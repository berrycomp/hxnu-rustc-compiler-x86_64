# Release Checklist: v0.1.0

This checklist is for preparing and validating the first public SDK-capable release.
Validated on: 2026-03-29

## 1. Branch and Version

- [x] Work on branch `codex/release/v0.1.0`
- [x] Verify workspace version is `0.1.0`
- [x] Confirm target triple is fixed as `x86_64-unknown-hxnu`

## 2. Validation (Local)

- [x] `cargo test -p hxnu-target-spec -p hxnu-cargo`
- [x] `cargo check -p hxnu-rustc -p hxnu-cargo -p hxnu-sdk`
- [x] `./scripts/release-artifacts.sh`

Expected outcomes:

- `hxnu-sdk bundle build|pack|install` succeeds
- Installed SDK builds both examples (`no_std-hello`, `init-like`)
- ELF verification passes (`ELF64`, `x86-64`, at least one `PT_LOAD`)

## 3. SDK Artifact Contract

- [x] Bundle root contains `bin/`, `targets/`, `sysroot/`, `examples/`, `docs/`
- [x] `bin/` includes `hxnu-rustc`, `hxnu-cargo`
- [x] `targets/` includes `x86_64-unknown-hxnu.json`
- [x] `sysroot/lib/rustlib/x86_64-unknown-hxnu/lib` includes `libcore-*`, `liballoc-*`, `libcompiler_builtins-*`
- [x] Packaged archive exists at `build/hxnu-rustc-compiler-x86_64-0.1.0.tar.gz`

## 4. Docs and Consumer Contract

- [x] `README.md` quickstart matches current commands
- [x] `docs/usage.md` includes bundle + install + example build flow
- [x] `docs/integration.md` includes HXNU consumer contract and acceptance baseline

## 5. Git Tagging

- [x] Clean working tree
- [x] Create annotated tag:

```bash
git tag -a v0.1.0 -m "HXNU Rust Compiler SDK v0.1.0"
```

- [x] Verify:

```bash
git show v0.1.0 --stat
```
