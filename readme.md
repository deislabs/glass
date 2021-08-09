# Glass

_A brittle prototype for building, distributing, and running composable
WebAssembly modules as lightweight cloud services._

### Using Glass

Starting the runtime with a reference to a remote WASI component will first
download the missing components of the WASI component, link it locally, then
pre-instantiate a WebAssembly module:

```
➜ glass --server <WASI component registry> --reference rust-service/0.1.0
[2021-08-09T06:06:39Z INFO  glass_runtime::ctx] Downloaded bindle in: 38.7931ms
[2021-08-09T06:06:39Z INFO  wacm_bindle::utils] Setting interface glass_runtime.witx as main interface for glass_runtime from bindle rust-service/0.1.0
[2021-08-09T06:06:39Z INFO  wacm_bindle::utils] Pushing library rust-service.wasm in the list for bindle rust-service/0.10.0
[2021-08-09T06:06:39Z INFO  glass_runtime::ctx] Linked bindle in: 2.0787ms
[2021-08-09T06:06:39Z INFO  glass_runtime::ctx] Wrote entrypoint to file in : 371.1µs
[2021-08-09T06:06:39Z INFO  glass_runtime::ctx] Created runtime from module in: 30.877ms
```

Then, for each request, it will instantiate the module and execute the `handler`
entrypoint function from the module:

```
➜ curl -X POST localhost:3000/<some-path>
request: Method::Get, "/", ["host:localhost:3000", "user-agent:curl/7.71.1", "accept:*/*"]
[2021-08-09T06:08:01Z INFO  glass_runtime::ctx] Result status code: 200
[2021-08-09T06:08:01Z INFO  glass_runtime::ctx] Total execution time: 152.8µs
```

### Writing handlers

Currently, the entire interface for writing handlers is very simple:

```fsharp
type http_status = u16
type body = list<u8>
type headers = list<string>
type params = list<string>
type uri = string

type request = tuple<method, uri, headers, option<params>, option<body>>
type response = tuple<http_status, option<headers>, option<body>>

enum method {
    get,
    post,
    put,
    delete,
    patch,
}

handler: function(req: request) -> response
```

This means that any language that can generate bindings from a WITX file can be
used for writing handlers. Currently, the list contains Rust, C, and C++.

Complete example in Rust:

```rust
wacm::export!("glass_runtime");

struct GlassRuntime {}

impl glass_runtime::GlassRuntime for GlassRuntime {
    fn handler(req: glass_runtime::Request) -> glass_runtime::Response {
        println!("request: {:?}, {:?}, {:?}", req.0, req.1, req.2);

        (200, None, None)
    }
}
```

The [WASI components manager][wacm] can be used to distribute the modules, which
can then be pulled by the runtime.

Example for C++:

```cpp
#include <glass_runtime.h>
#include <stdio.h>

void glass_runtime_handler(
    glass_runtime_request_t *req,
    glass_runtime_http_status_t *status,
    glass_runtime_option_headers_t *headers,
    glass_runtime_option_body_t *body
)
{
    *status = 200;
}
```

### Writing applications in JavaScript

> Note that this is currently not fully functional.

```javascript
async function handler(req, res) {
  console.log(
    "The answer to life, the universe, and everything: " + (await getAnswer())
  );

  console.log(req.method());
  res.status = 200;
}

async function getAnswer() {
  return 42;
}
```

Running this with the example JS file from the runtime root results in the
following:

```
The answer to life, the universe, and everything: 42
[2021-08-09T06:22:10Z INFO  glass_runtime::ctx] Result status code: 200
[2021-08-09T06:22:10Z INFO  glass_runtime::ctx] Total execution time: 27.9057ms
```

### Notes and acknowledgements

This project is inspired from a number of projects from the cloud and
WebAssembly ecosystems:

- [`wasmtime-functions`][wf]
- [Wagi][wagi]

[wf]: https://github.com/peterhuene/wasmtime-functions
[wagi]: https://github.com/deislabs/wagi
[wacm]: https://github.com/deislabs/wacm
