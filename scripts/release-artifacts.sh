#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERIFY_ELF="$ROOT/scripts/verify-elf.sh"
TEMP_PREFIX="$(mktemp -d /tmp/hxnu-sdk-release.XXXXXX)"

cleanup() {
  rm -rf "$TEMP_PREFIX"
}
trap cleanup EXIT

cargo test -p hxnu-target-spec --manifest-path "$ROOT/Cargo.toml"
cargo test -p hxnu-cargo --manifest-path "$ROOT/Cargo.toml"
cargo check -p hxnu-rustc -p hxnu-sdk --manifest-path "$ROOT/Cargo.toml"

cargo run -p hxnu-sdk --manifest-path "$ROOT/Cargo.toml" -- bundle build
cargo run -p hxnu-sdk --manifest-path "$ROOT/Cargo.toml" -- bundle pack
cargo run -p hxnu-sdk --manifest-path "$ROOT/Cargo.toml" -- bundle install --prefix "$TEMP_PREFIX"

SDK_ROOT="$(find "$TEMP_PREFIX" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
if [[ -z "$SDK_ROOT" ]]; then
  echo "failed to locate installed sdk root under $TEMP_PREFIX" >&2
  exit 1
fi

NO_STD_TARGET_DIR="$TEMP_PREFIX/no-std-build"
INIT_TARGET_DIR="$TEMP_PREFIX/init-build"

TARGETS=(
  "x86_64-unknown-hxnu|x86-64|LSB"
  "aarch64-unknown-hxnu|aarch64|LSB"
  "powerpc64le-unknown-hxnu|PowerPC|LSB"
  "powerpc64-unknown-hxnu|PowerPC|MSB"
)

for spec in "${TARGETS[@]}"; do
  IFS='|' read -r TARGET_TRIPLE EXPECTED_MACHINE EXPECTED_ENDIAN <<<"$spec"

  "$SDK_ROOT/bin/hxnu-cargo" build \
    --manifest-path "$SDK_ROOT/examples/no_std-hello/Cargo.toml" \
    --release \
    --target "$TARGET_TRIPLE" \
    --target-dir "$NO_STD_TARGET_DIR"
  "$VERIFY_ELF" \
    "$NO_STD_TARGET_DIR/$TARGET_TRIPLE/release/no-std-hello" \
    "$EXPECTED_MACHINE" \
    "$EXPECTED_ENDIAN"

  "$SDK_ROOT/bin/hxnu-cargo" build \
    --manifest-path "$SDK_ROOT/examples/init-like/Cargo.toml" \
    --release \
    --target "$TARGET_TRIPLE" \
    --target-dir "$INIT_TARGET_DIR"
  "$VERIFY_ELF" \
    "$INIT_TARGET_DIR/$TARGET_TRIPLE/release/init-like" \
    "$EXPECTED_MACHINE" \
    "$EXPECTED_ENDIAN"
done

echo "release artifact flow completed successfully"
