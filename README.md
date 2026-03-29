# HXNU Rust Compiler (x86_64)

Rust-first compiler/toolchain workspace for HXNU.

## Scope

- `x86_64-unknown-hxnu` target bootstrap
- `hxnu-rustc` rustc_driver-based frontend
- `hxnu-cargo` Cargo wrapper for target defaults
- `hxnu-sdk` bundle build/pack/install workflow

This repository is independent from the kernel repository and is consumed as an external toolchain.
