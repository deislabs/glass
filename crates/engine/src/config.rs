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
