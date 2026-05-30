# Contributing to NeuroWealth

Thank you for your interest in contributing to NeuroWealth! We welcome contributions from everyone.

This guide will help you get started with our development process, issue labeling, and coding standards.

## Table of Contents
- [Good First Issues](#good-first-issues)
- [Development Setup](#development-setup)
  - [Prerequisites](#prerequisites)
  - [Building the Contract](#building-the-contract)
  - [Running Tests](#running-tests)
- [Issue Labeling](#issue-labeling)
- [CI Requirements](#ci-requirements)
- [Coding Standards](#coding-standards)
- [Submitting a Pull Request](#submitting-a-pull-request)

## Good First Issues

If you're new to the project, a great place to start is our [good first issues](https://github.com/NeuroWealth/NeuroWealth-Smartcontract/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22). These are typically smaller tasks that help you get familiar with the codebase.

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

## Issue Labeling

We use the following labels to categorize issues:

- `bug`: Something isn't working as expected.
- `enhancement`: New feature or request.
- `documentation`: Improvements or additions to documentation.
- `good first issue`: Good for newcomers.
- `security`: Security-related issues or improvements.
- `help wanted`: Extra attention needed.

## CI Requirements

Our CI pipeline (defined in `.github/workflows/ci.yml`) runs on every push and pull request. For a PR to be merged, it must pass:

1. **Format Check**: `cargo fmt --all -- --check`
2. **Clippy Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Tests**: `cargo test --verbose`
4. **Build WASM**: Successful build of the contract WASM.

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
