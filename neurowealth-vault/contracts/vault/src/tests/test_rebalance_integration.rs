//! Integration tests for rebalance protocol-path behavior (Issue #46)
//!
//! These tests validate end-to-end rebalance transitions including:
//!  - Rebalance to Blend with pool configured
//!  - Rebalance to Blend without pool (expected panic)
//!  - Rebalance away from current protocol (none and switch cases)
//!  - Failed/partial protocol call handling
//!  - CurrentProtocol storage state correctness
//!  - Asset accounting invariants
//!  - Emitted rebalance events

extern crate std;

use super::utils::*;
use crate::{BlendSupplyEvent, BlendWithdrawEvent, RebalanceEvent};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, TryFromVal};

// ============================================================================
// HELPERS
// ============================================================================

/// Total USDC held by the vault contract.
fn vault_usdc_balance(env: &Env, token: &Address, contract_id: &Address) -> i128 {
    TestTokenClient::new(env, token).balance(contract_id)
}

/// Return all decoded RebalanceEvents emitted so far.
fn collect_rebalance_events(env: &Env) -> std::vec::Vec<RebalanceEvent> {
    find_events_by_topic(env.events().all(), env, symbol_short!("rebalance"))
        .into_iter()
        .map(|(_, _, data)| {
            RebalanceEvent::try_from_val(env, &data).expect("data should decode to RebalanceEvent")
        })
        .collect()
}

/// Return all decoded BlendWithdrawEvents emitted so far.
fn collect_blend_withdraw_events(env: &Env) -> std::vec::Vec<BlendWithdrawEvent> {
    find_events_by_topic(env.events().all(), env, symbol_short!("blend_wd"))
        .into_iter()
        .map(|(_, _, data)| {
            BlendWithdrawEvent::try_from_val(env, &data)
                .expect("data should decode to BlendWithdrawEvent")
        })
        .collect()
}

/// Return all decoded BlendSupplyEvents emitted so far.
fn collect_blend_supply_events(env: &Env) -> std::vec::Vec<BlendSupplyEvent> {
    find_events_by_topic(env.events().all(), env, symbol_short!("blend_sup"))
        .into_iter()
        .map(|(_, _, data)| {
            BlendSupplyEvent::try_from_val(env, &data)
                .expect("data should decode to BlendSupplyEvent")
        })
        .collect()
}

// ============================================================================
// 1. REBALANCE TO BLEND — POOL CONFIGURED
// ============================================================================

/// Happy-path: rebalance into Blend when a pool is configured and the vault
/// has a deposited balance.  
/// Validates: CurrentProtocol, asset accounting, and the emitted events.
#[test]
fn test_integration_rebalance_to_blend_with_pool_configured() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let blend_client = MockBlendPoolClient::new(&env, &blend_pool);

    // Pre-condition: no protocol, no pool configured
    assert_eq!(client.get_current_protocol(), symbol_short!("none"));
    assert!(client.get_blend_pool().is_none());

    // Configure pool
    client.set_blend_pool(&owner, &blend_pool);
    assert_eq!(client.get_blend_pool(), Some(blend_pool.clone()));

    // Deposit funds
    let deposit_amount = 15_000_000_i128; // 15 USDC
    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // Pre-rebalance state
    assert_eq!(
        vault_usdc_balance(&env, &usdc_token, &contract_id),
        deposit_amount
    );
    assert_eq!(blend_client.supplied(&usdc_token), 0);
    assert_eq!(client.get_total_assets(), deposit_amount);

    // Rebalance to Blend
    let apy = 700_i128; // 7%
    client.rebalance(&symbol_short!("blend"), &apy, &0_i128);

    // ---- CurrentProtocol ----
    assert_eq!(
        client.get_current_protocol(),
        symbol_short!("blend"),
        "CurrentProtocol should switch to 'blend'"
    );

    // ---- Asset accounting ----
    // All vault USDC moved to Blend; total assets unchanged
    assert_eq!(
        vault_usdc_balance(&env, &usdc_token, &contract_id),
        0,
        "Vault should have 0 idle USDC after supplying to Blend"
    );
    assert_eq!(
        blend_client.supplied(&usdc_token),
        deposit_amount,
        "Blend pool should hold all deposited USDC"
    );
    assert_eq!(
        client.get_total_assets(),
        deposit_amount,
        "Total assets tracked by vault should be unchanged"
    );

    // ---- Emitted events ----
    let supply_events = collect_blend_supply_events(&env);
    assert!(
        !supply_events.is_empty(),
        "BlendSupplyEvent must be emitted"
    );
    let supply_event = supply_events.last().unwrap();
    assert_eq!(supply_event.amount_actual, deposit_amount);
    assert!(supply_event.success);

    let rebalance_events = collect_rebalance_events(&env);
    assert!(
        !rebalance_events.is_empty(),
        "RebalanceEvent must be emitted"
    );
    let rebalance_event = rebalance_events.last().unwrap();
    assert_eq!(rebalance_event.protocol, symbol_short!("blend"));
    assert_eq!(rebalance_event.expected_apy, apy);
}

/// Re-entrant call: rebalancing to Blend while already deployed to Blend
/// should not duplicate supply or cause double-accounting.
#[test]
fn test_integration_rebalance_to_blend_already_in_blend_is_idempotent() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let blend_client = MockBlendPoolClient::new(&env, &blend_pool);

    client.set_blend_pool(&owner, &blend_pool);

    let deposit_amount = 10_000_000_i128;
    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // First rebalance to Blend
    client.rebalance(&symbol_short!("blend"), &500_i128, &0_i128);
    assert_eq!(client.get_current_protocol(), symbol_short!("blend"));
    assert_eq!(blend_client.supplied(&usdc_token), deposit_amount);

    // Second rebalance to Blend — no vault idle balance, nothing more to supply
    client.rebalance(&symbol_short!("blend"), &500_i128, &0_i128);

    let rebalance_events = collect_rebalance_events(&env);
    assert_eq!(
        rebalance_events.last().unwrap().status,
        symbol_short!("noop"),
        "Redundant blend rebalance with zero idle should be noop"
    );

    // State should remain consistent
    assert_eq!(client.get_current_protocol(), symbol_short!("blend"));
    assert_eq!(
        blend_client.supplied(&usdc_token),
        deposit_amount,
        "Supplied amount should not change on redundant rebalance to blend"
    );
    assert_eq!(
        vault_usdc_balance(&env, &usdc_token, &contract_id),
        0,
        "Vault should still hold 0 idle USDC"
    );

    // Total assets must remain the same
    assert_eq!(client.get_total_assets(), deposit_amount);
}

// ============================================================================
// 2. REBALANCE TO BLEND — NO POOL (EXPECTED PANIC)
// ============================================================================

/// Rebalancing to Blend without a pool configured must panic with the
/// "vault: blend pool not configured" message when the vault holds funds.
#[test]
#[should_panic(expected = "vault: blend pool not configured")]
fn test_integration_rebalance_to_blend_without_pool_panics_with_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Deposit so vault_balance > 0 which triggers the pool lookup
    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, 5_000_000_i128);

    // No blend pool set → must panic
    client.rebalance(&symbol_short!("blend"), &500_i128, &0_i128);
}

/// Rebalancing to Blend without a pool configured must panic even when the
/// vault has zero balance (the check is on pool existence, not balance).
#[test]
#[should_panic(expected = "vault: blend pool not configured")]
fn test_integration_rebalance_to_blend_without_pool_panics_zero_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // No deposit, no pool — still panics because pool check comes first
    client.rebalance(&symbol_short!("blend"), &500_i128, &0_i128);
}

// ============================================================================
// 3. REBALANCE AWAY FROM CURRENT PROTOCOL
// ============================================================================

/// Switching from Blend back to "none" must:
/// - Withdraw all funds from Blend
/// - Update CurrentProtocol to "none"
/// - Emit BlendWithdrawEvent and RebalanceEvent
/// - Restore full asset accounting in vault
#[test]
fn test_integration_rebalance_from_blend_to_none_withdraws_all() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let blend_client = MockBlendPoolClient::new(&env, &blend_pool);

    client.set_blend_pool(&owner, &blend_pool);

    let deposit_amount = 25_000_000_i128;
    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // Move funds into Blend
    client.rebalance(&symbol_short!("blend"), &800_i128, &0_i128);
    assert_eq!(blend_client.supplied(&usdc_token), deposit_amount);

    // Switch back to none
    client.rebalance(&symbol_short!("none"), &0_i128, &0_i128);

    // ---- CurrentProtocol ----
    assert_eq!(
        client.get_current_protocol(),
        symbol_short!("none"),
        "CurrentProtocol should revert to 'none'"
    );

    // ---- Asset accounting ----
    assert_eq!(
        blend_client.supplied(&usdc_token),
        0,
        "Blend pool should be fully emptied"
    );
    assert_eq!(
        vault_usdc_balance(&env, &usdc_token, &contract_id),
        deposit_amount,
        "All funds must return to the vault"
    );
    assert_eq!(client.get_total_assets(), deposit_amount);

    // ---- Events ----
    let wd_events = collect_blend_withdraw_events(&env);
    assert!(!wd_events.is_empty(), "BlendWithdrawEvent must be emitted");
    let wd_event = wd_events.last().unwrap();
    assert_eq!(wd_event.amount_actual, deposit_amount);
    assert!(wd_event.success, "Withdrawal should be marked successful");

    let rebalance_events = collect_rebalance_events(&env);
    let none_event = rebalance_events
        .iter()
        .find(|e| e.protocol == symbol_short!("none"))
        .expect("RebalanceEvent for 'none' must be emitted");
    assert_eq!(none_event.expected_apy, 0_i128);
}

/// Rebalancing from "none" to "none" is a no-op and should succeed without
/// touching any pool or changing any state.
#[test]
fn test_integration_rebalance_none_to_none_is_noop() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let deposit_amount = 8_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // Rebalance none → none
    client.rebalance(&symbol_short!("none"), &0_i128, &0_i128);

    assert_eq!(client.get_current_protocol(), symbol_short!("none"));
    assert_eq!(
        vault_usdc_balance(&env, &usdc_token, &contract_id),
        deposit_amount,
        "Vault funds should be untouched when protocol stays 'none'"
    );
    assert_eq!(client.get_total_assets(), deposit_amount);

    let rebalance_events = collect_rebalance_events(&env);
    assert_eq!(rebalance_events.len(), 1, "Expected one RebalanceEvent");
    assert_eq!(
        rebalance_events[0].status,
        symbol_short!("noop"),
        "none→none with no fund movement should be a noop rebalance"
    );
}

/// Full round-trip: none → blend → none → blend, validating state at each step.
#[test]
fn test_integration_full_protocol_round_trip() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let blend_client = MockBlendPoolClient::new(&env, &blend_pool);

    client.set_blend_pool(&owner, &blend_pool);

    let deposit_amount = 20_000_000_i128;
    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // Step 1: none → blend
    client.rebalance(&symbol_short!("blend"), &600_i128, &0_i128);
    assert_eq!(client.get_current_protocol(), symbol_short!("blend"));
    assert_eq!(blend_client.supplied(&usdc_token), deposit_amount);
    assert_eq!(vault_usdc_balance(&env, &usdc_token, &contract_id), 0);

    // Step 2: blend → none
    client.rebalance(&symbol_short!("none"), &0_i128, &0_i128);
    assert_eq!(client.get_current_protocol(), symbol_short!("none"));
    assert_eq!(blend_client.supplied(&usdc_token), 0);
    assert_eq!(
        vault_usdc_balance(&env, &usdc_token, &contract_id),
        deposit_amount
    );

    // Step 3: none → blend again
    client.rebalance(&symbol_short!("blend"), &550_i128, &0_i128);
    assert_eq!(client.get_current_protocol(), symbol_short!("blend"));
    assert_eq!(blend_client.supplied(&usdc_token), deposit_amount);
    assert_eq!(vault_usdc_balance(&env, &usdc_token, &contract_id), 0);
    assert_eq!(client.get_total_assets(), deposit_amount);
}

// ============================================================================
// 4. FAILED / PARTIAL PROTOCOL CALL HANDLING
// ============================================================================

/// When the vault has a zero balance (no deposits), rebalancing to Blend
/// must not panic — it should complete successfully even though there is
/// nothing to supply to the pool.
///
/// Note: The contract only writes CurrentProtocol = "blend" inside
/// `supply_to_blend`, which is guarded by `vault_balance > 0`.
/// When vault balance is zero, rebalance to blend is a noop but must update
/// CurrentProtocol to "blend" so protocol tracking is always consistent with
/// the agent's intent (Issue #146).
#[test]
fn test_integration_rebalance_blend_zero_vault_balance_no_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.set_blend_pool(&owner, &blend_pool);

    // No deposit — vault has zero balance; this must NOT panic
    client.rebalance(&symbol_short!("blend"), &400_i128, &0_i128);

    // Even with zero vault balance, CurrentProtocol must be updated to "blend"
    // so state reflects the agent's intent (Issue #146).
    assert_eq!(
        client.get_current_protocol(),
        symbol_short!("blend"),
        "CurrentProtocol must update to 'blend' even when no funds are moved"
    );

    let rebalance_events = collect_rebalance_events(&env);
    assert_eq!(rebalance_events.len(), 1, "Expected one RebalanceEvent");
    assert_eq!(
        rebalance_events[0].status,
        symbol_short!("noop"),
        "blend rebalance with zero idle balance should be a noop"
    );
}

/// Unsupported protocol names must panic with the canonical error message
/// so that callers receive a deterministic, explicit failure.
#[test]
#[should_panic(expected = "vault: unsupported protocol")]
fn test_integration_rebalance_unknown_protocol_is_explicit_failure() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.rebalance(&symbol_short!("aave"), &1200_i128, &0_i128);
}

/// Paused-vault rebalance must fail with an explicit "vault: paused" panic
/// regardless of which protocol is targeted.
#[test]
#[should_panic(expected = "vault: paused")]
fn test_integration_rebalance_while_paused_explicit_failure() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.set_blend_pool(&owner, &blend_pool);
    client.pause(&owner);
    assert!(client.is_paused());

    client.rebalance(&symbol_short!("blend"), &500_i128, &0_i128);
}

// ============================================================================
// 5. ASSET ACCOUNTING ACROSS DEPOSIT / REBALANCE / WITHDRAW
// ============================================================================

/// Verifies that total assets remain conserved across a deposit → blend supply
/// → partial user withdraw → blend → none cycle.
#[test]
fn test_integration_asset_accounting_invariant_across_full_cycle() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let blend_client = MockBlendPoolClient::new(&env, &blend_pool);

    client.set_blend_pool(&owner, &blend_pool);

    // Two users deposit
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let amount_a = 12_000_000_i128;
    let amount_b = 8_000_000_i128;
    let total = amount_a + amount_b;

    mint_and_deposit(&env, &client, &usdc_token, &user_a, amount_a);
    mint_and_deposit(&env, &client, &usdc_token, &user_b, amount_b);
    assert_eq!(client.get_total_assets(), total);

    // Rebalance to Blend
    client.rebalance(&symbol_short!("blend"), &700_i128, &0_i128);
    assert_eq!(client.get_current_protocol(), symbol_short!("blend"));
    assert_eq!(blend_client.supplied(&usdc_token), total);
    assert_eq!(vault_usdc_balance(&env, &usdc_token, &contract_id), 0);

    // User A withdraws
    client.withdraw(&user_a, &amount_a);
    let expected_in_blend = blend_client.supplied(&usdc_token);
    assert_eq!(
        expected_in_blend, amount_b,
        "Blend should retain only user B's share after user A withdraws"
    );
    assert_eq!(client.get_total_assets(), amount_b);

    // Rebalance back to none
    client.rebalance(&symbol_short!("none"), &0_i128, &0_i128);
    assert_eq!(client.get_current_protocol(), symbol_short!("none"));
    assert_eq!(blend_client.supplied(&usdc_token), 0);
    assert_eq!(
        vault_usdc_balance(&env, &usdc_token, &contract_id),
        amount_b,
        "Vault should hold exactly user B's remaining funds"
    );
    assert_eq!(client.get_total_assets(), amount_b);
}

// ============================================================================
// 6. EVENT SCHEMA VALIDATION
// ============================================================================

/// Validates that every rebalance emits exactly one RebalanceEvent with the
/// correct protocol and expected_apy fields.
#[test]
fn test_integration_rebalance_event_schema_correctness() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.set_blend_pool(&owner, &blend_pool);

    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, 10_000_000_i128);

    // Rebalance 1: none (initial state → none)
    client.rebalance(&symbol_short!("none"), &0_i128, &0_i128);
    // Rebalance 2: none → blend
    client.rebalance(&symbol_short!("blend"), &850_i128, &0_i128);
    // Rebalance 3: blend → none
    client.rebalance(&symbol_short!("none"), &0_i128, &0_i128);

    let events = collect_rebalance_events(&env);
    assert_eq!(events.len(), 3, "Expected exactly 3 RebalanceEvents");

    assert_eq!(events[0].protocol, symbol_short!("none"));
    assert_eq!(events[0].expected_apy, 0_i128);

    assert_eq!(events[1].protocol, symbol_short!("blend"));
    assert_eq!(events[1].expected_apy, 850_i128);

    assert_eq!(events[2].protocol, symbol_short!("none"));
    assert_eq!(events[2].expected_apy, 0_i128);
}

/// Validates that a Blend supply emits BlendSupplyEvent with the correct
/// asset address and success flag.
#[test]
fn test_integration_blend_supply_event_fields_are_correct() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.set_blend_pool(&owner, &blend_pool);

    let deposit_amount = 18_000_000_i128;
    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    client.rebalance(&symbol_short!("blend"), &900_i128, &0_i128);

    let supply_events = collect_blend_supply_events(&env);
    assert_eq!(
        supply_events.len(),
        1,
        "Exactly one BlendSupplyEvent expected"
    );

    let evt = &supply_events[0];
    assert_eq!(
        evt.asset, usdc_token,
        "BlendSupplyEvent.asset must be the USDC token"
    );
    assert_eq!(evt.amount_actual, deposit_amount);
    assert!(evt.success);
}

/// Validates that a Blend withdrawal emits BlendWithdrawEvent with correct
/// amount_actual and success fields.
#[test]
fn test_integration_blend_withdraw_event_fields_on_protocol_switch() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.set_blend_pool(&owner, &blend_pool);

    let deposit_amount = 22_000_000_i128;
    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // Move to Blend then away
    client.rebalance(&symbol_short!("blend"), &600_i128, &0_i128);
    client.rebalance(&symbol_short!("none"), &0_i128, &0_i128);

    let wd_events = collect_blend_withdraw_events(&env);
    assert!(!wd_events.is_empty(), "BlendWithdrawEvent must be emitted");

    let evt = wd_events.last().unwrap();
    assert_eq!(
        evt.amount_actual, deposit_amount,
        "Amount actual should match full deposited balance"
    );
    assert!(evt.success, "Withdrawal event must be marked successful");
}

/// User withdrawal that pulls all funds out of Blend must update
/// CurrentProtocol to "none".
#[test]
fn test_integration_withdraw_all_updates_current_protocol_to_none() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token, blend_pool) =
        setup_vault_with_token_and_blend(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let blend_client = MockBlendPoolClient::new(&env, &blend_pool);

    client.set_blend_pool(&owner, &blend_pool);

    let deposit_amount = 25_000_000_i128;
    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // Move funds into Blend
    client.rebalance(&symbol_short!("blend"), &800_i128, &0_i128);
    assert_eq!(client.get_current_protocol(), symbol_short!("blend"));
    assert_eq!(blend_client.supplied(&usdc_token), deposit_amount);

    // User withdraws everything
    client.withdraw_all(&user);

    // Vault should have pulled everything from Blend
    assert_eq!(blend_client.supplied(&usdc_token), 0);

    // CurrentProtocol should be updated to none
    assert_eq!(
        client.get_current_protocol(),
        symbol_short!("none"),
        "CurrentProtocol should switch to 'none' when all funds are withdrawn from Blend"
    );
}
