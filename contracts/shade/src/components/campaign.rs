use crate::components::{core, reentrancy};
use crate::errors::ContractError;
use crate::events;
use crate::types::{Campaign, CampaignAffiliate, CampaignParticipant, DataKey};
use soroban_sdk::{panic_with_error, Address, Env, String, Vec};

pub fn create_campaign(
    env: &Env,
    caller: &Address,
    name: &String,
    charity: bool,
    fee_waiver_bps: u32,
    discount_bps: u32,
    stake_required: i128,
) -> u64 {
    reentrancy::enter(env);
    caller.require_auth();

    if fee_waiver_bps > 10_000 || discount_bps > 10_000 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }
    if stake_required < 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let campaign_id = get_campaign_count(env) + 1;
    let campaign = Campaign {
        id: campaign_id,
        owner: caller.clone(),
        name: name.clone(),
        charity,
        fee_waiver_bps,
        discount_bps,
        stake_required,
        total_raised: 0,
        total_staked: 0,
        total_slashed: 0,
        total_commissions_paid: 0,
        active: true,
        created_at: env.ledger().timestamp(),
    };

    env.storage().persistent().set(&DataKey::Campaign(campaign_id), &campaign);
    env.storage().persistent().set(&DataKey::CampaignCount, &campaign_id);
    env.storage()
        .persistent()
        .set(&DataKey::CampaignParticipants(campaign_id), &Vec::<Address>::new(env));

    events::publish_campaign_created_event(
        env,
        campaign_id,
        caller.clone(),
        name.clone(),
        charity,
        fee_waiver_bps,
        discount_bps,
        env.ledger().timestamp(),
    );

    reentrancy::exit(env);
    campaign_id
}

pub fn configure_campaign_fee_policy(
    env: &Env,
    caller: &Address,
    campaign_id: u64,
    fee_waiver_bps: u32,
    discount_bps: u32,
) {
    reentrancy::enter(env);
    caller.require_auth();

    if fee_waiver_bps > 10_000 || discount_bps > 10_000 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let mut campaign = get_campaign(env, campaign_id);
    let admin = core::get_admin(env);
    if *caller != campaign.owner && *caller != admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    campaign.fee_waiver_bps = fee_waiver_bps;
    campaign.discount_bps = discount_bps;
    env.storage().persistent().set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_fee_policy_configured_event(
        env,
        campaign_id,
        caller.clone(),
        fee_waiver_bps,
        discount_bps,
        env.ledger().timestamp(),
    );
    reentrancy::exit(env);
}

pub fn calculate_campaign_discounted_amount(env: &Env, campaign_id: u64, amount: i128) -> i128 {
    if amount <= 0 {
        return 0;
    }

    let campaign = get_campaign(env, campaign_id);
    let waiver = (amount * i128::from(campaign.fee_waiver_bps)) / 10_000i128;
    let discount = (amount * i128::from(campaign.discount_bps)) / 10_000i128;
    amount - waiver - discount
}

pub fn record_campaign_contribution(env: &Env, caller: &Address, campaign_id: u64, amount: i128) {
    reentrancy::enter(env);
    caller.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let mut campaign = get_campaign(env, campaign_id);
    let mut participant = get_participant(env, campaign_id, caller);

    campaign.total_raised += amount;
    participant.contributed += amount;
    participant.score += amount;

    store_participant(env, campaign_id, &participant);
    env.storage().persistent().set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_contribution_recorded_event(
        env,
        campaign_id,
        caller.clone(),
        amount,
        campaign.total_raised,
        env.ledger().timestamp(),
    );
    reentrancy::exit(env);
}

pub fn stake_campaign(env: &Env, caller: &Address, campaign_id: u64, amount: i128) {
    reentrancy::enter(env);
    caller.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let mut campaign = get_campaign(env, campaign_id);
    let mut participant = get_participant(env, campaign_id, caller);

    participant.staked += amount;
    participant.score += amount;
    campaign.total_staked += amount;

    store_participant(env, campaign_id, &participant);
    env.storage().persistent().set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_staked_event(
        env,
        campaign_id,
        caller.clone(),
        amount,
        participant.staked,
        env.ledger().timestamp(),
    );
    reentrancy::exit(env);
}

pub fn slash_campaign_stake(
    env: &Env,
    caller: &Address,
    campaign_id: u64,
    participant_address: &Address,
    amount: i128,
) {
    reentrancy::enter(env);
    caller.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let mut campaign = get_campaign(env, campaign_id);
    let admin = core::get_admin(env);
    if *caller != campaign.owner && *caller != admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let mut participant = get_participant(env, campaign_id, participant_address);
    if participant.staked < amount {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    participant.staked -= amount;
    participant.slashed += amount;
    participant.score -= amount;
    campaign.total_staked -= amount;
    campaign.total_slashed += amount;

    store_participant(env, campaign_id, &participant);
    env.storage().persistent().set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_slashed_event(
        env,
        campaign_id,
        participant_address.clone(),
        amount,
        participant.staked,
        env.ledger().timestamp(),
    );
    reentrancy::exit(env);
}

pub fn register_affiliate(
    env: &Env,
    caller: &Address,
    campaign_id: u64,
    affiliate_address: &Address,
    commission_bps: u32,
) {
    reentrancy::enter(env);
    caller.require_auth();

    if commission_bps > 10_000 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let campaign = get_campaign(env, campaign_id);
    let admin = core::get_admin(env);
    if *caller != campaign.owner && *caller != admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let affiliate = CampaignAffiliate {
        campaign_id,
        affiliate: affiliate_address.clone(),
        commission_bps,
        total_paid: 0,
        active: true,
    };

    env.storage().persistent().set(
        &DataKey::CampaignAffiliate(campaign_id, affiliate_address.clone()),
        &affiliate,
    );

    events::publish_affiliate_registered_event(
        env,
        campaign_id,
        affiliate_address.clone(),
        commission_bps,
        env.ledger().timestamp(),
    );
    reentrancy::exit(env);
}

pub fn pay_affiliate_commission(
    env: &Env,
    caller: &Address,
    campaign_id: u64,
    affiliate_address: &Address,
    amount: i128,
) {
    reentrancy::enter(env);
    caller.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let mut campaign = get_campaign(env, campaign_id);
    let admin = core::get_admin(env);
    if *caller != campaign.owner && *caller != admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let mut affiliate = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignAffiliate(campaign_id, affiliate_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::AffiliateNotFound));

    affiliate.total_paid += amount;
    campaign.total_commissions_paid += amount;

    env.storage().persistent().set(
        &DataKey::CampaignAffiliate(campaign_id, affiliate_address.clone()),
        &affiliate,
    );
    env.storage().persistent().set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_affiliate_commission_paid_event(
        env,
        campaign_id,
        affiliate_address.clone(),
        amount,
        affiliate.total_paid,
        env.ledger().timestamp(),
    );
    reentrancy::exit(env);
}

pub fn get_campaign(env: &Env, campaign_id: u64) -> Campaign {
    env.storage()
        .persistent()
        .get(&DataKey::Campaign(campaign_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::CampaignNotFound))
}

pub fn get_campaign_participant(env: &Env, campaign_id: u64, participant: &Address) -> CampaignParticipant {
    get_participant(env, campaign_id, participant)
}

pub fn get_campaign_affiliate(env: &Env, campaign_id: u64, affiliate: &Address) -> CampaignAffiliate {
    env.storage()
        .persistent()
        .get(&DataKey::CampaignAffiliate(campaign_id, affiliate.clone()))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::AffiliateNotFound))
}

pub fn get_campaign_leaderboard(env: &Env, campaign_id: u64, limit: u32) -> Vec<(Address, i128)> {
    let participant_ids = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignParticipants(campaign_id))
        .unwrap_or_else(|| Vec::new(env));

    let mut rows: Vec<(Address, i128)> = Vec::new(env);
    for participant_id in participant_ids.iter() {
        let participant = get_participant(env, campaign_id, &participant_id);
        rows.push_back((participant_id.clone(), participant.score));
    }

    let n = rows.len();
    let mut i: u32 = 1;
    while i < n {
        let mut j = i;
        while j > 0 {
            let prev = rows.get_unchecked(j - 1);
            let curr = rows.get_unchecked(j);
            if curr.1 > prev.1 {
                rows.set(j - 1, curr);
                rows.set(j, prev);
                j -= 1;
            } else {
                break;
            }
        }
        i += 1;
    }

    while rows.len() > limit {
        rows.pop_back();
    }
    rows
}

fn get_campaign_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::CampaignCount)
        .unwrap_or(0)
}

fn get_participant(env: &Env, campaign_id: u64, participant: &Address) -> CampaignParticipant {
    env.storage()
        .persistent()
        .get(&DataKey::CampaignParticipant(campaign_id, participant.clone()))
        .unwrap_or(CampaignParticipant {
            campaign_id,
            participant: participant.clone(),
            contributed: 0,
            staked: 0,
            slashed: 0,
            commissions_paid: 0,
            score: 0,
        })
}

fn store_participant(env: &Env, campaign_id: u64, participant: &CampaignParticipant) {
    let participant_ids = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignParticipants(campaign_id))
        .unwrap_or_else(|| Vec::new(env));

    let mut exists = false;
    for existing in participant_ids.iter() {
        if existing == participant.participant {
            exists = true;
            break;
        }
    }

    if !exists {
        let mut updated_ids = Vec::new(env);
        for existing in participant_ids.iter() {
            updated_ids.push_back(existing);
        }
        updated_ids.push_back(participant.participant.clone());
        env.storage()
            .persistent()
            .set(&DataKey::CampaignParticipants(campaign_id), &updated_ids);
    }

    env.storage().persistent().set(
        &DataKey::CampaignParticipant(campaign_id, participant.participant.clone()),
        participant,
    );
}
