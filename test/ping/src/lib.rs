use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use deislabs_ping_v01::{DeislabsPingV01, DeislabsPingV01Data};
use glass_engine::Executor;
use glass_pipeline::{EventSource, Subscription, Trigger};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time;

witx_bindgen_wasmtime::export!("test/ping/deislabs_ping_v01.witx");

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

type WasiExecutionContext = glass_engine::WasiExecutionContext<DeislabsPingV01Data>;

#[derive(Clone)]
pub struct PingEngine(pub Arc<WasiExecutionContext>);

#[async_trait]
impl Executor for PingEngine {
    type Input = String;
    type Output = String;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Error> {
        let start = Instant::now();
        let (mut store, instance) = self.0.prepare_exec(None)?;

        let pr = DeislabsPingV01::new(&mut store, &instance, |host| {
            host.runtime_data.as_mut().unwrap()
        })?;

        let res = pr.ping(&mut store, input.as_str())?;

        log::info!("Total execution time: {:?}", start.elapsed());
        Ok(res)
    }
}
