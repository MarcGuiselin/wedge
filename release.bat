cargo +nightly fmt
:: Installer requires the other binaries to be compiled first
cargo build --release --workspace --exclude installer
:: Now build installer
cargo build --release --package installer
:: Generate checksum file
@echo off
for /f "skip=1" %%a in (
  'certutil -hashfile ./target/release/installer.exe SHA256'
) do if not defined sha set "sha=%%a"
echo %sha% installer.exe > ./target/release/checksums.txt
