use crate::trigger::PingExecutor;
use anyhow::Error;
use async_trait::async_trait;
use deislabs_ping_v01::{DeislabsPingV01, DeislabsPingV01Data};
use std::{
    sync::Arc,
    time::Instant,
};
use wasmtime::{Instance, Store};

witx_bindgen_wasmtime::export!("crates/ping/deislabs_ping_v01.witx");

type WasiExecutionContext = glass_engine::WasiExecutionContext<DeislabsPingV01Data>;
type DataContext = glass_engine::Context<DeislabsPingV01Data>;

#[derive(Clone)]
pub struct PingEngine(pub Arc<WasiExecutionContext>);

#[async_trait]
impl PingExecutor for PingEngine {
    async fn execute(&self, input: String) -> Result<String, Error> {
        let start = Instant::now();
        let (store, instance) = self.0.prepare_exec(None)?;
        let res = self.execute_impl(store, instance, input).await?;
        log::info!("Total execution time: {:?}", start.elapsed());
        Ok(res)
    }
}

impl PingEngine {
    async fn execute_impl(
        &self,
        mut store: Store<DataContext>,
        instance: Instance,
        input: String,
    ) -> Result<String, Error> {
        let r = DeislabsPingV01::new(&mut store, &instance, |host| {
            host.runtime_data.as_mut().unwrap()
        })?;

        let res = r.ping(&mut store, input.as_str())?;
        Ok(res)
    }
}
