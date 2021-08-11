use crate::{
    bindings::deislabs_http_v01::{DeislabsHttpV01, DeislabsHttpV01Data, Method},
    listener,
};
use anyhow::Error;
use async_trait::async_trait;
use glass_runtime_context::RegistryHelper;
use hyper::Body;
use std::{str::FromStr, sync::Arc, time::Instant};
use wasmtime::{Engine, Instance, InstancePre, Module, Store};

type Context = glass_runtime_context::Context<DeislabsHttpV01Data>;

const HTTP_INTERFACE: &str = "deislabs_http_v01";

#[derive(Clone)]
pub struct Runtime {
    entrypoint_path: String,
    vars: Vec<(String, String)>,
    preopen_dirs: Vec<(String, String)>,
    allowed_http_hosts: Option<Vec<String>>,

    pre: Arc<InstancePre<Context>>,
    engine: Engine,
}

#[async_trait]
impl listener::HttpRuntime for Runtime {
    async fn execute(&self, req: hyper::Request<Body>) -> Result<hyper::Response<Body>, Error> {
        let start = Instant::now();

        let vars = &self.vars;
        let preopen_dirs = &self.preopen_dirs;
        let mut store =
            Context::store_with_data(&self.engine, None, vars.clone(), preopen_dirs.clone())?;
        let instance = self.pre.instantiate(&mut store)?;

        let res = self.execute_impl(store, instance, req).await?;

        log::info!("Total request execution time: {:#?}", start.elapsed());

        Ok(res)
    }
}
impl Runtime {
    async fn execute_impl(
        &self,
        mut store: Store<Context>,
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
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<Self, Error> {
        let entrypoint_path =
            RegistryHelper::entrypoint_from_bindle(&server, &reference, HTTP_INTERFACE).await?;

        Self::create_runtime_context(entrypoint_path, vars, preopen_dirs, allowed_http_hosts)
    }

    pub fn new_from_local(
        entrypoint_path: String,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<Self, Error> {
        Self::create_runtime_context(entrypoint_path, vars, preopen_dirs, allowed_http_hosts)
    }

    fn create_runtime_context(
        entrypoint_path: String,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<Self, Error> {
        let start = Instant::now();

        let ctx = Context::default();
        let (engine, mut store, linker) = ctx.get_engine_store_linker(
            vars.clone(),
            preopen_dirs.clone(),
            allowed_http_hosts.clone(),
        )?;

        let module = Module::from_file(linker.engine(), entrypoint_path.clone())?;
        let pre = Arc::new(linker.instantiate_pre(&mut store, &module)?);

        log::info!("Created runtime from module in: {:#?}", start.elapsed());

        Ok(Runtime {
            entrypoint_path,
            vars,
            preopen_dirs,
            allowed_http_hosts,
            pre,
            engine,
        })
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
