use crate::components::core;
use crate::components::reentrancy;
use crate::errors::ContractError;
use crate::events;
use crate::types::{
    BackerKycStatus, CampaignKycStatus, DataKey, KycRequest, VerificationStatus, VerificationType,
};
use soroban_sdk::{panic_with_error, Address, Env, String, Vec};

// ── KYC Request Management ─────────────────────────────────────────────────────

/// Submit a KYC verification request for a user.
pub fn submit_kyc_verification(
    env: &Env,
    subject: &Address,
    verification_type: VerificationType,
    metadata: &String,
) -> u64 {
    subject.require_auth();
    reentrancy::enter(env);

    // Check if already verified or pending
    let current_status = get_kyc_status(env, subject);
    match current_status {
        VerificationStatus::Approved => {
            reentrancy::exit(env);
            panic_with_error!(env, ContractError::MerchantAlreadyRegistered);
        }
        VerificationStatus::Pending => {
            reentrancy::exit(env);
            panic_with_error!(env, ContractError::PlanNotActive);
        }
        _ => {}
    }

    // Generate new request ID
    let request_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::KycRequestCount)
        .unwrap_or(0);
    let request_id = request_count + 1;

    let now = env.ledger().timestamp();
    let kyc_request = KycRequest {
        id: request_id,
        subject: subject.clone(),
        verification_type,
        submitted_at: now,
        reviewed_at: 0,
        reviewer: Address::from_contract_id(env, &soroban_sdk::BytesN::zero(env)),
        status: VerificationStatus::Pending,
        document_count: 0,
        metadata: metadata.clone(),
    };

    // Store request
    env.storage()
        .persistent()
        .set(&DataKey::KycRequest(request_id), &kyc_request);
    env.storage()
        .persistent()
        .set(&DataKey::KycRequestCount, &request_id);

    // Update user status
    env.storage().persistent().set(
        &DataKey::KycVerificationStatus(subject.clone()),
        &VerificationStatus::Pending,
    );

    // Add to pending list
    let mut pending = get_pending_kyc_approvals(env);
    pending.push_back(request_id);
    env.storage()
        .persistent()
        .set(&DataKey::PendingKycApproval, &pending);

    // Emit event
    events::publish_kyc_request_submitted_event(
        env,
        request_id,
        subject.clone(),
        verification_type,
        now,
    );

    reentrancy::exit(env);
    request_id
}

/// Approve a KYC verification request (admin/reviewer only).
pub fn approve_kyc_request(env: &Env, reviewer: &Address, request_id: u64, expiration_days: u64) {
    reviewer.require_auth();
    reentrancy::enter(env);

    // Check reviewer is authorized
    assert_kyc_reviewer(env, reviewer);

    // Get request
    let mut kyc_request: KycRequest = env
        .storage()
        .persistent()
        .get(&DataKey::KycRequest(request_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound));

    // Check status is pending
    if kyc_request.status != VerificationStatus::Pending {
        reentrancy::exit(env);
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    let now = env.ledger().timestamp();
    let expiration_date = now + (expiration_days * 86400); // 86400 seconds per day

    // Update request
    kyc_request.status = VerificationStatus::Approved;
    kyc_request.reviewed_at = now;
    kyc_request.reviewer = reviewer.clone();

    env.storage()
        .persistent()
        .set(&DataKey::KycRequest(request_id), &kyc_request);

    // Update user status
    env.storage().persistent().set(
        &DataKey::KycVerificationStatus(kyc_request.subject.clone()),
        &VerificationStatus::Approved,
    );

    // Set expiration date
    env.storage().persistent().set(
        &DataKey::KycExpirationDate(kyc_request.subject.clone()),
        &expiration_date,
    );

    // Add to approved list
    let mut approved = get_approved_kyc_list(env);
    if !contains_address(&approved, &kyc_request.subject) {
        approved.push_back(kyc_request.subject.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ApprovedKycList, &approved);
    }

    // Remove from pending list
    remove_from_pending_kyc(env, request_id);

    // Emit event
    events::publish_kyc_request_approved_event(
        env,
        request_id,
        kyc_request.subject.clone(),
        reviewer.clone(),
        expiration_date,
        now,
    );

    reentrancy::exit(env);
}

/// Reject a KYC verification request (admin/reviewer only).
pub fn reject_kyc_request(env: &Env, reviewer: &Address, request_id: u64, reason: &String) {
    reviewer.require_auth();
    reentrancy::enter(env);

    // Check reviewer is authorized
    assert_kyc_reviewer(env, reviewer);

    // Get request
    let mut kyc_request: KycRequest = env
        .storage()
        .persistent()
        .get(&DataKey::KycRequest(request_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound));

    // Check status is pending
    if kyc_request.status != VerificationStatus::Pending {
        reentrancy::exit(env);
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    let now = env.ledger().timestamp();

    // Update request
    kyc_request.status = VerificationStatus::Rejected;
    kyc_request.reviewed_at = now;
    kyc_request.reviewer = reviewer.clone();

    env.storage()
        .persistent()
        .set(&DataKey::KycRequest(request_id), &kyc_request);

    // Update user status
    env.storage().persistent().set(
        &DataKey::KycVerificationStatus(kyc_request.subject.clone()),
        &VerificationStatus::Rejected,
    );

    // Store rejection reason
    env.storage()
        .persistent()
        .set(&DataKey::KycRejectionReason(request_id), reason);

    // Add to rejected list
    let mut rejected = get_rejected_kyc_list(env);
    if !contains_address(&rejected, &kyc_request.subject) {
        rejected.push_back(kyc_request.subject.clone());
        env.storage()
            .persistent()
            .set(&DataKey::RejectedKycList, &rejected);
    }

    // Remove from pending list
    remove_from_pending_kyc(env, request_id);

    // Emit event
    events::publish_kyc_request_rejected_event(
        env,
        request_id,
        kyc_request.subject.clone(),
        reviewer.clone(),
        reason.clone(),
        now,
    );

    reentrancy::exit(env);
}

/// Suspend a user's KYC approval (admin/reviewer only).
pub fn suspend_kyc(env: &Env, reviewer: &Address, subject: &Address, reason: &String) {
    reviewer.require_auth();
    reentrancy::enter(env);

    // Check reviewer is authorized
    assert_kyc_reviewer(env, reviewer);

    // Get current status
    let current_status = get_kyc_status(env, subject);
    if current_status != VerificationStatus::Approved {
        reentrancy::exit(env);
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    let now = env.ledger().timestamp();

    // Update user status to suspended
    env.storage().persistent().set(
        &DataKey::KycVerificationStatus(subject.clone()),
        &VerificationStatus::Suspended,
    );

    // Emit event
    events::publish_kyc_suspended_event(
        env,
        subject.clone(),
        reviewer.clone(),
        reason.clone(),
        now,
    );

    reentrancy::exit(env);
}

// ── KYC Status Queries ─────────────────────────────────────────────────────────

/// Get the KYC status of a user.
pub fn get_kyc_status(env: &Env, subject: &Address) -> VerificationStatus {
    env.storage()
        .persistent()
        .get(&DataKey::KycVerificationStatus(subject.clone()))
        .unwrap_or(VerificationStatus::Unverified)
}

/// Get a KYC request by ID.
pub fn get_kyc_request(env: &Env, request_id: u64) -> KycRequest {
    env.storage()
        .persistent()
        .get(&DataKey::KycRequest(request_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound))
}

/// Check if a user is KYC approved.
pub fn is_kyc_approved(env: &Env, subject: &Address) -> bool {
    let status = get_kyc_status(env, subject);
    status == VerificationStatus::Approved && !is_kyc_expired(env, subject)
}

/// Check if a user's KYC has expired.
pub fn is_kyc_expired(env: &Env, subject: &Address) -> bool {
    if let Some(expiration_date) = env
        .storage()
        .persistent()
        .get::<_, u64>(&DataKey::KycExpirationDate(subject.clone()))
    {
        let now = env.ledger().timestamp();
        now > expiration_date
    } else {
        false
    }
}

// ── Campaign KYC Management ───────────────────────────────────────────────────

/// Register a campaign for KYC verification.
pub fn register_campaign_for_kyc(
    env: &Env,
    creator: &Address,
    campaign_id: u64,
    require_backer_kyc: bool,
) {
    creator.require_auth();
    reentrancy::enter(env);

    // Check creator's KYC is approved
    if !is_kyc_approved(env, creator) {
        reentrancy::exit(env);
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let now = env.ledger().timestamp();

    // Create campaign KYC status
    let campaign_kyc = CampaignKycStatus {
        campaign_id,
        creator: creator.clone(),
        kyc_status: VerificationStatus::Pending,
        min_backer_kyc_required: require_backer_kyc,
        created_at: now,
        verified_at: 0,
        verified_by: Address::from_contract_id(env, &soroban_sdk::BytesN::zero(env)),
    };

    env.storage()
        .persistent()
        .set(&DataKey::CampaignKycStatus(campaign_id), &campaign_kyc);

    // Emit event
    events::publish_campaign_kyc_registered_event(
        env,
        campaign_id,
        creator.clone(),
        require_backer_kyc,
        now,
    );

    reentrancy::exit(env);
}

/// Verify a campaign's KYC (admin/reviewer only).
pub fn verify_campaign(env: &Env, reviewer: &Address, campaign_id: u64) {
    reviewer.require_auth();
    reentrancy::enter(env);

    // Check reviewer is authorized
    assert_kyc_reviewer(env, reviewer);

    // Get campaign KYC status
    let mut campaign_kyc: CampaignKycStatus = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignKycStatus(campaign_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::EventNotFound));

    let now = env.ledger().timestamp();

    // Update campaign status
    campaign_kyc.kyc_status = VerificationStatus::Approved;
    campaign_kyc.verified_at = now;
    campaign_kyc.verified_by = reviewer.clone();

    env.storage()
        .persistent()
        .set(&DataKey::CampaignKycStatus(campaign_id), &campaign_kyc);

    // Emit event
    events::publish_campaign_kyc_verified_event(
        env,
        campaign_id,
        campaign_kyc.creator.clone(),
        reviewer.clone(),
        now,
    );

    reentrancy::exit(env);
}

/// Get campaign KYC status.
pub fn get_campaign_kyc_status(env: &Env, campaign_id: u64) -> CampaignKycStatus {
    env.storage()
        .persistent()
        .get(&DataKey::CampaignKycStatus(campaign_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::EventNotFound))
}

// ── Backer KYC Tracking ────────────────────────────────────────────────────────

/// Record a backer's contribution to a campaign.
pub fn record_backer_contribution(env: &Env, backer: &Address, campaign_id: u64, amount: i128) {
    reentrancy::enter(env);

    // Get or create backer KYC status
    let mut backer_status: BackerKycStatus = env
        .storage()
        .persistent()
        .get(&DataKey::BackerKycStatus(backer.clone()))
        .unwrap_or_else(|| BackerKycStatus {
            backer: backer.clone(),
            kyc_status: get_kyc_status(env, backer),
            campaigns_backed: 0,
            total_backed_amount: 0,
            last_kyc_check: env.ledger().timestamp(),
        });

    // Update backer contribution tracking
    backer_status.campaigns_backed += 1;
    backer_status.total_backed_amount += amount;
    backer_status.last_kyc_check = env.ledger().timestamp();

    env.storage()
        .persistent()
        .set(&DataKey::BackerKycStatus(backer.clone()), &backer_status);

    // Emit event
    // TEMPORARILY DISABLED: events::publish_backer_contribution_recorded_event(
    //     env,
    //     backer.clone(),
    //     campaign_id,
    //     amount,
    //     env.ledger().timestamp(),
    // );

    reentrancy::exit(env);
}

/// Get backer KYC status.
pub fn get_backer_kyc_status(env: &Env, backer: &Address) -> BackerKycStatus {
    env.storage()
        .persistent()
        .get(&DataKey::BackerKycStatus(backer.clone()))
        .unwrap_or_else(|| BackerKycStatus {
            backer: backer.clone(),
            kyc_status: get_kyc_status(env, backer),
            campaigns_backed: 0,
            total_backed_amount: 0,
            last_kyc_check: env.ledger().timestamp(),
        })
}

// ── KYC Reviewer Role Management ───────────────────────────────────────────────

/// Grant KYC reviewer role to a user (admin only).
pub fn grant_kyc_reviewer_role(env: &Env, admin: &Address, reviewer: &Address) {
    core::assert_admin(env, admin);
    reentrancy::enter(env);

    env.storage()
        .persistent()
        .set(&DataKey::KycReviewerRole(reviewer.clone()), &true);

    events::publish_kyc_reviewer_role_granted_event(
        env,
        admin.clone(),
        reviewer.clone(),
        env.ledger().timestamp(),
    );

    reentrancy::exit(env);
}

/// Revoke KYC reviewer role from a user (admin only).
pub fn revoke_kyc_reviewer_role(env: &Env, admin: &Address, reviewer: &Address) {
    core::assert_admin(env, admin);
    reentrancy::enter(env);

    env.storage()
        .persistent()
        .remove(&DataKey::KycReviewerRole(reviewer.clone()));

    events::publish_kyc_reviewer_role_revoked_event(
        env,
        admin.clone(),
        reviewer.clone(),
        env.ledger().timestamp(),
    );

    reentrancy::exit(env);
}

/// Check if a user has KYC reviewer role.
pub fn has_kyc_reviewer_role(env: &Env, user: &Address) -> bool {
    let admin = core::get_admin(env);
    if *user == admin {
        return true;
    }

    env.storage()
        .persistent()
        .has(&DataKey::KycReviewerRole(user.clone()))
}

// ── Helper Functions ──────────────────────────────────────────────────────────

fn assert_kyc_reviewer(env: &Env, reviewer: &Address) {
    if !has_kyc_reviewer_role(env, reviewer) {
        panic_with_error!(env, ContractError::NotAuthorized);
    }
}

fn get_pending_kyc_approvals(env: &Env) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::PendingKycApproval)
        .unwrap_or_else(|| Vec::new(env))
}

fn get_approved_kyc_list(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::ApprovedKycList)
        .unwrap_or_else(|| Vec::new(env))
}

fn get_rejected_kyc_list(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::RejectedKycList)
        .unwrap_or_else(|| Vec::new(env))
}

fn remove_from_pending_kyc(env: &Env, request_id: u64) {
    let pending = get_pending_kyc_approvals(env);
    let mut updated = Vec::new(env);

    for req_id in pending.iter() {
        if req_id != request_id {
            updated.push_back(req_id);
        }
    }

    env.storage()
        .persistent()
        .set(&DataKey::PendingKycApproval, &updated);
}

fn contains_address(addresses: &Vec<Address>, target: &Address) -> bool {
    for addr in addresses.iter() {
        if addr == *target {
            return true;
        }
    }
    false
}
