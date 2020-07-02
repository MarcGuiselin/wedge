cargo +nightly fmt
:: Installer requires the other binaries to be compiled first
cargo build --release --workspace --exclude installer
:: Now build installer
cargo build --release --package installer