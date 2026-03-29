# Examples

- `no_std-hello`: minimal freestanding no_std executable with `_start`.
- `init-like`: placeholder Unix-like `/init` style entry artifact with `_start`.

Build with installed SDK:

```bash
SDK_ROOT=/path/to/hxnu-rustc-compiler-x86_64-0.1.0
"$SDK_ROOT/bin/hxnu-cargo" build --manifest-path "$SDK_ROOT/examples/no_std-hello/Cargo.toml" --release
"$SDK_ROOT/bin/hxnu-cargo" build --manifest-path "$SDK_ROOT/examples/init-like/Cargo.toml" --release
```
