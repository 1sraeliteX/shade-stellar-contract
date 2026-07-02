use crate::components::{
    nft as nft_component,
    access_control as access_control_component, admin as admin_component, core as core_component,
    invoice as invoice_component, merchant as merchant_component, pausable as pausable_component,
    subscription as subscription_component, upgrade as upgrade_component,
    history as history_component, escrow as escrow_component,
};
use crate::errors::ContractError;
use crate::events;
use crate::shade_interface::ShadeTrait;
use crate::types::{
    ContractInfo, CrossChainBridgePayload, DataKey, Event, Invoice, InvoiceFilter, Merchant,
    MerchantAnalytics, MerchantAnalyticsSummary, MerchantFilter, OracleConfig, PaymentPayload,
    PendingFee, Role, Subscription, SubscriptionPlan, Ticket, TokenAnalytics, Transaction, Escrow,
    BackerCampaign, BackerRewardTier, ContractInfo, CrossChainBridgePayload, DataKey, Event, Invoice,
    InvoiceFilter, Merchant, Nft, NftCollection, MerchantAnalytics, MerchantAnalyticsSummary, MerchantFilter,
    OracleConfig, PaymentPayload, PendingFee, Role, Subscription, SubscriptionPlan, Ticket,
    TokenAnalytics, Transaction,
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
        admin_component::get_token_market_share(&env, token)
    }

    fn create_escrow(
        env: Env,
        seller: Address,
        buyer: Address,
        token: Address,
        amount: i128,
        invoice_id: Option<u64>,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        escrow_component::create_escrow(&env, &seller, &buyer, &token, amount, invoice_id)
    }

    fn get_escrow(env: Env, escrow_id: u64) -> Escrow {
        escrow_component::get_escrow(&env, escrow_id)
    }

    fn fund_escrow(env: Env, buyer: Address, escrow_id: u64) {
        pausable_component::assert_not_paused(&env);
        escrow_component::fund_escrow(&env, &buyer, escrow_id)
    }

    fn release_escrow(env: Env, buyer: Address, escrow_id: u64) {
        pausable_component::assert_not_paused(&env);
        escrow_component::release_escrow(&env, &buyer, escrow_id)
    }

    fn refund_escrow(env: Env, seller: Address, escrow_id: u64) {
        pausable_component::assert_not_paused(&env);
        escrow_component::refund_escrow(&env, &seller, escrow_id)
    }

    // ── NFT minting & distribution ────────────────────────────────────────────

    fn create_nft_collection(
        env: Env,
        merchant: Address,
        name: String,
        base_uri: String,
        max_supply: u64,
        royalty_bps: u32,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        nft_component::create_nft_collection(&env, &merchant, &name, &base_uri, max_supply, royalty_bps)
    }

    fn mint_nft(
        env: Env,
        merchant: Address,
        collection_id: u64,
        recipient: Address,
        token_uri: String,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        nft_component::mint_nft(&env, &merchant, collection_id, &recipient, &token_uri)
    }

    fn batch_mint_nfts(
        env: Env,
        merchant: Address,
        collection_id: u64,
        recipients: Vec<Address>,
        token_uris: Vec<String>,
    ) -> Vec<u64> {
        pausable_component::assert_not_paused(&env);
        nft_component::batch_mint_nfts(&env, &merchant, collection_id, &recipients, &token_uris)
    }

    fn transfer_nft(env: Env, from: Address, to: Address, nft_id: u64) {
        pausable_component::assert_not_paused(&env);
        nft_component::transfer_nft(&env, &from, &to, nft_id)
    }

    fn burn_nft(env: Env, owner: Address, nft_id: u64) {
        pausable_component::assert_not_paused(&env);
        nft_component::burn_nft(&env, &owner, nft_id)
    }

    fn claim_nft_reward(env: Env, claimer: Address, nft_id: u64) {
        pausable_component::assert_not_paused(&env);
        nft_component::claim_nft_reward(&env, &claimer, nft_id)
    }

    fn deactivate_nft_collection(env: Env, merchant: Address, collection_id: u64) {
        pausable_component::assert_not_paused(&env);
        nft_component::deactivate_nft_collection(&env, &merchant, collection_id)
    }

    fn get_nft_collection(env: Env, collection_id: u64) -> NftCollection {
        nft_component::get_nft_collection(&env, collection_id)
    }

    fn get_nft(env: Env, nft_id: u64) -> Nft {
        nft_component::get_nft(&env, nft_id)
    }

    fn get_collection_nfts(env: Env, collection_id: u64) -> Vec<u64> {
        nft_component::get_collection_nfts(&env, collection_id)
    }

    fn get_user_nfts(env: Env, user: Address) -> Vec<u64> {
        nft_component::get_user_nfts(&env, &user)
    }
    fn create_backer_campaign(
        env: Env,
        merchant: Address,
        name: String,
        token: Address,
        deadline: u64,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        crate::components::backer_rewards::create_backer_campaign(
            &env, merchant, name, token, deadline,
        )
    }

    fn get_backer_campaign(env: Env, campaign_id: u64) -> BackerCampaign {
        crate::components::backer_rewards::get_backer_campaign(&env, campaign_id)
    }

    fn set_backer_reward_tiers(
        env: Env,
        merchant: Address,
        campaign_id: u64,
        tiers: Vec<BackerRewardTier>,
    ) {
        pausable_component::assert_not_paused(&env);
        crate::components::backer_rewards::set_backer_reward_tiers(
            &env, merchant, campaign_id, tiers,
        );
    }

    fn get_backer_reward_tiers(env: Env, campaign_id: u64) -> Vec<BackerRewardTier> {
        crate::components::backer_rewards::get_backer_reward_tiers(&env, campaign_id)
    }

    fn pledge_to_campaign(env: Env, backer: Address, campaign_id: u64, amount: i128) {
        pausable_component::assert_not_paused(&env);
        crate::components::backer_rewards::pledge_to_campaign(&env, backer, campaign_id, amount);
    }

    fn get_backer_pledge(env: Env, campaign_id: u64, backer: Address) -> i128 {
        crate::components::backer_rewards::get_backer_pledge(&env, campaign_id, backer)
    }

    fn select_backer_reward_tier(
        env: Env,
        backer: Address,
        campaign_id: u64,
        tier_index: u32,
    ) {
        pausable_component::assert_not_paused(&env);
        crate::components::backer_rewards::select_backer_reward_tier(
            &env, backer, campaign_id, tier_index,
        );
    }

    fn get_backer_selected_tier(env: Env, campaign_id: u64, backer: Address) -> Option<u32> {
        crate::components::backer_rewards::get_backer_selected_tier(&env, campaign_id, backer)
    }

    fn fulfill_backer_reward(
        env: Env,
        merchant: Address,
        campaign_id: u64,
        backer: Address,
    ) {
        pausable_component::assert_not_paused(&env);
        crate::components::backer_rewards::fulfill_backer_reward(
            &env, merchant, campaign_id, backer,
        );
    }

    fn is_backer_reward_fulfilled(env: Env, campaign_id: u64, backer: Address) -> bool {
        crate::components::backer_rewards::is_backer_reward_fulfilled(&env, campaign_id, backer)
    }

    fn claim_backer_perk(env: Env, backer: Address, campaign_id: u64, perk_index: u32) {
        pausable_component::assert_not_paused(&env);
        crate::components::backer_rewards::claim_backer_perk(&env, backer, campaign_id, perk_index);
    }

    fn is_backer_perk_claimed(
        env: Env,
        campaign_id: u64,
        backer: Address,
        perk_index: u32,
    ) -> bool {
        crate::components::backer_rewards::is_backer_perk_claimed(
            &env, campaign_id, backer, perk_index,
        )
    }

    fn create_vesting_timeline(
        env: Env,
        admin: Address,
        name: String,
        cliff_duration: u64,
        vesting_duration: u64,
        unlock_percentage: i128,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        vesting_component::create_vesting_timeline(
            &env,
            admin,
            name,
            cliff_duration,
            vesting_duration,
            unlock_percentage,
        )
    }

    fn get_vesting_timeline(env: Env, timeline_id: u64) -> VestingTimeline {
        vesting_component::get_vesting_timeline(&env, timeline_id)
    }

    fn update_vesting_timeline(
        env: Env,
        admin: Address,
        timeline_id: u64,
        cliff_duration: u64,
        vesting_duration: u64,
    ) {
        pausable_component::assert_not_paused(&env);
        vesting_component::update_vesting_timeline(
            &env,
            admin,
            timeline_id,
            cliff_duration,
            vesting_duration,
        )
    }

    fn configure_crowdfund_vesting(
        env: Env,
        admin: Address,
        crowdfund_id: u64,
        timeline_id: u64,
        total_vesting_amount: i128,
    ) {
        pausable_component::assert_not_paused(&env);
        vesting_component::configure_crowdfund_vesting(&env, admin, crowdfund_id, timeline_id, total_vesting_amount)
    }

    fn get_crowdfund_vesting_config(env: Env, crowdfund_id: u64) -> CrowdfundVestingConfig {
        vesting_component::get_crowdfund_vesting_config(&env, crowdfund_id)
    }

    fn add_vesting_schedule(
        env: Env,
        admin: Address,
        timeline_id: u64,
        tranche_index: u64,
        unlock_amount: i128,
        unlock_timestamp: u64,
    ) {
        pausable_component::assert_not_paused(&env);
        vesting_component::add_vesting_schedule(
            &env,
            admin,
            timeline_id,
            tranche_index,
            unlock_amount,
            unlock_timestamp,
        )
    }

    fn release_vesting_schedule(
        env: Env,
        admin: Address,
        timeline_id: u64,
        tranche_index: u64,
    ) {
        pausable_component::assert_not_paused(&env);
        vesting_component::release_vesting_schedule(&env, admin, timeline_id, tranche_index)
    }

    fn create_comment(
        env: Env,
        author: Address,
        crowdfund_id: u64,
        content: String,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        comments_component::create_comment(&env, author, crowdfund_id, content)
    }

    fn get_comment(env: Env, comment_id: u64) -> BackerComment {
        comments_component::get_comment(&env, comment_id)
    }

    fn flag_comment(env: Env, flagger: Address, comment_id: u64, reason: String) {
        pausable_component::assert_not_paused(&env);
        comments_component::flag_comment(&env, flagger, comment_id, reason)
    }

    fn remove_comment(env: Env, moderator: Address, comment_id: u64) {
        pausable_component::assert_not_paused(&env);
        comments_component::remove_comment(&env, moderator, comment_id)
    }

    fn approve_flagged_comment(env: Env, moderator: Address, comment_id: u64) {
        pausable_component::assert_not_paused(&env);
        comments_component::approve_flagged_comment(&env, moderator, comment_id)
    }

    fn get_crowdfund_comments(env: Env, crowdfund_id: u64) -> Vec<u64> {
        comments_component::get_crowdfund_comments(&env, crowdfund_id)
    }

    fn get_user_comments(env: Env, user: Address) -> Vec<u64> {
        comments_component::get_user_comments(&env, user)
    }

    fn initiate_hard_cap_voting(
        env: Env,
        crowdfund_id: u64,
        proposed_cap: i128,
        voting_duration: u64,
    ) {
        pausable_component::assert_not_paused(&env);
        voting_component::initiate_hard_cap_voting(&env, crowdfund_id, proposed_cap, voting_duration)
    }

    fn get_hard_cap_voting(env: Env, crowdfund_id: u64) -> HardCapVoting {
        voting_component::get_hard_cap_voting(&env, crowdfund_id)
    }

    fn vote_on_hard_cap(env: Env, voter: Address, crowdfund_id: u64, support: bool) {
        pausable_component::assert_not_paused(&env);
        voting_component::vote_on_hard_cap(&env, voter, crowdfund_id, support)
    }

    fn finalize_hard_cap_voting(env: Env, admin: Address, crowdfund_id: u64) {
        pausable_component::assert_not_paused(&env);
        voting_component::finalize_hard_cap_voting(&env, admin, crowdfund_id)
    }

    fn get_dynamic_hard_cap(env: Env, crowdfund_id: u64) -> DynamicHardCapConfig {
        voting_component::get_dynamic_hard_cap(&env, crowdfund_id)
    }

    fn get_crowdfund_hard_cap(env: Env, crowdfund_id: u64) -> i128 {
        voting_component::get_crowdfund_hard_cap(&env, crowdfund_id)
    }

    fn create_stretch_goal(
        env: Env,
        merchant: Address,
        crowdfund_id: u64,
        target_amount: i128,
        description: String,
        reward_description: String,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        stretch_goals_component::create_stretch_goal(
            &env,
            merchant,
            crowdfund_id,
            target_amount,
            description,
            reward_description,
        )
    }

    fn get_stretch_goal(env: Env, goal_id: u64) -> StretchGoal {
        stretch_goals_component::get_stretch_goal(&env, goal_id)
    }

    fn unlock_stretch_goal(env: Env, admin: Address, goal_id: u64, current_amount: i128) {
        pausable_component::assert_not_paused(&env);
        stretch_goals_component::unlock_stretch_goal(&env, admin, goal_id, current_amount)
    }

    fn distribute_stretch_goal_reward(
        env: Env,
        admin: Address,
        goal_id: u64,
        backer: Address,
        reward_amount: i128,
    ) {
        pausable_component::assert_not_paused(&env);
        stretch_goals_component::distribute_stretch_goal_reward(
            &env,
            admin,
            goal_id,
            backer,
            reward_amount,
        )
    }

    fn claim_stretch_goal_reward(env: Env, backer: Address, goal_id: u64) {
        pausable_component::assert_not_paused(&env);
        stretch_goals_component::claim_stretch_goal_reward(&env, backer, goal_id)
    }

    fn get_crowdfund_stretch_goals(env: Env, crowdfund_id: u64) -> Vec<u64> {
        stretch_goals_component::get_crowdfund_stretch_goals(&env, crowdfund_id)
    }

    fn get_stretch_goal_reward(env: Env, goal_id: u64) -> StretchGoalReward {
        stretch_goals_component::get_stretch_goal_reward(&env, goal_id)
    }

    // ── Pledge / crowdfund campaign system ────────────────────────────────────

    fn create_campaign(
        env: Env,
        merchant: Address,
        title: String,
        goal: i128,
        token: Address,
        deadline: u64,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        pledge_component::create_campaign(&env, &merchant, &title, goal, &token, deadline)
    }

    fn get_campaign(env: Env, campaign_id: u64) -> Campaign {
        pledge_component::get_campaign(&env, campaign_id)
    }

    fn pledge(
        env: Env,
        contributor: Address,
        campaign_id: u64,
        amount: i128,
        token: Address,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        pledge_component::pledge(&env, &contributor, campaign_id, amount, &token)
    }

    fn execute_campaign(env: Env, merchant: Address, campaign_id: u64) {
        pausable_component::assert_not_paused(&env);
        pledge_component::execute_campaign(&env, &merchant, campaign_id);
    }

    fn cancel_campaign(env: Env, merchant: Address, campaign_id: u64) {
        pausable_component::assert_not_paused(&env);
        pledge_component::cancel_campaign(&env, &merchant, campaign_id);
    }

    fn claim_refund(env: Env, contributor: Address, campaign_id: u64) {
        pausable_component::assert_not_paused(&env);
        pledge_component::claim_refund(&env, &contributor, campaign_id);
    }

    fn batch_refund(env: Env, campaign_id: u64) {
        pausable_component::assert_not_paused(&env);
        pledge_component::batch_refund(&env, campaign_id);
    }

    fn get_pledge(env: Env, pledge_id: u64) -> Pledge {
        pledge_component::get_pledge(&env, pledge_id)
    }

    fn get_campaign_pledges(env: Env, campaign_id: u64) -> Vec<Pledge> {
        pledge_component::get_campaign_pledges(&env, campaign_id)
    }

    fn get_contributor_pledges(env: Env, contributor: Address) -> Vec<Pledge> {
        pledge_component::get_contributor_pledges(&env, &contributor)
    }

    // ── Campaign announcements (Issue #335) ───────────────────────────────────

    fn create_campaign(
        env: Env,
        merchant: Address,
        title: String,
        description: String,
        goal_amount: i128,
        token: Address,
        end_date: u64,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        campaign_component::create_campaign(
            &env,
            &merchant,
            &title,
            &description,
            goal_amount,
            &token,
            end_date,
        )
    }

    fn get_campaign(env: Env, campaign_id: u64) -> Campaign {
        campaign_component::get_campaign(&env, campaign_id)
    }

    fn get_merchant_campaigns(env: Env, merchant: Address) -> Vec<u64> {
        campaign_component::get_merchant_campaigns(&env, &merchant)
    }

    fn update_campaign(
        env: Env,
        merchant: Address,
        campaign_id: u64,
        title: String,
        description: String,
        end_date: u64,
    ) {
        pausable_component::assert_not_paused(&env);
        campaign_component::update_campaign(&env, &merchant, campaign_id, &title, &description, end_date);
    }

    fn cancel_campaign(env: Env, caller: Address, campaign_id: u64) {
        pausable_component::assert_not_paused(&env);
        campaign_component::cancel_campaign(&env, &caller, campaign_id);
    }

    fn end_campaign(env: Env, merchant: Address, campaign_id: u64) {
        pausable_component::assert_not_paused(&env);
        campaign_component::end_campaign(&env, &merchant, campaign_id);
    }

    fn post_campaign_announcement(
        env: Env,
        merchant: Address,
        campaign_id: u64,
        title: String,
        content: String,
    ) -> u64 {
        pausable_component::assert_not_paused(&env);
        campaign_component::post_campaign_announcement(&env, &merchant, campaign_id, &title, &content)
    }

    fn get_campaign_announcements(env: Env, campaign_id: u64) -> Vec<CampaignAnnouncement> {
        campaign_component::get_campaign_announcements(&env, campaign_id)
    }
}
