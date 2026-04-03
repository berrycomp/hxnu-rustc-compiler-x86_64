# HXNU Consumer Contract

This toolchain is consumed from external repositories (for example the HXNU kernel repo) without converting to monorepo.

## Integration Steps

1. Install SDK bundle:
   - `hxnu-sdk bundle install --prefix /opt/hxnu-sdk` (or any local prefix)
2. Export SDK binaries:
   - `export PATH="/opt/hxnu-sdk/hxnu-rustc-compiler-x86_64-0.1.0/bin:$PATH"`
3. Build consumer crate with wrapper:
   - `hxnu-cargo build --release`

## Runtime Contract

- Default target: `x86_64-unknown-hxnu`
- Supported explicit targets:
  - `x86_64-unknown-hxnu`
  - `aarch64-unknown-hxnu`
  - `powerpc64le-unknown-hxnu`
  - `powerpc64-unknown-hxnu`
- `hxnu-cargo` sets:
  - `RUSTC=hxnu-rustc`
  - `RUST_TARGET_PATH` to include bundled `targets/`
  - `RUSTFLAGS` to include `-C panic=abort`
  - `-Z build-std=core,alloc,compiler_builtins` for compile-like commands
- `hxnu-rustc` enforces:
  - target default when missing
  - custom target compatibility (`-Z unstable-options`)
  - no-std-compatible panic strategy (`panic=abort`)

## Acceptance Baseline

- Kernel or userspace consumer builds with `hxnu-cargo` without modifying the kernel compiler internals.
- Produced artifacts stay in ELF64 class with target-expected machine/endian pairing.
- Program headers contain `PT_LOAD` segments for HXNU loader compatibility.
