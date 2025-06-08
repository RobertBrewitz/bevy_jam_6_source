#!/usr/bin/make

BIN_NAME != grep -Ei name Cargo.toml | head -n1 | grep -oEi '".*"' | cut -d '"' -f 2
VERSION != grep -Ei version Cargo.toml | head -n1 | grep -oEi '".*"' | cut -d '"' -f 2
ITCHIO_NAME != grep -Ei ITCHIO .cargo/config.toml | head -n1 | grep -oEi '".*"' | cut -d '"' -f 2

all: build-linux build-windows build-wasm

.PHONY: build
build:
	cargo build

.PHONY: dev
dev:
	cargo +nightly run -Zcodegen-backend --features dev

.PHONY: build-wasm
build-wasm:
	rm -rf ./builds/release-wasm
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --no-typescript --out-name game --out-dir builds/release-wasm --target web ./target/wasm32-unknown-unknown/release/$(BIN_NAME).wasm
	cp -r public/* ./builds/release-wasm/
	cp -r assets ./builds/release-wasm/

.PHONY: run-wasm
run-wasm:
	cargo run --release --target wasm32-unknown-unknown

.PHONY: publish-wasm
publish-wasm: build-wasm
	rm -f ./builds/wasm.zip
	zip --recurse-paths ./builds/wasm.zip ./builds/release-wasm
	butler push \
	  --fix-permissions \
	  --userversion="$(VERSION)" \
	  ./builds/wasm.zip \
	  ${ITCHIO_NAME}:wasm

.PHONY: build-windows
build-windows:
	rm -rf ./builds/release-windows
	mkdir -p ./builds/release-windows
	cargo build --release --target x86_64-pc-windows-gnu
	mv ./target/x86_64-pc-windows-gnu/release/$(BIN_NAME).exe ./builds/release-windows/
	cp -r assets ./builds/release-windows/

.PHONY: build-linux
build-linux:
	rm -rf ./builds/release-linux
	mkdir -p ./builds/release-linux
	cargo build --release --target x86_64-unknown-linux-gnu
	mv ./target/x86_64-unknown-linux-gnu/release/$(BIN_NAME) ./builds/release-linux/
	cp -r assets ./builds/release-linux/
