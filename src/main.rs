use std::net::SocketAddr;

use anyhow::Error;
use glass_runtime::Runtime;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Server,
};
use structopt::{clap::AppSettings, StructOpt};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    env_logger::init();
    let cmd = Opt::from_args();
    cmd.run().await
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "glass",
    author = "DeisLabs at Microsoft Azure",
    global_settings = &[AppSettings::ColoredHelp]
    )]
pub struct Opt {
    #[structopt(
        long = "server",
        default_value = "http://localhost:8000/v1",
        help = "URL of registry server used to pull WASI components"
    )]
    pub server: String,

    #[structopt(
        long = "listen",
        default_value = "127.0.0.1:3000",
        help = "IP address and port to listen on"
    )]
    pub address: String,

    #[structopt(
        long = "reference",
        help = "The full bindle name and version for the entrypoint component",
        required_unless = "local"
    )]
    pub reference: Option<String>,

    #[structopt(long = "local", global = true, help = "Path to local WASI component")]
    pub local: Option<String>,
}

impl Opt {
    pub async fn run(&self) -> Result<(), Error> {
        let runtime = match &self.reference {
            Some(r) => Runtime::new(&self.server, &r).await?,
            None => {
                match &self.local {
                    Some(l) => Runtime::new_from_local(l.into())?,
                    None => panic!("either a remote registry reference or local file must be passed to start the server")
                }
            }
        };

        let mk_svc = make_service_fn(move |_: &AddrStream| {
            let r = runtime.clone();
            async move {
                Ok::<_, std::convert::Infallible>(service_fn(move |req| {
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
