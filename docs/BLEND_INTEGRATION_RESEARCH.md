# Blend Protocol Integration Research

## Overview

This document contains research findings for integrating the NeuroWealth Vault with Blend Protocol's Soroban pool contract for on-chain yield generation.

## Production Soroban Interface (Blend v2)

The vault integrates via **request-based** fund management (not legacy `deposit`/`redeem` names):

| Entrypoint | Purpose |
|------------|---------|
| `submit_with_allowance(from, spender, to, requests)` | Supply assets (request type `0`) after USDC `approve` |
| `submit(from, to, requests)` | Withdraw assets (request type `1`) |
| `balance(asset, user)` | Supplied balance for the vault position |

Request struct (contract-local mirror):

```rust
struct BlendRequest {
    request_type: u32,  // 0 = supply, 1 = withdraw
    address: Address,   // USDC token
    amount: i128,
}
```

Implementation: `BlendPoolClient` in `neurowealth-vault/contracts/vault/src/lib.rs`.

References:

- https://docs.blend.capital/tech-docs/core-contracts/lending-pool/fund-management
- https://github.com/blend-capital/blend-contracts-v2

## Cross-Contract Call Pattern

```rust
env.invoke_contract::<Val>(
    &pool_address,
    &Symbol::new(env, "submit_with_allowance"),
    args,
);
```

Supply flow:

1. Vault `approve`s the Blend pool for the supply amount.
2. Vault calls `submit_with_allowance` with a type-0 request.
3. Blend pulls USDC via `transfer_from` (authorized sub-invocation).

Withdraw flow:

1. Vault calls `submit` with a type-1 request.
2. Blend transfers USDC back to the vault.

## Testing

| Layer | Command |
|-------|---------|
| Unit / mock pool | `cargo test -p neurowealth-vault` |
| Blend interface (feature) | `cargo test -p neurowealth-vault --features blend-devnet` |

Manual devnet smoke (replace addresses):

```bash
soroban contract invoke --id "$BLEND_POOL" --network testnet -- balance \
  --asset "$USDC" --user "$VAULT"
```

## Protocol Tracking

`DataKey::CurrentProtocol`:

- `"none"`: Funds not deployed (or idle in vault only)
- `"blend"`: Funds deployed to Blend

`ProtocolChangedEvent` (`proto_chg`) is emitted whenever `CurrentProtocol` changes.

## Rebalance API (agent)

```rust
pub fn rebalance(env: Env, protocol: Symbol, expected_apy: i128, min_out: i128);
```

- `min_out`: minimum assets received per supply/withdraw leg; `0` disables slippage checks.
- `RebalanceEvent.status == "noop"`: no funds moved (e.g. already in Blend with zero idle USDC).

## Security Considerations

1. **Reentrancy**: Blend calls follow state updates where applicable (CEI on protocol transitions).
2. **Incomplete exit**: Rebalance aborts if a protocol switch cannot withdraw the full deployed balance.
3. **Slippage**: Optional `min_out` guard on supply/withdraw legs.

## Status

1. ✅ Research Blend interface (this document)
2. ✅ Implement `BlendPoolClient` with production entrypoints
3. ✅ `ProtocolChangedEvent` for indexers
4. ✅ Rebalance `min_out` slippage guard
5. ✅ No-op rebalance semantics (`status: "noop"`)
6. ⏳ Measure gas on testnet
7. ⏳ Security review of cross-contract call patterns

## References

- Blend GitHub: https://github.com/blend-capital
- Blend Documentation: https://docs.blend.capital
- Soroban SDK Documentation: https://soroban.stellar.org/docs
