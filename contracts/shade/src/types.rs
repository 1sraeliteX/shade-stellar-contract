use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    FeeInBasisPoints(Address),
    FeeAmount(Address),
    ContractInfo,
    AcceptedTokens,
    Merchant(u64),
    MerchantKey(Address),
    MerchantCount,
    MerchantId(Address),
    TokenFee(Address),
    MerchantTokens(Address),
    MerchantBalance(Address),
    MerchantAccount(u64),
    Invoice(u64),
    InvoiceCount,
    ReentrancyStatus,
    AccountWasmHash,
    Role(Address, Role),
    UsedNonce(Address, BytesN<32>),
    // --- Subscription engine ---
    SubscriptionPlan(u64),
    Subscription(u64),
    PlanCount,
    SubscriptionCount,
    // --- Time-locked fee updates ---
    PendingTokenFee(Address),
    // --- Fee discount system ---
    MerchantVolume(Address, Address),
    UserTransactions(Address),
    MerchantAnalytics(Address, Address),
    MerchantAnalyticsSummary(Address),
    PlatformAccount,
    TokenOracle(Address),
    MerchantAutoWithdrawalThreshold(u64, Address),
    MerchantAutoWithdrawalRecipient(u64),
    // --- Event system ---
    Event(u64),
    EventCount,
    Ticket(u64),
    TicketCount,
    EventTickets(u64),
    UserTickets(Address),
    // --- Global token analytics ---
    TokenAnalytics(Address),
    TokenVolume(Address),
    // --- Bridge listener / external deposits ---
    /// Allowlist flag for an authorized bridge listener (relayer) address.
    BridgeListener(Address),
    /// Number of currently registered bridge listeners.
    BridgeListenerCount,
    /// Persisted external-deposit record keyed by sequential id.
    BridgeDeposit(u64),
    /// Monotonic counter of recorded external deposits.
    BridgeDepositCount,
    /// Replay-protection flag keyed by the source-chain transaction hash.
    ProcessedBridgeDeposit(BytesN<32>),
    /// Cumulative amount credited to a recipient per token via the bridge.
    BridgeCredit(Address, Address),
    // --- DAO governance for protocol upgrades ---
    /// Singleton governance state: voting params + member/proposal counters.
    /// Bundled into one key to stay within the `#[contracttype]` 50-case cap
    /// and to minimize the number of distinct storage entries.
    GovState,
    /// Allowlist flag for a governance council member.
    GovMember(Address),
    /// Persisted upgrade proposal keyed by sequential id.
    GovProposal(u64),
    /// Records a member's vote on a proposal (presence ⇒ voted).
    GovVote(u64, Address),
}

/// A single per-token auto-withdrawal threshold. Stored inside [`Merchant`] so
/// no extra `DataKey` variants are consumed (the enum is at its 50-case cap).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoWithdrawalThreshold {
    pub token: Address,
    pub threshold: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractInfo {
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merchant {
    pub id: u64,
    pub address: Address,
    pub active: bool,
    pub verified: bool,
    pub date_registered: u64,
    pub account: Address,
    pub webhook: String,
    /// Optional recipient for auto-withdrawals. Defaults to the merchant
    /// address when unset.
    pub auto_withdrawal_recipient: Option<Address>,
    /// Per-token auto-withdrawal thresholds.
    pub auto_withdrawal_thresholds: Vec<AutoWithdrawalThreshold>,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum InvoiceStatus {
    Pending = 0,
    Paid = 1,
    Cancelled = 2,
    Refunded = 3,
    PartiallyRefunded = 4,
    PartiallyPaid = 5,
    Draft = 6,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum InvoicePricingMode {
    FixedCrypto = 0,
    FixedFiat = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FiatPricing {
    pub currency: String,
    pub amount: i128,
    pub decimals: u32,
}

/// Soroban-compatible optional wrapper for FiatPricing.
/// `Option<FiatPricing>` cannot be used directly inside a `#[contracttype]`
/// struct because the SDK does not implement the required XDR conversions for
/// `Option<T>` where T is a user-defined struct. An explicit enum variant is
/// the idiomatic workaround.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FiatPricingData {
    None,
    Some(FiatPricing),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    pub id: u64,
    pub description: soroban_sdk::String,
    pub amount: i128,
    pub token: Address,
    pub status: InvoiceStatus,
    pub merchant_id: u64,
    pub payer: Option<Address>,
    pub date_created: u64,
    pub date_paid: Option<u64>,
    pub amount_paid: i128,
    pub amount_refunded: i128,
    pub expires_at: Option<u64>,
    pub pricing_mode: InvoicePricingMode,
    pub fiat_pricing: FiatPricingData,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantFilter {
    pub is_active: Option<bool>,
    pub is_verified: Option<bool>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceFilter {
    pub status: Option<u32>,
    pub merchant: Option<Address>,
    pub min_amount: Option<u128>,
    pub max_amount: Option<u128>,
    pub start_date: Option<u64>,
    pub end_date: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Manager,
    Operator,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeDiscount {
    pub min_volume: i128,
    pub discount_bps: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleConfig {
    pub contract: Address,
    pub price_decimals: u32,
    pub token_decimals: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantAnalytics {
    pub merchant: Address,
    pub token: Address,
    pub total_volume: i128,
    pub total_fees: i128,
    pub transaction_count: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantAnalyticsSummary {
    pub merchant: Address,
    pub total_volume: i128,
    pub total_fees: i128,
    pub transaction_count: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainBridgePayload {
    pub invoice_id: u64,
    pub merchant: Address,
    pub payer: Option<Address>,
    pub source_chain: String,
    pub destination_chain: String,
    pub token: Address,
    pub amount: i128,
    pub destination_recipient: String,
    pub memo: Option<String>,
}

/// A confirmed external-chain deposit recorded by an authorized bridge listener.
///
/// The `source_tx_id` is the 32-byte transaction hash on the origin chain and
/// doubles as the global idempotency key (see `DataKey::ProcessedBridgeDeposit`).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeDeposit {
    pub id: u64,
    pub source_chain: String,
    pub source_tx_id: BytesN<32>,
    pub listener: Address,
    pub token: Address,
    pub amount: i128,
    pub recipient: Address,
    pub timestamp: u64,
}

// ── Time-locked fee update ────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingFee {
    pub token: Address,
    pub fee: i128,
    pub proposed_at: u64,
}

// --- Subscription engine ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionPlan {
    pub id: u64,
    /// Numeric merchant ID - used to look up the merchant's account contract.
    pub merchant_id: u64,
    /// The merchant's wallet address - needed for event emission and auth checks.
    pub merchant: Address,
    /// Human-readable description of the plan.
    pub description: soroban_sdk::String,
    /// Token used for billing.
    pub token: Address,
    /// Amount charged per interval (in token base units).
    pub amount: i128,
    /// Billing interval in seconds (e.g. 2_592_000 = 30 days).
    pub interval: u64,
    /// Whether this plan is accepting new subscribers.
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub id: u64,
    pub plan_id: u64,
    pub customer: Address,
    /// Copied from the plan for quick access during auth checks.
    pub merchant_id: u64,
    pub status: SubscriptionStatus,
    pub date_created: u64,
    /// Ledger timestamp of the last successful charge.
    /// Starts at 0 so the first charge is available immediately.
    pub last_charged: u64,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SubscriptionStatus {
    Active = 0,
    Cancelled = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenAnalytics {
    pub token: Address,
    pub total_volume: i128,
    pub total_fees: i128,
    pub transaction_count: u64,
    pub unique_merchants: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TransactionType {
    InvoicePayment = 0,
    SubscriptionCharge = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub ref_id: u64,
    pub amount: i128,
    pub token: Address,
    pub description: soroban_sdk::String,
    pub date: u64,
    pub merchant_id: u64,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EventStatus {
    Active = 0,
    Cancelled = 1,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EscrowStatus {
    Created = 0,
    Funded = 1,
    Released = 2,
    Refunded = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
    pub invoice_id: Option<u64>,
    pub date_created: u64,
    pub date_funded: Option<u64>,
    pub date_released: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub id: u64,
    pub merchant_id: u64,
    pub name: String,
    pub ticket_price: i128,
    pub token: Address,
    pub capacity: u32,
    pub sold: u32,
    pub date: u64,
    /// Scheduled event date (unix seconds). Must be >= ledger timestamp at creation.
    pub event_date: u64,
    /// Royalty paid to the organizer on each resale, in basis points (10_000 = 100%).
    pub royalty_bps: u32,
    /// Early-bird cutoff timestamp. `0` disables early-bird pricing.
    pub early_bird_end: u64,
    /// Discount during early-bird period, in basis points.
    pub early_bird_discount_bps: u32,
    /// Markup applied after early-bird period, in basis points.
    pub late_markup_bps: u32,
    /// True once the event is cancelled.
    pub cancelled: bool,
    /// True once all ticket refunds have been processed.
    pub refunds_processed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ticket {
    pub id: u64,
    pub event_id: u64,
    pub owner: Address,
    pub minted_at: u64,
    /// Amount paid on primary purchase, used for cancellation refunds.
    pub purchase_price: i128,
}

// ── Campaign announcements (Issue #335) ──────────────────────────────────────

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CampaignStatus {
    Active = 0,
    Ended = 1,
    Cancelled = 2,
}

/// On-chain fundraising / promotional campaign created by a merchant.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Campaign {
    pub id: u64,
    pub merchant_id: u64,
    pub merchant: Address,
    pub title: String,
    pub description: String,
    /// Fundraising goal in token base units. 0 = open-ended (no specific goal).
    pub goal_amount: i128,
    pub token: Address,
    pub status: CampaignStatus,
    pub created_at: u64,
    pub updated_at: u64,
    /// Unix timestamp when the campaign stops accepting new backers.
    pub end_date: u64,
}

/// A timestamped update / news post published by the merchant on an active campaign.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignAnnouncement {
    pub id: u64,
    pub campaign_id: u64,
    pub title: String,
    pub content: String,
    pub posted_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentRoute {
    Direct,
    Swap(SwapRoute),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapRoute {
    pub router: Address,
    pub path: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentPayload {
    pub input_token: Address,
    pub settlement_token: Address,
    pub route: PaymentRoute,
    pub max_slippage_bps: Option<u32>,
}

// ── DAO governance for protocol upgrades ──────────────────────────────────────

/// Singleton governance configuration and counters. `voting_period == 0` is the
/// sentinel for "not yet configured".
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovState {
    pub voting_period: u64,
    pub quorum_bps: u32,
    pub member_count: u32,
    pub proposal_count: u64,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalStatus {
    /// Open for voting.
    Active = 0,
    /// Passed quorum + majority and the upgrade was applied.
    Executed = 1,
    /// Failed quorum or majority after the voting window closed.
    Defeated = 2,
}

/// A council-governed proposal to upgrade the contract's WASM to `wasm_hash`.
/// Voting is one-member-one-vote; `approvals`/`rejections` are head counts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    pub id: u64,
    pub proposer: Address,
    pub wasm_hash: BytesN<32>,
    pub created_at: u64,
    pub voting_ends_at: u64,
    pub approvals: u32,
    pub rejections: u32,
    pub status: ProposalStatus,
}
