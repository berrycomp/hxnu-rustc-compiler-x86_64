# Release Checklist: v0.1.0

This checklist is for preparing and validating the first public SDK-capable release.

## 1. Branch and Version

- [ ] Work on branch `codex/release/v0.1.0`
- [ ] Verify workspace version is `0.1.0`
- [ ] Confirm target triple is fixed as `x86_64-unknown-hxnu`

## 2. Validation (Local)

- [ ] `cargo test -p hxnu-target-spec -p hxnu-cargo`
- [ ] `cargo check -p hxnu-rustc -p hxnu-cargo -p hxnu-sdk`
- [ ] `./scripts/release-artifacts.sh`

Expected outcomes:

- `hxnu-sdk bundle build|pack|install` succeeds
- Installed SDK builds both examples (`no_std-hello`, `init-like`)
- ELF verification passes (`ELF64`, `x86-64`, at least one `PT_LOAD`)

## 3. SDK Artifact Contract

- [ ] Bundle root contains `bin/`, `targets/`, `sysroot/`, `examples/`, `docs/`
- [ ] `bin/` includes `hxnu-rustc`, `hxnu-cargo`
- [ ] `targets/` includes `x86_64-unknown-hxnu.json`
- [ ] `sysroot/lib/rustlib/x86_64-unknown-hxnu/lib` includes:
  - [ ] `libcore-*`
  - [ ] `liballoc-*`
  - [ ] `libcompiler_builtins-*`
- [ ] Packaged archive exists at `build/hxnu-rustc-compiler-x86_64-0.1.0.tar.gz`

## 4. Docs and Consumer Contract

- [ ] `README.md` quickstart matches current commands
- [ ] `docs/usage.md` includes bundle + install + example build flow
- [ ] `docs/integration.md` includes HXNU consumer contract and acceptance baseline

## 5. Git Tagging (No Push in Current Flow)

- [ ] Clean working tree
- [ ] Create annotated tag:

```bash
git tag -a v0.1.0 -m "HXNU Rust Compiler SDK v0.1.0"
```

- [ ] Verify:

```bash
git show v0.1.0 --stat
```

Push is intentionally skipped in this workflow.
