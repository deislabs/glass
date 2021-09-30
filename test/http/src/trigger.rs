use anyhow::Error;
use async_trait::async_trait;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::net::SocketAddr;

#[async_trait]
pub trait HttpEngine: Clone + Send + Sync + 'static {
    async fn execute(&self, req: Request<Body>) -> Result<Response<Body>, Error>;
}

pub struct Trigger {
    pub address: String,
}

impl Trigger {
    pub async fn run(&self, runtime: impl HttpEngine) -> Result<(), Error> {
        let mk_svc = make_service_fn(move |_: &AddrStream| {
            let r = runtime.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let r2 = r.clone();
                    async move { r2.execute(req).await }
                }))
            }
        });

        let addr: SocketAddr = self.address.parse()?;
        Server::bind(&addr).serve(mk_svc).await?;

        Ok(())
    }
}
