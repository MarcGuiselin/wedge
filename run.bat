:: ===============================
:: Builds debug and runs installer 
:: ===============================

cargo build --workspace --exclude installer
cargo run --package installer