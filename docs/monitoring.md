# NeuroWealth Vault — Monitoring & Audit Trail Strategy

Operations guide for running the NeuroWealth Vault in production.
All signals reference on-chain state read from the Stellar/Soroban ledger.

---

## 1. Routine Signals

Monitor these metrics continuously across every ledger window.

| Signal | How to Measure | Healthy Range |
|--------|----------------|---------------|
| TVL (TotalAssets) | `get_total_assets()` per ledger | Monotonically non-decreasing absent withdrawals |
| TVL growth rate | `(TotalAssets_now - TotalAssets_1h_ago) / TotalAssets_1h_ago` | Positive or flat; sharp drops warrant investigation |
| Deposit volume per ledger | Count `deposit()` calls + sum of amounts in ledger window | Tracks user inflow |
| Withdrawal volume per ledger | Count `withdraw()` + `withdraw_all()` calls + amounts | Tracks user outflow |
| Rebalance frequency | Count `rebalance()` calls per hour; compare to `MinRebalanceInterval` | Never more frequent than cooldown allows |
| Share price | `get_total_assets() / get_total_shares()` | Must be monotonically non-decreasing |
| Yield accrual | `get_total_assets()` before and after each `update_total_assets()` call | Delta ≥ 0 (no unexpected decrease) |
| TVL headroom | `(TvlCap - TotalAssets) / TvlCap` | Alert when < 5% headroom remains |

---

## 2. Warning Signals (Anomalies)

These conditions indicate abnormal behavior and require prompt investigation.

| Anomaly | Condition | Severity |
|---------|-----------|----------|
| Sudden TVL drop | `TotalAssets_now < TotalAssets_1h_ago * 0.80` | Critical |
| Extended pause | `Paused == true` for more than 24 h | High |
| Withdrawal spike | `withdrawal_volume_1h > withdrawal_volume_30d_avg * 3` | High |
| Cap saturation | Repeated `Error(Contract, #41)` rejections | Medium |
| Cooldown violation attempt | `rebalance()` called before cooldown elapsed | Medium |
| Share price decrease | `current_share_price < previous_share_price` | Critical |
| `update_total_assets` reporting lower value | New value < stored TotalAssets without `allow_decrease=true` | High |
| Vault contract upgrade | `upgrade()` called | High — requires governance sign-off |

---

## 3. Audit Trail

Track these on-chain events and storage mutations. Soroban events are indexed by
topic; the vault emits structured events for every significant state change.

### Admin Actions

| Action | Contract Function | Event Topic | Who |
|--------|------------------|-------------|-----|
| Pause vault | `pause()` | `pause` | Owner |
| Unpause vault | `unpause()` | `unpause` | Owner |
| Emergency pause | `emergency_pause()` | `emergency_pause` | Owner |
| Set TVL cap | `set_tvl_cap()` | `set_tvl_cap` | Owner |
| Transfer ownership | `set_owner()` | `set_owner` | Owner |
| Upgrade contract | `upgrade()` | `upgrade` | Owner |

### Parameter Changes

| Action | Contract Function | What Changes |
|--------|------------------|--------------|
| Set per-user deposit cap | `set_user_deposit_cap()` | Max single-user cumulative deposit |
| Set minimum deposit | `set_min_deposit()` | Smallest accepted deposit amount |
| Set Blend pool | `set_blend_pool()` | Target Blend pool address |
| Set rebalance interval | `set_min_rebalance_interval()` | Cooldown between rebalances |

### Rebalance Executions

Each `rebalance()` call must be logged with:
- Source protocol (prior `CurrentProtocol`)
- Destination protocol (new `CurrentProtocol`)
- Amount moved
- Ledger sequence (timestamp proxy)
- Agent address

### Large Transactions

Flag any single `deposit()` or `withdraw()` where:

```
amount > get_total_assets() * 0.01
```

A deposit or withdrawal exceeding 1% of TVL in a single transaction warrants
manual review.

---

## 4. Alert Examples

```
ALERT: tvl_drop_20pct
  condition: get_total_assets() < TotalAssets_1h_ago * 0.80
  severity: critical
  action: Page on-call; suspend agent rebalance authority until reviewed

ALERT: pause_duration_exceeded
  condition: Paused == true AND current_ledger > pause_start_ledger + 17280
  note: 17280 ledgers ≈ 24 h at ~5 s/ledger
  severity: high
  action: Notify owner; investigate reason for extended pause

ALERT: withdrawal_spike
  condition: withdrawal_volume_1h > withdrawal_volume_30d_avg * 3
  severity: high
  action: Review for coordinated exit; check protocol health

ALERT: tvl_cap_approach
  condition: get_total_assets() > TvlCap * 0.95
  severity: medium
  action: Consider raising cap or preparing user communication

ALERT: share_price_decrease
  condition: (get_total_assets() / get_total_shares()) < previous_share_price
  severity: critical
  action: Halt new deposits; investigate slashing or accounting error

ALERT: rapid_rebalance_attempts
  condition: rebalance() called more than once within MinRebalanceInterval
  severity: medium
  action: Audit agent key; verify no unauthorized rebalance calls
```

---

## 5. Suspicious Activity Indicators

These patterns may indicate manipulation, insider abuse, or a compromised key.

| Pattern | Description | Response |
|---------|-------------|----------|
| Deposit-withdraw cycling | Multiple accounts depositing near the cap and immediately withdrawing | Investigate for fee extraction or share-price manipulation |
| Admin address change without delay | `set_owner()` called without a governance timelock or multisig | Verify legitimacy; check for key compromise |
| Rapid emergency pause cycles | `emergency_pause()` / `unpause()` called multiple times within 24 h | Treat as potential exploit attempt; freeze agent authority |
| `update_total_assets()` reporting decrease | `allow_decrease=false` but a lower value was passed (would revert) | Indicates misconfigured yield reporter or off-chain bug |
| Unusual `upgrade()` timing | `upgrade()` called outside scheduled maintenance windows | Mandatory governance review before execution |
| Agent calling non-agent functions | Agent address calling `pause()`, `set_tvl_cap()`, etc. | Key misuse; rotate agent key immediately |
| TVL cap set to 0 | `set_tvl_cap(0)` effectively blocks all deposits | Verify intent; could be accidental denial-of-service |

---

## 6. Ledger-to-Time Conversion Reference

Soroban does not expose wall-clock time natively. Use ledger sequence as a proxy.

| Duration | Approximate Ledger Count (5 s/ledger) |
|----------|---------------------------------------|
| 1 hour   | 720 ledgers                           |
| 6 hours  | 4 320 ledgers                         |
| 24 hours | 17 280 ledgers                        |
| 7 days   | 120 960 ledgers                       |
| 30 days  | 518 400 ledgers                       |

These are estimates. Use `env.ledger().sequence()` for precise comparisons in
contract code; cross-reference with Stellar Horizon for wall-clock mapping in
off-chain monitoring.
