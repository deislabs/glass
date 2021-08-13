use anyhow::Error;
use std::ffi::OsStr;
use std::process::{self, Command, Output};

const WITX_BINDGEN_REV: &str = "f16233e3907d080bad595b42b7b4a083098861d5";
const WITX_BINDGEN_REPO: &str = "https://github.com/bytecodealliance/witx-bindgen";
const WITX_BINDGEN_CLI_CRATE: &str = "witx-bindgen-cli";

pub fn cargo_build_example(dir: &str, example: &str) -> Result<Output, Error> {
    let dir = format!("{}/{}", dir, example);

    run(
        vec!["cargo", "build", "--target", "wasm32-wasi", "--release"],
        Some(dir),
    )
}

pub fn clang_build_example(dir: &str, example: &str) -> Result<Output, Error> {
    let dir = format!("{}/{}", dir, example);

    run(vec!["make"], Some(dir))
}

pub fn generate_bindings(
    toolchain: &str,
    direction: &str,
    out_dir: &str,
    source: &str,
) -> Result<Output, Error> {
    check_witx_bindgen()?;
    run(
        vec![
            "witx-bindgen",
            toolchain,
            direction,
            "--out-dir",
            out_dir,
            source,
        ],
        None,
    )
}

pub fn check_witx_bindgen() -> Result<(), Error> {
    match process::Command::new("witx-bindgen").spawn() {
        Ok(_) => {
            eprintln!("witx-bindgen already installed");
            Ok(())
        }
        Err(_) => {
            println!("cannot find witx-bindgen, attempting to install");
            run(
                vec![
                    "cargo",
                    "install",
                    "--git",
                    WITX_BINDGEN_REPO,
                    "--rev",
                    WITX_BINDGEN_REV,
                    WITX_BINDGEN_CLI_CRATE,
                ],
                None,
            )?;
            Ok(())
        }
    }
}

pub fn run<S: Into<String> + AsRef<OsStr>>(
    args: Vec<S>,
    dir: Option<String>,
) -> Result<Output, Error> {
    let mut cmd = Command::new(get_os_process());
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());

    if let Some(dir) = dir {
        cmd.current_dir(dir);
    };

    cmd.arg("-c");
    cmd.arg(
        args.into_iter()
            .map(Into::into)
            .collect::<Vec<String>>()
            .join(" "),
    );

    println!("running {:#?}", cmd);

    Ok(cmd.output()?)
}

pub fn get_os_process() -> String {
    if cfg!(target_os = "windows") {
        String::from("powershell.exe")
    } else {
        String::from("/bin/bash")
    }
}
