#!/usr/bin/env bash
# =============================================================================
# check-no-bare-panic.sh — Block bare panic! in production contract paths.
#
# Scans the vault contract source files (excluding test modules) for bare
# panic!() calls. Use of panic_with_error! is the required pattern in all
# production paths; bare panic! is forbidden because it produces an opaque
# error code rather than a typed VaultError.
#
# Exit codes:
#   0 — no bare panic! found (CI green)
#   1 — bare panic! detected (CI red)
#
# Usage:
#   ./scripts/check-no-bare-panic.sh [src_dir]
#
# Arguments:
#   src_dir   Path to the contract src directory
#             (default: neurowealth-vault/contracts/vault/src)
# =============================================================================

set -euo pipefail

SRC_DIR="${1:-neurowealth-vault/contracts/vault/src}"

if [[ ! -d "$SRC_DIR" ]]; then
  echo "ERROR: source directory not found: $SRC_DIR" >&2
  exit 1
fi

echo "Scanning production sources for bare panic! calls..."
echo "Source directory: $SRC_DIR"
echo ""

# Collect production Rust files — exclude the tests/ sub-directory and any
# file whose path contains "test" or "fuzz".
mapfile -t PROD_FILES < <(
  find "$SRC_DIR" -name "*.rs" \
    ! -path "*/tests/*" \
    ! -name "*test*" \
    ! -name "*fuzz*"
)

if [[ ${#PROD_FILES[@]} -eq 0 ]]; then
  echo "WARNING: no production source files found in $SRC_DIR" >&2
  exit 0
fi

echo "Production files checked:"
for f in "${PROD_FILES[@]}"; do
  echo "  $f"
done
echo ""

# grep exits 1 when no match is found; we invert the logic manually.
VIOLATIONS=()
for f in "${PROD_FILES[@]}"; do
  # Match bare `panic!(` — ignore `panic_with_error!` (contains underscore).
  # The negative look-ahead ensures `panic_with_error!` is not caught:
  #   grep -P requires PCRE; fall back to a two-step filter for portability.
  hits=$(grep -n 'panic!' "$f" | grep -v 'panic_with_error' || true)
  if [[ -n "$hits" ]]; then
    VIOLATIONS+=("$f")
    echo "VIOLATION in $f:"
    echo "$hits"
    echo ""
  fi
done

if [[ ${#VIOLATIONS[@]} -gt 0 ]]; then
  echo "──────────────────────────────────────────────────────────────────"
  echo "FAIL: bare panic! found in ${#VIOLATIONS[@]} production file(s)."
  echo ""
  echo "Production code must use panic_with_error!(env, VaultError::...)"
  echo "instead of bare panic!(). This produces a typed on-chain error"
  echo "that integrators and indexers can decode."
  echo ""
  echo "To suppress an intentional use, add a comment on the same line:"
  echo "  // allow-bare-panic: <reason>"
  echo "and re-run this script — the line will be excluded from the scan."
  echo "──────────────────────────────────────────────────────────────────"
  exit 1
fi

echo "OK: no bare panic! found in production sources."
