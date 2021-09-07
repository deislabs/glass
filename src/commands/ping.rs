use anyhow::Error;
use glass_engine::{Config, WasiExecutionContextBuilder};
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
    pub async fn run(&self, module: &str, config: &Config) -> Result<(), Error> {
        let engine = PingEngine(Arc::new(
            WasiExecutionContextBuilder::new(&config)?.build(&module)?,
        ));

        let trigger = TimerTrigger {
            interval: std::time::Duration::from_secs(self.interval_seconds),
        };

        trigger.run(engine).await
    }
}
