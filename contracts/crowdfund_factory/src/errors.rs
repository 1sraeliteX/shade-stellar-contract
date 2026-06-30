use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FactoryError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    WasmHashNotSet = 3,
    CampaignNotFound = 4,
    // DAO governance has not been initialised via `init_dao` (#359).
    DaoNotInitialized = 5,
    DaoAlreadyInitialized = 6,
    NotDaoAdmin = 7,
    AlreadyDaoMember = 8,
    NotDaoMember = 9,
    // Quorum must be > 0 and <= 10_000 basis points.
    InvalidQuorum = 10,
    InvalidVotingPeriod = 11,
    DaoProposalNotFound = 12,
    // Proposal voting window has closed, or proposal is no longer in `Voting` status.
    VotingClosed = 13,
    // Proposal voting window has not yet closed.
    VotingNotClosed = 14,
    AlreadyVoted = 15,
    DaoProposalAlreadyExecuted = 16,
}
