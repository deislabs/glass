pub mod config;
pub mod context;
pub use crate::{
    config::Config,
    context::{Context, RegistryHelper},
};

use anyhow::Error;
use std::{sync::Arc, time::Instant};
use wasmtime::{Instance, InstancePre, Module, Store};

#[derive(Clone)]
pub struct WasiExecutionEngine<T: Default> {
    pub entrypoint_path: String,
    pub config: Config,

    pre: Arc<InstancePre<Context<T>>>,
    engine: wasmtime::Engine,
}

impl<T: Default> WasiExecutionEngine<T> {
    pub async fn new(
        server: &str,
        reference: &str,
        interface_name: String,
        config: Config,
    ) -> Result<Self, Error> {
        let entrypoint_path =
            RegistryHelper::entrypoint_from_bindle(&server, &reference, &interface_name).await?;

        Self::create_runtime_context(entrypoint_path, config)
    }

    pub fn new_from_local(entrypoint_path: String, config: Config) -> Result<Self, Error> {
        Self::create_runtime_context(entrypoint_path, config)
    }

    fn create_runtime_context(entrypoint_path: String, config: Config) -> Result<Self, Error> {
        let start = Instant::now();

        let ctx = Context::default();
        let (engine, mut store, linker) = ctx.get_engine_store_linker(config.clone())?;
        let module = Module::from_file(linker.engine(), entrypoint_path.clone())?;
        let pre = Arc::new(linker.instantiate_pre(&mut store, &module)?);

        log::info!(
            "Created engine from WASI component in: {:?}",
            start.elapsed()
        );

        Ok(Self {
            entrypoint_path,
            config,
            pre,
            engine,
        })
    }

    pub fn prepare_exec(&self, data: Option<T>) -> Result<(Store<Context<T>>, Instance), Error> {
        let mut store = Context::store_with_data(&self.engine, data, self.config.clone())?;
        let instance = self.pre.instantiate(&mut store)?;

        Ok((store, instance))
    }
}
