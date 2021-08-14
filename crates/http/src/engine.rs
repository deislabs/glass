use crate::trigger::HttpEngine;
use anyhow::Error;
use async_trait::async_trait;
use deislabs_http_v01::{DeislabsHttpV01, DeislabsHttpV01Data, Method};
use hyper::{Body, Request, Response};
use std::{str::FromStr, sync::Arc, time::Instant};
use wasmtime::{Instance, Store};

witx_bindgen_wasmtime::export!("crates/http/deislabs_http_v01.witx");

type WasiEngine = glass_engine::WasiExecutionEngine<DeislabsHttpV01Data>;
type InnerContext = glass_engine::Context<DeislabsHttpV01Data>;

#[derive(Clone)]
pub struct Engine(pub Arc<WasiEngine>);

#[async_trait]
impl HttpEngine for Engine {
    async fn execute(
        &self,
        req: hyper::Request<hyper::Body>,
    ) -> Result<hyper::Response<hyper::Body>, Error> {
        let start = Instant::now();
        let (store, instance) = self.0.prepare_exec(None)?;
        let res = self.execute_impl(store, instance, req).await?;
        log::info!("Total request execution time: {:#?}", start.elapsed());
        Ok(res)
    }
}

impl Engine {
    async fn execute_impl(
        &self,
        mut store: Store<InnerContext>,
        instance: Instance,
        req: Request<Body>,
    ) -> Result<Response<Body>, Error> {
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

        let headers = Self::header_map_to_vec(req.headers())?;
        let headers: Vec<&str> = headers.iter().map(|s| &**s).collect();

        let (_, b) = req.into_parts();
        let b = hyper::body::to_bytes(b).await?.to_vec();
        let req = (m, u.as_str(), &headers[..], None, Some(&b[..]));

        let (status, headers, body) = r.handler(&mut store, req)?;
        log::info!("Result status code: {}", status);
        let mut hr = http::Response::builder().status(status);
        Self::append_headers(hr.headers_mut().unwrap(), headers)?;

        let body = match body {
            Some(b) => Body::from(b),
            None => Body::empty(),
        };

        Ok(hr.body(body)?)
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
