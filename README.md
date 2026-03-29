# HXNU Rust Compiler (x86_64)

Rust-first compiler/toolchain workspace for HXNU.

## Scope

- Fixed target identity: `x86_64-unknown-hxnu`
- `hxnu-rustc`: `rustc_driver`-based frontend
- `hxnu-cargo`: Cargo wrapper with default target and `build-std` contract
- `hxnu-sdk`: SDK bundle build/pack/install workflow

This repository is independent from the HXNU kernel repository and is consumed as an external toolchain.

## Quickstart

```bash
cargo run -p hxnu-sdk -- bundle build
cargo run -p hxnu-sdk -- bundle pack
cargo run -p hxnu-sdk -- bundle install --prefix /tmp/hxnu-sdk
```

For end-to-end validation (tests + bundle + install + ELF checks):

```bash
./scripts/release-artifacts.sh
```
