use anyhow::Error;
use async_trait::async_trait;
use deislabs_ping_v01::{DeislabsPingV01, DeislabsPingV01Data};
use std::{sync::Arc, time::Instant};

witx_bindgen_wasmtime::export!("crates/ping/deislabs_ping_v01.witx");

#[async_trait]
pub trait PingEngineTrait: Clone + Send + Sync + 'static {
    async fn execute(&self, input: String) -> Result<String, Error>;
}

pub struct TimerTrigger {
    pub interval: std::time::Duration,
}

impl TimerTrigger {
    pub async fn run(&self, runtime: impl PingEngineTrait) -> Result<(), Error> {
        let mut interval = tokio::time::interval(self.interval);
        loop {
            interval.tick().await;
            let res = runtime
                .execute(format!(
                    "{}",
                    chrono::Local::now().format("%Y-%m-%d][%H:%M:%S")
                ))
                .await?;

            log::info!("Result: {}", res);
        }
    }
}

type InnerEngine = glass_engine::InnerEngine<DeislabsPingV01Data>;

#[derive(Clone)]
pub struct PingEngine(pub Arc<InnerEngine>);

#[async_trait]
impl PingEngineTrait for PingEngine {
    async fn execute(&self, input: String) -> Result<String, Error> {
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
