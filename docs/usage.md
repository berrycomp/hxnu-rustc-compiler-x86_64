# Usage

## Bootstrap

```bash
cargo build -p hxnu-rustc -p hxnu-cargo -p hxnu-sdk
```

## Build SDK Bundle

```bash
cargo run -p hxnu-sdk -- bundle build
cargo run -p hxnu-sdk -- bundle pack
cargo run -p hxnu-sdk -- bundle install --prefix /tmp/hxnu-sdk
```

## Consume from HXNU kernel repo

- Add `<prefix>/bin` to `PATH`.
- Use `hxnu-cargo` instead of `cargo`.
- Keep kernel repository unchanged.
