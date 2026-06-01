# Contract Specification Generation

This directory contains scripts to generate and validate a JSON specification of the NeuroWealth Vault Soroban smart contract. This specification enables frontend and agent clients to understand the contract interface without manually discovering function names, parameter types, and event schemas from Rust source code.

## Overview

The specification generation system consists of:

1. **Generator Script** (`generate-spec.py`) - Parses Rust contract source and generates `contract-spec.json`
2. **Validation Script** (`validate-spec.py`) - Ensures spec completeness and accuracy
3. **CI Workflow** (`.github/workflows/contract-spec.yml`) - Automatically generates and validates spec on commits
4. **JSON Specification** (`contract-spec.json`) - The generated specification for client use

## Quick Start

### Generate Specification

```bash
python3 scripts/generate-spec.py
```

This generates `contract-spec.json` in the repository root with:
- All 45 public contract functions
- All 23 contract events
- Type definitions and error codes
- Constants and parameter descriptions

### Validate Specification

```bash
python3 scripts/validate-spec.py
```

Validates that:
- All contract functions are documented in spec
- All contract events are documented in spec
- Spec structure is valid and complete

### Both Generate and Validate

```bash
python3 scripts/generate-spec.py && python3 scripts/validate-spec.py
```

## Specification Structure

The generated `contract-spec.json` has the following structure:

```json
{
  "version": "1.0.0",
  "contract": "NeuroWealth Vault",
  "network": "Stellar Soroban",
  "description": "ERC-4626 inspired vault contract...",
  "decimals": 7,
  "token": "USDC",
  
  "functions": [
    {
      "name": "deposit",
      "category": "liquidity",
      "access": "public",
      "description": "Deposit USDC into the vault and receive vault shares",
      "parameters": [...],
      "returns": {...},
      "requires_auth": true,
      "state_changing": true,
      "constraints": [...],
      "events": ["DepositEvent"]
    },
    ...
  ],
  
  "events": [
    {
      "name": "DepositEvent",
      "topic": "deposit",
      "description": "Emitted when user deposits USDC",
      "fields": [...]
    },
    ...
  ],
  
  "errors": {
    "VaultError::NegativeMin": {...},
    ...
  },
  
  "types": {
    "UserInfo": {...},
    "Address": {...},
    ...
  },
  
  "constants": {
    "DEFAULT_USER_DEPOSIT_CAP": {...},
    ...
  }
}
```

## Using the Specification

### For Frontend Clients

Use the spec to:
- Validate function calls before submission
- Display human-readable parameter descriptions
- Mock contract responses during development
- Generate TypeScript types and API clients

Example TypeScript generation:

```typescript
import spec from './contract-spec.json';

// Get function signature
const depositFunc = spec.functions.find(f => f.name === 'deposit');

// Display parameters to user
depositFunc.parameters.forEach(param => {
  console.log(`${param.name} (${param.type}): ${param.description}`);
});
```

### For Agent Clients

Use the spec to:
- Understand event schemas for monitoring
- Parse event data correctly
- Construct rebalance transactions with proper parameters
- Validate response data types

Example event monitoring:

```python
import json

spec = json.load(open('contract-spec.json'))

# Get DepositEvent schema
deposit_event = next(e for e in spec['events'] if e['name'] == 'DepositEvent')

# Know exactly what fields to expect
fields = {f['name']: f['type'] for f in deposit_event['fields']}
# Output: {'user': 'Address', 'amount': 'i128', 'shares': 'i128'}
```

### For Documentation

Use the spec to auto-generate API documentation:

```bash
# Generate markdown documentation from spec
python3 -c "
import json
with open('contract-spec.json') as f:
    spec = json.load(f)
print(f'# {spec[\"contract\"]}\n')
print(f'{spec[\"description\"]}\n\n')
print(f'## Functions ({len(spec[\"functions\"])})\n')
for func in spec['functions']:
    print(f'### {func[\"name\"]}')
    print(f'{func.get(\"description\", \"\")}')
"
```

## Function Categories

Functions are organized by category:

- **initialization** - Contract setup (initialize)
- **liquidity** - User deposits/withdrawals (deposit, withdraw)
- **management** - AI agent rebalancing (rebalance, update_total_assets)
- **administration** - Owner configuration (pause, set_caps, transfer_ownership)
- **queries** - Read-only getters (get_balance, get_agent, etc.)

## Access Control

Functions specify their access requirements:

- **public** - Anyone can call
- **owner-only** - Contract owner required
- **agent-only** - Authorized AI agent required
- **once** - Can only be called once (initialize)
- **pending-owner-only** - Pending owner in 2-step transfer

## Event Monitoring

All state-changing operations emit events:

```solidity
// User deposits USDC
DepositEvent {
  user: Address,
  amount: i128,        // USDC amount
  shares: i128         // Vault shares minted
}

// Agent rebalances
RebalanceEvent {
  protocol: Symbol,     // "blend" or "none"
  expected_apy: i128,   // Expected APY
  min_out: i128         // Slippage protection
}

// Total assets updated
AssetsUpdatedEvent {
  old_total: i128,
  new_total: i128
}
```

AI agents should monitor `DepositEvent` to detect new deposits and trigger yield deployment, and monitor `RebalanceEvent` to confirm strategy changes.

## CI/CD Integration

The spec is automatically:

1. **Generated** on every commit that changes contract code
2. **Validated** to ensure spec ↔ implementation consistency
3. **Uploaded** as a build artifact
4. **Committed** back to the repo (on main branch)
5. **Posted** as a PR comment (on pull requests)

### CI Workflow

The workflow runs on:

- **Push** to `main` or `develop` branches when contract files change
- **Pull Requests** to `main` or `develop` when spec changes
- **Manual trigger** via `workflow_dispatch`

To manually trigger:

```bash
gh workflow run contract-spec.yml --ref main
```

## Updating the Specification

The specification is automatically generated from the contract source code. To update it:

1. **Modify the contract** in `neurowealth-vault/contracts/vault/src/lib.rs`
2. **Run generation**:
   ```bash
   python3 scripts/generate-spec.py
   ```
3. **Validate**:
   ```bash
   python3 scripts/validate-spec.py
   ```
4. **Commit** the updated `contract-spec.json`

The spec should always be kept in sync with the contract implementation. The validation script ensures this.

## Schema Reference

### Function Definition

```json
{
  "name": "string",                    // Function name
  "category": "string",                // Function category
  "access": "string",                  // Access level (public, owner-only, etc.)
  "description": "string",             // Human-readable description
  "parameters": [                      // Function parameters
    {
      "name": "string",               // Parameter name
      "type": "string",               // Parameter type (Address, i128, etc.)
      "description": "string"         // Parameter description
    }
  ],
  "returns": "string | object",        // Return type or {type, description}
  "requires_auth": boolean,            // Requires signature authorization
  "state_changing": boolean,           // Modifies contract state
  "constraints": ["string"],           // Validation constraints
  "events": ["string"],                // Events emitted
  "deprecated": boolean                // Whether function is deprecated
}
```

### Event Definition

```json
{
  "name": "string",                    // Event name
  "topic": "string",                   // Event topic for filtering
  "description": "string",             // Event description
  "fields": [                          // Event fields
    {
      "name": "string",               // Field name
      "type": "string",               // Field type
      "indexed": boolean,             // Indexed for filtering
      "description": "string"         // Field description
    }
  ]
}
```

## Common Tasks

### List All Functions

```bash
jq '.functions[].name' contract-spec.json
```

### List Agent-Only Functions

```bash
jq '.functions[] | select(.access == "agent-only") | .name' contract-spec.json
```

### Get Function Parameter Types

```bash
jq '.functions[] | select(.name == "deposit") | .parameters' contract-spec.json
```

### Find Events Emitted by Function

```bash
jq '.functions[] | select(.name == "deposit") | .events' contract-spec.json
```

### List All Error Codes

```bash
jq '.errors | to_entries[] | "\(.key): \(.value.code) - \(.value.description)"' contract-spec.json
```

## Troubleshooting

### Validation Fails: "Function X in contract but not in spec"

The contract has a new function that the spec generator doesn't know about. Update `scripts/generate-spec.py` to include it in the appropriate function list.

### Validation Fails: "Event X defined in contract but not documented"

A new event was added to the contract. Add it to the `_get_events()` method in `scripts/generate-spec.py`.

### Spec Doesn't Update After Contract Changes

Run the generator manually:
```bash
python3 scripts/generate-spec.py
```

The CI workflow may not have been triggered due to file path filters.

## Support

For issues with spec generation or validation:

1. Check that Python 3.7+ is installed
2. Verify contract file exists at `neurowealth-vault/contracts/vault/src/lib.rs`
3. Run validation with verbose output: `python3 scripts/validate-spec.py`
4. Check GitHub Actions logs for CI failures

## License

The specification generation scripts are part of NeuroWealth and use the same license as the main project.
