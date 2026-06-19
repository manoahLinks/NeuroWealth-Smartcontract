## Summary

Closes #225 — adds regression tests proving non-owners cannot call `set_deposit_limits`, `set_tvl_cap`, and `set_limits`.
Closes #224 — enforces an optimised WASM size limit in CI and fixes incorrect NEAR references in `docs/WASM_SIZE.md`.

---

### Issue #225 — Access-control tests for `set_deposit_limits`

- **`test_non_owner_cannot_set_deposit_limits`** — required test from the issue; verifies `set_deposit_limits` rejects calls without owner auth.
- **`test_non_owner_cannot_set_tvl_cap`** — parity test; verifies `set_tvl_cap` rejects calls without owner auth.
- **`test_non_owner_cannot_set_limits`** — parity test; verifies `set_limits` rejects calls without owner auth.

#### Why a different pattern from the existing non-owner tests

The existing negative tests (`test_non_owner_cannot_pause`, `test_non_owner_cannot_upgrade`, etc.) work by passing a freshly-generated address to a function that takes an explicit owner parameter and does an identity comparison.

`set_deposit_limits`, `set_tvl_cap`, and `set_limits` have no such parameter — ownership is enforced via `require_is_owner`, which fetches the stored owner address and calls `owner.require_auth()`. The only way to test rejection is to ensure the stored owner's auth is absent at call time.

The tests follow the stricter pattern already established in `test_auth.rs`:
1. Call `env.mock_all_auths()` during setup (initialization is incidental setup noise, not the behavior under test).
2. Call `env.mock_auths(&[])` immediately before the function under test, revoking all authorization.
3. Use the `try_*` client variant and `assert!(result.is_err())`.

If the `require_is_owner` guard is ever removed from any of these functions, valid inputs would complete successfully, `result.is_err()` would be `false`, and the test would fail — exactly what the issue requires.

---

### Issue #224 — CI WASM size gate

**`.github/workflows/ci.yml`**

- Added `WASM_SIZE_LIMIT_BYTES: 1500000` as a workflow-level `env` variable — configurable without touching the shell script logic.
- After `wasm-opt` in the `build-wasm` step:
  - Measures the optimised WASM size with `stat -c%s`.
  - Writes a markdown table (optimised size / limit / status) to `$GITHUB_STEP_SUMMARY`.
  - Fails the job with a `::error::` annotation if size exceeds the limit; prints a link to `docs/WASM_SIZE.md` for reduction tips.
  - Exports `WASM_FILE` to `$GITHUB_ENV` for the follow-on upload step.
- Added an "Upload optimised WASM artifact" step (guarded by `matrix.step == 'build-wasm'`) that uploads the file via `actions/upload-artifact@v4` with 14-day retention.

**`docs/WASM_SIZE.md`**

- Removed all NEAR Protocol references (wrong network entirely).
- Updated the constraint description to reference Soroban's `maxContractSizeBytes` network parameter.
- Fixed reduction tips to reference `soroban-sdk` instead of `near-sdk`.
- Aligned the local `wasm-opt` example command with the flags used in CI.

## Test plan

**#225**
- [ ] `cargo test -p neurowealth-vault test_non_owner_cannot_set_deposit_limits` passes
- [ ] `cargo test -p neurowealth-vault test_non_owner_cannot_set_tvl_cap` passes
- [ ] `cargo test -p neurowealth-vault test_non_owner_cannot_set_limits` passes
- [ ] Removing `require_is_owner` from any of the three functions causes the corresponding test to fail

**#224**
- [ ] CI passes on current `main` (WASM is under 1.5 MB)
- [ ] Step summary shows a WASM size table in the Build WASM job
- [ ] Optimised WASM is uploaded as the `neurowealth-vault-wasm` artifact
- [ ] Setting `WASM_SIZE_LIMIT_BYTES: 1` in the workflow causes the Build WASM job to fail
- [ ] `docs/WASM_SIZE.md` contains no NEAR references
