use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    Reentrancy = 4,
    MerchantAlreadyRegistered = 5,
    MerchantNotFound = 6,
    InvalidAmount = 7,
    InvoiceNotFound = 8,
    ContractPaused = 9,
    ContractNotPaused = 10,
    MerchantKeyNotFound = 11,
    TokenNotAccepted = 12,
    InvalidSignature = 13,
    NonceAlreadyUsed = 14,
    MerchantAccountNotFound = 15,
    InvalidInvoiceStatus = 16,
    RefundPeriodExpired = 17,
    WasmHashNotSet = 18,
    InvoiceAlreadyPaid = 19,
    MerchantAccountNotSet = 20,
    InvalidInterval = 21,
    PlanNotFound = 22,
    PlanNotActive = 23,
    SubscriptionNotFound = 24,
    SubscriptionNotActive = 25,
    ChargeTooEarly = 26,
    InvoiceExpired = 27,
    InvoiceNotPaid = 28,
    PayerNotAvailable = 29,
    InsufficientBalance = 30,
    InsufficientAllowance = 31,
    MerchantNotActive = 32,
    InvalidDescription = 33,
    OracleNotConfigured = 34,
    OraclePriceUnavailable = 35,
    TokenNotAcceptedByMerchant = 41,
    FeeUpdateTooEarly = 42,
    NoPendingFeeUpdate = 43,
    InvalidSwapPath = 44,
    InvalidSlippage = 45,
    EventNotFound = 46,
    EventSoldOut = 47,
    InvalidCapacity = 48,
    InvalidEventDate = 49,
    InvalidRoyaltyBps = 50,
    TicketNotFound = 51,
    NotTicketOwner = 52,
    TicketEventMismatch = 53,
    InvalidResalePrice = 54,
    /// An external deposit with this origin-chain tx hash was already credited.
    EscrowNotFound = 55,
    InvalidEscrowStatus = 56,
    CampaignNotFound = 55,
    AffiliateNotFound = 56,
    NftError = 55,
    CampaignNotFound = 55,
    InvalidRewardTier = 56,
    PledgeBelowTierMinimum = 57,
    RewardTierAtCapacity = 58,
    BackerRewardAlreadyFulfilled = 59,
    NotBacker = 60,
    CampaignEnded = 61,
    InvalidCampaignDeadline = 62,
    PerkNotFound = 63,
    PerkAlreadyClaimed = 64,
    BackerRewardNotFulfilled = 65,
    InvalidTierOrdering = 66,
    CampaignNotActive = 67,
  BridgeDepositProcessed = 68,
}

/// DAO governance errors. Kept in a separate enum (codes offset to 100+) so the
/// `ContractError` enum can stay within Soroban's hard cap of 50 cases while
/// governance still has its own distinct, unambiguous error codes.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GovernanceError {
    /// Caller is not a registered governance council member.
    NotGovMember = 100,
    /// Governance voting parameters have not been configured.
    GovNotConfigured = 101,
    /// Supplied governance config is invalid (zero period or quorum > 100%).
    InvalidGovConfig = 102,
    /// No proposal exists for the supplied id.
    ProposalNotFound = 103,
    /// The proposal is no longer open (already executed or defeated).
    ProposalNotActive = 104,
    /// The voting window for this proposal has closed.
    VotingClosed = 105,
    /// The voting window is still open; the proposal cannot be finalized yet.
    VotingStillOpen = 106,
    /// This member has already voted on the proposal.
    AlreadyVoted = 107,
}

/// Escrow / expired-refund errors. Kept in a separate enum so `ContractError`
/// can stay within Soroban's hard cap of 50 cases. The numeric codes (44/45)
/// are scoped to this enum and are only ever returned from the escrow-refund
/// path, so there is no on-chain ambiguity with `ContractError`'s own 44/45.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EscrowError {
    /// The escrow invoice has not yet reached its expiration timestamp.
    EscrowNotExpired = 44,
    /// The escrow invoice has already been fully refunded.
    EscrowAlreadyRefunded = 45,
    
}
