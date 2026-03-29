# HXNU Consumer Contract

This toolchain is consumed from external repositories (for example the HXNU kernel repo).

Contract:

1. Add SDK `bin/` to `PATH`.
2. Invoke `hxnu-cargo build` in consumer repository.
3. Toolchain injects `x86_64-unknown-hxnu` target defaults when missing.
4. Artifact class remains ELF64 x86_64 freestanding outputs.
