use std::{fs::OpenOptions, io::Write, path::PathBuf, time::Instant};

use anyhow::Error;
use bindle::client::Client;
use wasi_cap_std_sync::{Dir, WasiCtxBuilder};
use wasi_common::WasiCtx;
use wasi_experimental_http_wasmtime::HttpCtx;
use wasi_nn_onnx_wasmtime::WasiNnTractCtx;
use wasmtime::{Engine, Linker, Store};

use crate::Config;

const WACM_CACHE_DIR: &str = ".wasi";

#[derive(Default)]
pub struct Context<T> {
    pub wasi_ctx: Option<WasiCtx>,
    pub tract_ctx: Option<WasiNnTractCtx>,
    pub runtime_data: Option<T>,
}

impl<T: Default> Context<T> {
    #[allow(clippy::type_complexity)]
    pub fn get_engine_store_linker(
        &self,
        config: Config,
    ) -> Result<(Engine, Store<Context<T>>, Linker<Context<T>>), Error> {
        let (wasi_config, vars, preopen_dirs, allowed_http_hosts) = (
            config.wasi_config.clone(),
            config.vars.clone(),
            config.preopen_dirs.clone(),
            config.allowed_http_hosts,
        );

        let engine = Engine::new(&wasi_config)?;
        let mut linker: Linker<Context<T>> = Linker::new(&engine);
        let mut store: Store<Context<T>> = Store::new(&engine, Context::default());

        linker.allow_unknown_exports(true);
        linker.allow_shadowing(true);

        let preopen_dirs = Self::compute_preopen_dirs(preopen_dirs)?;

        Context::populate_with_wasi(
            &mut store,
            &mut linker,
            vars,
            preopen_dirs,
            allowed_http_hosts,
        )?;

        Ok((engine, store, linker))
    }

    pub fn populate_with_wasi(
        store: &mut Store<Context<T>>,
        mut linker: &mut Linker<Context<T>>,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, Dir)>,
        allowed_http_hosts: Option<Vec<String>>,
    ) -> Result<(), Error> {
        wasmtime_wasi::add_to_linker(&mut linker, |host| host.wasi_ctx.as_mut().unwrap())?;

        let mut builder = WasiCtxBuilder::new()
            .inherit_stdin()
            .inherit_stdout()
            .inherit_stderr()
            .envs(&vars)?;

        for (name, dir) in preopen_dirs.into_iter() {
            builder = builder.preopened_dir(dir, name)?;
        }
        store.data_mut().wasi_ctx = Some(builder.build());

        let http = HttpCtx::new(allowed_http_hosts, None)?;
        http.add_to_generic_linker(&mut linker)?;

        wasi_nn_onnx_wasmtime::add_to_linker(linker, |host| host.tract_ctx.as_mut().unwrap())?;
        store.data_mut().tract_ctx = Some(WasiNnTractCtx::default());

        Ok(())
    }

    pub fn store_with_data(
        engine: &Engine,
        data: Option<T>,
        config: Config,
    ) -> Result<Store<Context<T>>, Error> {
        let mut store: Store<Context<T>> = Store::new(&engine, Context::default());
        store.data_mut().runtime_data = data;

        let mut builder = WasiCtxBuilder::new()
            .inherit_stdin()
            .inherit_stdout()
            .inherit_stderr()
            .envs(&config.vars)?;

        let preopen_dirs = Self::compute_preopen_dirs(config.preopen_dirs)?;

        for (name, dir) in preopen_dirs.into_iter() {
            builder = builder.preopened_dir(dir, name)?;
        }
        store.data_mut().wasi_ctx = Some(builder.build());

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
}

pub struct RegistryHelper {}
impl RegistryHelper {
    pub async fn entrypoint_from_bindle(
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
}
