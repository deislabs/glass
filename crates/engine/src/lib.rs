pub mod context;
pub use context::{Context, RegistryHelper};

use anyhow::Error;
use std::{sync::Arc, time::Instant};
use wasmtime::{Instance, InstancePre, Module, Store};

#[derive(Clone)]
pub struct InnerEngine<T: Default> {
    pub entrypoint_path: String,
    pub vars: Vec<(String, String)>,
    pub preopen_dirs: Vec<(String, String)>,
    pub allowed_http_hosts: Option<Vec<String>>,

    pub pre: Arc<InstancePre<Context<T>>>,
    pub engine: wasmtime::Engine,
}

impl<T: Default> InnerEngine<T> {
    pub async fn new(
        server: &str,
        reference: &str,
        interface_name: String,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<Self, Error> {
        let entrypoint_path =
            RegistryHelper::entrypoint_from_bindle(&server, &reference, &interface_name).await?;

        Self::create_runtime_context(entrypoint_path, vars, preopen_dirs, allowed_http_hosts)
    }

    pub fn new_from_local(
        entrypoint_path: String,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<Self, Error> {
        Self::create_runtime_context(entrypoint_path, vars, preopen_dirs, allowed_http_hosts)
    }

    fn create_runtime_context(
        entrypoint_path: String,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<Self, Error> {
        let start = Instant::now();

        let ctx = Context::default();
        let (engine, mut store, linker) = ctx.get_engine_store_linker(
            vars.clone(),
            preopen_dirs.clone(),
            allowed_http_hosts.clone(),
        )?;

        let module = Module::from_file(linker.engine(), entrypoint_path.clone())?;
        let pre = Arc::new(linker.instantiate_pre(&mut store, &module)?);

        log::info!("Created runtime from module in: {:#?}", start.elapsed());

        Ok(Self {
            entrypoint_path,
            vars,
            preopen_dirs,
            allowed_http_hosts,
            pre,
            engine,
        })
    }

    pub fn prepare_exec(&self, data: Option<T>) -> Result<(Store<Context<T>>, Instance), Error> {
        let vars = &self.vars;
        let preopen_dirs = &self.preopen_dirs;
        let mut store =
            Context::store_with_data(&self.engine, data, vars.clone(), preopen_dirs.clone())?;
        let instance = self.pre.instantiate(&mut store)?;

        Ok((store, instance))
    }
}
