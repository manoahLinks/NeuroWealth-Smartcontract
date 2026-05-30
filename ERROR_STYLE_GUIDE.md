# NeuroWealth Vault Error Message Style Guide

This document defines the standardized format and wording for all error messages in the NeuroWealth Vault contract.

## Error Message Philosophy

Error messages should be:
- **Clear**: Users and developers should understand what went wrong
- **Consistent**: Same concepts use the same wording throughout
- **Concise**: Short but descriptive, avoiding unnecessary words
- **Actionable**: When possible, indicate what can be done to fix the issue

## Message Format

All error messages follow this pattern:
```
vault: <category> <description>
```

Where:
- `vault:` - Prefix indicating the error source
- `<category>` - Error category (e.g., "amount", "auth", "state")
- `<description>` - Brief, specific description

## Error Categories

### Amount Errors
Format: `vault: amount <description>`

Used for:
- Invalid amounts (zero, negative)
- Amounts exceeding limits
- Insufficient balances

Examples:
- `vault: amount must be positive`
- `vault: amount exceeds maximum`
- `vault: insufficient balance`

### Authorization Errors
Format: `vault: <role> <description>`

Used for:
- Missing permissions
- Incorrect caller role
- Unauthorized access attempts

Examples:
- `vault: only owner can pause`
- `vault: only agent can rebalance`
- `vault: caller is not the owner`

### State Errors
Format: `vault: <state> <description>`

Used for:
- Contract state violations
- Invalid operation sequences
- Configuration issues

Examples:
- `vault: already initialized`
- `vault: paused`
- `vault: not initialized.`

### Validation Errors
Format: `vault: <field> <description>`

Used for:
- Input validation failures
- Parameter constraints
- Business rule violations

Examples:
- `vault: below minimum deposit`
- `vault: exceeds TVL cap`
- `vault: insufficient shares`

### Protocol Errors
Format: `vault: <protocol> <description>`

Used for:
- External protocol failures
- Integration issues
- Protocol-specific errors

Examples:
- `vault: blend pool not configured`
- `vault: unsupported protocol`
- `vault: protocol call failed`

## Standardized Messages

### Core Operations

#### Deposit Errors
- `vault: amount must be positive`
- `vault: below minimum deposit`
- `vault: maximum deposit exceeded`
- `vault: exceeds user deposit cap`
- `vault: exceeds TVL cap`
- `vault: paused`

#### Withdraw Errors
- `vault: amount must be positive`
- `vault: insufficient shares`
- `vault: insufficient liquidity`
- `vault: paused`

#### Share/Asset Conversions
- `vault: shares to mint must be positive`
- `vault: shares to burn must be positive`
- `vault: insufficient shares for requested amount`

### Administrative Errors

#### Pause/Unpause
- `vault: only owner can pause`
- `vault: only owner can unpause`
- `vault: only agent can emergency pause`

#### Ownership Transfer
- `vault: caller is not the owner`
- `vault: no pending owner`
- `vault: no pending owner to cancel`

#### Configuration
- `vault: already initialized`
- `vault: minimum deposit must be at least 1 USDC`
- `vault: maximum deposit must be >= minimum`

### Protocol Integration

#### Blend Protocol
- `vault: blend pool not configured`
- `vault: unsupported protocol`

## Wording Guidelines

### Use Simple Language
- ✅ `vault: amount must be positive`
- ❌ `vault: the provided amount is not greater than zero`

### Be Specific but Concise
- ✅ `vault: only owner can pause`
- ❌ `vault: pause operation restricted to owner role only`

### Use Consistent Terminology
- Always use "owner" for contract owner
- Always use "agent" for AI agent
- Always use "user" for regular users
- Use "deposit" and "withdraw" consistently
- Use "assets" and "shares" consistently

### Avoid Technical Jargon
- ✅ `vault: insufficient balance`
- ❌ `vault: account has insufficient token balance`

### Include Context When Helpful
- ✅ `vault: below minimum deposit`
- ❌ `vault: invalid amount`

## Current Error Messages Audit

### Messages That Follow Standard
- `vault: already initialized` ✅
- `vault: paused` ✅
- `vault: only owner can pause` ✅
- `vault: only owner can unpause` ✅
- `vault: only agent can emergency pause` ✅
- `vault: caller is not the owner` ✅
- `vault: only agent can update total assets` ✅
- `vault: blend pool not configured` ✅
- `vault: unsupported protocol` ✅

### Messages That Need Updates
- `vault: minimum deposit must be at least 1 USDC` → `vault: minimum deposit too low`
- `vault: maximum deposit must be >= minimum` → `vault: maximum deposit below minimum`
- `vault: exceeds TVL cap` ✅ (already follows standard)
- `vault: exceeds user deposit cap` ✅ (already follows standard)

## Implementation Guidelines

### When Adding New Errors

1. Check if an existing message covers the case
2. Choose the appropriate category
3. Follow the format: `vault: <category> <description>`
4. Keep it under 50 characters when possible
5. Update tests to use the exact message

### When Updating Existing Errors

1. Update the error message in the contract
2. Update all test expectations
3. Update this documentation
4. Consider backward compatibility for external integrators

## Testing Requirements

All error messages must have corresponding tests that:
1. Trigger the error condition
2. Verify the exact error message
3. Use `#[should_panic(expected = "...")]` attribute

Example:
```rust
#[test]
#[should_panic(expected = "vault: amount must be positive")]
fn test_deposit_zero_amount_panics() {
    // Test code that triggers the error
}
```

## Migration Process

When updating error messages:
1. Update the contract code
2. Update all affected tests
3. Update this documentation
4. Communicate changes to external integrators
5. Consider deprecation period for breaking changes

## Review Checklist

Before finalizing error messages:
- [ ] Follows `vault: <category> <description>` format
- [ ] Uses consistent terminology
- [ ] Is clear and actionable
- [ ] Has corresponding tests
- [ ] Is documented in this guide
- [ ] Is under 50 characters when possible

This style guide ensures that all error messages in the NeuroWealth Vault are consistent, clear, and maintainable.
