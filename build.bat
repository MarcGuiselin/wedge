:: =========================================
:: Formats, builds, and checks debug version
:: =========================================

cargo +nightly fmt
:: Installer requires the other binaries to be compiled first
cargo build --workspace --exclude installer
:: Now build installer
cargo build --package installer
:: Run unit tests
cargo test --all
:: Run clippy
cargo clippy