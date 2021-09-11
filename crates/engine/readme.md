# The Glass Engine

This crate represents the main engine for a Glass runtime. It is intended to
only provide the low level WASI execution context, and to provide a way to
execute clearly defined entrypoints.

See the tests for examples on using this crate to build specialized execution
engines.

### Building and running

```
➜ rustup target add wasm32-wasi
➜ cargo build
➜ cargo test --all --all-features -- --nocapture
```
