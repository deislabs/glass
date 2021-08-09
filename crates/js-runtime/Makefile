# DEFINES := -DDEBUG -DJS_DEBUG

# # ROOT_SRC?=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/
# ROOT_SRC?=$(shell pwd)/
# SM_SRC=$(ROOT_SRC)/spidermonkey/
# SM_OBJ=$(SM_SRC)lib/*
# FSM_SRC=$(ROOT_SRC)

# WASI_CC ?= /opt/wasi-sdk/bin/clang
# WASI_CXX ?= /opt/wasi-sdk/bin/clang++
# WIZER ?= wizer

# CXX_FLAGS := -std=gnu++17 -Wall -fno-exceptions

# FSM_CPP := $(wildcard $(FSM_SRC)*.c**)
# FSM_DEP := $(patsubst $(FSM_SRC)%.c**,$(OBJDIR)%.d,$(FSM_CPP))
# FSM_OBJ := $(patsubst $(FSM_SRC)%.c**,$(OBJDIR)%.o,$(FSM_CPP))

# # ifneq (, $(shell which wasm-opt))
# # WASM_STRIP = wasm-opt --strip-debug -o $@ $@
# # endif

# all: spidermonkey.wasm

# -include $(FSM_DEP)

# %.o: $(FSM_SRC)%.c $(FSM_SRC)Makefile
# 	$(WASI_CXX) $(CXX_FLAGS) $(DEFINES) -I $(SM_SRC)include -I $(ROOT_SRC)include -O3 -MMD -MP -c -o $@ $<

# spidermonkey.wasm: $(FSM_OBJ) $(SM_OBJ)
# 	$(WASI_CXX) $(CXX_FLAGS) $(DEFINES) -fvisibility=default -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576 -mexec-model=command -O3 -o $@ $^
# 	$(WASM_STRIP)

# runtime.wasm: $(FSM_OBJ) $(SM_OBJ)
# 	$(WASI_CC) $(CXX_FLAGS) $(DEFINES) -fvisibility=default -Wl,--export-all -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576 -mexec-model=reactor -o glass_runtime.o -o $@ $^
# 	$(WASM_STRIP)

# bootstrap:
# 	wget https://github.com/tschneidereit/spidermonkey-wasi-embedding/releases/download/rev_18af21b26bacfe3ad846d2e002871bdbe704886f/spidermonkey-wasm-static-lib_debug.tar.gz
# 	mkdir spidermonkey
# 	tar xfv spidermonkey-wasm-static-lib_debug.tar.gz -C spidermonkey
# 	rm spidermonkey-wasm-static-lib_debug.tar.gz


DEFINES := -DDEBUG -DJS_DEBUG

ROOT_SRC?=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/
SM_SRC=$(ROOT_SRC)/spidermonkey/
SM_OBJ=$(SM_SRC)lib/*
FSM_SRC=$(ROOT_SRC)

WASI_CC ?= /opt/wasi-sdk/bin/clang
WASI_CXX ?= /opt/wasi-sdk/bin/clang++
WIZER ?= wizer

CXX_FLAGS := -std=gnu++17 -Wall -fno-exceptions

FSM_CPP := $(wildcard $(FSM_SRC)*.cpp)
FSM_DEP := $(patsubst $(FSM_SRC)%.cpp,$(OBJDIR)%.d,$(FSM_CPP))
FSM_OBJ := $(patsubst $(FSM_SRC)%.cpp,$(OBJDIR)%.o,$(FSM_CPP))

ifneq (, $(shell which wasm-opt))
WASM_STRIP = wasm-opt --strip-debug -o $@ $@
endif

all: spidermonkey.wasm

-include $(FSM_DEP)

glass_runtime.o: $(FSM_SRC)Makefile
	$(WASI_CC) $(DEFINES) -I $(ROOT_SRC)include -c -o $@ glass_runtime.c

%.o: $(FSM_SRC)%.cpp $(FSM_SRC)Makefile
	$(WASI_CXX) $(CXX_FLAGS) $(DEFINES) -I $(SM_SRC)include -I $(ROOT_SRC)include -O3 -MMD -MP -c -o $@ $<

spidermonkey.wasm: $(FSM_OBJ) $(SM_OBJ)
	$(WASI_CXX) $(CXX_FLAGS) $(DEFINES) -fvisibility=hidden -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576 -mexec-model=command -O3 -o $@ $^
	$(WASM_STRIP)

runtime.wasm: $(FSM_OBJ) $(SM_OBJ)
	make glass_runtime.o
	$(WASI_CXX) $(CXX_FLAGS) $(DEFINES) -fvisibility=hidden -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576 -mexec-model=command -O3 -o $@ glass_runtime.o $^
	$(WASM_STRIP)

yolo.wasm: $(FSM_OBJ) $(SM_OBJ)
	$(WASI_CC) -mexec-model=reactor -o $@ glass_runtime.o $^



bootstrap:
	wget https://github.com/tschneidereit/spidermonkey-wasi-embedding/releases/download/rev_18af21b26bacfe3ad846d2e002871bdbe704886f/spidermonkey-wasm-static-lib_debug.tar.gz
	mkdir spidermonkey
	tar xfv spidermonkey-wasm-static-lib_debug.tar.gz -C spidermonkey
	rm spidermonkey-wasm-static-lib_debug.tar.gz
