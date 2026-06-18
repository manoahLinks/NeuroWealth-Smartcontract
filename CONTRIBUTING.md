# Contributing to NeuroWealth

Thank you for your interest in contributing to NeuroWealth! We welcome contributions from everyone.

This guide will help you get started with our development process, issue labeling, and coding standards.

## Table of Contents
- [Good First Issues](#good-first-issues)
- [Reporting Issues](#reporting-issues)
- [Development Setup](#development-setup)
  - [Prerequisites](#prerequisites)
  - [Building the Contract](#building-the-contract)
  - [Running Tests](#running-tests)
- [CI Requirements](#ci-requirements)
- [Coding Standards](#coding-standards)
- [Submitting a Pull Request](#submitting-a-pull-request)

## Good First Issues

If you're new to the project, a great place to start is our [good first issues](https://github.com/NeuroWealth/NeuroWealth-Smartcontract/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22). These are typically smaller tasks that help you get familiar with the codebase.

## Reporting Issues

We use standardized issue templates to ensure that bug reports and feature requests contain all the necessary information for the team to respond effectively.

### Bug Reports

Use the [bug report template](/.github/ISSUE_TEMPLATE/bug_report.md) when:
- You've found a defect in the smart contract
- You've encountered unexpected behavior
- You have a reproducible test case

The bug report template will guide you to provide:
- Soroban network context (devnet, testnet, or mainnet)
- Contract ID and affected function
- Steps to reproduce the issue
- Expected vs. actual behavior
- Test command output and environment details
- Verification checklist to ensure completeness

### Feature Requests

Use the [feature request template](/.github/ISSUE_TEMPLATE/feature_request.md) when:
- You want to propose a new capability
- You want to suggest an enhancement
- You have an idea for improving the project

The feature request template will guide you to provide:
- Clear feature description
- Use case and motivation
- Proposed solution and alternative approaches
- Security and network-specific considerations
- Architecture alignment
- Related issues or pull requests

### Security Issues

**For security-related issues**, please follow the [Security Policy](SECURITY.md) and report via GitHub's security advisory system instead of creating a public issue. Do not disclose security vulnerabilities publicly.

### Issue Labels

We use the following labels to categorize issues:

- `bug`: Something isn't working as expected
- `enhancement`: New feature or request
- `documentation`: Improvements or additions to documentation
- `good first issue`: Good for newcomers
- `security`: Security-related issues or improvements
- `help wanted`: Extra attention needed

## Development Setup

### Prerequisites

To contribute to the smart contracts, you'll need the following installed:

- **Rust**: Latest stable version. [Install Rust](https://rustup.rs/)
- **WASM Target**: `rustup target add wasm32-unknown-unknown`
- **Stellar CLI**: We recommend version **21.2.0** or later.
  ```bash
  cargo install --locked stellar-cli --version 21.2.0
  ```
- **Node.js & npm**: For agent and frontend development (LTS version recommended).

### Building the Contract

Navigate to the contract directory and use the Stellar CLI to build:

```bash
cd neurowealth-vault
stellar contract build
```

### Running Tests

We prioritize high test coverage. Always run the test suite before submitting a PR:

```bash
cd neurowealth-vault
cargo test
```

For frontend or agent changes, run:
```bash
npm test
```

## CI Requirements

Our CI pipeline (defined in `.github/workflows/ci.yml`) runs on every push and pull request. For a PR to be merged, it must pass:

1. **Format Check**: `cargo fmt --all -- --check`
2. **Clippy Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Tests**: `cargo test --verbose`
4. **Build WASM**: Successful build of the contract WASM.
5. **Dependency Audit**: `cargo-deny check` runs against the policy in [`deny.toml`](deny.toml), blocking dependencies with disallowed licenses or known security advisories.

### Dependency Audit & Advisory Exceptions

The dependency audit ([`EmbarkStudios/cargo-deny-action`](https://github.com/EmbarkStudios/cargo-deny-action)) enforces the license, advisory, and source policies defined in [`deny.toml`](deny.toml). To reproduce it locally:

```bash
cargo install --locked cargo-deny
cargo deny --manifest-path neurowealth-vault/Cargo.toml check --config deny.toml
```

Occasionally a published advisory cannot be resolved immediately (for example, the upstream fix has not yet been released). In that case you can request a **time-limited exception** rather than disabling the check:

1. Open a tracking issue describing the advisory and the remediation plan.
2. Open a PR that adds an `[[advisories.ignore]]` entry to [`deny.toml`](deny.toml). Each entry must include the RustSec `id`, the affected `crate`, and a `reason` that justifies the exception and links to the tracking issue. See the commented example and process notes in the *Exceptions* section of [`deny.toml`](deny.toml).
3. Exceptions are reviewed and re-evaluated every sprint, and should be removed as soon as an upstream fix is available.

License exceptions follow the same PR-based process but additionally require legal sign-off before the `[[licenses.exceptions]]` entry is added.

## Coding Standards

- **Error Messages**: All error messages must follow the [Error Message Style Guide](ERROR_STYLE_GUIDE.md).
- **Architecture**: Ensure changes align with the project [Architecture Documentation](ARCHITECTURE.md).
- **Events**: Every state change should emit a corresponding event as defined in [EVENTS.md](EVENTS.md).
- **Safety**: Always use `checked_*` arithmetic operations for financial calculations.

## Submitting a Pull Request

1. **Fork the repository** and create your branch from `develop`.
2. **Make your changes**, ensuring you add or update tests.
3. **Verify locally** that all tests pass and there are no linting errors.
4. **Commit your changes** with a clear and descriptive message.
5. **Push to your fork** and open a Pull Request against the `develop` branch.
6. **Provide a detailed description** in the PR of what you changed and why.

---

By contributing, you agree that your contributions will be licensed under the project's license.
