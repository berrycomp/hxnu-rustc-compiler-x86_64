#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 3 ]]; then
  echo "usage: $0 <artifact> [machine-substring] [endian-substring]" >&2
  exit 1
fi

ARTIFACT="$1"
EXPECTED_MACHINE="${2:-x86-64}"
EXPECTED_ENDIAN="${3:-LSB}"
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
if [[ "$FILE_OUTPUT" != *"$EXPECTED_ENDIAN"* ]]; then
  echo "artifact endian mismatch ($EXPECTED_ENDIAN): $FILE_OUTPUT" >&2
  exit 1
fi
if [[ "$FILE_OUTPUT" != *"$EXPECTED_MACHINE"* ]]; then
  echo "artifact machine mismatch ($EXPECTED_MACHINE): $FILE_OUTPUT" >&2
  exit 1
fi

PROGRAM_HEADERS="$(objdump -p "$ARTIFACT")"
if ! grep -qE '^[[:space:]]+LOAD[[:space:]]' <<<"$PROGRAM_HEADERS"; then
  echo "artifact does not contain PT_LOAD program headers" >&2
  exit 1
fi

echo "verified ELF artifact: $ARTIFACT ($EXPECTED_MACHINE, $EXPECTED_ENDIAN)"
