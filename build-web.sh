#!/usr/bin/env sh

export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'
cargo build --release --target wasm32-unknown-unknown --features web
echo running wasm-bindgen...
wasm-bindgen --out-dir ./web-out/ --target web ./target/wasm32-unknown-unknown/release/quaternions-offline.wasm
echo done
echo running wasm-opt...
wasm-opt -Oz -o ./web-out/quaternions-offline_bg.wasm ./web-out/quaternions-offline_bg.wasm
echo done

cp ./web/* ./web-out/
