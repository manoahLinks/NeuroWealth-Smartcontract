# Changelog

All notable changes to this repository are documented in this file.
This changelog is tied to the vault contract `Version` storage value. Each released contract upgrade should add a new entry matching the stored version number.

## [Unreleased]
- Add pending contract changes here.
- Include the target `Version` storage value for any upgrade.
- Update the PR description and reviewer notes when contract behavior changes.

## [1]
- Initial vault implementation with ERC-4626-inspired share accounting.
- `get_version()` returns the contract version from `DataKey::Version`.
- `UpgradedEvent` emits both `old_version` and `new_version` for on-chain auditability.
