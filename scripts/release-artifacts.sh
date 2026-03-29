#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

cargo run -p hxnu-sdk --manifest-path "$ROOT/Cargo.toml" -- bundle build
cargo run -p hxnu-sdk --manifest-path "$ROOT/Cargo.toml" -- bundle pack
