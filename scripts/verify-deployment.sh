#!/usr/bin/env bash
# =============================================================================
# NeuroWealth Vault — Post-Deploy Verification (Smoke Checks)
# =============================================================================
#
# Reads scripts/devnet-contracts.env (written by deploy-devnet.sh) and verifies
# that the vault is initialized with the expected agent, USDC token, and pause
# state via read-only contract invocations.
#
# Usage:
#   ./scripts/verify-deployment.sh [--help] [ENV_FILE]
#
# Arguments:
#   ENV_FILE   Path to contract env file (default: scripts/devnet-contracts.env)
#
# Environment:
#   Variables are loaded from ENV_FILE. Required keys:
#     VAULT_CONTRACT_ID, USDC_TOKEN_ADDRESS, AGENT_ADDRESS, AGENT_SECRET_KEY
#     SOROBAN_RPC_URL, SOROBAN_NETWORK_PASSPHRASE
#
# Exit codes:
#   0  — All checks passed
#   1  — One or more checks failed
#   2  — Invalid usage or missing configuration
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_ENV_FILE="$SCRIPT_DIR/devnet-contracts.env"

ENV_FILE="${1:-$DEFAULT_ENV_FILE}"
FAILURES=0

timestamp() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }
log() { echo "[$(timestamp)] $*"; }

show_help() {
  cat << EOF
NeuroWealth Vault — Post-Deploy Verification

USAGE:
    $0 [--help] [ENV_FILE]

ARGUMENTS:
    ENV_FILE    Contract env file (default: $DEFAULT_ENV_FILE)

EXAMPLE:
    ./scripts/deploy-devnet.sh
    ./scripts/verify-deployment.sh
EOF
}

fail() {
  log "FAIL: $*"
  FAILURES=$((FAILURES + 1))
}

pass() {
  log "PASS: $*"
}

normalize_address() {
  # stellar CLI may return JSON strings or bare addresses
  local raw="$1"
  raw="${raw//\"/}"
  raw="${raw//$'\n'/}"
  raw="${raw//[[:space:]]/}"
  echo "$raw"
}

invoke_view() {
  local description="$1"
  shift
  log "  invoke: $description"
  local output
  if ! output=$(stellar contract invoke \
    --id "$VAULT_CONTRACT_ID" \
    --source-account "$AGENT_SECRET_KEY" \
    --network "$SOROBAN_NETWORK_PASSPHRASE" \
    --rpc-url "$SOROBAN_RPC_URL" \
    --send=no \
    -- "$@" 2>&1); then
    log "  error output: $output"
    return 1
  fi
  echo "$output"
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  show_help
  exit 0
fi

if [[ ! -f "$ENV_FILE" ]]; then
  log "ERROR: env file not found: $ENV_FILE"
  log "Run ./scripts/deploy-devnet.sh first or pass a valid ENV_FILE path."
  exit 2
fi

# shellcheck disable=SC1090
source "$ENV_FILE"

for var in VAULT_CONTRACT_ID USDC_TOKEN_ADDRESS AGENT_ADDRESS AGENT_SECRET_KEY \
           SOROBAN_RPC_URL SOROBAN_NETWORK_PASSPHRASE; do
  if [[ -z "${!var:-}" ]]; then
    log "ERROR: missing required variable in $ENV_FILE: $var"
    exit 2
  fi
done

if ! command -v stellar &> /dev/null; then
  log "ERROR: stellar CLI not found"
  exit 2
fi

log "Verifying deployment using $ENV_FILE"
log "  Vault:  $VAULT_CONTRACT_ID"
log "  Token:  $USDC_TOKEN_ADDRESS"
log "  Agent:  $AGENT_ADDRESS"

# ---------------------------------------------------------------------------
# Initialization / agent
# ---------------------------------------------------------------------------

AGENT_OUTPUT=$(invoke_view "get_agent" get_agent) || {
  fail "get_agent invocation failed (vault may not be initialized)"
  AGENT_OUTPUT=""
}

if [[ -n "$AGENT_OUTPUT" ]]; then
  ON_CHAIN_AGENT=$(normalize_address "$AGENT_OUTPUT")
  EXPECTED_AGENT=$(normalize_address "$AGENT_ADDRESS")
  if [[ "$ON_CHAIN_AGENT" == "$EXPECTED_AGENT" ]]; then
    pass "get_agent matches AGENT_ADDRESS"
  else
    fail "get_agent mismatch: on-chain=$ON_CHAIN_AGENT expected=$EXPECTED_AGENT"
  fi
fi

# ---------------------------------------------------------------------------
# USDC token
# ---------------------------------------------------------------------------

TOKEN_OUTPUT=$(invoke_view "get_usdc_token" get_usdc_token) || {
  fail "get_usdc_token invocation failed"
  TOKEN_OUTPUT=""
}

if [[ -n "$TOKEN_OUTPUT" ]]; then
  ON_CHAIN_TOKEN=$(normalize_address "$TOKEN_OUTPUT")
  EXPECTED_TOKEN=$(normalize_address "$USDC_TOKEN_ADDRESS")
  if [[ "$ON_CHAIN_TOKEN" == "$EXPECTED_TOKEN" ]]; then
    pass "get_usdc_token matches USDC_TOKEN_ADDRESS"
  else
    fail "get_usdc_token mismatch: on-chain=$ON_CHAIN_TOKEN expected=$EXPECTED_TOKEN"
  fi
fi

# ---------------------------------------------------------------------------
# Pause flag
# ---------------------------------------------------------------------------

PAUSED_OUTPUT=$(invoke_view "is_paused" is_paused) || {
  fail "is_paused invocation failed"
  PAUSED_OUTPUT=""
}

if [[ -n "$PAUSED_OUTPUT" ]]; then
  PAUSED_NORM=$(echo "$PAUSED_OUTPUT" | tr -d '[:space:]"')
  if [[ "$PAUSED_NORM" == "false" || "$PAUSED_NORM" == "False" ]]; then
    pass "is_paused is false (vault operational)"
  else
    fail "is_paused expected false, got: $PAUSED_OUTPUT"
  fi
fi

# ---------------------------------------------------------------------------
# Init state sanity (version + zero TVL at fresh deploy)
# ---------------------------------------------------------------------------

VERSION_OUTPUT=$(invoke_view "get_version" get_version) || {
  fail "get_version invocation failed"
  VERSION_OUTPUT=""
}

if [[ -n "$VERSION_OUTPUT" ]]; then
  VERSION_NORM=$(echo "$VERSION_OUTPUT" | tr -d '[:space:]"')
  if [[ "$VERSION_NORM" =~ ^[1-9][0-9]*$ ]]; then
    pass "get_version returned $VERSION_NORM (initialized)"
  else
    fail "get_version unexpected value: $VERSION_OUTPUT"
  fi
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

echo ""
if [[ "$FAILURES" -eq 0 ]]; then
  log "All deployment smoke checks passed."
  exit 0
else
  log "$FAILURES check(s) failed."
  exit 1
fi
