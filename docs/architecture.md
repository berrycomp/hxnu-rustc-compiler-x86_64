# Architecture

HXNU compiler workspace is split into four crates:

- `hxnu-rustc`: rustc_driver entrypoint that enforces HXNU target defaults.
- `hxnu-cargo`: Cargo wrapper that wires `hxnu-rustc`, build-std defaults, and target.
- `hxnu-sdk`: SDK bundle build/pack/install orchestration.
- `hxnu-target-spec`: shared target constants, argument utilities, and target-spec validation.

Design goals:

- Keep target identity fixed as `x86_64-unknown-hxnu`.
- Keep repository independent from kernel source tree.
- Keep SDK layout stable for CI/automation consumption.
