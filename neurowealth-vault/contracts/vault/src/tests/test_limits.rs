//! Tests for deposit limits and caps

use super::utils::*;
use crate::{DataKey, DEFAULT_MIN_DEPOSIT};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_owner_can_set_tvl_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let tvl_cap = 100_000_000_000_i128; // 100K USDC
    client.set_tvl_cap(&tvl_cap);

    assert_eq!(client.get_tvl_cap(), tvl_cap);
}

#[test]
fn test_owner_can_set_user_deposit_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user_cap = 50_000_000_000_i128; // 50K USDC
    client.set_user_deposit_cap(&user_cap);

    assert_eq!(client.get_user_deposit_cap(), user_cap);
}

#[test]
fn test_set_deposit_limits() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let min = 2_000_000_i128; // 2 USDC
    let max = 20_000_000_000_i128; // 20K USDC

    client.set_deposit_limits(&min, &max);

    assert_eq!(client.get_min_deposit(), min);
    assert_eq!(client.get_max_deposit(), max);
}

#[test]
#[should_panic(expected = "vault: minimum deposit too low")]
fn test_set_deposit_limits_min_too_low() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let min = 999_999_i128; // Less than 1 USDC
    let max = 20_000_000_000_i128;

    client.set_deposit_limits(&min, &max);
}

#[test]
#[should_panic(expected = "vault: maximum deposit below minimum")]
fn test_set_deposit_limits_max_less_than_min() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let min = 5_000_000_i128;
    let max = 4_000_000_i128; // Less than min

    client.set_deposit_limits(&min, &max);
}

#[test]
#[should_panic(expected = "vault: exceeds TVL cap")]
fn test_deposit_enforces_tvl_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    // Set TVL cap to 10 USDC
    let tvl_cap = 10_000_000_i128;
    client.set_tvl_cap(&tvl_cap);

    let user = Address::generate(&env);
    let amount = 11_000_000_i128; // 11 USDC — exceeds TVL cap

    token_client.mint(&user, &amount);
    client.deposit(&user, &amount);
}

#[test]
#[should_panic(expected = "vault: exceeds user deposit cap")]
fn test_deposit_enforces_user_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    // Set user deposit cap to 5 USDC
    let user_cap = 5_000_000_i128;
    client.set_user_deposit_cap(&user_cap);

    let user = Address::generate(&env);
    let amount = 6_000_000_i128; // 6 USDC — exceeds user cap

    token_client.mint(&user, &amount);
    client.deposit(&user, &amount);
}

#[test]
fn test_tvl_cap_zero_means_unlimited() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // cap = 0 disables enforcement
    client.set_tvl_cap(&0_i128);

    let user = Address::generate(&env);
    let amount = 5_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    assert_eq!(client.get_total_deposits(), amount);
}

#[test]
fn test_user_deposit_cap_zero_means_unlimited() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // cap = 0 disables enforcement
    client.set_user_deposit_cap(&0_i128);

    let user = Address::generate(&env);
    let amount = 5_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    assert_eq!(client.get_shares(&user), amount);
}

#[test]
fn test_get_min_deposit_uses_default_when_key_missing() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    env.as_contract(&contract_id, || {
        env.storage().instance().remove(&DataKey::MinDeposit);
    });

    assert_eq!(client.get_min_deposit(), DEFAULT_MIN_DEPOSIT);
}

#[test]
#[should_panic(expected = "vault: below minimum deposit")]
fn test_deposit_uses_default_minimum_when_key_missing() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    env.as_contract(&contract_id, || {
        env.storage().instance().remove(&DataKey::MinDeposit);
    });

    let user = Address::generate(&env);
    let below_default_min = DEFAULT_MIN_DEPOSIT - 1;
    token_client.mint(&user, &below_default_min);

    client.deposit(&user, &below_default_min);
}
