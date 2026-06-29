use crate::components::{admin as admin_component, core as core_component, merchant};
use crate::errors::ContractError;
use crate::events;
use crate::types::{Campaign, CampaignAnnouncement, CampaignStatus, DataKey};
use soroban_sdk::{panic_with_error, Address, Env, String, Vec};

// ── Campaign creation (Issue #335) ───────────────────────────────────────────

pub fn create_campaign(
    env: &Env,
    merchant_addr: &Address,
    title: &String,
    description: &String,
    goal_amount: i128,
    token: &Address,
    end_date: u64,
) -> u64 {
    merchant_addr.require_auth();

    if end_date <= env.ledger().timestamp() {
        panic_with_error!(env, ContractError::InvalidCampaignEndDate);
    }
    if goal_amount < 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }
    if !admin_component::is_accepted_token(env, token) {
        panic_with_error!(env, ContractError::TokenNotAccepted);
    }

    let merchant_id = merchant::get_merchant_id(env, merchant_addr);
    let merchant_record = merchant::get_merchant(env, merchant_id);
    if !merchant_record.active {
        panic_with_error!(env, ContractError::MerchantNotActive);
    }

    let id = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignCount)
        .unwrap_or(0u64)
        + 1;

    let now = env.ledger().timestamp();
    let campaign = Campaign {
        id,
        merchant_id,
        merchant: merchant_addr.clone(),
        title: title.clone(),
        description: description.clone(),
        goal_amount,
        token: token.clone(),
        status: CampaignStatus::Active,
        created_at: now,
        updated_at: now,
        end_date,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Campaign(id), &campaign);
    env.storage()
        .persistent()
        .set(&DataKey::CampaignCount, &id);

    let mut merchant_campaigns: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantCampaigns(merchant_addr.clone()))
        .unwrap_or_else(|| Vec::new(env));
    merchant_campaigns.push_back(id);
    env.storage().persistent().set(
        &DataKey::MerchantCampaigns(merchant_addr.clone()),
        &merchant_campaigns,
    );

    events::publish_campaign_created_event(
        env,
        id,
        merchant_addr.clone(),
        merchant_id,
        title.clone(),
        goal_amount,
        token.clone(),
        end_date,
        now,
    );

    id
}

// ── Campaign queries ──────────────────────────────────────────────────────────

pub fn get_campaign(env: &Env, campaign_id: u64) -> Campaign {
    env.storage()
        .persistent()
        .get(&DataKey::Campaign(campaign_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::CampaignNotFound))
}

pub fn get_merchant_campaigns(env: &Env, merchant: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::MerchantCampaigns(merchant.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

// ── Campaign mutations ────────────────────────────────────────────────────────

pub fn update_campaign(
    env: &Env,
    merchant_addr: &Address,
    campaign_id: u64,
    title: &String,
    description: &String,
    end_date: u64,
) {
    merchant_addr.require_auth();

    let mut campaign = get_campaign(env, campaign_id);

    if campaign.merchant != *merchant_addr {
        panic_with_error!(env, ContractError::NotCampaignMerchant);
    }
    if campaign.status == CampaignStatus::Cancelled {
        panic_with_error!(env, ContractError::CampaignNotActive);
    }
    if campaign.status == CampaignStatus::Ended {
        panic_with_error!(env, ContractError::CampaignEnded);
    }
    if end_date <= env.ledger().timestamp() {
        panic_with_error!(env, ContractError::InvalidCampaignEndDate);
    }

    campaign.title = title.clone();
    campaign.description = description.clone();
    campaign.end_date = end_date;
    campaign.updated_at = env.ledger().timestamp();

    env.storage()
        .persistent()
        .set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_updated_event(
        env,
        campaign_id,
        merchant_addr.clone(),
        title.clone(),
        description.clone(),
        end_date,
        env.ledger().timestamp(),
    );
}

pub fn cancel_campaign(env: &Env, caller: &Address, campaign_id: u64) {
    caller.require_auth();

    let mut campaign = get_campaign(env, campaign_id);

    // Only the campaign's merchant or the contract admin may cancel.
    let admin = core_component::get_admin(env);
    if campaign.merchant != *caller && admin != *caller {
        panic_with_error!(env, ContractError::NotCampaignMerchant);
    }
    if campaign.status == CampaignStatus::Cancelled {
        panic_with_error!(env, ContractError::CampaignNotActive);
    }
    if campaign.status == CampaignStatus::Ended {
        panic_with_error!(env, ContractError::CampaignEnded);
    }

    campaign.status = CampaignStatus::Cancelled;
    campaign.updated_at = env.ledger().timestamp();

    env.storage()
        .persistent()
        .set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_cancelled_event(
        env,
        campaign_id,
        campaign.merchant.clone(),
        caller.clone(),
        env.ledger().timestamp(),
    );
}

pub fn end_campaign(env: &Env, merchant_addr: &Address, campaign_id: u64) {
    merchant_addr.require_auth();

    let mut campaign = get_campaign(env, campaign_id);

    if campaign.merchant != *merchant_addr {
        panic_with_error!(env, ContractError::NotCampaignMerchant);
    }
    if campaign.status == CampaignStatus::Cancelled {
        panic_with_error!(env, ContractError::CampaignNotActive);
    }
    if campaign.status == CampaignStatus::Ended {
        panic_with_error!(env, ContractError::CampaignEnded);
    }

    campaign.status = CampaignStatus::Ended;
    campaign.updated_at = env.ledger().timestamp();

    env.storage()
        .persistent()
        .set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_ended_event(
        env,
        campaign_id,
        merchant_addr.clone(),
        env.ledger().timestamp(),
    );
}

// ── Campaign announcements ────────────────────────────────────────────────────

pub fn post_campaign_announcement(
    env: &Env,
    merchant_addr: &Address,
    campaign_id: u64,
    title: &String,
    content: &String,
) -> u64 {
    merchant_addr.require_auth();

    let campaign = get_campaign(env, campaign_id);

    if campaign.merchant != *merchant_addr {
        panic_with_error!(env, ContractError::NotCampaignMerchant);
    }
    if campaign.status == CampaignStatus::Cancelled {
        panic_with_error!(env, ContractError::CampaignNotActive);
    }
    if campaign.status == CampaignStatus::Ended {
        panic_with_error!(env, ContractError::CampaignEnded);
    }

    let announcement_id = env
        .storage()
        .persistent()
        .get(&DataKey::AnnouncementCount)
        .unwrap_or(0u64)
        + 1;

    let announcement = CampaignAnnouncement {
        id: announcement_id,
        campaign_id,
        title: title.clone(),
        content: content.clone(),
        posted_at: env.ledger().timestamp(),
    };

    let mut announcements: Vec<CampaignAnnouncement> = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignAnnouncements(campaign_id))
        .unwrap_or_else(|| Vec::new(env));
    announcements.push_back(announcement);
    env.storage().persistent().set(
        &DataKey::CampaignAnnouncements(campaign_id),
        &announcements,
    );
    env.storage()
        .persistent()
        .set(&DataKey::AnnouncementCount, &announcement_id);

    events::publish_campaign_announcement_posted_event(
        env,
        announcement_id,
        campaign_id,
        merchant_addr.clone(),
        title.clone(),
        env.ledger().timestamp(),
    );

    announcement_id
}

pub fn get_campaign_announcements(env: &Env, campaign_id: u64) -> Vec<CampaignAnnouncement> {
    env.storage()
        .persistent()
        .get(&DataKey::CampaignAnnouncements(campaign_id))
        .unwrap_or_else(|| Vec::new(env))
}
