# fix: protect initialize() from front-running & add indexed user topic to events

Closes #117 · Closes #162

---

## Summary

### #117 — Protect `initialize()` against front-running takeover

`initialize()` was already guarded by a two-factor check:

1. **Contract-address derivation** — the `deployer` argument plus `salt` must
   reproduce the current contract address via
   `env.deployer().with_address(deployer, salt).deployed_address()`.  An
   attacker who passes their own address as `deployer` cannot satisfy this
   check.
2. **`deployer.require_auth()`** — even if an attacker somehow knows the real
   deployer address and the salt (both visible on-chain), they cannot produce
   the deployer's signature.

This PR adds:
- **Two regression tests** in `test_initialize.rs` that prove both attack
  vectors are rejected:
  - `test_front_runner_without_deployer_auth_is_rejected` — calls
    `initialize()` without mocking any auth; `require_auth()` panics.
  - `test_front_runner_with_own_address_as_deployer_is_rejected` — attacker
    substitutes their own address; the address-derivation check panics with
    `"vault: unauthorized deployer"`.
- **Secure Deployment Sequence** section in `README.md` with step-by-step CLI
  commands covering: keypair generation → contract deploy with salt → immediate
  `initialize()` call from the same keypair → verification → keypair disposal.

### #162 — Add indexed user topic to Deposit and Withdraw events

Previously, `DepositEvent` and `WithdrawEvent` were published with a single
symbol topic `("deposit",)` / `("withdraw",)`.  Filtering events by user
required scanning the full payload on every indexer node.

This PR publishes the user address as a second **indexed topic**:

```
("deposit",  <user: Address>)   // DepositEvent
("withdraw", <user: Address>)   // WithdrawEvent  (withdraw + withdraw_all)
```

Indexers and AI agents can now filter by user via `topic[1]` directly.

- **`lib.rs`** — three publish sites updated (`deposit`, `withdraw`,
  `withdraw_all`), each using `user.clone()` so the address is still available
  for the payload struct.
- **`test_event_schema.rs`** — new test
  `test_deposit_withdraw_user_indexed_topic` asserts that the user address
  appears as an indexed topic in both event types.
- **`EVENTS.md`** — updated event descriptions for `DepositEvent` and
  `WithdrawEvent` with topic-position tables.

---

## Files changed

| File | Change |
|---|---|
| `neurowealth-vault/contracts/vault/src/lib.rs` | Add `user.clone()` as second indexed topic in deposit/withdraw/withdraw_all event publishes |
| `neurowealth-vault/contracts/vault/src/tests/test_event_schema.rs` | Add `test_deposit_withdraw_user_indexed_topic` |
| `neurowealth-vault/contracts/vault/src/tests/test_initialize.rs` | Add two front-running regression tests |
| `EVENTS.md` | Document indexed user topic and topic-position tables for Deposit/Withdraw events |
| `README.md` | Add Secure Deployment Sequence section |

---

## Test plan

- [x] `cargo test` — all 238 tests pass, 0 failures
- [x] New test `test_deposit_withdraw_user_indexed_topic` confirms user address in topics
- [x] New test `test_front_runner_without_deployer_auth_is_rejected` confirms auth guard
- [x] New test `test_front_runner_with_own_address_as_deployer_is_rejected` confirms address guard
- [x] All pre-existing event schema tests continue to pass (existing `find_events_by_topic` searches all topic slots, so the added address topic does not break any existing assertions)

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
