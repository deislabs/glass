use crate::trigger::Ping;
use anyhow::Error;
use async_trait::async_trait;
use deislabs_ping_v01::{DeislabsPingV01, DeislabsPingV01Data};
use std::{
    sync::Arc,
    time::Instant,
};

witx_bindgen_wasmtime::export!("crates/ping/deislabs_ping_v01.witx");

type WasiExecutionContext = glass_engine::WasiExecutionContext<DeislabsPingV01Data>;

#[derive(Clone)]
pub struct PingEngine(pub Arc<WasiExecutionContext>);

#[async_trait]
impl Ping for PingEngine {
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
