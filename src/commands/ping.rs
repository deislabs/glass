use anyhow::Error;
use async_trait::async_trait;
use glass_engine::{Config, WasiExecutionContextBuilder};
use glass_ping::{PingEngine, TimerTrigger};
use glass_pipeline::{Binding, Pipeline};
use std::sync::Arc;
use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Start a simple ping/pong STDIN trigger",
    global_settings = &[AppSettings::ColoredHelp, AppSettings::ArgRequiredElseHelp]
)]
pub struct PingCmd {
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

        let pipeline = Pipeline::new(
            trigger,
            time_to_str,
            engine,
            identity::<String>,
            ConsoleOutputBinding {},
        );

        pipeline.run().await
    }
}

#[derive(Clone)]
struct ConsoleOutputBinding;

#[async_trait]
impl Binding for ConsoleOutputBinding {
    type DataType = String;

    async fn propagate_result(&self, input: Self::DataType) -> Result<(), Error> {
        tokio::time::sleep(tokio::time::Duration::from_millis(2750)).await;
        let now = chrono::Local::now().format("%H:%M:%S");
        println!("OUTPUT BINDING': {} at {}", input, now);
        Ok(())
    }
}

fn time_to_str(t: chrono::DateTime<chrono::Local>) -> Result<String, Error> {
    Ok(format!("{}", t.format("%Y-%m-%d][%H:%M:%S")))
}

fn identity<T>(t: T) -> Result<T, Error> {
    Ok(t)
}
