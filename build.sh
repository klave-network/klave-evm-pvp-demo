echo building multiple apps.wasm from Rust...
rm Cargo.lock
rm -rf target
cargo clean

rm -f apps/klave-evm-pvp-app/src/bindings.rs

cargo component build --target wasm32-unknown-unknown --release
base64 -w 0 target/wasm32-unknown-unknown/release/klave_evm_pvp_app.wasm > ./klave_evm_pvp_app.b64

echo done