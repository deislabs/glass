use crate::{
    bindings::deislabs_http_v01::{DeislabsHttpV01, DeislabsHttpV01Data, Method},
    listener,
};
use anyhow::Error;
use async_trait::async_trait;
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

#[async_trait]
impl listener::HttpRuntime for Runtime {
    async fn execute(&self, req: hyper::Request<Body>) -> Result<hyper::Response<Body>, Error> {
        let start = Instant::now();

        // create a new store so each request gets its own instance and data
        let mut store = self.store(Vec::new(), Vec::new())?;
        let instance = self.pre.instantiate(&mut store)?;

        // execute the instance's entrypoint using the request data and return the response
        let res = self.execute_impl(store, instance, req).await?;

        log::info!("Total execution time: {:#?}", start.elapsed());

        Ok(res)
    }
}
impl Runtime {
    async fn execute_impl(
        &self,
        mut store: Store<Ctx>,
        instance: Instance,
        req: hyper::Request<Body>,
    ) -> Result<hyper::Response<Body>, Error> {
        let r = DeislabsHttpV01::new(&mut store, &instance, |host| {
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

        let headers = Runtime::header_map_to_vec(req.headers())?;
        let headers: Vec<&str> = headers.iter().map(|s| &**s).collect();

        let (_, b) = req.into_parts();
        let b = hyper::body::to_bytes(b).await?.to_vec();
        let req = (m, u.as_str(), &headers[..], None, Some(&b[..]));

        let (status, headers, body) = r.handler(&mut store, req)?;
        log::info!("Result status code: {}", status);
        let mut hr = http::Response::builder().status(status);
        Runtime::append_headers(hr.headers_mut().unwrap(), headers)?;

        let body = match body {
            Some(b) => Body::from(b),
            None => Body::empty(),
        };

        Ok(hr.body(body)?)
    }

    pub async fn new(
        server: &str,
        reference: &str,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, Dir)>,
    ) -> Result<Self, Error> {
        let entrypoint_path = Runtime::entrypoint_from_bindle(&server, &reference).await?;

        Self::create_runtime(entrypoint_path, vars, preopen_dirs)
    }

    pub fn new_from_local(
        entrypoint_path: String,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, Dir)>,
    ) -> Result<Self, Error> {
        Self::create_runtime(entrypoint_path, vars, preopen_dirs)
    }
    // TODO
    // Populate the store with runtime specific data.
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

    fn create_runtime(
        entrypoint_path: String,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, Dir)>,
    ) -> Result<Self, Error> {
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
        Runtime::populate_with_wasi(&mut store, &mut linker, vars, preopen_dirs)?;

        let module = Module::from_file(linker.engine(), entrypoint_path)?;
        let pre = linker.instantiate_pre(&mut store, &module)?;
        let pre = Arc::new(pre);

        log::info!("Created runtime from module in: {:#?}", start.elapsed());

        Ok(Runtime { pre, engine })
    }

    async fn entrypoint_from_bindle(server: &str, reference: &str) -> Result<String, Error> {
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

    /// Generate a string vector from an HTTP header map.
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
    fn append_headers(
        res_headers: &mut http::HeaderMap,
        source: Option<Vec<String>>,
    ) -> Result<(), Error> {
        match source {
            Some(h) => {
                for pair in h {
                    let mut parts = pair.splitn(2, ':');
                    let k = parts.next().ok_or_else(|| {
                        anyhow::format_err!("Invalid serialized header: [{}]", pair)
                    })?;
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
}

#[derive(Default)]
pub struct Ctx {
    pub wasi_ctx: Option<WasiCtx>,
    pub runtime_data: Option<DeislabsHttpV01Data>,
}
