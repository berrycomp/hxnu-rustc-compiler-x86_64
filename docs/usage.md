# Usage

## Prerequisites

- Rust toolchain pinned by `rust-toolchain.toml` (`nightly-2026-03-20`)
- Host tools: `cargo`, `file`, `objdump`

## Build Locally

```bash
cargo build -p hxnu-rustc -p hxnu-cargo -p hxnu-sdk
cargo test -p hxnu-target-spec
cargo test -p hxnu-cargo
```

## Build and Package SDK

```bash
cargo run -p hxnu-sdk -- bundle build
cargo run -p hxnu-sdk -- bundle pack
cargo run -p hxnu-sdk -- bundle install --prefix /tmp/hxnu-sdk
```

Default SDK output contract:

- `<sdk-root>/bin`: `hxnu-rustc`, `hxnu-cargo`
- `<sdk-root>/targets`: `x86_64-unknown-hxnu.json`
- `<sdk-root>/sysroot/lib/rustlib/x86_64-unknown-hxnu/lib`: prebuilt `core`, `alloc`, `compiler_builtins`
- `<sdk-root>/examples`: `no_std-hello`, `init-like`
- `<sdk-root>/docs`: usage and integration guides

## Compile Example with Installed SDK

```bash
SDK_ROOT=/tmp/hxnu-sdk/hxnu-rustc-compiler-x86_64-0.1.0
"$SDK_ROOT/bin/hxnu-cargo" build \
  --manifest-path "$SDK_ROOT/examples/no_std-hello/Cargo.toml" \
  --release \
  --target-dir /tmp/hxnu-example-build
```

Validate produced artifact:

```bash
./scripts/verify-elf.sh /tmp/hxnu-example-build/x86_64-unknown-hxnu/release/no-std-hello
```

## End-to-End Validation

```bash
./scripts/release-artifacts.sh
```
