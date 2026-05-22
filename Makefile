.PHONY: all build run install clean

BIN = concierge

all: build

build:
	cargo build --release

run:
	cargo run --bin $(BIN)

install: build
	cp target/release/$(BIN) /usr/local/bin/$(BIN)

clean:
	cargo clean
