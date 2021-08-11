use anyhow::{bail, Context, Error};
use glass_runtime_http::{listener::Listener, runtime::Runtime};
use structopt::{clap::AppSettings, StructOpt};
use wasi_cap_std_sync::Dir;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    env_logger::init();
    let cmd = Opt::from_args();
    cmd.run().await
}

impl Opt {
    pub async fn run(&self) -> Result<(), Error> {
        let dirs = compute_preopen_dirs(self.dirs.clone(), self.map_dirs.clone())?;

        match &self.cmd {
            SubCommand::Http(h) => {
                h.run(
                    self.server.clone(),
                    self.reference.clone(),
                    self.local.clone(),
                    self.vars.clone(),
                    dirs,
                )
                .await
            }
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "glass",
    author = "DeisLabs at Microsoft Azure",
    global_settings = &[AppSettings::ColoredHelp]
    )]
pub struct Opt {
    #[structopt(
        long = "server",
        default_value = "http://localhost:8000/v1",
        global = true,
        help = "URL of registry server used to pull WASI components"
    )]
    pub server: String,

    #[structopt(
        short = "e",
        long = "env",
        value_name = "NAME=VAL",
        parse(try_from_str = parse_env_var),
        global = true,
        help = "Pass an environment variable to the program"
    )]
    vars: Vec<(String, String)>,

    #[structopt(long = "dir", number_of_values = 1, value_name = "DIRECTORY")]
    dirs: Vec<String>,

    #[structopt(long = "mapdir", number_of_values = 1, value_name = "GUEST_DIR::HOST_DIR", parse(try_from_str = parse_map_dirs))]
    map_dirs: Vec<(String, String)>,

    #[structopt(
        long = "reference",
        global = true,
        help = "The full bindle name and version for the entrypoint component"
    )]
    pub reference: Option<String>,

    #[structopt(long = "local", global = true, help = "Path to local WASI component")]
    pub local: Option<String>,

    #[structopt(subcommand)]
    pub cmd: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    Http(HttpCmd),
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Start the default HTTP listener",
    global_settings = &[AppSettings::ColoredHelp, AppSettings::ArgRequiredElseHelp]
)]
pub struct HttpCmd {
    #[structopt(
        long = "listen",
        default_value = "127.0.0.1:3000",
        help = "IP address and port to listen on"
    )]
    pub address: String,
}

impl HttpCmd {
    async fn run(
        &self,
        server: String,
        reference: Option<String>,
        local: Option<String>,
        vars: Vec<(String, String)>,
        preopen_dirs: Vec<(String, Dir)>,
    ) -> Result<(), Error> {
        let runtime = match reference {
            Some(r) => Runtime::new(&server, &r, vars, preopen_dirs).await?,
            None => {
                match local {
                    Some(l) => Runtime::new_from_local(l, vars, preopen_dirs)?,
                    None => panic!("either a remote registry reference or local file must be passed to start the server")
                }
            }
        };

        let listener = Listener {
            address: self.address.clone(),
        };

        listener.run(runtime).await
    }
}

fn parse_env_var(s: &str) -> Result<(String, String), Error> {
    let parts: Vec<_> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        bail!("must be of the form `key=value`");
    }
    Ok((parts[0].to_owned(), parts[1].to_owned()))
}

fn parse_map_dirs(s: &str) -> Result<(String, String), Error> {
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() != 2 {
        bail!("must contain exactly one double colon ('::')");
    }
    Ok((parts[0].into(), parts[1].into()))
}

fn compute_preopen_dirs(
    dirs: Vec<String>,
    map_dirs: Vec<(String, String)>,
) -> Result<Vec<(String, Dir)>, Error> {
    let mut preopen_dirs = Vec::new();

    for dir in dirs.iter() {
        preopen_dirs.push((
            dir.clone(),
            unsafe { Dir::open_ambient_dir(dir) }
                .with_context(|| format!("failed to open directory '{}'", dir))?,
        ));
    }

    for (guest, host) in map_dirs.iter() {
        preopen_dirs.push((
            guest.clone(),
            unsafe { Dir::open_ambient_dir(host) }
                .with_context(|| format!("failed to open directory '{}'", host))?,
        ));
    }

    Ok(preopen_dirs)
}
