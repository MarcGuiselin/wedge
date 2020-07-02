:: =================================
:: Prepares environment for building
:: =================================

:: We need clippy and rustfmt(nightly)
rustup component add clippy
rustup component add rustfmt --toolchain nightly