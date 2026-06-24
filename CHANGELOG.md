# Changelog

All notable changes to this repository are documented in this file.
This changelog is tied to the vault contract `Version` storage value. Each released contract upgrade should add a new entry matching the stored version number.

## [Unreleased]
- Add pending contract changes here.
- Include the target `Version` storage value for any upgrade.
- Update the PR description and reviewer notes when contract behavior changes.
- **DEX liquidity pool integration (Issue #228):** the vault can now deploy USDC
  to a Stellar DEX liquidity pool in addition to Blend, implementing the
  on-chain side of the Balanced/Growth strategies.
  - Added owner-configurable `DataKey::DexPool` with `set_dex_pool` / `get_dex_pool`.
  - Added `supply_to_dex` / `withdraw_from_dex` internal helpers mirroring Blend.
  - `rebalance` now accepts the `"dex"` protocol symbol with `min_out` slippage
    protection; `CurrentProtocol` and `ProtocolChangedEvent` reflect DEX deployments.
  - User `withdraw` / `withdraw_all` pull liquidity back from the DEX when needed.
  - New events: `DexSupplyEvent` (`dex_sup`), `DexWithdrawEvent` (`dex_wd`),
    `DexPoolConfiguredEvent` (`dex_cfg`).
  - New errors: `DexPoolNotConfigured` (#46), `OnlyOwnerCanSetDexPool` (#47).
  - New `dex-devnet` test feature flag. No `Version` bump (additive, pre-mainnet).
  - See `docs/DEX_INTEGRATION.md`.

## [1]
- Initial vault implementation with ERC-4626-inspired share accounting.
- `get_version()` returns the contract version from `DataKey::Version`.
- `UpgradedEvent` emits both `old_version` and `new_version` for on-chain auditability.
