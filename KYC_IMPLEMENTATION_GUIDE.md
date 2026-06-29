# Campaign KYC and Verification System - Implementation Guide

## Overview

This document provides a comprehensive guide for implementing a Campaign KYC (Know Your Customer) and Verification System for the Shade Protocol Soroban smart contract. This system enables secure crowdfunding by enforcing identity verification for campaign creators and backers.

## Architecture & Design

### 1. **Core Concepts**

#### KYC Workflow

```
User → Submit KYC Request 
  → Admin Review 
    → Approve (with expiration date) OR Reject 
      → User can participate in campaigns
```

#### Roles

- **Admin**: Initializes KYC settings, can grant reviewer roles
- **KYC Reviewer**: Approves/rejects KYC requests, can suspend users
- **Campaign Creator**: Must submit KYC before creating campaigns
- **Backer**: Must submit KYC if campaign requires it

#### State Transitions

```
Unverified 
  → Pending (on submit_kyc)
    → Approved (on approve_kyc)
    → Rejected (on reject_kyc)
    → Suspended (on suspend_kyc from Approved)

Approved
  → Expired (time-based, on expiration check)
  → Suspended (by reviewer)
```

### 2. **Data Model & Storage**

Due to Soroban SDK's `#[contracttype]` serialization limits, the DataKey enum cannot accommodate all KYC variants. The solution uses a **Map-based storage pattern** where KYC data is stored using flexible key generation.

#### Type Definitions (Already in types.rs)

```rust
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VerificationStatus {
    Unverified = 0,
    Pending = 1,
    Approved = 2,
    Rejected = 3,
    Suspended = 4,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VerificationType {
    Individual = 0,
    CampaignCreator = 1,
    Backer = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KycRequest {
    pub id: u64,
    pub subject: Address,
    pub verification_type: VerificationType,
    pub submitted_at: u64,
    pub reviewed_at: u64,
    pub reviewer: Address,
    pub status: VerificationStatus,
    pub document_count: u32,
    pub metadata: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignKycStatus {
    pub campaign_id: u64,
    pub creator: Address,
    pub kyc_status: VerificationStatus,
    pub min_backer_kyc_required: bool,
    pub created_at: u64,
    pub verified_at: u64,
    pub verified_by: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackerKycStatus {
    pub backer: Address,
    pub kyc_status: VerificationStatus,
    pub campaigns_backed: u64,
    pub total_backed_amount: i128,
    pub last_kyc_check: u64,
}
```

#### Storage Keys (Using Map Pattern)

The KYC module uses Soroban's `Map` with Symbol keys:

| Storage Purpose | Key Pattern | Value Type |
|---|---|---|
| KYC Request by ID | `map(KycRequestMap, Symbol::short("req:{id}"))` | `KycRequest` |
| Global Request Counter | `persistent.get(Symbol::short("req_cnt"))` | `u64` |
| User Verification Status | `map(KycStatusMap, user_address)` | `VerificationStatus` |
| KYC Expiration Date | `map(KycExpirationMap, user_address)` | `u64` |
| Rejection Reason | `map(RejectionReasonMap, Symbol::short("rej:{id}"))` | `String` |
| Pending Requests List | `persistent.get(Symbol::short("pnd_list"))` | `Vec<u64>` |
| Approved Users List | `persistent.get(Symbol::short("app_list"))` | `Vec<Address>` |
| Rejected Users List | `persistent.get(Symbol::short("rjc_list"))` | `Vec<Address>` |
| Reviewer Roles | `map(ReviewerRoleMap, reviewer_address)` | `bool` |
| Campaign KYC Status | `map(CampaignKycMap, campaign_id)` | `CampaignKycStatus` |
| Backer Contribution Tracking | `map(BackerKycMap, backer_address)` | `BackerKycStatus` |

### 3. **Core Functions**

#### KYC Request Management

**`submit_kyc_verification(subject, verification_type, metadata) -> u64`**
- **Auth**: User must authenticate
- **Validation**: 
  - User not already approved or pending
  - Metadata not empty
- **Effects**:
  - Creates new KycRequest with Pending status
  - Increments request counter
  - Adds to pending requests list
  - Emits KycRequestSubmittedEvent
- **Returns**: Request ID

**`approve_kyc_request(reviewer, request_id, expiration_days)`**
- **Auth**: Reviewer must authenticate
- **Validation**: 
  - Reviewer has KYC reviewer role
  - Request exists and is Pending
  - Expiration days >= 1
- **Effects**:
  - Sets request status to Approved
  - Sets user verification status to Approved
  - Records expiration date as `now + (expiration_days * 86400)`
  - Adds user to approved list
  - Removes request from pending list
  - Emits KycRequestApprovedEvent

**`reject_kyc_request(reviewer, request_id, reason)`**
- **Auth**: Reviewer must authenticate
- **Validation**: 
  - Reviewer has KYC reviewer role
  - Request exists and is Pending
  - Reason not empty
- **Effects**:
  - Sets request status to Rejected
  - Sets user verification status to Rejected
  - Stores rejection reason
  - Adds user to rejected list
  - Removes request from pending list
  - Emits KycRequestRejectedEvent

**`suspend_kyc(reviewer, subject, reason)`**
- **Auth**: Reviewer must authenticate
- **Validation**: 
  - Reviewer has KYC reviewer role
  - User status is currently Approved
  - Reason not empty
- **Effects**:
  - Sets user verification status to Suspended
  - Clears expiration date
  - Emits KycSuspendedEvent
  - User cannot participate in campaigns until approved again

#### Status Queries

**`get_kyc_status(subject) -> VerificationStatus`**
- Returns current verification status
- Default: Unverified

**`get_kyc_request(request_id) -> KycRequest`**
- Panics if request not found

**`is_kyc_approved(subject) -> bool`**
- Returns true if status is Approved AND not expired
- Checks expiration date dynamically

**`is_kyc_expired(subject) -> bool`**
- Returns true if expiration date exists and is in the past
- Compares against `env.ledger().timestamp()`

#### Campaign KYC Management

**`register_campaign_for_kyc(creator, campaign_id, require_backer_kyc)`**
- **Auth**: Creator must authenticate
- **Validation**: 
  - Creator's KYC is approved and not expired
  - Campaign not already registered
- **Effects**:
  - Creates CampaignKycStatus with Pending status
  - Stores backer KYC requirement flag
  - Emits CampaignKycRegisteredEvent

**`verify_campaign(reviewer, campaign_id)`**
- **Auth**: Reviewer must authenticate
- **Validation**: 
  - Reviewer has KYC reviewer role
  - Campaign exists and status is Pending
- **Effects**:
  - Sets campaign status to Approved
  - Records verification timestamp and reviewer
  - Emits CampaignKycVerifiedEvent

**`get_campaign_kyc_status(campaign_id) -> CampaignKycStatus`**
- Panics if campaign not found

#### Backer Tracking

**`record_backer_contribution(backer, campaign_id, amount)`**
- **Auth**: Internal (called by crowdfund contract)
- **Validation**: 
  - Amount >= 0
- **Effects**:
  - Creates or updates BackerKycStatus
  - Increments campaign count
  - Adds amount to total backed
  - Updates last check timestamp
  - Emits BackerContributionRecordedEvent

**`get_backer_kyc_status(backer) -> BackerKycStatus`**
- Returns backer's contribution tracking

#### Reviewer Role Management

**`grant_kyc_reviewer_role(admin, reviewer)`**
- **Auth**: Admin must authenticate
- **Validation**: Admin is contract admin
- **Effects**:
  - Marks reviewer address with reviewer role
  - Emits KycReviewerRoleGrantedEvent

**`revoke_kyc_reviewer_role(admin, reviewer)`**
- **Auth**: Admin must authenticate
- **Validation**: Admin is contract admin
- **Effects**:
  - Removes reviewer role
  - Emits KycReviewerRoleRevokedEvent

**`has_kyc_reviewer_role(user) -> bool`**
- Returns true if user is admin or has reviewer role

### 4. **Events**

All events include timestamp from `env.ledger().timestamp()`:

```rust
event KycRequestSubmittedEvent {
    request_id: u64,
    subject: Address,
    verification_type: VerificationType,
    timestamp: u64,
}

event KycRequestApprovedEvent {
    request_id: u64,
    subject: Address,
    reviewer: Address,
    expiration_date: u64,
    timestamp: u64,
}

event KycRequestRejectedEvent {
    request_id: u64,
    subject: Address,
    reviewer: Address,
    reason: String,
    timestamp: u64,
}

event KycSuspendedEvent {
    subject: Address,
    reviewer: Address,
    reason: String,
    timestamp: u64,
}

event CampaignKycRegisteredEvent {
    campaign_id: u64,
    creator: Address,
    require_backer_kyc: bool,
    timestamp: u64,
}

event CampaignKycVerifiedEvent {
    campaign_id: u64,
    creator: Address,
    reviewer: Address,
    timestamp: u64,
}

event BackerContributionRecordedEvent {
    backer: Address,
    campaign_id: u64,
    amount: i128,
    timestamp: u64,
}

event KycReviewerRoleGrantedEvent {
    admin: Address,
    reviewer: Address,
    timestamp: u64,
}

event KycReviewerRoleRevokedEvent {
    admin: Address,
    reviewer: Address,
    timestamp: u64,
}
```

## Security Considerations

### 1. **Authentication & Authorization**
- All privileged operations require `require_auth()`
- Only admin can grant/revoke reviewer roles
- Only KYC reviewers (or admin) can approve/reject
- Users can only submit KYC for themselves

### 2. **Reentrancy Protection**
- All state-modifying functions use reentrancy guards via `enter()` and `exit()`
- Prevents recursive calls during execution

### 3. **Input Validation**
- Empty metadata/reason strings rejected
- Expiration days must be >= 1
- Amounts must be non-negative
- Status checks prevent invalid transitions

### 4. **Storage Efficiency**
- Uses Maps instead of large enums to respect Soroban limits
- Pending/approved/rejected lists maintained for off-chain indexing
- Pagination support through list queries for large datasets

### 5. **Data Integrity**
- KYC requests immutable once stored
- Verification status changes only through explicit functions
- Expiration dates prevent permanent access from expired approvals

## Implementation Patterns

### 1. **Reentrancy Pattern**

```rust
pub fn some_kyc_function(env: &Env, ...) {
    // Authenticate caller
    caller.require_auth();
    reentrancy::enter(env);
    
    // ... perform operations ...
    
    reentrancy::exit(env);
}
```

### 2. **Storage Access Pattern**

```rust
// Get from persistent storage
let value: SomeType = env
    .storage()
    .persistent()
    .get(&key)
    .unwrap_or_default_value();

// Set in persistent storage
env.storage()
    .persistent()
    .set(&key, &value);

// Check existence
if env.storage().persistent().has(&key) {
    // Key exists
}
```

### 3. **Status Validation Pattern**

```rust
match get_kyc_status(env, user) {
    VerificationStatus::Approved => {
        // Proceed
    }
    VerificationStatus::Pending => {
        panic_with_error!(env, ContractError::SomeError);
    }
    _ => {
        // Handle other statuses
    }
}
```

### 4. **Event Emission Pattern**

```rust
events::publish_kyc_request_approved_event(
    env,
    request_id,
    subject.clone(),
    reviewer.clone(),
    expiration_date,
    timestamp,
);
```

## Testing Strategy

### 1. **Unit Tests**
- Test each function with valid inputs
- Test each function with invalid inputs
- Test status transitions
- Test expiration logic
- Test authorization checks

### 2. **Integration Tests**
- Test full KYC workflow (submit → approve → verify campaign)
- Test rejection workflow
- Test suspension workflow
- Test backer contribution tracking
- Test concurrent KYC requests

### 3. **Security Tests**
- Test unauthorized access prevention
- Test reentrancy attacks
- Test data integrity
- Test edge cases (max u64 values, empty collections)

### 4. **Testnet Deployment
- Deploy to local Soroban testnet
- Verify all functions execute correctly
- Verify events emit with correct data
- Test with real transaction scenarios

## Deployment Checklist

- [ ] Types defined in types.rs
- [ ] Events defined and published correctly
- [ ] KYC component module implemented
- [ ] ShadeTrait interface updated with KYC methods
- [ ] shade.rs impl block includes KYC delegations
- [ ] components/mod.rs exports kyc module
- [ ] All tests pass locally
- [ ] Cargo check passes without errors
- [ ] Cargo build completes successfully
- [ ] No clippy warnings
- [ ] Events tested on testnet
- [ ] Storage efficiency verified (rent costs)
- [ ] Error messages are informative
- [ ] Documentation is complete
- [ ] Code follows project conventions

## Performance Optimization

### Storage Cost Reduction

1. **Use compact representations**
   - VerificationStatus uses u32 enum representation
   - Timestamps stored as u64 (not String)
   - Addresses stored natively (not serialized)

2. **Minimize list sizes**
   - Pending/rejected lists cleaned periodically
   - Approved list can be pruned after expiration
   - Implement pagination for large datasets

3. **Lazy evaluation**
   - Expiration checked on-demand, not proactively
   - Lists rebuilt only when modified
   - Request details loaded only when needed

### Gas Efficiency

1. **Function signatures**
   - Minimal parameters (combine when possible)
   - Return only necessary data
   - Use enums over strings where feasible

2. **Storage operations**
   - Batch updates when possible
   - Avoid redundant reads
   - Cache frequently accessed values

3. **Event emission**
   - Include all necessary metadata
   - Avoid duplicate events
   - Use compact types

## Future Enhancements

1. **Tiered KYC Levels**
   - Different verification requirements
   - Risk-based KYC thresholds
   - Progressive verification for higher stakes

2. **Automated KYC**
   - Integration with oracle-based identity services
   - Automatic approval based on credit scores
   - Real-time risk scoring

3. **KYC Data Privacy**
   - Encrypted storage of sensitive data
   - Zero-knowledge proofs of identity
   - GDPR-compliant data handling

4. **Advanced Features**
   - KYC renewal reminders
   - Bulk approval/rejection
   - KYC analytics dashboard
   - Fraud detection integration

## References

- [Soroban Contract Development Guide](https://developers.stellar.org/docs)
- [ContractType Serialization](https://soroban.stellar.org/)
- [KYC Regulations Overview](https://en.wikipedia.org/wiki/Know_your_customer)
- Shade Protocol Smart Contract Standards

---

**Document Version**: 1.0
**Last Updated**: June 2026
**Status**: Implementation Ready
