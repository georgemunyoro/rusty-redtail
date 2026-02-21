EXE = redtail

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

rule:
	cargo clean
	cargo rustc --release --bin redtail -- -C target-cpu=native --emit link=$(NAME)


