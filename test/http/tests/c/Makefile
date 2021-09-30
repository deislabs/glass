WASI_CC ?= /opt/wasi-sdk/bin/clang
WITX_BINDGEN ?= witx-bindgen

example:
	$(WASI_CC) -I . -c -o http_v01.o deislabs_http_v01.c
	$(WASI_CC) -mexec-model=reactor lib.c http_v01.o -o ctest.wasm

bindgen:
	$(WITX_BINDGEN) c --export ../../deislabs_http_v01.witx
