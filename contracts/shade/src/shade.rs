use crate::components::{
    access_control as access_control_component, admin as admin_component,
    auto_withdrawal as auto_withdrawal_component, bridge as bridge_component,
    core as core_component, governance as governance_component, history as history_component,
    invoice as invoice_component, merchant as merchant_component, pausable as pausable_component,
    subscription as subscription_component, upgrade as upgrade_component,
};
use crate::errors::ContractError;
use crate::events;
use crate::interface::ShadeTrait;
use crate::types::{
    BridgeDeposit, ContractInfo, CrossChainBridgePayload, DataKey, Event, Invoice, InvoiceFilter,
    Merchant, MerchantAnalytics, MerchantAnalyticsSummary, MerchantFilter, OracleConfig,
    PendingFee, Role, Subscription, SubscriptionPlan, Ticket, TokenAnalytics, Transaction,
    UpgradeProposal,
};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, BytesN, Env, String, Vec};

#[contract]
pub struct Shade;

#[contractimpl]
impl ShadeTrait for Shade {
    fn initialize(env: Env, admin: Address) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }
        let contract_info = ContractInfo {
            admin: admin.clone(),
            timestamp: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::PlatformAccount, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::ContractInfo, &contract_info);
        events::publish_initialized_event(&env, admin, env.ledger().timestamp());
    }

    fn get_admin(env: Env) -> Address {
        core_component::get_admin(&env)
    }

    fn add_accepted_token(env: Env, admin: Address, token: Address) {
        pausable_component::assert_not_paused(&env);
        admin_component::add_accepted_token(&env, &admin, &token);
    }

    fn add_accepted_tokens(env: Env, admin: Address, tokens: Vec<Address>) {
        pausable_component::assert_not_paused(&env);
        admin_component::add_accepted_tokens(&env, &admin, &tokens);
    }

    fn remove_accepted_token(env: Env, admin: Address, token: Address) {
        pausable_component::assert_not_paused(&env);
        admin_component::remove_accepted_token(&env, &admin, &token);
    }

    fn is_accepted_token(env: Env, token: Address) -> bool {
        admin_component::is_accepted_token(&env, &token)
    }

    fn set_account_wasm_hash(env: Env, admin: Address, wasm_hash: soroban_sdk::BytesN<32>) {
        admin_component::set_account_wasm_hash(&env, &admin, &wasm_hash);
    }

    fn set_fee(env: Env, admin: Address, token: Address, fee: i128) {
        pausable_component::assert_not_paused(&env);
        admin_component::set_fee(&env, &admin, &token, fee);
    }

    fn get_fee(env: Env, token: Address) -> i128 {
        admin_component::get_fee(&env, &token)
    }

    fn set_platform_account(env: Env, admin: Address, account: Address) {
        pausable_component::assert_not_paused(&env);
        admin_component::set_platform_account(&env, &admin, &account);
    }

    fn get_platform_account(env: Env) -> Address {
        admin_component::get_platform_account(&env)
    }

    fn set_token_oracle(env: Env, admin: Address, token: Address, oracle: OracleConfig) {
        pausable_component::assert_not_paused(&env);
        admin_component::set_token_oracle(&env, &admin, &token, &oracle);
    }

    fn get_token_oracle(env: Env, token: Address) -> OracleConfig {
        admin_component::get_token_oracle(&env, &token)
    }

    fn propose_fee(env: Env, admin: Address, token: Address, fee: i128) {
        pausable_component::assert_not_paused(&env);
        admin_component::propose_fee(&env, &admin, &token, fee);
    }

    fn execute_fee(env: Env, admin: Address, token: Address) {
        pausable_component::assert_not_paused(&env);
        admin_component::execute_fee(&env, &admin, &token);
    }

    fn get_pending_fee(env: Env, token: Address) -> PendingFee {
        admin_component::get_pending_fee(&env, &token)
    }

    fn register_merchant(env: Env, merchant: Address) {
        pausable_component::assert_not_paused(&env);
        merchant_component::register_merchant(&env, &merchant);
    }

    fn get_merchant(env: Env, merchant_id: u64) -> Merchant {
        merchant_component::get_merchant(&env, merchant_id)
    }

    fn get_merchants(env: Env, filter: MerchantFilter) -> Vec<Merchant> {
        merchant_component::get_merchants(&env, filter)
    }

    fn is_merchant(env: Env, merchant: Address) -> bool {
        merchant_component::is_merchant(&env, &merchant)
    }

    fn set_merchant_status(env: Env, admin: Address, merchant_id: u64, status: bool) {
        merchant_component::set_merchant_status(&env, &admin, merchant_id, status);
    }

    fn is_merchant_active(env: Env, merchant_id: u64) -> bool {
        merchant_component::is_merchant_active(&env, merchant_id)
    }

    fn verify_merchant(env: Env, admin: Address, merchant_id: u64, status: bool) {
        merchant_component::verify_merchant(&env, &admin, merchant_id, status);
    }

    fn is_merchant_verified(env: Env, merchant_id: u64) -> bool {
        merchant_component::is_merchant_verified(&env, merchant_id)
    }

    fn create_invoice(
        env: Env,
        merchant: Address,
        description: String,
        amount: i128,
        token: Address,
        expires_at: Option<u64>,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        invoice_component::create_invoice(&env, &merchant, &description, amount, &token, expires_at)
    }

    fn create_fiat_invoice(
        env: Env,
        merchant: Address,
        description: String,
        fiat_amount: i128,
        fiat_currency: String,
        fiat_decimals: u32,
        token: Address,
        expires_at: Option<u64>,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        invoice_component::create_fiat_invoice(
            &env,
            &merchant,
            &description,
            fiat_amount,
            &fiat_currency,
            fiat_decimals,
            &token,
            expires_at,
        )
    }

    fn create_invoice_draft(
        env: Env,
        merchant: Address,
        description: String,
        amount: i128,
        token: Address,
        expires_at: Option<u64>,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        invoice_component::create_invoice_draft(
            &env,
            &merchant,
            &description,
            amount,
            &token,
            expires_at,
        )
    }

    fn finalize_invoice(env: Env, merchant: Address, invoice_id: u64) {
        pausable_component::assert_not_paused(&env);
        invoice_component::finalize_invoice(&env, &merchant, invoice_id);
    }

    #[allow(clippy::too_many_arguments)]
    fn create_invoice_signed(
        env: Env,
        caller: Address,
        merchant: Address,
        description: String,
        amount: i128,
        token: Address,
        nonce: BytesN<32>,
        signature: BytesN<64>,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        invoice_component::create_invoice_signed(
            &env,
            &caller,
            &merchant,
            &description,
            amount,
            &token,
            &nonce,
            &signature,
        )
    }

    fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        invoice_component::get_invoice(&env, invoice_id)
    }

    fn resolve_invoice_amount(env: Env, invoice_id: u64) -> i128 {
        invoice_component::resolve_invoice_amount(&env, invoice_id)
    }

    fn refund_invoice(env: Env, merchant: Address, invoice_id: u64) {
        pausable_component::assert_not_paused(&env);
        invoice_component::refund_invoice(&env, &merchant, invoice_id);
    }

    fn set_merchant_key(env: Env, merchant: Address, key: BytesN<32>) {
        merchant_component::set_merchant_key(&env, &merchant, &key);
    }

    fn get_merchant_key(env: Env, merchant: Address) -> BytesN<32> {
        merchant_component::get_merchant_key(&env, &merchant)
    }

    fn grant_role(env: Env, admin: Address, user: Address, role: Role) {
        access_control_component::grant_role(&env, &admin, &user, role);
    }

    fn revoke_role(env: Env, admin: Address, user: Address, role: Role) {
        access_control_component::revoke_role(&env, &admin, &user, role);
    }

    fn has_role(env: Env, user: Address, role: Role) -> bool {
        access_control_component::has_role(&env, &user, role)
    }

    fn get_invoices(env: Env, filter: InvoiceFilter) -> Vec<Invoice> {
        invoice_component::get_invoices(&env, filter)
    }

    fn refund_invoice_partial(env: Env, merchant: Address, invoice_id: u64, amount: i128) {
        pausable_component::assert_not_paused(&env);
        invoice_component::refund_invoice_partial(&env, &merchant, invoice_id, amount);
    }

    fn pause(env: Env, admin: Address) {
        pausable_component::pause(&env, &admin);
    }

    fn unpause(env: Env, admin: Address) {
        pausable_component::unpause(&env, &admin);
    }

    fn is_paused(env: Env) -> bool {
        pausable_component::is_paused(&env)
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        upgrade_component::upgrade(&env, &new_wasm_hash);
    }

    fn restrict_merchant_account(
        env: Env,
        caller: Address,
        merchant_address: Address,
        status: bool,
    ) {
        merchant_component::restrict_merchant_account(&env, &caller, &merchant_address, status);
    }

    fn calculate_fee(env: Env, merchant: Address, token: Address, amount: i128) -> i128 {
        admin_component::calculate_fee(&env, &merchant, &token, amount)
    }

    fn get_merchant_volume(env: Env, merchant: Address, token: Address) -> i128 {
        admin_component::get_merchant_volume(&env, &merchant, &token)
    }

    fn get_merchant_analytics(env: Env, merchant: Address, token: Address) -> MerchantAnalytics {
        admin_component::get_merchant_analytics(&env, &merchant, &token)
    }

    fn get_merchant_analytics_summary(env: Env, merchant: Address) -> MerchantAnalyticsSummary {
        admin_component::get_merchant_analytics_summary(&env, &merchant)
    }

    fn set_merchant_account(env: Env, merchant: Address, account: Address) {
        merchant_component::set_merchant_account(&env, &merchant, &account);
    }

    fn get_merchant_account(env: Env, merchant_id: u64) -> Address {
        merchant_component::get_merchant_account(&env, merchant_id)
    }

    fn set_auto_withdrawal_threshold(env: Env, merchant: Address, token: Address, threshold: i128) {
        pausable_component::assert_not_paused(&env);
        auto_withdrawal_component::set_auto_withdrawal_threshold(
            &env, &merchant, &token, threshold,
        );
    }

    fn get_auto_withdrawal_threshold(env: Env, merchant_id: u64, token: Address) -> Option<i128> {
        auto_withdrawal_component::get_auto_withdrawal_threshold(&env, merchant_id, &token)
    }

    fn set_auto_withdrawal_recipient(env: Env, merchant: Address, recipient: Address) {
        pausable_component::assert_not_paused(&env);
        auto_withdrawal_component::set_auto_withdrawal_recipient(&env, &merchant, &recipient);
    }

    fn get_auto_withdrawal_recipient(env: Env, merchant_id: u64) -> Option<Address> {
        auto_withdrawal_component::get_auto_withdrawal_recipient(&env, merchant_id)
    }

    fn claim_refund(env: Env, buyer: Address, invoice_id: u64) {
        pausable_component::assert_not_paused(&env);
        invoice_component::claim_refund(&env, &buyer, invoice_id);
    }

    fn pay_invoice(env: Env, payer: Address, invoice_id: u64) {
        pausable_component::assert_not_paused(&env);
        invoice_component::pay_invoice(&env, &payer, invoice_id);
    }

    fn pay_invoices_batch(env: Env, payer: Address, invoice_ids: Vec<u64>) {
        pausable_component::assert_not_paused(&env);
        invoice_component::pay_invoices_batch(&env, &payer, &invoice_ids);
    }

    fn pay_invoice_partial(env: Env, payer: Address, invoice_id: u64, amount: i128) {
        pausable_component::assert_not_paused(&env);
        invoice_component::pay_invoice_partial(&env, &payer, invoice_id, amount);
    }

    fn validate_payment_payload(env: Env, payload: crate::types::PaymentPayload) {
        crate::components::payment::validate_payment_payload(&env, &payload);
    }

    fn void_invoice(env: Env, merchant: Address, invoice_id: u64) {
        pausable_component::assert_not_paused(&env);
        invoice_component::void_invoice(&env, &merchant, invoice_id);
    }

    fn amend_invoice(
        env: Env,
        merchant: Address,
        invoice_id: u64,
        new_amount: Option<i128>,
        new_description: Option<String>,
    ) {
        pausable_component::assert_not_paused(&env);
        invoice_component::amend_invoice(&env, &merchant, invoice_id, new_amount, new_description);
    }

    fn propose_admin_transfer(env: Env, admin: Address, new_admin: Address) {
        admin_component::propose_admin_transfer(&env, &admin, &new_admin);
    }

    fn accept_admin_transfer(env: Env, new_admin: Address) {
        admin_component::accept_admin_transfer(&env, &new_admin);
    }

    // ── Subscription engine ───────────────────────────────────────────────────

    fn create_subscription_plan(
        env: Env,
        merchant: Address,
        description: String,
        token: Address,
        amount: i128,
        interval: u64,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        subscription_component::create_subscription_plan(
            &env,
            merchant,
            description,
            token,
            amount,
            interval,
        )
    }

    fn get_subscription_plan(env: Env, plan_id: u64) -> SubscriptionPlan {
        subscription_component::get_subscription_plan(&env, plan_id)
    }

    fn subscribe(env: Env, customer: Address, plan_id: u64) -> u64 {
        pausable_component::assert_not_paused(&env);
        subscription_component::subscribe(&env, customer, plan_id)
    }

    fn get_subscription(env: Env, subscription_id: u64) -> Subscription {
        subscription_component::get_subscription(&env, subscription_id)
    }

    fn charge_subscription(env: Env, subscription_id: u64) {
        pausable_component::assert_not_paused(&env);
        subscription_component::charge_subscription(&env, subscription_id);
    }

    fn cancel_subscription(env: Env, caller: Address, subscription_id: u64) {
        pausable_component::assert_not_paused(&env);
        subscription_component::cancel_subscription(&env, caller, subscription_id);
    }

    fn deactivate_plan(env: Env, caller: Address, plan_id: u64) {
        pausable_component::assert_not_paused(&env);
        subscription_component::deactivate_plan(&env, caller, plan_id);
    }

    fn set_merchant_webhook(env: Env, merchant: Address, webhook: String) {
        pausable_component::assert_not_paused(&env);
        merchant_component::set_merchant_webhook(&env, &merchant, &webhook);
    }

    fn get_merchant_webhook(env: Env, merchant_id: u64) -> String {
        merchant_component::get_merchant_webhook(&env, merchant_id)
    }

    fn set_merchant_accepted_tokens(env: Env, merchant: Address, tokens: Vec<Address>) {
        pausable_component::assert_not_paused(&env);
        merchant_component::set_merchant_accepted_tokens(&env, &merchant, &tokens);
    }

    fn get_merchant_accepted_tokens(env: Env, merchant: Address) -> Vec<Address> {
        merchant_component::get_merchant_accepted_tokens(&env, &merchant)
    }

    fn remove_merchant_accepted_token(env: Env, merchant: Address, token: Address) {
        pausable_component::assert_not_paused(&env);
        merchant_component::remove_merchant_accepted_token(&env, &merchant, &token);
    }

    fn is_token_accepted_for_merchant(env: Env, merchant: Address, token: Address) -> bool {
        merchant_component::is_token_accepted_for_merchant(&env, &merchant, &token)
    }

    fn get_user_transactions(env: Env, user: Address) -> Vec<Transaction> {
        history_component::get_user_transactions(&env, user)
    }

    fn emit_bridge_placeholder(env: Env, caller: Address, payload: CrossChainBridgePayload) {
        pausable_component::assert_not_paused(&env);
        caller.require_auth();
        events::publish_bridge_placeholder_event(&env, caller, payload, env.ledger().timestamp());
    }

    // ── Bridge listener / external deposits ──────────────────────────────────

    fn register_bridge_listener(env: Env, admin: Address, listener: Address) {
        pausable_component::assert_not_paused(&env);
        bridge_component::register_bridge_listener(&env, &admin, &listener);
    }

    fn remove_bridge_listener(env: Env, admin: Address, listener: Address) {
        pausable_component::assert_not_paused(&env);
        bridge_component::remove_bridge_listener(&env, &admin, &listener);
    }

    fn is_bridge_listener(env: Env, listener: Address) -> bool {
        bridge_component::is_bridge_listener(&env, &listener)
    }

    fn record_bridge_deposit(
        env: Env,
        listener: Address,
        source_chain: String,
        source_tx_id: BytesN<32>,
        token: Address,
        amount: i128,
        recipient: Address,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        bridge_component::record_bridge_deposit(
            &env,
            &listener,
            source_chain,
            source_tx_id,
            token,
            amount,
            recipient,
        )
    }

    fn get_bridge_deposit(env: Env, deposit_id: u64) -> Option<BridgeDeposit> {
        bridge_component::get_bridge_deposit(&env, deposit_id)
    }

    fn is_bridge_deposit_processed(env: Env, source_tx_id: BytesN<32>) -> bool {
        bridge_component::is_bridge_deposit_processed(&env, &source_tx_id)
    }

    fn get_bridge_deposit_count(env: Env) -> u64 {
        bridge_component::get_bridge_deposit_count(&env)
    }

    fn get_bridge_credit(env: Env, recipient: Address, token: Address) -> i128 {
        bridge_component::get_bridge_credit(&env, &recipient, &token)
    }

    // ── DAO governance for protocol upgrades ─────────────────────────────────

    fn add_gov_member(env: Env, admin: Address, member: Address) {
        pausable_component::assert_not_paused(&env);
        governance_component::add_gov_member(&env, &admin, &member);
    }

    fn remove_gov_member(env: Env, admin: Address, member: Address) {
        pausable_component::assert_not_paused(&env);
        governance_component::remove_gov_member(&env, &admin, &member);
    }

    fn is_gov_member(env: Env, member: Address) -> bool {
        governance_component::is_gov_member(&env, &member)
    }

    fn get_gov_member_count(env: Env) -> u32 {
        governance_component::get_gov_member_count(&env)
    }

    fn set_governance_config(env: Env, admin: Address, voting_period: u64, quorum_bps: u32) {
        pausable_component::assert_not_paused(&env);
        governance_component::set_governance_config(&env, &admin, voting_period, quorum_bps);
    }

    fn propose_upgrade(env: Env, proposer: Address, wasm_hash: BytesN<32>) -> u64 {
        pausable_component::assert_not_paused(&env);
        governance_component::propose_upgrade(&env, &proposer, wasm_hash)
    }

    fn vote_on_upgrade(env: Env, voter: Address, proposal_id: u64, approve: bool) {
        pausable_component::assert_not_paused(&env);
        governance_component::vote_on_upgrade(&env, &voter, proposal_id, approve);
    }

    fn finalize_upgrade(env: Env, caller: Address, proposal_id: u64) {
        pausable_component::assert_not_paused(&env);
        governance_component::finalize_upgrade(&env, &caller, proposal_id);
    }

    fn get_upgrade_proposal(env: Env, proposal_id: u64) -> Option<UpgradeProposal> {
        governance_component::get_upgrade_proposal(&env, proposal_id)
    }

    fn has_voted_on_upgrade(env: Env, proposal_id: u64, member: Address) -> bool {
        governance_component::has_voted(&env, proposal_id, &member)
    }

    // --- Event ticketing system ---
    #[allow(clippy::too_many_arguments)]
    fn create_event(
        env: Env,
        merchant: Address,
        name: String,
        ticket_price: i128,
        token: Address,
        capacity: u32,
        event_date: u64,
        royalty_bps: u32,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        crate::components::event::create_event(
            &env,
            &merchant,
            &name,
            &ticket_price,
            &token,
            &capacity,
            &event_date,
            &royalty_bps,
        )
    }

    fn purchase_ticket(env: Env, event_id: u64, buyer: Address) -> u64 {
        pausable_component::assert_not_paused(&env);
        crate::components::event::purchase_ticket(&env, &event_id, &buyer)
    }

    fn configure_dynamic_pricing(
        env: Env,
        merchant: Address,
        event_id: u64,
        early_bird_end: u64,
        early_bird_discount_bps: u32,
        late_markup_bps: u32,
    ) {
        pausable_component::assert_not_paused(&env);
        crate::components::event::configure_dynamic_pricing(
            &env,
            &merchant,
            event_id,
            early_bird_end,
            early_bird_discount_bps,
            late_markup_bps,
        );
    }

    fn get_current_ticket_price(env: Env, event_id: u64) -> i128 {
        crate::components::event::get_current_ticket_price(&env, event_id)
    }

    fn cancel_event_and_batch_refund(env: Env, merchant: Address, event_id: u64) {
        pausable_component::assert_not_paused(&env);
        crate::components::event::cancel_event_and_batch_refund(&env, &merchant, event_id);
    }

    fn resell_ticket(
        env: Env,
        seller: Address,
        buyer: Address,
        ticket_id: u64,
        resale_price: i128,
    ) {
        pausable_component::assert_not_paused(&env);
        crate::components::event::resell_ticket(&env, &seller, &buyer, ticket_id, resale_price);
    }

    fn get_event(env: Env, event_id: u64) -> Event {
        crate::components::event::get_event(&env, &event_id)
    }

    fn get_ticket(env: Env, ticket_id: u64) -> Ticket {
        crate::components::event::get_ticket(&env, ticket_id)
    }

    fn get_event_tickets(env: Env, event_id: u64) -> Vec<u64> {
        crate::components::event::get_event_tickets(&env, event_id)
    }

    fn get_user_tickets(env: Env, user: Address) -> Vec<u64> {
        crate::components::event::get_user_tickets(&env, &user)
    }
    fn purchase_tickets_bulk(
        env: Env,
        event_id: u64,
        buyer: Address,
        quantity: u32,
        shade_token: Address,
        merchant_account: Address,
    ) {
        pausable_component::assert_not_paused(&env);
        crate::components::event::purchase_tickets_bulk(
            &env,
            &event_id,
            &buyer,
            quantity,
            &shade_token,
            &merchant_account,
        );
    }

    fn get_token_analytics(env: Env, token: Address) -> TokenAnalytics {
        admin_component::get_token_analytics(&env, &token)
    }

    fn get_token_volume(env: Env, token: Address) -> i128 {
        admin_component::get_token_volume(&env, &token)
    }

    fn get_token_dominance_metrics(env: Env, tokens: Vec<Address>) -> Vec<(Address, i128)> {
        admin_component::get_token_dominance_metrics(&env, &tokens)
    }

    fn get_top_tokens_by_volume(env: Env, limit: u32) -> Vec<(Address, i128)> {
        admin_component::get_top_tokens_by_volume(&env, limit)
    }

    fn get_token_market_share(env: Env, token: Address) -> i128 {
        admin_component::get_token_market_share(&env, &token)
    }
}
