EXE ?= redtail
all:
	cargo build --release
	cp target/release/redtail $(EXE)
