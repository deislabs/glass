use anyhow::{bail, Error};
use glass::{HttpCmd, PingCmd};
use glass_engine::Config;
use structopt::{clap::AppSettings, StructOpt};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    env_logger::init();
    let cmd = Opt::from_args();
    cmd.run().await
}

impl Opt {
    pub async fn run(&self) -> Result<(), Error> {
        let dirs = compute_preopen_dirs(self.dirs.clone(), self.map_dirs.clone())?;
        let config = Config::new(self.vars.clone(), dirs, self.allowed_hosts.clone());

        match &self.cmd {
            SubCommand::Http(h) => h.run(&self.module, &config).await,
            SubCommand::Ping(p) => p.run(&self.module, &config).await,
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
        short = "e",
        long = "env",
        value_name = "NAME=VAL",
        parse(try_from_str = parse_env_var),
        global = true,
        help = "Pass an environment variable to the program"
    )]
    vars: Vec<(String, String)>,

    #[structopt(
        long = "dir",
        global = true,
        number_of_values = 1,
        value_name = "DIRECTORY"
    )]
    dirs: Vec<String>,

    #[structopt(
        long = "mapdir",
        global = true,
        number_of_values = 1,
        value_name = "GUEST_DIR::HOST_DIR",
        parse(try_from_str = parse_map_dirs)
    )]
    map_dirs: Vec<(String, String)>,

    #[structopt(
        short = "a",
        long = "allowed-host",
        global = true,
        help = "Host the guest module is allowed to make outbound HTTP requests to"
    )]
    allowed_hosts: Option<Vec<String>>,

    #[structopt(
        long = "local",
        global = true,
        default_value = "",
        help = "Path to local WASI component"
    )]
    pub module: String,

    #[structopt(subcommand)]
    pub cmd: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    Http(HttpCmd),
    Ping(PingCmd),
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
) -> Result<Vec<(String, String)>, Error> {
    let mut preopen_dirs = Vec::new();

    for dir in dirs.iter() {
        preopen_dirs.push((dir.clone(), dir.clone()));
    }

    for (guest, host) in map_dirs.iter() {
        preopen_dirs.push((guest.clone(), host.clone()));
    }

    Ok(preopen_dirs)
}
