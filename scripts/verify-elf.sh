#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <artifact>" >&2
  exit 1
fi

ARTIFACT="$1"
if [[ ! -f "$ARTIFACT" ]]; then
  echo "artifact not found: $ARTIFACT" >&2
  exit 1
fi

if ! command -v file >/dev/null 2>&1; then
  echo "required command is missing: file" >&2
  exit 1
fi
if ! command -v objdump >/dev/null 2>&1; then
  echo "required command is missing: objdump" >&2
  exit 1
fi

FILE_OUTPUT="$(file "$ARTIFACT")"
if [[ "$FILE_OUTPUT" != *"ELF 64-bit"* ]]; then
  echo "artifact is not ELF64: $FILE_OUTPUT" >&2
  exit 1
fi
if [[ "$FILE_OUTPUT" != *"x86-64"* ]]; then
  echo "artifact machine is not x86-64: $FILE_OUTPUT" >&2
  exit 1
fi

PROGRAM_HEADERS="$(objdump -p "$ARTIFACT")"
if ! grep -qE '^[[:space:]]+LOAD[[:space:]]' <<<"$PROGRAM_HEADERS"; then
  echo "artifact does not contain PT_LOAD program headers" >&2
  exit 1
fi

echo "verified ELF artifact: $ARTIFACT"
