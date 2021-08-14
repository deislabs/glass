use anyhow::Error;
use glass_engine::{Config, WasiExecutionContext};
use glass_ping::{PingEngine, TimerTrigger};
use std::sync::Arc;
use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Start a simple ping/pong STDIN trigger",
    global_settings = &[AppSettings::ColoredHelp, AppSettings::ArgRequiredElseHelp]
)]
pub struct PingCmd {
    #[structopt(
        long = "interface",
        default_value = "deislabs_ping_v01",
        help = "WASI interface the entrypoint component implements"
    )]
    pub interface: String,

    #[structopt(
        long = "interval-seconds",
        default_value = "2",
        help = "Interval in seconds"
    )]
    pub interval_seconds: u64,
}

impl PingCmd {
    pub async fn run(
        &self,
        server: String,
        reference: Option<String>,
        local: Option<String>,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<(), Error> {
        let config = Config::new(vars, preopen_dirs, allowed_http_hosts);

        let engine = match reference {
            Some(r) => PingEngine(Arc::new(WasiExecutionContext::new(&server, &r, self.interface.clone(), config).await?)),
            None => {
                match local {
                    Some(l) => PingEngine(Arc::new(WasiExecutionContext::new_from_local(l, config)?)),
                    None => panic!("either a remote registry reference or local file must be passed to start the server")
                }
            }
        };

        let trigger = TimerTrigger {
            interval: std::time::Duration::from_secs(self.interval_seconds),
        };

        trigger.run(engine).await
    }
}
