# Glass: a toolkit for building WebAssembly cloud services

> This project is experimental, and in its very early stages of development.
> They authors strongly encourage people not to take a dependency on the project
> at this time.

Glass is a collection of building blocks for defining and implementing
lightweight WebAssembly runtimes, and couple event triggering with the execution
of WebAssembly modules.

Check out [the documentation](docs/readme.md) or see example on how to use Glass
in [a simple ping example](crates/engine/test/ping), or
[an HTTP engine](crates/engine/test/http).

_More details on using Glass soon._

### Notes and acknowledgements

This project is inspired from a number of projects from the cloud and
WebAssembly ecosystems:

- [`wasmtime-functions`][wf]
- [Wagi][wagi]

[wf]: https://github.com/peterhuene/wasmtime-functions
[wagi]: https://github.com/deislabs/wagi
[wacm]: https://github.com/deislabs/wacm
