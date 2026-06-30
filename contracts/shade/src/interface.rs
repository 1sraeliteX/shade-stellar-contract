use crate::types::{
    BackerComment, CrossChainBridgePayload, CrowdfundVestingConfig, DynamicHardCapConfig,
    Event, HardCapVoting, Invoice, InvoiceFilter, Merchant, MerchantAnalytics,
    MerchantAnalyticsSummary, MerchantFilter, OracleConfig, PaymentPayload, PendingFee, Role,
    StretchGoal, StretchGoalReward, Subscription, SubscriptionPlan, Ticket, TokenAnalytics,
    Transaction, VestingTimeline,
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

    fn set_merchant_webhook(env: Env, merchant: Address, webhook: String);
    fn get_merchant_webhook(env: Env, merchant_id: u64) -> String;

    fn set_merchant_accepted_tokens(env: Env, merchant: Address, tokens: Vec<Address>);
    fn get_merchant_accepted_tokens(env: Env, merchant: Address) -> Vec<Address>;
    fn remove_merchant_accepted_token(env: Env, merchant: Address, token: Address);
    fn is_token_accepted_for_merchant(env: Env, merchant: Address, token: Address) -> bool;

    // ── Admin transfer (two-step handover) ───────────────────────────────────

    /// Step 1: Current admin proposes a new admin address.
    fn propose_admin_transfer(env: Env, admin: Address, new_admin: Address);

    /// Step 2: Proposed new admin accepts and takes ownership.
    fn accept_admin_transfer(env: Env, new_admin: Address);

    // ── Subscription engine ───────────────────────────────────────────────────

    /// Create a recurring billing plan.
    /// Only `merchant` can call this (requires auth). Returns new plan ID.
    fn create_subscription_plan(
        env: Env,
        merchant: Address,
        description: String,
        token: Address,
        amount: i128,
        interval: u64,
    ) -> u64;

    /// Fetch a plan by ID.
    fn get_subscription_plan(env: Env, plan_id: u64) -> SubscriptionPlan;

    /// Subscribe a customer to a plan.
    /// The customer must have already called `token.approve` to grant the Shade
    /// contract sufficient allowance for recurring charges.
    /// Returns the new subscription ID.
    fn subscribe(env: Env, customer: Address, plan_id: u64) -> u64;

    /// Fetch a subscription by ID.
    fn get_subscription(env: Env, subscription_id: u64) -> Subscription;

    /// Trigger a charge for a subscription.
    /// Callable by anyone (merchant or automated bot).
    /// Panics if the billing interval has not yet elapsed or subscription is not active.
    fn charge_subscription(env: Env, subscription_id: u64);

    /// Cancel a subscription. Either the customer or the merchant may call this.
    fn cancel_subscription(env: Env, caller: Address, subscription_id: u64);

    /// Deactivate a subscription plan so that no new customers can enroll.
    /// Only the merchant who owns the plan may call this.
    fn deactivate_plan(env: Env, caller: Address, plan_id: u64);

    /// Get all transactions executed by a specific customer address.
    fn get_user_transactions(env: Env, user: Address) -> Vec<Transaction>;

    // ── Cross-chain bridge placeholder ───────────────────────────────────────
    fn emit_bridge_placeholder(
        env: Env,
        caller: Address,
        payload: CrossChainBridgePayload,
    );

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

    /// Purchase multiple tickets in a single call.
    /// Applies automatic group discount in Shade tokens:
    /// 5–9 tickets → 5%, 10–19 → 10%, 20+ → 15%.
    fn purchase_tickets_bulk(
        env: Env,
        event_id: u64,
        buyer: Address,
        quantity: u32,
        shade_token: Address,
        merchant_account: Address,
    );

    // ── Token analytics ────────────────────────────────────────────────────────

    /// Get comprehensive analytics for a specific token
    fn get_token_analytics(env: Env, token: Address) -> TokenAnalytics;

    /// Get total volume for a specific token
    fn get_token_volume(env: Env, token: Address) -> i128;

    /// Get token dominance metrics sorted by volume (descending)
    fn get_token_dominance_metrics(env: Env, tokens: Vec<Address>) -> Vec<(Address, i128)>;

    /// Get top tokens by volume with limit
    fn get_top_tokens_by_volume(env: Env, limit: u32) -> Vec<(Address, i128)>;

    /// Get market share of a token as basis points (10000 = 100%)
    fn get_token_market_share(env: Env, token: Address) -> i128;

    // ── Vesting Timeline Configuration ─────────────────────────────────────

    /// Create a new vesting timeline for campaigns
    fn create_vesting_timeline(
        env: Env,
        admin: Address,
        name: String,
        cliff_duration: u64,
        vesting_duration: u64,
        unlock_percentage: i128,
    ) -> u64;

    /// Fetch a vesting timeline by ID
    fn get_vesting_timeline(env: Env, timeline_id: u64) -> VestingTimeline;

    /// Update vesting timeline parameters
    fn update_vesting_timeline(
        env: Env,
        admin: Address,
        timeline_id: u64,
        cliff_duration: u64,
        vesting_duration: u64,
    );

    /// Configure vesting for a crowdfund campaign
    fn configure_crowdfund_vesting(
        env: Env,
        admin: Address,
        crowdfund_id: u64,
        timeline_id: u64,
        total_vesting_amount: i128,
    );

    /// Get vesting configuration for a crowdfund
    fn get_crowdfund_vesting_config(env: Env, crowdfund_id: u64) -> CrowdfundVestingConfig;

    /// Add a vesting schedule tranche
    fn add_vesting_schedule(
        env: Env,
        admin: Address,
        timeline_id: u64,
        tranche_index: u64,
        unlock_amount: i128,
        unlock_timestamp: u64,
    );

    /// Release a vesting schedule tranche
    fn release_vesting_schedule(
        env: Env,
        admin: Address,
        timeline_id: u64,
        tranche_index: u64,
    );

    // ── Backer Comment and Feedback Moderation ──────────────────────────────

    /// Create a comment on a crowdfunding campaign
    fn create_comment(
        env: Env,
        author: Address,
        crowdfund_id: u64,
        content: String,
    ) -> u64;

    /// Get a comment by ID
    fn get_comment(env: Env, comment_id: u64) -> BackerComment;

    /// Flag a comment for moderation review
    fn flag_comment(env: Env, flagger: Address, comment_id: u64, reason: String);

    /// Remove a comment (admin only)
    fn remove_comment(env: Env, moderator: Address, comment_id: u64);

    /// Approve a flagged comment (admin only)
    fn approve_flagged_comment(env: Env, moderator: Address, comment_id: u64);

    /// Get all comments for a crowdfund campaign
    fn get_crowdfund_comments(env: Env, crowdfund_id: u64) -> Vec<u64>;

    /// Get all comments by a specific user
    fn get_user_comments(env: Env, user: Address) -> Vec<u64>;

    // ── Dynamic Hard Caps based on Community Voting ───────────────────────

    /// Initiate a voting session to adjust campaign hard cap
    fn initiate_hard_cap_voting(
        env: Env,
        crowdfund_id: u64,
        proposed_cap: i128,
        voting_duration: u64,
    );

    /// Get voting session details
    fn get_hard_cap_voting(env: Env, crowdfund_id: u64) -> HardCapVoting;

    /// Cast a vote on hard cap adjustment
    fn vote_on_hard_cap(env: Env, voter: Address, crowdfund_id: u64, support: bool);

    /// Finalize voting and execute hard cap change if passed
    fn finalize_hard_cap_voting(env: Env, admin: Address, crowdfund_id: u64);

    /// Get current dynamic hard cap configuration
    fn get_dynamic_hard_cap(env: Env, crowdfund_id: u64) -> DynamicHardCapConfig;

    /// Get current hard cap value for a crowdfund
    fn get_crowdfund_hard_cap(env: Env, crowdfund_id: u64) -> i128;

    // ── Automated Stretch Goal Unlocking ────────────────────────────────────

    /// Create a new stretch goal for a campaign
    fn create_stretch_goal(
        env: Env,
        merchant: Address,
        crowdfund_id: u64,
        target_amount: i128,
        description: String,
        reward_description: String,
    ) -> u64;

    /// Get a stretch goal by ID
    fn get_stretch_goal(env: Env, goal_id: u64) -> StretchGoal;

    /// Unlock a stretch goal when target is reached
    fn unlock_stretch_goal(env: Env, admin: Address, goal_id: u64, current_amount: i128);

    /// Distribute rewards to backers for unlocked goal
    fn distribute_stretch_goal_reward(
        env: Env,
        admin: Address,
        goal_id: u64,
        backer: Address,
        reward_amount: i128,
    );

    /// Claim stretch goal reward
    fn claim_stretch_goal_reward(env: Env, backer: Address, goal_id: u64);

    /// Get all stretch goals for a campaign
    fn get_crowdfund_stretch_goals(env: Env, crowdfund_id: u64) -> Vec<u64>;

    /// Get reward details for a stretch goal
    fn get_stretch_goal_reward(env: Env, goal_id: u64) -> StretchGoalReward;
}
