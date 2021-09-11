use anyhow::Error;
use async_trait::async_trait;
use deislabs_ping_v01::{DeislabsPingV01, DeislabsPingV01Data};
use glass_engine::Executor;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time;

witx_bindgen_wasmtime::export!("crates/engine/test/ping/deislabs_ping_v01.witx");

#[async_trait]
pub trait Ping: Clone + Send + Sync + 'static {
    async fn execute(&self, input: String) -> Result<String, Error>;
}

pub struct TimerTrigger {
    pub interval: Duration,
}

impl TimerTrigger {
    pub async fn run(&self, runtime: impl Ping) -> Result<(), Error> {
        let mut interval = time::interval(self.interval);
        loop {
            interval.tick().await;
            let res = runtime
                .execute(format!(
                    "{}",
                    chrono::Local::now().format("%Y-%m-%d][%H:%M:%S")
                ))
                .await?;

            log::info!("{}\n", res);
        }
    }
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
