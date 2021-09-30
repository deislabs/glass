use glass_engine::{Executor, WasiExecutionContextBuilder};
use glass_ping::PingEngine;
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
    let pe = PingEngine(Arc::new(
        WasiExecutionContextBuilder::build_default(entrypoint).unwrap(),
    ));
    let res = pe.execute(input).await.unwrap();

    assert_eq!(res, exp);
    println!("result: {}", res);
}
