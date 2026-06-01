//! Tests for persistent-storage TTL behavior on user share entries.

use super::utils::*;
use crate::DataKey;
use soroban_sdk::testutils::storage::Persistent as _;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_get_shares_does_not_extend_ttl() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    mint_and_deposit(&env, &client, &usdc_token, &user, 5_000_000);

    let ttl_before = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get_ttl(&DataKey::Shares(user.clone()))
    });

    let _ = client.get_shares(&user);

    let ttl_after_read = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get_ttl(&DataKey::Shares(user.clone()))
    });

    assert_eq!(
        ttl_before, ttl_after_read,
        "get_shares must not extend Shares TTL"
    );
}

#[test]
fn test_get_balance_does_not_extend_ttl() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    mint_and_deposit(&env, &client, &usdc_token, &user, 5_000_000);

    let ttl_before = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get_ttl(&DataKey::Shares(user.clone()))
    });

    let _ = client.get_balance(&user);

    let ttl_after_read = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get_ttl(&DataKey::Shares(user.clone()))
    });

    assert_eq!(
        ttl_before, ttl_after_read,
        "get_balance must not extend Shares TTL"
    );
}

#[test]
fn test_touch_user_ttl_extends_shares_entry() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    mint_and_deposit(&env, &client, &usdc_token, &user, 5_000_000);

    let ttl_before = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get_ttl(&DataKey::Shares(user.clone()))
    });

    assert!(client.touch_user_ttl(&user));

    let ttl_after_touch = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get_ttl(&DataKey::Shares(user.clone()))
    });

    assert!(
        ttl_after_touch >= ttl_before,
        "touch_user_ttl should extend or preserve Shares TTL"
    );
}

#[test]
fn test_touch_user_ttl_no_entry_returns_false() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    assert!(!client.touch_user_ttl(&user));
}
