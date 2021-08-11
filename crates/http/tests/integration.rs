use glass_runtime_http::{listener::HttpRuntime, runtime::Runtime};
use hyper::body;

const SIMPLE_RUST_MODULE: &str = "tests/rust/target/wasm32-wasi/release/simple_rust.wasm";
const SIMPLE_C_MODULE: &str = "tests/c/ctest.wasm";

#[tokio::test]
async fn test_rust_handler() {
    let exp_status = 200;
    let exp_body =
        "Nerva, Trajan, Hadrian, Pius, and Marcus Aurelius are the five best emperors. Don't @ me."
            .as_bytes()
            .to_vec();

    test_example(SIMPLE_RUST_MODULE, exp_status, exp_body).await;
}

#[tokio::test]
async fn test_c_handler() {
    let exp_status = 418;
    let exp_body = "Octavian was a pretty good emperor".as_bytes().to_vec();

    test_example(SIMPLE_C_MODULE, exp_status, exp_body).await;
}

async fn test_example(entrypoint: &str, exp_status: u16, exp_body: Vec<u8>) {
    let req = http::Request::builder()
        .method("GET")
        .uri("https://www.rust-lang.org/")
        .header("X-Custom-Foo", "Bar")
        .header("X-Custom-Foo2", "Bar2")
        .body(body::Body::empty())
        .unwrap();
    let r = Runtime::new_from_local(entrypoint.to_string(), Vec::new(), Vec::new(), None).unwrap();
    let res = r.execute(req).await.unwrap();

    println!("response status: {:?}", res.status());
    assert_eq!(exp_status, res.status());

    let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
    assert_eq!(exp_body, body_bytes.to_vec());

    println!(
        "response body: {:?}",
        std::str::from_utf8(&body_bytes.to_vec()).unwrap()
    );
}
