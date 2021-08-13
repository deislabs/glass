use glass_build::{cargo_build_example, clang_build_example, generate_bindings};

const HTTP_WITX: &str = "crates/http/deislabs_http_v01.witx";
const HTTP_TESTS_DIR: &str = "crates/http/tests";

const PING_WITX: &str = "crates/http/deislabs_ping_v01.witx";
const PING_TESTS_DIR: &str = "crates/ping/tests";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    build_http_tests();
    build_ping_tests();
}

fn build_http_tests() {
    println!("cargo:rerun-if-changed={}", HTTP_WITX);
    println!("cargo:rerun-if-changed={}/rust/lib.rs", HTTP_TESTS_DIR);
    println!("cargo:rerun-if-changed={}/c/lib.c", HTTP_TESTS_DIR);

    generate_bindings(
        "c",
        "--export",
        format!("{}/c", HTTP_TESTS_DIR).as_str(),
        HTTP_WITX,
    )
    .unwrap();
    clang_build_example(HTTP_TESTS_DIR, "c").unwrap();
    cargo_build_example(HTTP_TESTS_DIR, "rust").unwrap();
}

fn build_ping_tests() {
    println!("cargo:rerun-if-changed={}", PING_WITX);
    println!("cargo:rerun-if-changed={}/c/lib.c", PING_TESTS_DIR);

    generate_bindings(
        "c",
        "--export",
        format!("{}/c", PING_TESTS_DIR).as_str(),
        PING_WITX,
    )
    .unwrap();
    clang_build_example(PING_TESTS_DIR, "c").unwrap();
}
