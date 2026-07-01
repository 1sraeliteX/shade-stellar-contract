use crate::types::{
    Campaign, CampaignAffiliate, CampaignParticipant, CrossChainBridgePayload, Event, Invoice,
    InvoiceFilter, Merchant, MerchantAnalytics, MerchantAnalyticsSummary, MerchantFilter,
    OracleConfig, PaymentPayload, PendingFee, Role, Subscription, SubscriptionPlan, Ticket,
    TokenAnalytics, Transaction,
};
use soroban_sdk::{contracttrait, Address, BytesN, Env, String, Vec};

#[contracttrait]
pub trait ShadeTrait {
    fn initialize(env: Env, admin: Address);
    fn get_admin(env: Env) -> Address;
    fn add_accepted_token(env: Env, admin: Address, token: Address);
    fn add_accepted_tokens(env: Env, admin: Address, tokens: Vec<Address>);
    fn remove_accepted_token(env: Env, admin: Address, token: Address);
    fn is_accepted_token(env: Env, token: Address) -> bool;
    fn set_account_wasm_hash(env: Env, admin: Address, wasm_hash: soroban_sdk::BytesN<32>);
    fn set_fee(env: Env, admin: Address, token: Address, fee: i128);
    fn get_fee(env: Env, token: Address) -> i128;
    fn set_platform_account(env: Env, admin: Address, account: Address);
    fn get_platform_account(env: Env) -> Address;
    fn set_token_oracle(env: Env, admin: Address, token: Address, oracle: OracleConfig);
    fn get_token_oracle(env: Env, token: Address) -> OracleConfig;
    fn propose_fee(env: Env, admin: Address, token: Address, fee: i128);
    fn execute_fee(env: Env, admin: Address, token: Address);
    fn get_pending_fee(env: Env, token: Address) -> PendingFee;
    fn register_merchant(env: Env, merchant: Address);
    fn get_merchant(env: Env, merchant_id: u64) -> Merchant;
    fn get_merchants(env: Env, filter: MerchantFilter) -> Vec<Merchant>;
    fn is_merchant(env: Env, merchant: Address) -> bool;
    fn set_merchant_status(env: Env, admin: Address, merchant_id: u64, status: bool);
    fn is_merchant_active(env: Env, merchant_id: u64) -> bool;
    fn verify_merchant(env: Env, admin: Address, merchant_id: u64, status: bool);
    fn is_merchant_verified(env: Env, merchant_id: u64) -> bool;
    fn create_invoice(
        env: Env,
        merchant: Address,
        description: String,
        amount: i128,
        token: Address,
        expires_at: Option<u64>,
    ) -> u64;
    fn create_fiat_invoice(
        env: Env,
        merchant: Address,
        description: String,
        fiat_amount: i128,
        fiat_currency: String,
        fiat_decimals: u32,
        token: Address,
        expires_at: Option<u64>,
    ) -> u64;
    fn create_invoice_draft(
        env: Env,
        merchant: Address,
        description: String,
        amount: i128,
        token: Address,
        expires_at: Option<u64>,
    ) -> u64;
    fn finalize_invoice(env: Env, merchant: Address, invoice_id: u64);
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
    ) -> u64;
    fn get_invoice(env: Env, invoice_id: u64) -> Invoice;
    fn resolve_invoice_amount(env: Env, invoice_id: u64) -> i128;
    fn refund_invoice(env: Env, merchant: Address, invoice_id: u64);
    fn set_merchant_key(env: Env, merchant: Address, key: BytesN<32>);
    fn get_merchant_key(env: Env, merchant: Address) -> BytesN<32>;
    fn grant_role(env: Env, admin: Address, user: Address, role: Role);
    fn revoke_role(env: Env, admin: Address, user: Address, role: Role);
    fn has_role(env: Env, user: Address, role: Role) -> bool;
    fn get_invoices(env: Env, filter: InvoiceFilter) -> Vec<Invoice>;
    fn refund_invoice_partial(env: Env, merchant: Address, invoice_id: u64, amount: i128);
    fn pause(env: Env, admin: Address);
    fn unpause(env: Env, admin: Address);
    fn is_paused(env: Env) -> bool;
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
    fn restrict_merchant_account(
        env: Env,
        caller: Address,
        merchant_address: Address,
        status: bool,
    );
    fn calculate_fee(env: Env, merchant: Address, token: Address, amount: i128) -> i128;
    fn get_merchant_volume(env: Env, merchant: Address, token: Address) -> i128;
    fn get_merchant_analytics(env: Env, merchant: Address, token: Address) -> MerchantAnalytics;
    fn get_merchant_analytics_summary(env: Env, merchant: Address) -> MerchantAnalyticsSummary;
    fn set_merchant_account(env: Env, merchant: Address, account: Address);
    fn get_merchant_account(env: Env, merchant_id: u64) -> Address;
    fn pay_invoice(env: Env, payer: Address, invoice_id: u64);
    fn pay_invoices_batch(env: Env, payer: Address, invoice_ids: Vec<u64>);
    fn pay_invoice_partial(env: Env, payer: Address, invoice_id: u64, amount: i128);
    fn validate_payment_payload(env: Env, payload: PaymentPayload);
    fn void_invoice(env: Env, merchant: Address, invoice_id: u64);
    fn amend_invoice(
        env: Env,
        merchant: Address,
        invoice_id: u64,
        new_amount: Option<i128>,
        new_description: Option<String>,
    );

    fn propose_admin_transfer(env: Env, admin: Address, new_admin: Address);
    fn accept_admin_transfer(env: Env, new_admin: Address);

    fn create_subscription_plan(
        env: Env,
        merchant: Address,
        description: String,
        token: Address,
        amount: i128,
        interval: u64,
    ) -> u64;
    fn get_subscription_plan(env: Env, plan_id: u64) -> SubscriptionPlan;
    fn subscribe(env: Env, customer: Address, plan_id: u64) -> u64;
    fn get_subscription(env: Env, subscription_id: u64) -> Subscription;
    fn charge_subscription(env: Env, subscription_id: u64);
    fn cancel_subscription(env: Env, caller: Address, subscription_id: u64);
    fn deactivate_plan(env: Env, caller: Address, plan_id: u64);
    fn set_merchant_webhook(env: Env, merchant: Address, webhook: String);
    fn get_merchant_webhook(env: Env, merchant_id: u64) -> String;
    fn set_merchant_accepted_tokens(env: Env, merchant: Address, tokens: Vec<Address>);
    fn get_merchant_accepted_tokens(env: Env, merchant: Address) -> Vec<Address>;
    fn remove_merchant_accepted_token(env: Env, merchant: Address, token: Address);
    fn is_token_accepted_for_merchant(env: Env, merchant: Address, token: Address) -> bool;
    fn get_user_transactions(env: Env, user: Address) -> Vec<Transaction>;
    fn emit_bridge_placeholder(env: Env, caller: Address, payload: CrossChainBridgePayload);

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
    ) -> u64;
    fn purchase_ticket(env: Env, event_id: u64, buyer: Address) -> u64;
    fn configure_dynamic_pricing(
        env: Env,
        merchant: Address,
        event_id: u64,
        early_bird_end: u64,
        early_bird_discount_bps: u32,
        late_markup_bps: u32,
    );
    fn get_current_ticket_price(env: Env, event_id: u64) -> i128;
    fn cancel_event_and_batch_refund(env: Env, merchant: Address, event_id: u64);
    fn resell_ticket(
        env: Env,
        seller: Address,
        buyer: Address,
        ticket_id: u64,
        resale_price: i128,
    );
    fn get_event(env: Env, event_id: u64) -> Event;
    fn get_ticket(env: Env, ticket_id: u64) -> Ticket;
    fn get_event_tickets(env: Env, event_id: u64) -> Vec<u64>;
    fn get_user_tickets(env: Env, user: Address) -> Vec<u64>;
    fn purchase_tickets_bulk(
        env: Env,
        event_id: u64,
        buyer: Address,
        quantity: u32,
        shade_token: Address,
        merchant_account: Address,
    );

    fn get_token_analytics(env: Env, token: Address) -> TokenAnalytics;
    fn get_token_volume(env: Env, token: Address) -> i128;
    fn get_token_dominance_metrics(env: Env, tokens: Vec<Address>) -> Vec<(Address, i128)>;
    fn get_top_tokens_by_volume(env: Env, limit: u32) -> Vec<(Address, i128)>;
    fn get_token_market_share(env: Env, token: Address) -> i128;

    fn create_campaign(
        env: Env,
        caller: Address,
        name: String,
        charity: bool,
        fee_waiver_bps: u32,
        discount_bps: u32,
        stake_required: i128,
    ) -> u64;
    fn configure_campaign_fee_policy(
        env: Env,
        caller: Address,
        campaign_id: u64,
        fee_waiver_bps: u32,
        discount_bps: u32,
    );
    fn calculate_campaign_discounted_amount(env: Env, campaign_id: u64, amount: i128) -> i128;
    fn record_campaign_contribution(env: Env, caller: Address, campaign_id: u64, amount: i128);
    fn stake_campaign(env: Env, caller: Address, campaign_id: u64, amount: i128);
    fn slash_campaign_stake(
        env: Env,
        caller: Address,
        campaign_id: u64,
        participant: Address,
        amount: i128,
    );
    fn register_affiliate(
        env: Env,
        caller: Address,
        campaign_id: u64,
        affiliate: Address,
        commission_bps: u32,
    );
    fn pay_affiliate_commission(
        env: Env,
        caller: Address,
        campaign_id: u64,
        affiliate: Address,
        amount: i128,
    );
    fn get_campaign(env: Env, campaign_id: u64) -> Campaign;
    fn get_campaign_participant(env: Env, campaign_id: u64, participant: Address) -> CampaignParticipant;
    fn get_campaign_affiliate(env: Env, campaign_id: u64, affiliate: Address) -> CampaignAffiliate;
    fn get_campaign_leaderboard(env: Env, campaign_id: u64, limit: u32) -> Vec<(Address, i128)>;
}
