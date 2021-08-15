use anyhow::Error;
use bindle::client::Client;
use std::{fs::OpenOptions, io::Write, path::PathBuf, sync::Arc, time::Instant};
use wasi_cap_std_sync::{Dir, WasiCtxBuilder};
use wasi_common::WasiCtx;
use wasi_experimental_http_wasmtime::HttpCtx;
use wasi_nn_onnx_wasmtime::WasiNnTractCtx;
use wasmtime::{Engine, Instance, InstancePre, Linker, Module, Store};

/// Configuration for the engine.
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

/// The execution context used by engines.
/// This object can be directly created using its `new` functions,
/// or granularly configured using a `WasiExecutionContextBuilder`.
#[derive(Clone)]
pub struct WasiExecutionContext<T: Default> {
    pub entrypoint_path: String,
    pub config: Config,
    pre: Arc<InstancePre<Context<T>>>,
    engine: Engine,
}

/// Runtime data for the instances.
/// The generic type can either be directly the `-Data` type generated
/// by witx-bindgen, or if additional host imports need to be configured,
/// it can also contain state related to those imports.
#[derive(Default)]
pub struct Context<T> {
    pub wasi_ctx: Option<WasiCtx>,
    pub nn_ctx: Option<WasiNnTractCtx>,
    pub runtime_data: Option<T>,
}

/// A builder that helps configure and build `WasiExecutionContext` instances.
///
/// Additional host imports can be defined using the linker and store fields.
pub struct WasiExecutionContextBuilder<T: Default> {
    pub config: Config,
    pub engine: Engine,
    pub store: Store<Context<T>>,
    pub linker: Linker<Context<T>>,
}

impl<T: Default> WasiExecutionContextBuilder<T> {
    /// Create a new `WasiExecutionContextBuilder`.
    pub fn new(config: Config) -> Result<Self, Error> {
        let engine = Engine::new(&config.wasi_config)?;
        let linker: Linker<Context<T>> = Linker::new(&engine);
        let store: Store<Context<T>> = Store::new(&engine, Context::default());

        Ok(Self {
            config,
            engine,
            store,
            linker,
        })
    }

    /// Configure support for the core WASI API.
    pub fn add_wasi(&mut self) -> Result<&mut Self, Error> {
        wasmtime_wasi::add_to_linker(&mut self.linker, |host| host.wasi_ctx.as_mut().unwrap())?;
        Ok(self)
    }

    /// Configure support for experimental outbound HTTP support.
    pub fn add_experimental_http(&mut self) -> Result<&mut Self, Error> {
        HttpCtx::new(self.config.allowed_http_hosts.clone(), None)?
            .add_to_generic_linker(&mut self.linker)?;
        Ok(self)
    }

    /// Configure WASI NN for the current context using the self-contained
    /// ONNX implementation for WASI NN.
    pub fn add_nn(&mut self) -> Result<&mut Self, Error> {
        wasi_nn_onnx_wasmtime::add_to_linker(&mut self.linker, |host| {
            host.nn_ctx.as_mut().unwrap()
        })?;
        Ok(self)
    }

    /// Configure all available host imports.
    ///
    /// Currently, this includes core WASI, experimental HTTP
    /// support, and the ONNX implementation of WASI NN.
    pub fn add_all(&mut self) -> Result<&mut Self, Error> {
        self.add_wasi()?;
        self.add_experimental_http()?;
        self.add_nn()?;

        Ok(self)
    }

    /// Create a `WasiExecutionContext` using the configured store
    /// and linker, and pre-instantiate the entrypoint WebAssembly module.
    pub fn build(&mut self, entrypoint_path: String) -> Result<WasiExecutionContext<T>, Error> {
        let start = Instant::now();

        let (config, engine) = (self.config.clone(), self.engine.clone());
        let module = Module::from_file(&self.engine, entrypoint_path.clone())?;
        let pre = Arc::new(self.linker.instantiate_pre(&mut self.store, &module)?);

        log::info!(
            "Created engine from WASI component in: {:?}",
            start.elapsed()
        );

        Ok(WasiExecutionContext {
            entrypoint_path,
            config,
            pre,
            engine,
        })
    }
}

impl<T: Default> WasiExecutionContext<T> {
    /// Creates a new `WasiExecutionContext` based on the WASI component reference
    /// from the registry and configuration.
    ///
    /// The WASI context created in this way enables support for HTTP and WASI NN.
    /// This can be configured by using a `WasiExecutionContextBuilder`.
    pub async fn new(
        server: &str,
        reference: &str,
        interface_name: String,
        config: Config,
    ) -> Result<Self, Error> {
        let entrypoint_path = entrypoint_from_bindle(server, reference, &interface_name).await?;

        Self::preinstantiate(entrypoint_path, config)
    }

    /// Create a new `WasiExecutionContext` from a local file.
    /// See `WasiExecutionContext::new` for details about the resulting instance.
    pub fn new_from_local(entrypoint_path: String, config: Config) -> Result<Self, Error> {
        Self::preinstantiate(entrypoint_path, config)
    }

    fn preinstantiate(entrypoint_path: String, config: Config) -> Result<Self, Error> {
        let mut builder = WasiExecutionContextBuilder::new(config)?;
        builder.add_all()?;
        builder.build(entrypoint_path)
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

    /// Prepare the execution by finishing the instantiation proces for the module
    /// using real runtime data, then return the store and instance to be used by the engine.
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
