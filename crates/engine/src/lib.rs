use anyhow::Error;
use bindle::client::Client;
use std::{fs::OpenOptions, io::Write, path::PathBuf, sync::Arc, time::Instant};
use wasi_cap_std_sync::{Dir, WasiCtxBuilder};
use wasi_common::WasiCtx;
use wasi_experimental_http_wasmtime::HttpCtx;
use wasi_nn_onnx_wasmtime::WasiNnTractCtx;
use wasmtime::{Engine, Instance, InstancePre, Linker, Module, Store};

#[derive(Clone, Default)]
pub struct Config {
    pub vars: Vec<(String, String)>,
    pub preopen_dirs: Vec<(String, String)>,
    pub allowed_http_hosts: Option<Vec<String>>,

    pub wasi_config: wasmtime::Config,
}

impl Config {
    pub fn new(
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, String)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Self {
        let mut wasi_config = wasmtime::Config::default();
        wasi_config.wasm_multi_memory(true);
        wasi_config.wasm_module_linking(true);

        Self {
            vars,
            preopen_dirs,
            allowed_http_hosts,
            wasi_config,
        }
    }
}

#[derive(Clone)]
pub struct WasiExecutionContext<T: Default> {
    pub entrypoint_path: String,
    pub config: Config,

    pre: Arc<InstancePre<Context<T>>>,
    engine: Engine,
}

#[derive(Default)]
pub struct Context<T> {
    pub wasi_ctx: Option<WasiCtx>,
    pub nn_ctx: Option<WasiNnTractCtx>,
    pub runtime_data: Option<T>,
}

impl<T: Default> WasiExecutionContext<T> {
    pub async fn new(
        server: &str,
        reference: &str,
        interface_name: String,
        config: Config,
    ) -> Result<Self, Error> {
        let entrypoint_path = entrypoint_from_bindle(server, reference, &interface_name).await?;

        Self::preinstantiate(entrypoint_path, config)
    }

    pub fn new_from_local(entrypoint_path: String, config: Config) -> Result<Self, Error> {
        Self::preinstantiate(entrypoint_path, config)
    }

    fn preinstantiate(entrypoint_path: String, config: Config) -> Result<Self, Error> {
        let start = Instant::now();

        let engine = Engine::new(&config.wasi_config)?;
        let mut linker: Linker<Context<T>> = Linker::new(&engine);
        let mut store: Store<Context<T>> = Store::new(&engine, Context::default());

        Self::link(&mut linker, &config)?;
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

    fn link(mut linker: &mut Linker<Context<T>>, config: &Config) -> Result<(), Error> {
        wasmtime_wasi::add_to_linker(linker, |host| host.wasi_ctx.as_mut().unwrap())?;
        let http = HttpCtx::new(config.allowed_http_hosts.clone(), None)?;
        http.add_to_generic_linker(&mut linker)?;
        wasi_nn_onnx_wasmtime::add_to_linker(linker, |host| host.nn_ctx.as_mut().unwrap())?;

        Ok(())
    }

    fn create_store(&self, data: Option<T>) -> Result<Store<Context<T>>, Error> {
        let mut store: Store<Context<T>> = Store::new(&self.engine, Context::default());
        let mut builder = WasiCtxBuilder::new()
            .inherit_stdin()
            .inherit_stdout()
            .inherit_stderr()
            .envs(&self.config.vars)?;

        let preopen_dirs = Self::compute_preopen_dirs(self.config.preopen_dirs.clone())?;

        for (name, dir) in preopen_dirs.into_iter() {
            builder = builder.preopened_dir(dir, name)?;
        }

        store.data_mut().wasi_ctx = Some(builder.build());
        store.data_mut().nn_ctx = Some(WasiNnTractCtx::default());
        store.data_mut().runtime_data = data;

        Ok(store)
    }

    fn compute_preopen_dirs(map_dirs: Vec<(String, String)>) -> Result<Vec<(String, Dir)>, Error> {
        let mut preopen_dirs = Vec::new();
        for (guest, host) in map_dirs.iter() {
            preopen_dirs.push((
                guest.clone(),
                anyhow::Context::with_context(unsafe { Dir::open_ambient_dir(host) }, || {
                    format!("failed to open directory '{}'", host)
                })?,
            ));
        }

        Ok(preopen_dirs)
    }

    pub fn prepare_exec(&self, data: Option<T>) -> Result<(Store<Context<T>>, Instance), Error> {
        let mut store = self.create_store(data)?;
        let instance = self.pre.instantiate(&mut store)?;

        Ok((store, instance))
    }
}

const WACM_CACHE_DIR: &str = ".wasi";
async fn entrypoint_from_bindle(
    server: &str,
    reference: &str,
    desired_interface: &str,
) -> Result<String, Error> {
    let start = Instant::now();

    let linking_path = PathBuf::from(WACM_CACHE_DIR).join("_linking");
    let bindler = Client::new(server)?;

    wacm_bindle::utils::download_bindle(
        bindler,
        reference.to_string(),
        &linking_path,
        Some("dat".into()),
    )
    .await?;

    log::info!("Downloaded bindle in: {:?}", start.elapsed(),);

    let start = Instant::now();

    let out = wacm_bindle::linker::Linker::link_component_for_interface_to_bytes(
        linking_path.to_string_lossy().into(),
        desired_interface.to_string(),
    )?;

    log::info!("Linked bindle in: {:?}", start.elapsed(),);

    let start = Instant::now();

    let entrypoint_path = linking_path.join("entrypoint.wasm");
    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .open(entrypoint_path.clone())?;
    f.write_all(&out)?;

    log::info!("Wrote entrypoint to file in : {:?}", start.elapsed(),);

    Ok(entrypoint_path.to_string_lossy().to_string())
}
