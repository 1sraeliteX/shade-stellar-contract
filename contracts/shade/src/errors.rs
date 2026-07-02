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
    NonceAlreadyUsed = 14,
    InvalidInvoiceStatus = 16,
    RefundPeriodExpired = 17,
    WasmHashNotSet = 18,
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
    InvalidResalePrice = 54,
    NotFound = 55,
    // ── Multi-sig massive withdrawal ─────────────────────────────────────────
    /// The withdrawal amount is below the configured threshold; no multi-sig needed.
    BelowMultiSigThreshold = 55,
    /// No signers have been registered for multi-sig.
    MultiSigSignersNotSet = 56,
    /// The quorum value must be > 0 and ≤ the number of registered signers.
    InvalidQuorum = 57,
    /// The caller is not in the registered signer list.
    NotASigner = 58,
    /// This signer has already approved the given proposal.
    AlreadyApproved = 59,
    /// The referenced withdrawal proposal does not exist.
    ProposalNotFound = 60,
    /// The proposal is not in the Pending state and cannot be acted on.
    ProposalNotPending = 61,
    /// The proposal has not yet collected enough approvals for execution.
    QuorumNotReached = 62,
    /// Only the original proposer (merchant) may cancel their own proposal.
    NotProposer = 63,
    /// The multi-sig threshold has not been configured for this token.
    ThresholdNotSet = 64,
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
