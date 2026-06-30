//! Bridge listener interface for external (cross-chain) deposits.
//!
//! An authorized off-chain relayer — a *bridge listener* — watches an origin
//! chain and records confirmed deposits into Shade so that downstream
//! crowdfunding / payment flows and off-chain indexers can react to them.
//!
//! Access control is two-tiered:
//!   * Only the contract admin may register or remove bridge listeners.
//!   * Only a currently registered listener may record a deposit.
//!
//! Listener registration is idempotent so that retried or concurrent admin
//! calls converge on the same state. Deposit replay protection is keyed on the
//! origin-chain transaction hash (`source_tx_id`), so the same external
//! transfer can never be credited twice even under concurrent relayer
//! submissions.

use crate::components::{admin, core};
use crate::errors::ContractError;
use crate::events;
use crate::types::{BridgeDeposit, DataKey};
use soroban_sdk::{panic_with_error, Address, BytesN, Env, String};

fn get_listener_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::BridgeListenerCount)
        .unwrap_or(0)
}

fn get_deposit_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::BridgeDepositCount)
        .unwrap_or(0)
}

/// Register an address as an authorized bridge listener. Admin only.
/// Idempotent: re-registering an existing listener is a no-op.
pub fn register_bridge_listener(env: &Env, admin: &Address, listener: &Address) {
    core::assert_admin(env, admin);

    if is_bridge_listener(env, listener) {
        return;
    }

    env.storage()
        .persistent()
        .set(&DataKey::BridgeListener(listener.clone()), &true);
    env.storage().persistent().set(
        &DataKey::BridgeListenerCount,
        &(get_listener_count(env) + 1),
    );

    events::publish_bridge_listener_registered_event(
        env,
        admin.clone(),
        listener.clone(),
        env.ledger().timestamp(),
    );
}

/// Revoke a bridge listener's authorization. Admin only.
/// Idempotent: removing an address that is not a listener is a no-op.
pub fn remove_bridge_listener(env: &Env, admin: &Address, listener: &Address) {
    core::assert_admin(env, admin);

    if !is_bridge_listener(env, listener) {
        return;
    }

    env.storage()
        .persistent()
        .remove(&DataKey::BridgeListener(listener.clone()));
    env.storage().persistent().set(
        &DataKey::BridgeListenerCount,
        &get_listener_count(env).saturating_sub(1),
    );

    events::publish_bridge_listener_removed_event(
        env,
        admin.clone(),
        listener.clone(),
        env.ledger().timestamp(),
    );
}

pub fn is_bridge_listener(env: &Env, listener: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::BridgeListener(listener.clone()))
}

/// Record a confirmed external-chain deposit.
///
/// Callable only by a registered bridge listener (auth required). The deposit
/// is persisted, credited to `recipient` for the given token, and de-duplicated
/// on `source_tx_id`. Returns the new sequential deposit id.
#[allow(clippy::too_many_arguments)]
pub fn record_bridge_deposit(
    env: &Env,
    listener: &Address,
    source_chain: String,
    source_tx_id: BytesN<32>,
    token: Address,
    amount: i128,
    recipient: Address,
) -> u64 {
    listener.require_auth();

    if !is_bridge_listener(env, listener) {
        panic_with_error!(env, ContractError::NotAuthorized);
    }
    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }
    if !admin::is_accepted_token(env, &token) {
        panic_with_error!(env, ContractError::TokenNotAccepted);
    }
    if is_bridge_deposit_processed(env, &source_tx_id) {
        panic_with_error!(env, ContractError::BridgeDepositProcessed);
    }

    // Mark processed first so any re-entrant or concurrent retry is rejected.
    env.storage().persistent().set(
        &DataKey::ProcessedBridgeDeposit(source_tx_id.clone()),
        &true,
    );

    let deposit_id = get_deposit_count(env) + 1;
    env.storage()
        .persistent()
        .set(&DataKey::BridgeDepositCount, &deposit_id);

    let now = env.ledger().timestamp();
    let deposit = BridgeDeposit {
        id: deposit_id,
        source_chain: source_chain.clone(),
        source_tx_id: source_tx_id.clone(),
        listener: listener.clone(),
        token: token.clone(),
        amount,
        recipient: recipient.clone(),
        timestamp: now,
    };
    env.storage()
        .persistent()
        .set(&DataKey::BridgeDeposit(deposit_id), &deposit);

    // Maintain a running per-recipient, per-token credited total for queries.
    let credit_key = DataKey::BridgeCredit(recipient.clone(), token.clone());
    let credited: i128 = env.storage().persistent().get(&credit_key).unwrap_or(0);
    env.storage()
        .persistent()
        .set(&credit_key, &(credited + amount));

    events::publish_bridge_deposit_recorded_event(
        env,
        deposit_id,
        listener.clone(),
        source_chain,
        source_tx_id,
        token,
        amount,
        recipient,
        now,
    );

    deposit_id
}

/// Fetch a recorded deposit by id, or `None` if no such deposit exists.
pub fn get_bridge_deposit(env: &Env, deposit_id: u64) -> Option<BridgeDeposit> {
    env.storage()
        .persistent()
        .get(&DataKey::BridgeDeposit(deposit_id))
}

pub fn is_bridge_deposit_processed(env: &Env, source_tx_id: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::ProcessedBridgeDeposit(source_tx_id.clone()))
}

pub fn get_bridge_deposit_count(env: &Env) -> u64 {
    get_deposit_count(env)
}

pub fn get_bridge_credit(env: &Env, recipient: &Address, token: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::BridgeCredit(recipient.clone(), token.clone()))
        .unwrap_or(0)
}
