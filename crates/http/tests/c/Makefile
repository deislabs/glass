WASI_CC ?= /opt/wasi-sdk/bin/clang

example:
	$(WASI_CC) -I . -c -o http_v01.o deislabs_http_v01.c
	$(WASI_CC) -Wl,--export-all -mexec-model=reactor lib.c http_v01.o -o ctest.wasm
