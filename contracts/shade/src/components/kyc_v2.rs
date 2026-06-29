/// Campaign KYC and Verification System
/// 
/// This module implements a robust KYC (Know Your Customer) and verification system
/// for the Shade Protocol that supports campaign creators and backers.
/// 
/// Due to Soroban SDK limitations on #[contracttype] enum serialization,
/// storage uses a Map-based pattern rather than DataKey variants.

use crate::components::core;
use crate::components::reentrancy;
use crate::errors::ContractError;
use crate::events;
use crate::types::{
    BackerKycStatus, CampaignKycStatus, KycRequest, VerificationStatus, VerificationType,
};
use soroban_sdk::{panic_with_error, Address, Env, Map, String, Symbol, Vec};

// ── Storage Helper Symbols ─────────────────────────────────────────────────────

/// Get symbol for KYC request storage key by ID
fn kyc_request_symbol(id: u64) -> Symbol {
    Symbol::short(&format!("kyc_req_{}", id))
}

/// Symbol for global KYC request counter
fn kyc_request_count_symbol() -> Symbol {
    Symbol::short("kyc_cnt")
}

/// Symbol for pending KYC approvals list
fn kyc_pending_symbol() -> Symbol {
    Symbol::short("kyc_pnd")
}

/// Symbol for approved KYC users list
fn kyc_approved_symbol() -> Symbol {
    Symbol::short("kyc_app")
}

/// Symbol for rejected KYC users list
fn kyc_rejected_symbol() -> Symbol {
    Symbol::short("kyc_rjc")
}

/// Symbol for KYC reviewer role storage map key
fn kyc_reviewer_symbol() -> Symbol {
    Symbol::short("kyc_rev")
}

/// Symbol for user verification status map key
fn kyc_status_symbol() -> Symbol {
    Symbol::short("kyc_st")
}

/// Symbol for KYC expiration date map key
fn kyc_expiration_symbol() -> Symbol {
    Symbol::short("kyc_exp")
}

/// Symbol for campaign KYC status map key
fn campaign_kyc_symbol() -> Symbol {
    Symbol::short("cam_kyc")
}

/// Symbol for backer KYC status map key
fn backer_kyc_symbol() -> Symbol {
    Symbol::short("bkr_kyc")
}

/// Symbol for rejection reason map key
fn rejection_reason_symbol() -> Symbol {
    Symbol::short("rej_rsn")
}

// ── KYC Request Management ─────────────────────────────────────────────────────

/// Submit a KYC verification request for a user.
/// 
/// # Arguments
/// * `subject` - The user submitting KYC (must authenticate)
/// * `verification_type` - Type of verification (Individual, CampaignCreator, or Backer)
/// * `metadata` - Additional metadata (KYC document references, etc.)
/// 
/// # Returns
/// The newly generated request ID
/// 
/// # Panics
/// - If subject is already approved or has pending request
pub fn submit_kyc_verification(
    env: &Env,
    subject: &Address,
    verification_type: VerificationType,
    metadata: &String,
) -> u64 {
    subject.require_auth();
    reentrancy::enter(env);

    // Check current status
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

    // Generate request ID
    let request_count: u64 = env
        .storage()
        .persistent()
        .get(&kyc_request_count_symbol())
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

    // Store request in map
    let mut kyc_map: Map<u64, KycRequest> = env
        .storage()
        .persistent()
        .get(&kyc_request_symbol(0))
        .unwrap_or_else(|| Map::new(env));
    kyc_map.set(request_id, kyc_request);
    env.storage()
        .persistent()
        .set(&kyc_request_symbol(0), &kyc_map);

    // Update counter
    env.storage()
        .persistent()
        .set(&kyc_request_count_symbol(), &request_id);

    // Update status map
    let mut status_map: Map<Address, VerificationStatus> = env
        .storage()
        .persistent()
        .get(&kyc_status_symbol())
        .unwrap_or_else(|| Map::new(env));
    status_map.set(subject.clone(), VerificationStatus::Pending);
    env.storage()
        .persistent()
        .set(&kyc_status_symbol(), &status_map);

    // Add to pending list
    let mut pending: Vec<u64> = env
        .storage()
        .persistent()
        .get(&kyc_pending_symbol())
        .unwrap_or_else(|| Vec::new(env));
    pending.push_back(request_id);
    env.storage()
        .persistent()
        .set(&kyc_pending_symbol(), &pending);

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

/// Approve a KYC verification request.
/// 
/// # Arguments
/// * `reviewer` - The reviewer approving (must have reviewer role)
/// * `request_id` - The request ID to approve
/// * `expiration_days` - Number of days until approval expires (>= 1)
pub fn approve_kyc_request(env: &Env, reviewer: &Address, request_id: u64, expiration_days: u64) {
    reviewer.require_auth();
    reentrancy::enter(env);

    assert_kyc_reviewer(env, reviewer);

    // Get request
    let mut kyc_map: Map<u64, KycRequest> = env
        .storage()
        .persistent()
        .get(&kyc_request_symbol(0))
        .unwrap_or_else(|| Map::new(env));

    let mut kyc_request = kyc_map
        .get(request_id)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound));

    if kyc_request.status != VerificationStatus::Pending {
        reentrancy::exit(env);
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    let now = env.ledger().timestamp();
    let expiration_date = now + (expiration_days * 86400);

    // Update request
    kyc_request.status = VerificationStatus::Approved;
    kyc_request.reviewed_at = now;
    kyc_request.reviewer = reviewer.clone();

    kyc_map.set(request_id, kyc_request.clone());
    env.storage()
        .persistent()
        .set(&kyc_request_symbol(0), &kyc_map);

    // Update status
    let mut status_map: Map<Address, VerificationStatus> = env
        .storage()
        .persistent()
        .get(&kyc_status_symbol())
        .unwrap_or_else(|| Map::new(env));
    status_map.set(kyc_request.subject.clone(), VerificationStatus::Approved);
    env.storage()
        .persistent()
        .set(&kyc_status_symbol(), &status_map);

    // Set expiration date
    let mut exp_map: Map<Address, u64> = env
        .storage()
        .persistent()
        .get(&kyc_expiration_symbol())
        .unwrap_or_else(|| Map::new(env));
    exp_map.set(kyc_request.subject.clone(), expiration_date);
    env.storage()
        .persistent()
        .set(&kyc_expiration_symbol(), &exp_map);

    // Add to approved list
    let mut approved: Vec<Address> = env
        .storage()
        .persistent()
        .get(&kyc_approved_symbol())
        .unwrap_or_else(|| Vec::new(env));
    if !contains_address(&approved, &kyc_request.subject) {
        approved.push_back(kyc_request.subject.clone());
        env.storage()
            .persistent()
            .set(&kyc_approved_symbol(), &approved);
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

/// Reject a KYC verification request.
pub fn reject_kyc_request(env: &Env, reviewer: &Address, request_id: u64, reason: &String) {
    reviewer.require_auth();
    reentrancy::enter(env);

    assert_kyc_reviewer(env, reviewer);

    // Get request
    let mut kyc_map: Map<u64, KycRequest> = env
        .storage()
        .persistent()
        .get(&kyc_request_symbol(0))
        .unwrap_or_else(|| Map::new(env));

    let mut kyc_request = kyc_map
        .get(request_id)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound));

    if kyc_request.status != VerificationStatus::Pending {
        reentrancy::exit(env);
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    let now = env.ledger().timestamp();

    // Update request
    kyc_request.status = VerificationStatus::Rejected;
    kyc_request.reviewed_at = now;
    kyc_request.reviewer = reviewer.clone();

    kyc_map.set(request_id, kyc_request.clone());
    env.storage()
        .persistent()
        .set(&kyc_request_symbol(0), &kyc_map);

    // Update status
    let mut status_map: Map<Address, VerificationStatus> = env
        .storage()
        .persistent()
        .get(&kyc_status_symbol())
        .unwrap_or_else(|| Map::new(env));
    status_map.set(kyc_request.subject.clone(), VerificationStatus::Rejected);
    env.storage()
        .persistent()
        .set(&kyc_status_symbol(), &status_map);

    // Store rejection reason
    let mut reason_map: Map<u64, String> = env
        .storage()
        .persistent()
        .get(&rejection_reason_symbol())
        .unwrap_or_else(|| Map::new(env));
    reason_map.set(request_id, reason.clone());
    env.storage()
        .persistent()
        .set(&rejection_reason_symbol(), &reason_map);

    // Add to rejected list
    let mut rejected: Vec<Address> = env
        .storage()
        .persistent()
        .get(&kyc_rejected_symbol())
        .unwrap_or_else(|| Vec::new(env));
    if !contains_address(&rejected, &kyc_request.subject) {
        rejected.push_back(kyc_request.subject.clone());
        env.storage()
            .persistent()
            .set(&kyc_rejected_symbol(), &rejected);
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

/// Suspend a user's KYC approval.
pub fn suspend_kyc(env: &Env, reviewer: &Address, subject: &Address, reason: &String) {
    reviewer.require_auth();
    reentrancy::enter(env);

    assert_kyc_reviewer(env, reviewer);

    let current_status = get_kyc_status(env, subject);
    if current_status != VerificationStatus::Approved {
        reentrancy::exit(env);
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    let now = env.ledger().timestamp();

    // Update status
    let mut status_map: Map<Address, VerificationStatus> = env
        .storage()
        .persistent()
        .get(&kyc_status_symbol())
        .unwrap_or_else(|| Map::new(env));
    status_map.set(subject.clone(), VerificationStatus::Suspended);
    env.storage()
        .persistent()
        .set(&kyc_status_symbol(), &status_map);

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

// ── Status Queries ────────────────────────────────────────────────────────────

/// Get the current KYC status of a user.
pub fn get_kyc_status(env: &Env, subject: &Address) -> VerificationStatus {
    let status_map: Map<Address, VerificationStatus> = env
        .storage()
        .persistent()
        .get(&kyc_status_symbol())
        .unwrap_or_else(|| Map::new(env));
    
    status_map
        .get(subject.clone())
        .unwrap_or(VerificationStatus::Unverified)
}

/// Get a KYC request by ID.
pub fn get_kyc_request(env: &Env, request_id: u64) -> KycRequest {
    let kyc_map: Map<u64, KycRequest> = env
        .storage()
        .persistent()
        .get(&kyc_request_symbol(0))
        .unwrap_or_else(|| Map::new(env));

    kyc_map
        .get(request_id)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound))
}

/// Check if a user's KYC is approved and not expired.
pub fn is_kyc_approved(env: &Env, subject: &Address) -> bool {
    let status = get_kyc_status(env, subject);
    status == VerificationStatus::Approved && !is_kyc_expired(env, subject)
}

/// Check if a user's KYC has expired.
pub fn is_kyc_expired(env: &Env, subject: &Address) -> bool {
    let exp_map: Map<Address, u64> = env
        .storage()
        .persistent()
        .get(&kyc_expiration_symbol())
        .unwrap_or_else(|| Map::new(env));

    if let Some(expiration_date) = exp_map.get(subject.clone()) {
        let now = env.ledger().timestamp();
        now > expiration_date
    } else {
        false
    }
}

// ── Campaign KYC Management ──────────────────────────────────────────────────

/// Register a campaign for KYC verification.
pub fn register_campaign_for_kyc(
    env: &Env,
    creator: &Address,
    campaign_id: u64,
    require_backer_kyc: bool,
) {
    creator.require_auth();
    reentrancy::enter(env);

    if !is_kyc_approved(env, creator) {
        reentrancy::exit(env);
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let now = env.ledger().timestamp();
    let campaign_kyc = CampaignKycStatus {
        campaign_id,
        creator: creator.clone(),
        kyc_status: VerificationStatus::Pending,
        min_backer_kyc_required: require_backer_kyc,
        created_at: now,
        verified_at: 0,
        verified_by: Address::from_contract_id(env, &soroban_sdk::BytesN::zero(env)),
    };

    // Store in map
    let mut campaign_map: Map<u64, CampaignKycStatus> = env
        .storage()
        .persistent()
        .get(&campaign_kyc_symbol())
        .unwrap_or_else(|| Map::new(env));
    campaign_map.set(campaign_id, campaign_kyc);
    env.storage()
        .persistent()
        .set(&campaign_kyc_symbol(), &campaign_map);

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

/// Verify a campaign's KYC.
pub fn verify_campaign(env: &Env, reviewer: &Address, campaign_id: u64) {
    reviewer.require_auth();
    reentrancy::enter(env);

    assert_kyc_reviewer(env, reviewer);

    // Get campaign KYC status
    let mut campaign_map: Map<u64, CampaignKycStatus> = env
        .storage()
        .persistent()
        .get(&campaign_kyc_symbol())
        .unwrap_or_else(|| Map::new(env));

    let mut campaign_kyc = campaign_map
        .get(campaign_id)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::EventNotFound));

    let now = env.ledger().timestamp();

    // Update campaign status
    campaign_kyc.kyc_status = VerificationStatus::Approved;
    campaign_kyc.verified_at = now;
    campaign_kyc.verified_by = reviewer.clone();

    campaign_map.set(campaign_id, campaign_kyc.clone());
    env.storage()
        .persistent()
        .set(&campaign_kyc_symbol(), &campaign_map);

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
    let campaign_map: Map<u64, CampaignKycStatus> = env
        .storage()
        .persistent()
        .get(&campaign_kyc_symbol())
        .unwrap_or_else(|| Map::new(env));

    campaign_map
        .get(campaign_id)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::EventNotFound))
}

// ── Backer Tracking ──────────────────────────────────────────────────────────

/// Record a backer's contribution to a campaign.
pub fn record_backer_contribution(env: &Env, backer: &Address, campaign_id: u64, amount: i128) {
    reentrancy::enter(env);

    // Get or create backer status
    let mut backer_map: Map<Address, BackerKycStatus> = env
        .storage()
        .persistent()
        .get(&backer_kyc_symbol())
        .unwrap_or_else(|| Map::new(env));

    let mut backer_status = backer_map.get(backer.clone()).unwrap_or_else(|| BackerKycStatus {
        backer: backer.clone(),
        kyc_status: get_kyc_status(env, backer),
        campaigns_backed: 0,
        total_backed_amount: 0,
        last_kyc_check: env.ledger().timestamp(),
    });

    // Update tracking
    backer_status.campaigns_backed += 1;
    backer_status.total_backed_amount += amount;
    backer_status.last_kyc_check = env.ledger().timestamp();

    backer_map.set(backer.clone(), backer_status);
    env.storage()
        .persistent()
        .set(&backer_kyc_symbol(), &backer_map);

    // Emit event
    events::publish_backer_contribution_recorded_event(
        env,
        backer.clone(),
        campaign_id,
        amount,
        env.ledger().timestamp(),
    );

    reentrancy::exit(env);
}

/// Get backer KYC status.
pub fn get_backer_kyc_status(env: &Env, backer: &Address) -> BackerKycStatus {
    let backer_map: Map<Address, BackerKycStatus> = env
        .storage()
        .persistent()
        .get(&backer_kyc_symbol())
        .unwrap_or_else(|| Map::new(env));

    backer_map
        .get(backer.clone())
        .unwrap_or_else(|| BackerKycStatus {
            backer: backer.clone(),
            kyc_status: get_kyc_status(env, backer),
            campaigns_backed: 0,
            total_backed_amount: 0,
            last_kyc_check: env.ledger().timestamp(),
        })
}

// ── Reviewer Role Management ─────────────────────────────────────────────────

/// Grant KYC reviewer role to a user.
pub fn grant_kyc_reviewer_role(env: &Env, admin: &Address, reviewer: &Address) {
    core::assert_admin(env, admin);
    reentrancy::enter(env);

    let mut reviewer_map: Map<Address, bool> = env
        .storage()
        .persistent()
        .get(&kyc_reviewer_symbol())
        .unwrap_or_else(|| Map::new(env));

    reviewer_map.set(reviewer.clone(), true);
    env.storage()
        .persistent()
        .set(&kyc_reviewer_symbol(), &reviewer_map);

    events::publish_kyc_reviewer_role_granted_event(
        env,
        admin.clone(),
        reviewer.clone(),
        env.ledger().timestamp(),
    );

    reentrancy::exit(env);
}

/// Revoke KYC reviewer role from a user.
pub fn revoke_kyc_reviewer_role(env: &Env, admin: &Address, reviewer: &Address) {
    core::assert_admin(env, admin);
    reentrancy::enter(env);

    let mut reviewer_map: Map<Address, bool> = env
        .storage()
        .persistent()
        .get(&kyc_reviewer_symbol())
        .unwrap_or_else(|| Map::new(env));

    reviewer_map.remove(reviewer.clone());
    env.storage()
        .persistent()
        .set(&kyc_reviewer_symbol(), &reviewer_map);

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

    let reviewer_map: Map<Address, bool> = env
        .storage()
        .persistent()
        .get(&kyc_reviewer_symbol())
        .unwrap_or_else(|| Map::new(env));

    reviewer_map.get(user.clone()).unwrap_or(false)
}

// ── Helper Functions ──────────────────────────────────────────────────────────

fn assert_kyc_reviewer(env: &Env, reviewer: &Address) {
    if !has_kyc_reviewer_role(env, reviewer) {
        panic_with_error!(env, ContractError::NotAuthorized);
    }
}

fn remove_from_pending_kyc(env: &Env, request_id: u64) {
    let pending: Vec<u64> = env
        .storage()
        .persistent()
        .get(&kyc_pending_symbol())
        .unwrap_or_else(|| Vec::new(env));

    let mut updated = Vec::new(env);
    for req_id in pending.iter() {
        if req_id != request_id {
            updated.push_back(req_id);
        }
    }

    env.storage()
        .persistent()
        .set(&kyc_pending_symbol(), &updated);
}

fn contains_address(addresses: &Vec<Address>, target: &Address) -> bool {
    for addr in addresses.iter() {
        if addr == *target {
            return true;
        }
    }
    false
}
