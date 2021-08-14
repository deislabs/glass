use glass_engine::{Config, WasiExecutionContext};
use glass_ping::{PingEngine, PingEngineInterface};
use std::sync::Arc;

const SIMPLE_C_MODULE: &str = "tests/c/ctest.wasm";

#[tokio::test]
async fn test_c_ping() {
    let input =
        "Are you even a Roman emperor if you are not a delusional megalomaniac?".to_string();
    let exp = format!("PONG: {}", input);

    test_example(SIMPLE_C_MODULE, input, exp).await;
}

async fn test_example(entrypoint: &str, input: String, exp: String) {
    let ie = Arc::new(
        WasiExecutionContext::new_from_local(entrypoint.to_string(), Config::default()).unwrap(),
    );
    let pe = PingEngine(ie);
    let res = pe.execute(input).await.unwrap();

    assert_eq!(res, exp);
    println!("result: {}", res);
}
