//! Tests for pause/unpause functionality

use super::utils::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_owner_can_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    assert!(!client.is_paused(), "Vault should start unpaused");

    client.pause(&owner);

    assert!(client.is_paused(), "Vault should be paused");
}

#[test]
fn test_owner_can_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);
    assert!(client.is_paused());

    client.unpause(&owner);
    assert!(!client.is_paused(), "Vault should be unpaused");
}

#[test]
fn test_agent_can_emergency_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    assert!(!client.is_paused());

    client.emergency_pause(&owner);

    assert!(client.is_paused(), "Vault should be emergency paused");
}

#[test]
#[should_panic(expected = "vault: only owner can unpause")]
fn test_non_owner_cannot_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Owner pauses
    client.pause(&owner);
    assert!(client.is_paused());

    // A fresh address that is NOT the owner tries to unpause
    let non_owner = Address::generate(&env);
    client.unpause(&non_owner);
}

#[test]
#[should_panic]
fn test_unauthorized_users_cannot_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let unauthorized = Address::generate(&env);

    client.emergency_pause(&owner);
    // Fails because unauthorized != stored_owner
    client.pause(&unauthorized);
}

#[test]
#[should_panic(expected = "vault: paused")]
fn test_deposit_blocked_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    client.pause(&owner);
    assert!(client.is_paused());

    let user = Address::generate(&env);
    let amount = 5_000_000_i128;

    token_client.mint(&user, &amount);
    client.deposit(&user, &amount);
}

#[test]
#[should_panic(expected = "vault: paused")]
fn test_withdraw_blocked_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let amount = 5_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    client.pause(&owner);
    assert!(client.is_paused());

    let balance = client.get_balance(&user);
    client.withdraw(&user, &balance);
}

#[test]
#[should_panic(expected = "vault: paused")]
fn test_rebalance_blocked_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);
    assert!(client.is_paused());

    // require_not_paused fires before any blend check
    client.rebalance(&soroban_sdk::symbol_short!("blend"), &500_i128);
}

#[test]
fn test_pause_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);

    let pause_events = find_events_by_topic(
        env.events().all(),
        &env,
        soroban_sdk::symbol_short!("paused"),
    );
    assert!(!pause_events.is_empty(), "Pause should emit an event");
}

#[test]
fn test_emergency_pause_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.emergency_pause(&owner);

    let emergency_events = find_events_by_topic(
        env.events().all(),
        &env,
        soroban_sdk::symbol_short!("emerg"),
    );
    assert!(
        !emergency_events.is_empty(),
        "Emergency pause should emit an event"
    );
}
