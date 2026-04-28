#![cfg(test)]

use super::utils::*;
use soroban_sdk::{testutils::Events, vec, Env, IntoVal, Symbol, Val, Vec};

#[test]
fn test_blend_integration_supply_via_rebalance() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup: Vault + Token + Blend Pool
        let (vault_id, agent, owner, usdc_token, blend_pool) = setup_vault_with_token_and_blend(&env);
    let vault_client = NeuroWealthVaultClient::new(&env, &vault_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    // 1. Give vault some USDC
    let deposit_amount = 100_000_000; // 100 USDC
    token_client.mint(&vault_id, &deposit_amount);
    assert_eq!(token_client.balance(&vault_id), deposit_amount);

    // 2. Set Blend Pool address in Vault
    // Note: setup_vault_with_token_and_blend already initializes the vault,
    // but we need to set the blend pool.
    // In setup_vault_with_token_and_blend, agent is the owner.
        vault_client.set_blend_pool(&owner, &blend_pool);

    // 3. Trigger rebalance to Blend
    let protocol = Symbol::new(&env, "blend");
    vault_client.rebalance(&protocol, &850); // 8.5% expected APY

    // 4. Verify results
    // - Vault USDC balance should be 0 (transferred to Blend)
    // - Blend Pool USDC balance should be deposit_amount
    // - Vault CurrentProtocol should be "blend"
    assert_eq!(token_client.balance(&vault_id), 0);
    assert_eq!(token_client.balance(&blend_pool), deposit_amount);
    assert_eq!(vault_client.get_current_protocol(), protocol);

    // 5. Verify events
    let events = env.events().all();
    let blend_sup_events = find_events_by_topic(events, &env, Symbol::new(&env, "blend_sup"));
    assert_eq!(blend_sup_events.len(), 1);
}

#[test]
fn test_blend_integration_withdraw_via_rebalance() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup: Vault + Token + Blend Pool
        let (vault_id, agent, owner, usdc_token, blend_pool) = setup_vault_with_token_and_blend(&env);
    let vault_client = NeuroWealthVaultClient::new(&env, &vault_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

        vault_client.set_blend_pool(&owner, &blend_pool);

    // 1. Supply to Blend first
    let amount = 50_000_000;
    token_client.mint(&vault_id, &amount);
    vault_client.rebalance(&Symbol::new(&env, "blend"), &850);
    assert_eq!(token_client.balance(&blend_pool), amount);

    // 2. Withdraw from Blend by rebalancing to "none"
    vault_client.rebalance(&Symbol::new(&env, "none"), &0);

    // 3. Verify results
    // - Vault USDC balance should be restored
    // - Blend Pool USDC balance should be 0
    // - Vault CurrentProtocol should be "none"
    assert_eq!(token_client.balance(&vault_id), amount);
    assert_eq!(token_client.balance(&blend_pool), 0);
    assert_eq!(
        vault_client.get_current_protocol(),
        Symbol::new(&env, "none")
    );

    // 4. Verify events
    let events = env.events().all();
    let blend_wd_events = find_events_by_topic(events, &env, Symbol::new(&env, "blend_wd"));
    assert_eq!(blend_wd_events.len(), 1);
}

#[test]
fn test_blend_integration_balance_read() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup
        let (vault_id, agent, owner, usdc_token, blend_pool) = setup_vault_with_token_and_blend(&env);
    let vault_client = NeuroWealthVaultClient::new(&env, &vault_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

        vault_client.set_blend_pool(&owner, &blend_pool);

    // 1. Supply some funds
    let amount = 75_000_000;
    token_client.mint(&vault_id, &amount);
    vault_client.rebalance(&Symbol::new(&env, "blend"), &850);

    // 2. Check balance via vault's internal get_blend_pool getter
    // Note: The vault doesn't have a public get_blend_balance, but we can verify it indirectly
    // or by checking the mock's balance directly as a proxy of what the vault sees.

    // Let's use a manual contract call to verify the mock responds correctly to the vault's expectation
    let args: Vec<Val> = vec![&env, usdc_token.into_val(&env), vault_id.into_val(&env)];
    let balance: i128 = env.invoke_contract(&blend_pool, &Symbol::new(&env, "balance"), args);

    assert_eq!(balance, amount);
}
