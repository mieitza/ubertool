#!/usr/bin/env bash
# Verify ubertool.ocs.yaml is structurally consistent with `ubertool --help`.
# Fails (non-zero exit) if the spec declares any command the binary does not
# accept. This is the design-spec §9 drift check.
set -euo pipefail

SPEC="${SPEC:-ubertool.ocs.yaml}"
BIN="${BIN:-./target/debug/ubertool}"

if [[ ! -f "$SPEC" ]]; then
  echo "spec not found: $SPEC" >&2
  exit 1
fi
if [[ ! -x "$BIN" ]]; then
  echo "binary not found: $BIN (run: cargo build)" >&2
  exit 1
fi

# 1. The spec must validate.
ocli spec check "$SPEC"

# 2. Every leaf command in the spec must be accepted by --help.
if ! command -v yq >/dev/null 2>&1; then
  echo "yq is required (brew install yq)" >&2
  exit 1
fi

# Extract leaf command paths from the spec — exclude group placeholders.
fail=0
count=0
while IFS= read -r path; do
  # Skip empty lines (some yq versions emit them).
  [[ -z "$path" ]] && continue
  count=$((count + 1))

  # Parse the command path into parts
  read -r -a parts <<<"$path"
  if ! "$BIN" "${parts[@]}" --help >/dev/null 2>&1; then
    echo "FAIL: spec declares '$path' but CLI rejects it" >&2
    fail=1
  fi
done < <(
  yq -r '.commands | keys[]' "$SPEC" \
    | grep -v '{command}' \
    | sed -E 's/^ubertool //; s/ \[.*$//; s/ <.*$//'
)

if [[ $fail -eq 0 ]]; then
  echo "verify-spec: ok ($count leaf commands match)"
fi
exit "$fail"
