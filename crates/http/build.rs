use std::process::{self, Command};

const WITX_BINDGEN_REV: &str = "f16233e3907d080bad595b42b7b4a083098861d5";
const WITX_BINDGEN_REPO: &str = "https://github.com/bytecodealliance/witx-bindgen";
const WITX_BINDGEN_CLI_CRATE: &str = "witx-bindgen-cli";

const WITX_SOURCE: &str = "deislabs_http_v01.witx";
const WITX_BINDINGS_DESTINATION_DIR: &str = "src";
const WITX_BINDINGS_DESTINATION_FILE: &str = "bindings.rs";

const TESTS_DIR: &str = "tests";
const RUST_SIMPLE_TEST: &str = "rust";
const C_TEST: &str = "c";

fn main() {
    println!("cargo:rerun-if-changed={}", WITX_SOURCE);
    println!(
        "cargo:rerun-if-changed={}/{}",
        WITX_SOURCE, WITX_BINDINGS_DESTINATION_FILE
    );
    println!(
        "cargo:rerun-if-changed={}/{}/lib.rs",
        TESTS_DIR, RUST_SIMPLE_TEST
    );
    println!("cargo:rerun-if-changed={}/{}/lib.c", TESTS_DIR, C_TEST);

    generate_bindings();
    cargo_build_example(TESTS_DIR, RUST_SIMPLE_TEST);
    clang_build_example(TESTS_DIR, C_TEST);
}

fn cargo_build_example(dir: &str, example: &str) {
    let dir = format!("{}/{}", dir, example);

    run(
        vec!["cargo", "build", "--target", "wasm32-wasi", "--release"],
        Some(dir),
    );
}

fn clang_build_example(dir: &str, example: &str) {
    let dir = format!("{}/{}", dir, example);

    run(vec!["make"], Some(dir));
}

fn generate_bindings() {
    check_witx_bindgen();

    run(
        vec![
            "witx-bindgen",
            "wasmtime",
            "--export",
            "--out-dir",
            WITX_BINDINGS_DESTINATION_DIR,
            WITX_SOURCE,
        ],
        None,
    );

    run(
        vec![
            "witx-bindgen",
            "c",
            "--export",
            "--out-dir",
            format!("{}/{}", TESTS_DIR, C_TEST).as_str(),
            WITX_SOURCE,
        ],
        None,
    );
}

fn check_witx_bindgen() {
    match process::Command::new("witx-bindgen").spawn() {
        Ok(_) => {
            eprintln!("witx-bindgen already installed");
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
            );
        }
    }
}

fn run<S: Into<String> + AsRef<std::ffi::OsStr>>(args: Vec<S>, dir: Option<String>) {
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

    cmd.output().unwrap();
}

fn get_os_process() -> String {
    if cfg!(target_os = "windows") {
        String::from("powershell.exe")
    } else {
        String::from("/bin/bash")
    }
}
