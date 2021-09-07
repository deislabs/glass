use anyhow::Error;
use glass_engine::{Config, WasiExecutionContextBuilder};
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
    pub async fn run(&self, module: &str, config: &Config) -> Result<(), Error> {
        let engine = Engine(Arc::new(
            WasiExecutionContextBuilder::new(config)?.build(&module)?,
        ));

        let trigger = Trigger {
            address: self.address.clone(),
        };

        trigger.run(engine).await
    }
}
