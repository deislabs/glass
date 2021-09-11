use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use glass_engine::{Config, WasiExecutionContextBuilder};
use glass_ping::PingEngine;
use glass_pipeline::{Binding, Pipeline};
use glass_pipeline::{EventSource, Subscription, Trigger};
use std::sync::Arc;
use std::time::Duration;
use structopt::{clap::AppSettings, StructOpt};
use tokio::time;

pub struct TimerTrigger {
    pub interval: Duration,
}

#[async_trait]
impl Trigger for TimerTrigger {
    async fn run<S: Subscription<Self::Event>>(&self, subscription_broker: S) -> Result<(), Error> {
        let mut interval = time::interval(self.interval);
        loop {
            interval.tick().await;
            subscription_broker.on_event(Local::now())?;
        }
    }
}

impl EventSource for TimerTrigger {
    type Event = DateTime<chrono::Local>;
}
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
