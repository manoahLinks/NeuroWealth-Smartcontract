# CI/CD: Soroban Smart Contract Pipeline

**As a** Developer,  
**I want** to establish an automated Continuous Integration (CI) pipeline for our Soroban smart contracts so that all incoming code is properly formatted, passes all tests, and successfully compiles to WebAssembly (Wasm) before it can be merged.

## Description

We need to create a GitHub Actions workflow to ensure high code quality, prevent broken contracts from reaching production, and reduce the manual review burden for maintainers.

**Context:** The Rust workspace for the Soroban vault contract is located at `neurowealth-vault/` (workspace root). All `cargo` commands in the workflow should be run from that directory (e.g. set `working-directory: neurowealth-vault` for the job, or `cd neurowealth-vault` before each step).

## ✅ Requirements

- Create a new GitHub Actions workflow file at `.github/workflows/rust-ci.yml`.
- The workflow should trigger on:
  - Pushes to the `main` and `develop` branches.
  - All Pull Requests (opened, synchronized, reopened).
- The workflow job must run on `ubuntu-latest` and perform the following sequential steps:
  1. **Checkout** the repository.
  2. **Setup the Rust toolchain** (must include `rustfmt`, `clippy`, and the `wasm32-unknown-unknown` target).
  3. **Setup Rust caching** (to speed up subsequent action runs).
  4. **Enforce formatting** (`cargo fmt`).
  5. **Enforce strict linting** (`cargo clippy` with warnings treated as errors).
  6. **Run unit tests** (`cargo test`).
  7. **Verify Wasm compilation** (`cargo build --target wasm32-unknown-unknown --release`).

## 🎯 Acceptance Criteria

- [ ] Workflow file exists at `.github/workflows/rust-ci.yml`.
- [ ] Workflow successfully triggers on PRs (and on pushes to `main` and `develop`).
- [ ] Pipeline **fails** if the code is not formatted correctly (`cargo fmt --all -- --check`).
- [ ] Pipeline **fails** if there are any Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- [ ] Pipeline **passes** only when all tests pass and the code compiles to Wasm.

## Notes

- Use the [actions-rs/toolchain](https://github.com/actions-rs/toolchain) or the official [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) (or `rustup` in a setup step) to install Rust with `rustfmt`, `clippy`, and `wasm32-unknown-unknown`.
- Use [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) or [actions-rs/cache](https://github.com/actions-rs/cache) for caching the `target` directory and cargo registry.
- Ensure all steps run from the `neurowealth-vault` directory so that the workspace and vault contract are built and tested correctly.
