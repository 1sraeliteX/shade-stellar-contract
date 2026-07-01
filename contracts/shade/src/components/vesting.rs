use crate::components::admin;
use crate::errors::ContractError;
use crate::events;
use crate::types::{CrowdfundVestingConfig, DataKey, VestingSchedule, VestingTimeline};
use soroban_sdk::{panic_with_error, Address, Env, String};

fn get_vesting_timeline_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::VestingTimelineCount)
        .unwrap_or(0)
}

fn set_vesting_timeline_count(env: &Env, count: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::VestingTimelineCount, &count);
}

pub fn create_vesting_timeline(
    env: &Env,
    admin: Address,
    name: String,
    cliff_duration: u64,
    vesting_duration: u64,
    unlock_percentage: i128,
) -> u64 {
    admin.require_auth();
    let contract_admin = admin::get_admin(env);
    if admin != contract_admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    if cliff_duration == 0 || vesting_duration == 0 {
        panic_with_error!(env, ContractError::InvalidInterval);
    }
    if unlock_percentage <= 0 || unlock_percentage > 10000 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let timeline_id = get_vesting_timeline_count(env) + 1;
    set_vesting_timeline_count(env, timeline_id);

    let now = env.ledger().timestamp();
    let timeline = VestingTimeline {
        id: timeline_id,
        name: name.clone(),
        cliff_duration,
        vesting_duration,
        unlock_percentage,
        admin: admin.clone(),
        created_at: now,
    };

    env.storage()
        .persistent()
        .set(&DataKey::VestingTimeline(timeline_id), &timeline);

    events::publish_vesting_timeline_created_event(
        env,
        timeline_id,
        name,
        cliff_duration,
        vesting_duration,
        admin,
        now,
    );

    timeline_id
}

pub fn get_vesting_timeline(env: &Env, timeline_id: u64) -> VestingTimeline {
    env.storage()
        .persistent()
        .get(&DataKey::VestingTimeline(timeline_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidInterval))
}

pub fn update_vesting_timeline(
    env: &Env,
    admin: Address,
    timeline_id: u64,
    cliff_duration: u64,
    vesting_duration: u64,
) {
    admin.require_auth();
    let contract_admin = admin::get_admin(env);
    if admin != contract_admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    if cliff_duration == 0 || vesting_duration == 0 {
        panic_with_error!(env, ContractError::InvalidInterval);
    }

    let mut timeline = get_vesting_timeline(env, timeline_id);
    timeline.cliff_duration = cliff_duration;
    timeline.vesting_duration = vesting_duration;

    env.storage()
        .persistent()
        .set(&DataKey::VestingTimeline(timeline_id), &timeline);

    let now = env.ledger().timestamp();
    events::publish_vesting_timeline_updated_event(
        env,
        timeline_id,
        cliff_duration,
        vesting_duration,
        admin,
        now,
    );
}

pub fn configure_crowdfund_vesting(
    env: &Env,
    admin: Address,
    crowdfund_id: u64,
    timeline_id: u64,
    total_vesting_amount: i128,
) {
    admin.require_auth();
    let contract_admin = admin::get_admin(env);
    if admin != contract_admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    if total_vesting_amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let _timeline = get_vesting_timeline(env, timeline_id);

    let config = CrowdfundVestingConfig {
        crowdfund_id,
        timeline_id,
        total_vesting_amount,
        configured_at: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DataKey::CrowdfundVestingConfig(crowdfund_id), &config);

    let now = env.ledger().timestamp();
    events::publish_crowdfund_vesting_configured_event(
        env,
        crowdfund_id,
        timeline_id,
        total_vesting_amount,
        admin,
        now,
    );
}

pub fn get_crowdfund_vesting_config(env: &Env, crowdfund_id: u64) -> CrowdfundVestingConfig {
    env.storage()
        .persistent()
        .get(&DataKey::CrowdfundVestingConfig(crowdfund_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound))
}

pub fn add_vesting_schedule(
    env: &Env,
    admin: Address,
    timeline_id: u64,
    tranche_index: u64,
    unlock_amount: i128,
    unlock_timestamp: u64,
) {
    admin.require_auth();
    let contract_admin = admin::get_admin(env);
    if admin != contract_admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    if unlock_amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let _timeline = get_vesting_timeline(env, timeline_id);

    let schedule = VestingSchedule {
        timeline_id,
        tranche_index,
        unlock_amount,
        unlock_timestamp,
        released: false,
    };

    env.storage()
        .persistent()
        .set(&DataKey::VestingSchedule(timeline_id, tranche_index), &schedule);
}

pub fn release_vesting_schedule(
    env: &Env,
    admin: Address,
    timeline_id: u64,
    tranche_index: u64,
) {
    admin.require_auth();
    let contract_admin = admin::get_admin(env);
    if admin != contract_admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let mut schedule = env
        .storage()
        .persistent()
        .get::<_, VestingSchedule>(&DataKey::VestingSchedule(timeline_id, tranche_index))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound));

    if schedule.released {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    let now = env.ledger().timestamp();
    if now < schedule.unlock_timestamp {
        panic_with_error!(env, ContractError::InvoiceExpired);
    }

    schedule.released = true;
    env.storage()
        .persistent()
        .set(&DataKey::VestingSchedule(timeline_id, tranche_index), &schedule);

    events::publish_vesting_schedule_released_event(
        env,
        timeline_id,
        tranche_index,
        schedule.unlock_amount,
        now,
    );
}
