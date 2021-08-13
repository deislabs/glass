use anyhow::Error;
use glass_engine::InnerEngine;
use glass_http::{Engine, Trigger};
use std::sync::Arc;
use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Start the default HTTP listener",
    global_settings = &[AppSettings::ColoredHelp, AppSettings::ArgRequiredElseHelp]
)]
pub struct HttpCmd {
    #[structopt(
        long = "listen",
        default_value = "127.0.0.1:3000",
        help = "IP address and port to listen on"
    )]
    pub address: String,

    #[structopt(
        long = "interface",
        default_value = "deislabs_http_v01",
        help = "WASI interface the entrypoint component implements"
    )]
    pub interface: String,
}

impl HttpCmd {
    pub async fn run(
        &self,
        server: String,
        reference: Option<String>,
        local: Option<String>,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<(), Error> {
        let ie = match reference {
            Some(r) => Arc::new(InnerEngine::new(&server, &r, self.interface.clone(), vars, preopen_dirs, allowed_http_hosts).await?),
            None => {
                match local {
                    Some(l) => Arc::new(InnerEngine::new_from_local(l, vars, preopen_dirs, allowed_http_hosts)?),
                    None => panic!("either a remote registry reference or local file must be passed to start the server")
                }
            }
        };

        let engine = Engine(ie);

        let trigger = Trigger {
            address: self.address.clone(),
        };

        trigger.run(engine).await
    }
}
