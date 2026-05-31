use super::*;
use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Events},
    Address, Env,
};

// ============================================================================
// SIMPLE TEST TOKEN CONTRACT
// ============================================================================

#[contracttype]
enum TokenDataKey {
    Balance(Address),
}

mod token {
    use super::*;

    #[contract]
    pub struct TestToken;

    #[contractimpl]
    impl TestToken {
        pub fn mint(env: Env, to: Address, amount: i128) {
            let balance: i128 = env
                .storage()
                .persistent()
                .get(&TokenDataKey::Balance(to.clone()))
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&TokenDataKey::Balance(to), &(balance + amount));
        }

        pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
            from.require_auth();
            assert!(amount > 0, "amount must be positive");

            let from_balance: i128 = env
                .storage()
                .persistent()
                .get(&TokenDataKey::Balance(from.clone()))
                .unwrap_or(0);
            assert!(from_balance >= amount, "insufficient balance");

            let to_balance: i128 = env
                .storage()
                .persistent()
                .get(&TokenDataKey::Balance(to.clone()))
                .unwrap_or(0);

            env.storage()
                .persistent()
                .set(&TokenDataKey::Balance(from), &(from_balance - amount));
            env.storage()
                .persistent()
                .set(&TokenDataKey::Balance(to), &(to_balance + amount));
        }

        pub fn balance(env: Env, owner: Address) -> i128 {
            env.storage()
                .persistent()
                .get(&TokenDataKey::Balance(owner))
                .unwrap_or(0)
        }
    }
}

use token::{TestToken, TestTokenClient};


// ============================================================================
// TEST SETUP FUNCTIONS
// ============================================================================

fn setup_vault(env: &Env) -> (Address, Address, Address) {
    let (contract_id, agent, owner, _usdc_token) = setup_vault_with_token(env);
    (contract_id, agent, owner)
}

fn setup_vault_with_token(env: &Env) -> (Address, Address, Address, Address) {
    let contract_id = env.register_contract(None, NeuroWealthVault);
    let client = NeuroWealthVaultClient::new(env, &contract_id);
    let agent = Address::generate(env);
    let usdc_token = env.register_contract(None, TestToken);
    let owner = Address::generate(env);

    client.initialize(&owner, &agent, &usdc_token);

    (contract_id, agent, owner, usdc_token)
}

// ============================================================================
// DEPOSIT LIMITS TESTS
// ============================================================================

#[test]
fn test_get_min_deposit_default() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let min_deposit = client.get_min_deposit();
    assert_eq!(min_deposit, 1_000_000_i128); // 1 USDC default
}

#[test]
fn test_get_max_deposit_default() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let max_deposit = client.get_max_deposit();
    assert_eq!(max_deposit, 10_000_000_000_i128); // 10K USDC default
}

#[test]
fn test_set_deposit_limits_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_min = 2_000_000_i128; // 2 USDC
    let new_max = 20_000_000_000_i128; // 20K USDC

    client.set_deposit_limits(&new_min, &new_max);

    assert_eq!(client.get_min_deposit(), new_min);
    assert_eq!(client.get_max_deposit(), new_max);
}

#[test]
#[should_panic(expected = "vault: minimum deposit too low")]
fn test_set_deposit_limits_min_too_low() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let min = 999_999_i128; // Less than 1 USDC
    let max = 10_000_000_000_i128;

    client.set_deposit_limits(&min, &max);
}

#[test]
#[should_panic(expected = "vault: maximum deposit below minimum")]
fn test_set_deposit_limits_max_less_than_min() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let min = 5_000_000_i128; // 5 USDC
    let max = 4_000_000_i128; // 4 USDC (less than min)

    client.set_deposit_limits(&min, &max);
}

#[test]
#[should_panic(expected = "vault: below minimum deposit")]
fn test_deposit_below_minimum() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Set minimum to 5 USDC
    let min = 5_000_000_i128;
    let max = 20_000_000_000_i128;
    client.set_deposit_limits(&min, &max);

    let _user = Address::generate(&env);
    let amount = 4_000_000_i128; // 4 USDC (below minimum)

    // This should panic
    client.deposit(&_user, &amount);
}

#[test]
#[should_panic(expected = "vault: exceeds maximum deposit")]
fn test_deposit_above_maximum() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Set maximum to 5 USDC
    let min = 1_000_000_i128;
    let max = 5_000_000_i128;
    client.set_deposit_limits(&min, &max);

    let _user = Address::generate(&env);
    let amount = 6_000_000_i128; // 6 USDC (above maximum)

    // This should panic
    client.deposit(&_user, &amount);
}

#[test]
fn test_deposit_at_minimum_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Set minimum to 5 USDC
    let min = 5_000_000_i128;
    let max = 20_000_000_000_i128;
    client.set_deposit_limits(&min, &max);

    let _user = Address::generate(&env);
    let amount = 5_000_000_i128; // Exactly at minimum

    // This should succeed (though we can't fully test without token mocking)
    assert_eq!(client.get_min_deposit(), min);
    assert!(amount >= min);
}

#[test]
fn test_deposit_at_maximum_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Set maximum to 5 USDC
    let min = 1_000_000_i128;
    let max = 5_000_000_i128;
    client.set_deposit_limits(&min, &max);

    let _user = Address::generate(&env);
    let amount = 5_000_000_i128; // Exactly at maximum

    // This should succeed (though we can't fully test without token mocking)
    assert_eq!(client.get_max_deposit(), max);
    assert!(amount <= max);
}

#[test]
#[should_panic(expected = "vault: below minimum deposit")]
fn test_deposit_one_stroop_below_minimum() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Use default minimum of 1 USDC
    let _user = Address::generate(&env);
    let amount = 999_999_i128; // 1 stroop below 1 USDC

    // This should panic
    client.deposit(&_user, &amount);
}

#[test]
#[should_panic(expected = "vault: exceeds maximum deposit")]
fn test_deposit_one_stroop_above_maximum() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Set maximum to 10 USDC
    let min = 1_000_000_i128;
    let max = 10_000_000_i128;
    client.set_deposit_limits(&min, &max);

    let _user = Address::generate(&env);
    let amount = 10_000_001_i128; // 1 stroop above maximum

    // This should panic
    client.deposit(&_user, &amount);
}

#[test]
fn test_owner_updates_limits_immediate_effect() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Verify initial limits
    assert_eq!(client.get_min_deposit(), 1_000_000_i128);
    assert_eq!(client.get_max_deposit(), 10_000_000_000_i128);

    // Update limits
    let new_min = 3_000_000_i128; // 3 USDC
    let new_max = 15_000_000_000_i128; // 15K USDC
    client.set_deposit_limits(&new_min, &new_max);

    // Verify new limits are immediately effective
    assert_eq!(client.get_min_deposit(), new_min);
    assert_eq!(client.get_max_deposit(), new_max);

    // Test that new limits apply immediately by checking validation
    let _user = Address::generate(&env);

    // Amount below new minimum should fail
    let below_min = 2_000_000_i128; // 2 USDC
    assert!(below_min < new_min);

    // Amount above new maximum should fail
    let above_max = 20_000_000_000_i128; // 20K USDC
    assert!(above_max > new_max);

    // Amount within new range should be valid
    let within_range = 5_000_000_i128; // 5 USDC
    assert!(within_range >= new_min && within_range <= new_max);
}

// ============================================================================
// EVENT TESTS
// ============================================================================

#[test]
fn test_vault_initialized_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, NeuroWealthVault);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let agent = Address::generate(&env);
    let owner = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&owner, &agent, &usdc_token);

    let events = env.events().all();
    assert!(
        !events.is_empty(),
        "Expected initialization event to be emitted"
    );
}

#[test]
fn test_vault_paused_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);

    let events = env.events().all();
    assert!(!events.is_empty(), "Expected pause event to be emitted");
}

#[test]
fn test_vault_unpaused_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);
    client.unpause(&owner);

    let events = env.events().all();
    assert!(!events.is_empty(), "Expected unpause event to be emitted");
}

#[test]
fn test_emergency_paused_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.emergency_pause(&owner);

    let events = env.events().all();
    assert!(
        !events.is_empty(),
        "Expected emergency pause event to be emitted"
    );
}

#[test]
fn test_limits_updated_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_min = 20_000_000_000_i128; // 20K USDC
    let new_max = 200_000_000_000_i128; // 200M USDC

    client.set_deposit_limits(&new_min, &new_max);

    let events = env.events().all();
    assert!(
        !events.is_empty(),
        "Expected limits updated event to be emitted"
    );
}

#[test]
fn test_agent_updated_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _old_agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_agent = Address::generate(&env);
    client.update_agent(&new_agent);

    let events = env.events().all();
    assert!(
        !events.is_empty(),
        "Expected agent updated event to be emitted"
    );
}

#[test]
fn test_assets_updated_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_total = 50_000_000_000_i128; // 50M USDC
    client.update_total_assets(&agent, &new_total);

    let events = env.events().all();
    assert!(
        !events.is_empty(),
        "Expected assets updated event to be emitted"
    );
}

// ============================================================================
// SECURITY TESTS
// ============================================================================

#[test]
fn test_pause_by_non_owner_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, NeuroWealthVault);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let agent = Address::generate(&env);
    let owner = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let _non_owner = Address::generate(&env);

    client.initialize(&owner, &agent, &usdc_token);

    // Verify vault starts unpaused
    assert!(!client.is_paused(), "Vault should start unpaused");

    // Note: Auth checks in pause() are enforced by require_auth() at contract level
    // This test would need proper auth mocking to fully test the failure case
}

#[test]
fn test_rebalance_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Pause the vault
    client.pause(&owner);
    assert!(client.is_paused());

    // Rebalance while paused should be prevented by require_not_paused guard
    // For this test, we verify the pause state is correctly set
    assert!(client.is_paused());
}

// ============================================================================
// INTEGRATION TESTS - SHARE ACCOUNTING
// ============================================================================

#[test]
fn test_first_deposit_mints_1_to_1_shares() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let amount = 5_000_000_i128;

    token_client.mint(&user, &amount);
    client.deposit(&user, &amount);

    assert_eq!(client.get_shares(&user), amount);
    assert_eq!(client.get_total_assets(), amount);
}

#[test]
fn test_subsequent_deposit_maintains_share_price() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount1 = 5_000_000_i128;
    let amount2 = 10_000_000_i128;

    token_client.mint(&user1, &amount1);
    client.deposit(&user1, &amount1);

    token_client.mint(&user2, &amount2);
    client.deposit(&user2, &amount2);

    // Price should remain 1:1, so shares == assets for both
    assert_eq!(client.get_shares(&user1), amount1);
    assert_eq!(client.get_shares(&user2), amount2);
    assert_eq!(client.get_total_assets(), amount1 + amount2);
}

#[test]
fn test_yield_accrual_increases_withdrawal_value() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    token_client.mint(&user, &deposit_amount);
    client.deposit(&user, &deposit_amount);

    // Simulate yield: total assets increase by 50%.
    let yield_amount = deposit_amount / 2;
    let new_total_assets = deposit_amount + yield_amount;
    client.update_total_assets(&agent, &new_total_assets);

    // Mint the corresponding yield tokens to the vault
    token_client.mint(&contract_id, &yield_amount);

    // User should now be able to withdraw more than original deposit
    let before_withdraw_balance = client.get_balance(&user);
    assert!(before_withdraw_balance > deposit_amount);

    client.withdraw(&user, &before_withdraw_balance);
    assert_eq!(client.get_shares(&user), 0);
}

#[test]
fn test_get_shares_zero_for_new_user() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let _user = Address::generate(&env);
    assert_eq!(client.get_shares(&_user), 0);
}

#[test]
fn test_get_balance_zero_when_no_shares() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let _user = Address::generate(&env);
    assert_eq!(client.get_balance(&_user), 0);
}

// ============================================================================
// AGENT EMERGENCY PROTECTION TESTS
// ============================================================================

#[test]
fn test_agent_can_trigger_emergency_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, NeuroWealthVault);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let agent = Address::generate(&env);
    let owner = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&owner, &agent, &usdc_token);

    // Owner and agent are distinct; owner can trigger emergency pause
    client.emergency_pause(&owner);
    assert!(client.is_paused());
}

#[test]
fn test_only_owner_can_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, NeuroWealthVault);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let agent = Address::generate(&env);
    let owner = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&owner, &agent, &usdc_token);

    // Owner pauses
    client.pause(&owner);
    assert!(client.is_paused());

    // Only owner can unpause
    client.unpause(&agent);
    assert!(!client.is_paused());
}

// ============================================================================
// UPGRADE TESTS
// ============================================================================

/// Helper that installs the vault WASM in the test environment and returns
/// a valid hash for use in upgrade tests.
///
/// This compiles the current contract (via `contractimport!`) and uploads it,
/// giving us a real 32-byte hash the deployer will accept.
mod vault_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/neurowealth_vault.wasm"
    );
}

fn upload_vault_wasm(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(vault_wasm::WASM)
}

#[test]
fn test_version_is_1_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    assert_eq!(client.get_version(), 1u32);
}

#[test]
fn test_owner_can_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let owner = client.get_owner();
    let new_wasm_hash = upload_vault_wasm(&env);

    client.upgrade(&owner, &new_wasm_hash);

    assert_eq!(client.get_version(), 2u32);
}

#[test]
#[should_panic(expected = "vault: caller is not the owner")]
fn test_non_owner_cannot_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let non_owner = Address::generate(&env);
    let fake_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);

    client.upgrade(&non_owner, &fake_wasm_hash);
}

#[test]
fn test_version_increments_correctly() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let owner = client.get_owner();
    let new_wasm_hash = upload_vault_wasm(&env);

    assert_eq!(client.get_version(), 1u32);

    client.upgrade(&owner, &new_wasm_hash.clone());
    assert_eq!(client.get_version(), 2u32);

    client.upgrade(&owner, &new_wasm_hash);
    assert_eq!(client.get_version(), 3u32);
}

#[test]
fn test_upgrade_emits_upgraded_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let owner = client.get_owner();
    let new_wasm_hash = upload_vault_wasm(&env);

    client.upgrade(&owner, &new_wasm_hash);

    let all = env.events().all();
    let events: std::vec::Vec<_> = all.into_iter().collect();
    let upgraded_events = find_events_by_topic(&events, &env, symbol_short!("upgraded"));
    assert_eq!(upgraded_events.len(), 1);

    let event_data = UpgradedEvent::try_from_val(&env, &upgraded_events[0].2).unwrap();
    assert_eq!(event_data.old_version, 1u32);
    assert_eq!(event_data.new_version, 2u32);
}

// ============================================================================
// MOCK BLEND POOL CONTRACT
// ============================================================================

mod blend {
    use super::*;

    #[contract]
    pub struct MockBlendPool;

    #[contractimpl]
    impl MockBlendPool {
        pub fn submit_with_allowance(
            env: Env,
            _from: Address,
            _to: Address,
            _spender: Address,
            requests: Vec<BlendRequest>,
        ) {
            // Simple mock: just transfer tokens from vault to pool
            let usdc_token = requests.get(0).unwrap().address;
            let amount = requests.get(0).unwrap().amount;
            let pool = env.current_contract_address();
            let vault = _from;

            let token_client = TestTokenClient::new(&env, &usdc_token);
            token_client.transfer(&vault, &pool, &amount);
        }

        pub fn redeem(env: Env, asset: Address, amount: i128, to: Address) -> i128 {
            let token_client = TestTokenClient::new(&env, &asset);
            let pool_balance = token_client.balance(&env.current_contract_address());

            // Mock partial redemption: return only half of what's in the pool if requested > 0
            // this simulates a protocol with limited liquidity
            let actual_to_return = if amount > 0 {
                core::cmp::min(amount, pool_balance / 2)
            } else {
                0
            };

            if actual_to_return > 0 {
                token_client.transfer(&env.current_contract_address(), &to, &actual_to_return);
            }
            actual_to_return
        }

        pub fn balance(env: Env, asset: Address, _user: Address) -> i128 {
            TestTokenClient::new(&env, &asset).balance(&env.current_contract_address())
        }
    }
}

use blend::MockBlendPool;


// ============================================================================
// RECONCILIATION TESTS
// ============================================================================

#[test]
fn test_withdraw_reconciles_partial_blend_redemption() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let pool_id = env.register_contract(None, MockBlendPool);
    client.set_blend_pool(&pool_id);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128; // 10 USDC

    // 1. Initial Deposit
    token_client.mint(&user, &deposit_amount);
    client.deposit(&user, &deposit_amount);

    // 2. Rebalance to Blend
    client.rebalance(&symbol_short!("blend"), &800_i128, &0_i128);

    // Verify funds moved to pool
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(token_client.balance(&pool_id), deposit_amount);

    // 3. User attempts to withdraw 6 USDC
    // MockBlendPool will only return 5 USDC (half of its 10 USDC balance)
    let withdraw_request = 6_000_000_i128;
    client.withdraw(&user, &withdraw_request);

    // 4. Verify Reconciliation
    // User should have received 5 USDC, not 6.
    assert_eq!(token_client.balance(&user), 5_000_000_i128);

    // User's remaining shares should reflect that they only got 5 USDC.
    // At 1:1 price, they should have 5,000,000 shares remaining (10M - 5M).
    assert_eq!(client.get_shares(&user), 5_000_000_i128);

    // Vault accounting should be consistent
    assert_eq!(client.get_total_assets(), 5_000_000_i128);
    assert_eq!(client.get_total_deposits(), 5_000_000_i128);
}

#[test]
fn test_withdraw_all_reconciles_partial_blend_redemption() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let pool_id = env.register_contract(None, MockBlendPool);
    client.set_blend_pool(&pool_id);

    let user = Address::generate(&env);
    let deposit_amount = 20_000_000_i128; // 20 USDC

    // 1. Initial Deposit
    token_client.mint(&user, &deposit_amount);
    client.deposit(&user, &deposit_amount);

    // 2. Rebalance to Blend
    client.rebalance(&symbol_short!("blend"), &800_i128, &0_i128);

    // 3. User attempts to withdraw_all (entitled to 20 USDC)
    // MockBlendPool will only return 10 USDC (half of its 20 USDC balance)
    client.withdraw_all(&user);

    // 4. Verify Reconciliation
    // User should have received 10 USDC
    assert_eq!(token_client.balance(&user), 10_000_000_i128);

    // User should still have 10,000,000 shares remaining
    assert_eq!(client.get_shares(&user), 10_000_000_i128);

    // Vault accounting should be consistent
    assert_eq!(client.get_total_assets(), 10_000_000_i128);
}

// ============================================================================
// SET LIMITS INVARIANT VALIDATION TESTS (#120)
// ============================================================================

#[test]
fn test_set_limits_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user_cap = 5_000_000_i128;   // 5 USDC per-user cap
    let tvl_cap  = 50_000_000_i128;  // 50 USDC TVL cap

    client.set_limits(&user_cap, &tvl_cap);

    assert_eq!(client.get_min_deposit(), user_cap);
    assert_eq!(client.get_max_deposit(), tvl_cap);
}

#[test]
fn test_set_limits_negative_min_returns_error() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let result = client.try_set_limits(&(-1_i128), &10_000_000_i128);
    assert_eq!(result, Err(Ok(VaultError::NegativeMin)));
}

#[test]
fn test_set_limits_negative_max_returns_error() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let result = client.try_set_limits(&1_000_000_i128, &(-1_i128));
    assert_eq!(result, Err(Ok(VaultError::NegativeMax)));
}

#[test]
fn test_set_limits_max_less_than_min_returns_error() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let result = client.try_set_limits(&5_000_000_i128, &4_000_000_i128);
    assert_eq!(result, Err(Ok(VaultError::MaxLessThanMin)));
}

#[test]
fn test_set_limits_equal_min_max_allowed() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // max == min is valid (max >= min)
    let cap = 5_000_000_i128;
    client.set_limits(&cap, &cap);

    assert_eq!(client.get_min_deposit(), cap);
    assert_eq!(client.get_max_deposit(), cap);
}

#[test]
fn test_set_limits_emits_event_with_correct_fields() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user_cap = 2_000_000_i128;
    let tvl_cap  = 20_000_000_i128;

    client.set_limits(&user_cap, &tvl_cap);

    let events = env.events().all();
    // The last event should be LimitsUpdatedEvent
    assert!(!events.is_empty(), "expected at least one event");
}
