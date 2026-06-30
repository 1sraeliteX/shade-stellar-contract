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
    // ── Campaign categories & tagging (#352) ──────────────────────────────
    /// Referenced campaign category does not exist.
    CampaignCategoryNotFound = 55,
    /// A category with the supplied name has already been registered.
    CampaignCategoryAlreadyExists = 56,
    /// Referenced campaign category exists but is not active.
    CampaignCategoryInactive = 57,
    /// Referenced campaign tag does not exist.
    CampaignTagNotFound = 58,
    /// A tag with the supplied name has already been registered.
    CampaignTagAlreadyExists = 59,
    /// Referenced campaign does not exist.
    CampaignNotFound = 60,
    /// Campaign goal_amount must be positive.
    InvalidCampaignGoal = 61,
    /// Campaign deadline must be in the future.
    InvalidCampaignDeadline = 62,
    /// Operation referred to a campaign that has been deactivated.
    CampaignInactive = 63,
    /// The caller is not the merchant that owns the campaign.
    NotCampaignMerchant = 64,
    /// The campaign's deadline has passed and it can no longer accept contributions.
    CampaignExpired = 65,
}
