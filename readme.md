# Glass: a toolkit for building WebAssembly cloud services

Glass is a collection of building blocks for defining and implementing a
lightweight service that allows users to submit applications as WASI components
and execute them in a WebAssembly sandbox when a configured event source
triggers the execution.

More concretely, Glass is a toolkit for building WebAssembly-based
Function-as-a-Service engines that come with all the benefits of executing
untrusted client applications in a WebAssembly sandbox, and that gives
application developers the flexibility to compose and reuse language-independent
WASI components.

Core concepts:

- registry -- a public or private endpoint that can be used to efficiently
  distribute WASI components (and potentially other files, such as static
  assets)
- WASI component -- a collection of WebAssembly modules and interfaces
  distributed through a registry, together with an entrypoint that an engine can
  execute.
- engine (or execution context) -- a component that executes a Wasm module's
  entrypoint based on a clearly defined interface
- trigger -- a component that listens for events from an external source (HTTP
  trigger or webhook, events on a queue etc) and executes a WASI component using
  a configured engine

Check out [the documentation](docs/readme.md) or see example on how to use Glass
in [a simple ping example](crates/ping), or [an HTTP engine](crates/http).

_More details on using Glass soon._

### Notes and acknowledgements

This project is inspired from a number of projects from the cloud and
WebAssembly ecosystems:

- [`wasmtime-functions`][wf]
- [Wagi][wagi]

[wf]: https://github.com/peterhuene/wasmtime-functions
[wagi]: https://github.com/deislabs/wagi
[wacm]: https://github.com/deislabs/wacm
