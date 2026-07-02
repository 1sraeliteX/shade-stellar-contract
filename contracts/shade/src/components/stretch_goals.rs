use crate::components::admin;
use crate::errors::ContractError;
use crate::events;
use crate::types::{DataKey, StretchGoal, StretchGoalReward, StretchGoalStatus};
use soroban_sdk::{panic_with_error, Address, Env, String, Vec};

fn get_stretch_goal_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::StretchGoalCount)
        .unwrap_or(0)
}

fn set_stretch_goal_count(env: &Env, count: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::StretchGoalCount, &count);
}

pub fn create_stretch_goal(
    env: &Env,
    merchant: Address,
    crowdfund_id: u64,
    target_amount: i128,
    description: String,
    reward_description: String,
) -> u64 {
    merchant.require_auth();

    if target_amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    if description.len() == 0 || reward_description.len() == 0 {
        panic_with_error!(env, ContractError::InvalidDescription);
    }

    let goal_id = get_stretch_goal_count(env) + 1;
    set_stretch_goal_count(env, goal_id);

    let now = env.ledger().timestamp();
    let goal = StretchGoal {
        id: goal_id,
        crowdfund_id,
        target_amount,
        description: description.clone(),
        reward_description,
        status: StretchGoalStatus::Pending,
        created_at: now,
        unlocked_at: 0,
    };

    env.storage()
        .persistent()
        .set(&DataKey::StretchGoal(goal_id), &goal);

    let mut crowdfund_goals: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::CrowdfundStretchGoals(crowdfund_id))
        .unwrap_or_else(Vec::new);
    crowdfund_goals.push_back(goal_id);
    env.storage()
        .persistent()
        .set(&DataKey::CrowdfundStretchGoals(crowdfund_id), &crowdfund_goals);

    events::publish_stretch_goal_created_event(
        env,
        goal_id,
        crowdfund_id,
        target_amount,
        description.len() as u64,
        now,
    );

    goal_id
}

pub fn get_stretch_goal(env: &Env, goal_id: u64) -> StretchGoal {
    env.storage()
        .persistent()
        .get(&DataKey::StretchGoal(goal_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::StretchGoalNotFound))
}

pub fn unlock_stretch_goal(env: &Env, admin: Address, goal_id: u64, current_amount: i128) {
    admin.require_auth();
    let contract_admin = admin::get_admin(env);
    if admin != contract_admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let mut goal = get_stretch_goal(env, goal_id);

    if goal.status != StretchGoalStatus::Pending {
        panic_with_error!(env, ContractError::GoalAlreadyUnlocked);
    }

    if current_amount < goal.target_amount {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    goal.status = StretchGoalStatus::Unlocked;
    goal.unlocked_at = env.ledger().timestamp();

    env.storage()
        .persistent()
        .set(&DataKey::StretchGoal(goal_id), &goal);

    events::publish_stretch_goal_unlocked_event(
        env,
        goal_id,
        goal.crowdfund_id,
        current_amount,
        env.ledger().timestamp(),
    );
}

pub fn distribute_stretch_goal_reward(
    env: &Env,
    admin: Address,
    goal_id: u64,
    backer: Address,
    reward_amount: i128,
) {
    admin.require_auth();
    let contract_admin = admin::get_admin(env);
    if admin != contract_admin {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let goal = get_stretch_goal(env, goal_id);

    if goal.status != StretchGoalStatus::Unlocked {
        panic_with_error!(env, ContractError::StretchGoalNotUnlocked);
    }

    if reward_amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let reward = StretchGoalReward {
        goal_id,
        backer: backer.clone(),
        reward_amount,
        claimed: false,
        created_at: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DataKey::StretchGoalReward(goal_id), &reward);

    events::publish_stretch_goal_reward_distributed_event(
        env,
        goal_id,
        backer,
        reward_amount,
        env.ledger().timestamp(),
    );
}

pub fn claim_stretch_goal_reward(env: &Env, backer: Address, goal_id: u64) {
    backer.require_auth();

    let mut reward = env
        .storage()
        .persistent()
        .get::<_, StretchGoalReward>(&DataKey::StretchGoalReward(goal_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::StretchGoalNotFound));

    if reward.backer != backer {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    if reward.claimed {
        panic_with_error!(env, ContractError::RewardAlreadyClaimed);
    }

    reward.claimed = true;
    env.storage()
        .persistent()
        .set(&DataKey::StretchGoalReward(goal_id), &reward);

    events::publish_stretch_goal_reward_claimed_event(
        env,
        goal_id,
        backer,
        reward.reward_amount,
        env.ledger().timestamp(),
    );
}

pub fn get_crowdfund_stretch_goals(env: &Env, crowdfund_id: u64) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::CrowdfundStretchGoals(crowdfund_id))
        .unwrap_or_else(Vec::new)
}

pub fn get_stretch_goal_reward(env: &Env, goal_id: u64) -> StretchGoalReward {
    env.storage()
        .persistent()
        .get(&DataKey::StretchGoalReward(goal_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::StretchGoalNotFound))
}
