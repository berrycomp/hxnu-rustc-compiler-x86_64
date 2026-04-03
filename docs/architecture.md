# Architecture

HXNU compiler workspace is split into four crates:

- `hxnu-rustc`: rustc_driver entrypoint that enforces HXNU target defaults.
- `hxnu-cargo`: Cargo wrapper that wires `hxnu-rustc`, build-std defaults, and target.
- `hxnu-sdk`: SDK bundle build/pack/install orchestration.
- `hxnu-target-spec`: shared target constants, argument utilities, and target-spec validation.

Design goals:

- Keep default target identity as `x86_64-unknown-hxnu` while supporting multiple HXNU targets.
- Keep repository independent from kernel source tree.
- Keep SDK layout stable for CI/automation consumption.

SDK layout contract:

- `bin/`: `hxnu-rustc`, `hxnu-cargo`
- `targets/`: one JSON spec per target (`x86_64`, `aarch64`, `powerpc64le`, `powerpc64`)
- `sysroot/lib/rustlib/<triple>/lib`: prebuilt `core`, `alloc`, `compiler_builtins` per target
- `examples/`: `no_std-hello`, `init-like`
- `docs/`: consumer-facing usage and integration notes
