# Glass: a toolkit for building WebAssembly cloud services

> This project is experimental, and in its very early stages of development.
> They authors strongly encourage people not to take a dependency on the project
> at this time.

Glass is a collection of building blocks for defining and implementing
lightweight services that allow users to submit applications as Wasm components
and execute them in a WebAssembly sandbox when a configured event source
triggers the execution.

More concretely, Glass is a toolkit for building WebAssembly-based
Function-as-a-Service engines that come with all the benefits of executing
untrusted client applications in a WebAssembly sandbox, and that gives
application developers the flexibility to compose and reuse language-independent
WASI components.

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
