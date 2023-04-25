#!/usr/bin/env sh

cargo build --release --target wasm32-unknown-unknown --no-default-features
wasm-bindgen --out-dir ./web-out/ --target web ./target/wasm32-unknown-unknown/release/quaternions-offline.wasm
wasm-opt -Oz -o ./web-out/quaternions-offline_bg.wasm ./web-out/quaternions-offline_bg.wasm

cp ./web/* ./web-out/
