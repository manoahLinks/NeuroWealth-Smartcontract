# NeuroWealth Vault — E2E Devnet Validation

End-to-end tests that deploy the vault contract to Stellar testnet and verify
core flows against a live network.

## Quick Start

```bash
# 1. Build the contract WASM
cd neurowealth-vault
RUSTFLAGS="-C target-cpu=mvp" cargo build --target wasm32-unknown-unknown --release

# 2. Run E2E validation
cd ..
./scripts/e2e-devnet.sh
```

The script will generate testnet identities, deploy the contract, and run
through all scenarios automatically.

## Scenarios Covered

| # | Scenario | What it verifies |
|---|----------|------------------|
| 1 | Initialize | Vault init with agent + USDC token; double-init rejected |
| 2 | Deposit | Basic deposit flow, token transfer, balance update |
| 3 | Withdraw | Partial withdrawal, balance check |
| 4 | Pause / Unpause | Pause blocks deposits; unpause restores normal operation |
| 5 | Rebalance | Agent-initiated rebalance to "none" protocol |
| 6 | Withdraw All | Full balance withdrawal |
| 7 | Events | Verify emitted events (init, deposit, withdraw, pause, etc.) |

## Required Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SOROBAN_SECRET_KEY` | _(auto-generated)_ | Funded testnet secret key for the deployer identity. If not set, a fresh identity is generated and funded via friendbot. |
| `SOROBAN_RPC_URL` | `https://soroban-testnet.stellar.org` | Soroban RPC endpoint |
| `SOROBAN_NETWORK_PASSPHRASE` | `Test SDF Network ; September 2015` | Network passphrase |
| `E2E_TIMEOUT_SECS` | `300` | Per-command timeout in seconds |

## Artifacts

After a run, check `scripts/e2e-artifacts/` for:

| File | Contents |
|------|----------|
| `contract_id.txt` | Deployed vault contract ID |
| `usdc_token_id.txt` | Test USDC token contract ID |
| `deployer_address.txt` | Deployer/agent address |
| `user_address.txt` | Test user address |
| `tx_*.txt` | Raw output from each transaction |
| `events.txt` | Contract events fetched after all scenarios |
| `summary.txt` | Pass/fail summary for each scenario |
| `full_output.log` | Complete stdout (CI only) |

## Interpreting Results

**All passed:**
```
Scenarios passed: 7
Scenarios failed: 0
E2E VALIDATION PASSED
```

**Failures:** Each failed scenario includes a description of what went wrong.
Check the corresponding `tx_*.txt` artifact for the raw CLI output and error
messages.

**Event verification skipped:** Some testnet RPC endpoints do not support
event queries. The script handles this gracefully and marks the scenario as
skipped rather than failed.

## Debugging Failures

1. **Check the summary:** `cat scripts/e2e-artifacts/summary.txt`

2. **Check individual transactions:** Each scenario writes its output to
   `scripts/e2e-artifacts/tx_<scenario>.txt`.

3. **Look up on explorer:** Use the contract ID from
   `scripts/e2e-artifacts/contract_id.txt` on
   [StellarExpert](https://stellar.expert/explorer/testnet).

4. **Re-run a single invocation:** Copy the `stellar contract invoke` command
   from the log output and run it manually with `--verbose` for more detail.

5. **Insufficient funds:** If deposits fail with transfer errors, the testnet
   friendbot may not have funded accounts sufficiently. Try running
   `stellar keys fund e2e-user --network testnet` manually.

6. **CLI version mismatch:** The scripts support both `stellar` and `soroban`
   CLI names. Ensure your installed CLI version is compatible with
   `soroban-sdk = 21.0.0`.

## Cleanup

Remove local CLI identities and artifacts:

```bash
./scripts/e2e-restore.sh
```

Keep artifacts but remove identities:

```bash
./scripts/e2e-restore.sh --keep-artifacts
```

On-chain testnet state (deployed contracts, transactions) cannot be deleted.
The Stellar testnet is periodically reset by the SDF.

For a full explanation of what each artifact file contains, which outputs are
safe to delete, and how to restore from CI artifact bundles, see
[`docs/E2E_ARTIFACT_LIFECYCLE.md`](../docs/E2E_ARTIFACT_LIFECYCLE.md).

## CI Integration

The E2E tests run via `.github/workflows/e2e-devnet.yml`:

- **Manual trigger:** Use "Run workflow" in the GitHub Actions UI
- **Nightly schedule:** Runs automatically at 03:00 UTC
- **Artifacts:** Uploaded on both success and failure (14-day retention)
- **Job summary:** Pass/fail details and explorer links appear in the GitHub
  Actions job summary
