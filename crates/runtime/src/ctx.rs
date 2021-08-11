use crate::glass_runtime::{GlassRuntime, GlassRuntimeData, Method};
use anyhow::Error;
use bindle::client::Client;
use hyper::Body;
use std::{fs::OpenOptions, io::Write, path::PathBuf, str::FromStr, sync::Arc, time::Instant};
use wasmtime::{Config, Engine, Instance, InstancePre, Linker, Module, Store};
use wasmtime_wasi::*;

const WASMTIME_CACHE_DIR: &str = ".wasmtime-cache";
const WACM_CACHE_DIR: &str = ".wasi";
const RUNTIME_INTERFACE: &str = "glass_runtime";

#[derive(Clone)]
pub struct Runtime {
    pre: Arc<InstancePre<Ctx>>,
    engine: Engine,
}

impl Runtime {
    pub async fn execute(&self, req: hyper::Request<Body>) -> Result<hyper::Response<Body>, Error> {
        let start = Instant::now();

        // create a new store so each request gets its own instance and data
        let mut store = self.store(Vec::new(), Vec::new())?;
        let instance = self.pre.instantiate(&mut store)?;

        let res = self.execute_impl(store, instance, req).await?;

        log::info!("Total execution time: {:#?}", start.elapsed());

        Ok(res)
    }

    async fn execute_impl(
        &self,
        mut store: Store<Ctx>,
        instance: Instance,
        req: hyper::Request<Body>,
    ) -> Result<hyper::Response<Body>, Error> {
        let r = GlassRuntime::new(&mut store, &instance, |host| {
            host.runtime_data.as_mut().unwrap()
        })?;

        let m = match *req.method() {
            http::Method::GET => Method::Get,
            http::Method::POST => Method::Post,
            http::Method::PUT => Method::Put,
            http::Method::DELETE => Method::Delete,
            http::Method::PATCH => Method::Patch,
            _ => todo!(),
        };
        let u = req.uri().to_string();

        let headers = header_map_to_vec(req.headers())?;
        let headers: Vec<&str> = headers.iter().map(|s| &**s).collect();

        let (_, b) = req.into_parts();
        let b = hyper::body::to_bytes(b).await?.to_vec();
        let req = (m, u.as_str(), &headers[..], None, Some(&b[..]));

        let (status, headers, body) = r.handler(&mut store, req)?;
        log::info!("Result status code: {}", status);
        let mut hr = http::Response::builder().status(status);
        append_headers(hr.headers_mut().unwrap(), headers)?;

        let body = match body {
            Some(b) => Body::from(b),
            None => Body::empty(),
        };

        Ok(hr.body(body)?)
    }

    pub async fn new(server: &str, reference: &str) -> Result<Self, Error> {
        let entrypoint_path = Runtime::entrypoint_from_bindle(&server, &reference).await?;

        Self::runtime_from_module(entrypoint_path)
    }

    pub fn new_from_local(entrypoint_path: String) -> Result<Self, Error> {
        Self::runtime_from_module(entrypoint_path)
    }

    fn runtime_from_module(entrypoint_path: String) -> Result<Self, Error> {
        let start = Instant::now();

        let mut config = Config::default();
        config.wasm_multi_memory(true);
        config.wasm_module_linking(true);
        if let Ok(p) = std::fs::canonicalize(WASMTIME_CACHE_DIR) {
            config.cache_config_load(p)?;
        };

        let engine = Engine::new(&config)?;
        let mut linker: Linker<Ctx> = Linker::new(&engine);
        let mut store = Store::new(&engine, Ctx::default());

        linker.allow_unknown_exports(true);
        linker.allow_shadowing(true);
        populate_with_wasi(&mut store, &mut linker, Vec::new(), Vec::new())?;

        let module = Module::from_file(linker.engine(), entrypoint_path)?;
        let pre = linker.instantiate_pre(&mut store, &module)?;
        let pre = Arc::new(pre);

        log::info!("Created runtime from module in: {:#?}", start.elapsed());

        Ok(Runtime { pre, engine })
    }

    pub async fn entrypoint_from_bindle(server: &str, reference: &str) -> Result<String, Error> {
        let start = Instant::now();

        let linking_path = PathBuf::from(WACM_CACHE_DIR).join("_linking");
        let bindler = Client::new(server)?;

        wacm_bindle::utils::download_bindle(
            bindler,
            reference.to_string(),
            &linking_path,
            Some("dat".into()),
        )
        .await?;

        log::info!("Downloaded bindle in: {:?}", start.elapsed(),);

        let start = Instant::now();

        let out = wacm_bindle::linker::Linker::link_component_for_interface_to_bytes(
            linking_path.to_string_lossy().into(),
            RUNTIME_INTERFACE.to_string(),
        )?;

        log::info!("Linked bindle in: {:?}", start.elapsed(),);

        let start = Instant::now();

        let entrypoint_path = linking_path.join("entrypoint.wasm");
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(entrypoint_path.clone())?;
        f.write_all(&out)?;

        log::info!("Wrote entrypoint to file in : {:?}", start.elapsed(),);

        Ok(entrypoint_path.to_string_lossy().to_string())
    }

    // TODO
    // Populate the store.
    fn store(
        &self,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, Dir)>,
    ) -> Result<Store<Ctx>, Error> {
        let mut builder = WasiCtxBuilder::new()
            .inherit_stdin()
            .inherit_stdout()
            .inherit_stderr()
            .envs(&vars)?;

        for (name, dir) in preopen_dirs.into_iter() {
            builder = builder.preopened_dir(dir, name)?;
        }

        let mut store = Store::new(&self.engine, Ctx::default());

        store.data_mut().wasi_ctx = Some(builder.build());

        Ok(store)
    }
}

fn populate_with_wasi(
    store: &mut Store<Ctx>,
    linker: &mut Linker<Ctx>,
    vars: Vec<(String, String)>,
    preopen_dirs: Vec<(String, Dir)>,
) -> Result<(), Error> {
    wasmtime_wasi::add_to_linker(linker, |host| host.wasi_ctx.as_mut().unwrap())?;

    let mut builder = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdout()
        .inherit_stderr()
        .envs(&vars)?;

    for (name, dir) in preopen_dirs.into_iter() {
        builder = builder.preopened_dir(dir, name)?;
    }
    store.data_mut().wasi_ctx = Some(builder.build());

    Ok(())
}

fn header_map_to_vec(hm: &http::HeaderMap) -> Result<Vec<String>, Error> {
    let mut res = Vec::new();
    for (name, value) in hm
        .iter()
        .map(|(name, value)| (name.as_str(), std::str::from_utf8(value.as_bytes())))
    {
        let value = value?;
        anyhow::ensure!(
            !name
                .chars()
                .any(|x| x.is_control() || "(),/:;<=>?@[\\]{}".contains(x)),
            "Invalid header name"
        );
        anyhow::ensure!(
            !value.chars().any(|x| x.is_control()),
            "Invalid header value"
        );
        res.push(format!("{}:{}", name, value));
    }
    Ok(res)
}

/// Append a header map string to a mutable http::HeaderMap.
#[allow(unused)]
fn append_headers(
    res_headers: &mut http::HeaderMap,
    source: Option<Vec<String>>,
) -> Result<(), Error> {
    match source {
        Some(h) => {
            for pair in h {
                let mut parts = pair.splitn(2, ':');
                let k = parts
                    .next()
                    .ok_or_else(|| anyhow::format_err!("Invalid serialized header: [{}]", pair))?;
                let v = parts.next().unwrap();
                res_headers.insert(
                    http::header::HeaderName::from_str(k)?,
                    http::header::HeaderValue::from_str(v)?,
                );
            }

            Ok(())
        }
        None => Ok(()),
    }
}

#[derive(Default)]
pub struct Ctx {
    pub wasi_ctx: Option<WasiCtx>,
    pub runtime_data: Option<GlassRuntimeData>,
}

#[cfg(test)]
mod tests {
    use hyper::body;

    use crate::Runtime;

    #[tokio::test]
    async fn test_start_runtime() {
        env_logger::init();
        let request = http::Request::builder()
            .method("GET")
            .uri("https://www.rust-lang.org/")
            .header("X-Custom-Foo", "Bar")
            .header("ana-are-mere", "marcel-pavel")
            .body(body::Body::empty())
            .unwrap();
        let r = Runtime::new("http://localhost:8000/v1", "components-test/0.15.0")
            .await
            .unwrap();
        r.execute(request).await.unwrap();
    }
}
