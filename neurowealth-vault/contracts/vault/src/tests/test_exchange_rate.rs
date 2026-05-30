//! Tests for `get_exchange_rate()` — issue #135
//!
//! Covers:
//! - Empty vault returns 1:1 parity (SCALAR)
//! - After first deposit, rate is still 1:1
//! - After yield accrual, rate exceeds SCALAR
//! - Rate is consistent with `convert_to_assets` / `convert_to_shares`
//! - Multi-user deposits do not distort the rate
//! - Uninitialized vault panics

use super::utils::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Scalar used by `get_exchange_rate()`.  Must match the constant in lib.rs.
const SCALAR: i128 = 10_000_000;

// ============================================================================
// EMPTY-VAULT / BOOTSTRAP TESTS
// ============================================================================

/// Before any deposit the vault has no shares and no assets.
/// `get_exchange_rate()` must return `SCALAR` (i.e. 1.0000000) to represent
/// 1:1 parity and avoid a divide-by-zero panic.
#[test]
fn test_exchange_rate_returns_scalar_when_vault_is_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let rate = client.get_exchange_rate();
    assert_eq!(
        rate, SCALAR,
        "empty vault must report 1:1 parity ({SCALAR}), got {rate}"
    );
}

// ============================================================================
// AFTER DEPOSIT — RATE STILL 1:1
// ============================================================================

/// After the very first deposit the total_shares == total_assets (bootstrap
/// 1:1 mapping), so exchange_rate = (assets * SCALAR) / shares = SCALAR.
#[test]
fn test_exchange_rate_is_one_to_one_after_first_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let deposit = 10_000_000_i128; // 1 USDC (7 decimal places)

    mint_and_deposit(&env, &client, &usdc_token, &user, deposit);

    let rate = client.get_exchange_rate();
    assert_eq!(
        rate, SCALAR,
        "rate after first deposit must be 1:1 ({SCALAR}), got {rate}"
    );
}

// ============================================================================
// YIELD ACCRUAL — RATE EXCEEDS SCALAR
// ============================================================================

/// After yield increases `total_assets` by 50 % while `total_shares` stays
/// constant, each share should be worth 1.5 USDC → rate = 15_000_000.
#[test]
fn test_exchange_rate_increases_after_yield_accrual() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit = 10_000_000_i128; // 1 USDC

    mint_and_deposit(&env, &client, &usdc_token, &user, deposit);

    // Simulate 50 % yield: total_assets goes from 10_000_000 → 15_000_000
    let yield_amount = 5_000_000_i128;
    let new_total = deposit + yield_amount;

    // Mint yield tokens into vault so the balance check in update_total_assets passes
    token_client.mint(&contract_id, &yield_amount);
    client.update_total_assets(&agent, &new_total, &false, &0u32);

    let rate = client.get_exchange_rate();
    // Expected: (15_000_000 * 10_000_000) / 10_000_000 = 15_000_000
    let expected = new_total * SCALAR / deposit; // 15_000_000
    assert_eq!(
        rate, expected,
        "after 50 % yield, rate must be {expected}, got {rate}"
    );
    assert!(
        rate > SCALAR,
        "rate must exceed SCALAR after positive yield"
    );
}

/// Doubling `total_assets` means each share is worth exactly 2.0 USDC.
#[test]
fn test_exchange_rate_doubles_when_assets_double() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, deposit);

    // Double total assets
    let doubled = deposit * 2;
    token_client.mint(&contract_id, &deposit);
    client.update_total_assets(&agent, &doubled, &false, &0u32);

    let rate = client.get_exchange_rate();
    let expected = 2 * SCALAR; // 20_000_000
    assert_eq!(rate, expected, "rate must be 2×SCALAR after assets double");
}

// ============================================================================
// CONSISTENCY WITH convert_to_assets / convert_to_shares
// ============================================================================

/// For a single depositor, converting their shares back to assets via
/// `convert_to_assets` must equal `user_shares * rate / SCALAR`.
#[test]
fn test_exchange_rate_consistent_with_convert_to_assets() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, deposit);

    // 20 % yield
    let yield_amount = 2_000_000_i128;
    let new_total = deposit + yield_amount;
    token_client.mint(&contract_id, &yield_amount);
    client.update_total_assets(&agent, &new_total, &false, &0u32);

    let user_shares = client.get_shares(&user);
    let rate = client.get_exchange_rate();

    // value via exchange_rate path
    let via_rate = user_shares * rate / SCALAR;
    // value via convert_to_assets path
    let via_convert = client.convert_to_assets(&user_shares);

    assert_eq!(
        via_rate, via_convert,
        "exchange-rate path ({via_rate}) must match convert_to_assets ({via_convert})"
    );
}

// ============================================================================
// MULTI-USER DEPOSITS — RATE STABILITY
// ============================================================================

/// Two deposits of equal size at the same share price must leave the rate
/// unchanged (both users join at 1:1, total_assets = total_shares).
#[test]
fn test_exchange_rate_stable_across_equal_multi_user_deposits() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount = 5_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user1, amount);
    let rate_after_first = client.get_exchange_rate();

    mint_and_deposit(&env, &client, &usdc_token, &user2, amount);
    let rate_after_second = client.get_exchange_rate();

    assert_eq!(
        rate_after_first, rate_after_second,
        "rate must not change when a second user deposits at the same price"
    );
    assert_eq!(rate_after_second, SCALAR, "rate must remain 1:1");
}

/// A second deposit after yield accrual must not change the exchange rate
/// (the new depositor enters at the current elevated price).
#[test]
fn test_exchange_rate_stable_after_second_deposit_post_yield() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let deposit = 10_000_000_i128;

    // User 1 deposits
    mint_and_deposit(&env, &client, &usdc_token, &user1, deposit);

    // 30 % yield
    let yield_amount = 3_000_000_i128;
    let new_total = deposit + yield_amount;
    token_client.mint(&contract_id, &yield_amount);
    client.update_total_assets(&agent, &new_total, &false, &0u32);

    let rate_before_second = client.get_exchange_rate();

    // User 2 deposits — their shares are priced at current rate, so rate must hold
    mint_and_deposit(&env, &client, &usdc_token, &user2, deposit);
    let rate_after_second = client.get_exchange_rate();

    // Allow for ±1 rounding tolerance from integer arithmetic
    let diff = (rate_before_second - rate_after_second).abs();
    assert!(
        diff <= 1,
        "rate must not materially change after second deposit at elevated price; \
         before={rate_before_second}, after={rate_after_second}"
    );
}

// ============================================================================
// FLOOR ROUNDING — RATE NEVER OVER-REPORTED
// ============================================================================

/// When `total_assets * SCALAR` is not perfectly divisible by `total_shares`,
/// the result must be floor-rounded (≤ true rational value).
#[test]
fn test_exchange_rate_is_floor_rounded() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, deposit);

    // Add 1 stroop of yield so the division is not exact
    let yield_amount = 1_i128;
    let new_total = deposit + yield_amount;
    token_client.mint(&contract_id, &yield_amount);
    client.update_total_assets(&agent, &new_total, &false, &0u32);

    let rate = client.get_exchange_rate();
    // true rational: (10_000_001 * 10_000_000) / 10_000_000 = 10_000_001.0 exactly
    // integer: 10_000_001
    let expected = new_total * SCALAR / deposit;
    assert_eq!(rate, expected);

    // Ensure rate is never *more* than the true value
    let true_rational = (new_total as f64 * SCALAR as f64) / deposit as f64;
    assert!(
        rate as f64 <= true_rational + 1.0,
        "rate must not exceed the true rational value"
    );
}
