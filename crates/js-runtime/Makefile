DEFINES := -DDEBUG -DJS_DEBUG

ROOT_SRC?=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/
SM_SRC=$(ROOT_SRC)/spidermonkey/
SM_OBJ=$(SM_SRC)lib/*
FSM_SRC=$(ROOT_SRC)

WASI_CC ?= /opt/wasi-sdk/bin/clang
WASI_CXX ?= /opt/wasi-sdk/bin/clang++

WIZER ?= wizer
WITX_BINDGEN ?= witx-bindgen

CXX_FLAGS := -std=gnu++17 -Wall -fno-exceptions

FSM_CPP := $(wildcard $(FSM_SRC)*.cpp)
FSM_DEP := $(patsubst $(FSM_SRC)%.cpp,$(OBJDIR)%.d,$(FSM_CPP))
FSM_OBJ := $(patsubst $(FSM_SRC)%.cpp,$(OBJDIR)%.o,$(FSM_CPP))

BINDINGS_DIR=$(ROOT_SRC)/bindings/
B_CPP := $(wildcard $(BINDINGS_DIR)*.c)
B_DEP := $(patsubst $(BINDINGS_DIR)%.c,$(OBJDIR)%.d,$(FSM_CPP))
B_OBJ := $(patsubst $(BINDINGS_DIR)%.c,$(OBJDIR)%.o,$(FSM_CPP))

ifneq (, $(shell which wasm-opt))
WASM_STRIP = wasm-opt --strip-debug -o $@ $@
endif

all: bindgen runtime.wasm

-include $(FSM_DEP)


%.o: $(FSM_SRC)%.cpp $(FSM_SRC)Makefile
	$(WASI_CXX) $(CXX_FLAGS) $(DEFINES) -I $(SM_SRC)include -I $(ROOT_SRC)include -I $(BINDINGS_DIR) -O3 -MMD -MP -c -o $@ $<

runtime.wasm: $(FSM_OBJ) $(SM_OBJ)
	$(WASI_CXX) $(CXX_FLAGS) $(DEFINES) -I $(BINDINGS_DIR) -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576 -mexec-model=command -O3 -o $@ deislabs_http_v01.o $^
	$(WASM_STRIP)
	$(WIZER) --allow-wasi -r _start=wizer.resume --dir . -o initialized.wasm $@

bindgen: $(B_CPP)
	$(WITX_BINDGEN) c --export ../http/deislabs_http_v01.witx --out-dir $(BINDINGS_DIR)
	$(WASI_CC) -I$(BINDINGS_DIR) -c $(B_CPP)

clean:
	rm *.o && rm *.wasm && rm *.d

bootstrap:
	wget https://github.com/tschneidereit/spidermonkey-wasi-embedding/releases/download/rev_18af21b26bacfe3ad846d2e002871bdbe704886f/spidermonkey-wasm-static-lib_debug.tar.gz
	mkdir spidermonkey
	tar xfv spidermonkey-wasm-static-lib_debug.tar.gz -C spidermonkey
	rm spidermonkey-wasm-static-lib_debug.tar.gz
