use crate::components::{admin, merchant};
use crate::errors::ContractError;
use crate::events;
use crate::types::{Campaign, CampaignStatus, DataKey};
use soroban_sdk::{panic_with_error, token, Address, Env, Vec};

pub fn create_campaign(
    env: &Env,
    merchant_addr: &Address,
    goal: i128,
    token_addr: &Address,
    deadline: u64,
) -> u64 {
    merchant_addr.require_auth();

    if goal <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }
    if deadline <= env.ledger().timestamp() {
        panic_with_error!(env, ContractError::InvoiceExpired);
    }
    if !admin::is_accepted_token(env, token_addr) {
        panic_with_error!(env, ContractError::TokenNotAccepted);
    }
    if !merchant::is_token_accepted_for_merchant(env, merchant_addr, token_addr) {
        panic_with_error!(env, ContractError::TokenNotAcceptedByMerchant);
    }

    let merchant_id = merchant::get_merchant_id(env, merchant_addr);
    if !merchant::is_merchant_active(env, merchant_id) {
        panic_with_error!(env, ContractError::MerchantNotActive);
    }

    let id = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignCount)
        .unwrap_or(0u64)
        + 1;

    let campaign = Campaign {
        id,
        merchant_id,
        goal,
        raised: 0,
        token: token_addr.clone(),
        deadline,
        status: CampaignStatus::Active,
        created_at: env.ledger().timestamp(),
        finalized_at: None,
        total_refunded: 0,
        refund_count: 0,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Campaign(id), &campaign);
    env.storage().persistent().set(&DataKey::CampaignCount, &id);
    env.storage()
        .persistent()
        .set(&DataKey::CampaignBackers(id), &Vec::<Address>::new(env));
    env.storage()
        .persistent()
        .set(&DataKey::CampaignRefundCursor(id), &0u32);

    events::publish_campaign_created_event(
        env,
        id,
        merchant_addr.clone(),
        merchant_id,
        goal,
        token_addr.clone(),
        deadline,
        env.ledger().timestamp(),
    );

    id
}

pub fn pledge_campaign(env: &Env, backer: &Address, campaign_id: u64, amount: i128) {
    backer.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let mut campaign = get_campaign(env, campaign_id);
    if campaign.status != CampaignStatus::Active {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }
    if env.ledger().timestamp() > campaign.deadline {
        panic_with_error!(env, ContractError::InvoiceExpired);
    }
    if !admin::is_accepted_token(env, &campaign.token) {
        panic_with_error!(env, ContractError::TokenNotAccepted);
    }

    token::TokenClient::new(env, &campaign.token).transfer(
        backer,
        &env.current_contract_address(),
        &amount,
    );

    let pledge_key = DataKey::CampaignPledge(campaign_id, backer.clone());
    let previous = env.storage().persistent().get(&pledge_key).unwrap_or(0i128);
    env.storage()
        .persistent()
        .set(&pledge_key, &previous.saturating_add(amount));

    if previous == 0 {
        let backers_key = DataKey::CampaignBackers(campaign_id);
        let mut backers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&backers_key)
            .unwrap_or_else(|| Vec::new(env));
        backers.push_back(backer.clone());
        env.storage().persistent().set(&backers_key, &backers);
    }

    campaign.raised = campaign.raised.saturating_add(amount);
    env.storage()
        .persistent()
        .set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_pledged_event(
        env,
        campaign_id,
        campaign.merchant_id,
        backer.clone(),
        amount,
        campaign.raised,
        campaign.goal,
        campaign.token.clone(),
        env.ledger().timestamp(),
    );
}

pub fn finalize_campaign(env: &Env, caller: &Address, campaign_id: u64) {
    caller.require_auth();

    let mut campaign = get_campaign(env, campaign_id);
    if campaign.status != CampaignStatus::Active {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }
    if env.ledger().timestamp() <= campaign.deadline {
        panic_with_error!(env, ContractError::InvoiceNotPaid);
    }

    let merchant_addr = merchant_id_to_address(env, campaign.merchant_id);
    if *caller != merchant_addr {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    if campaign.raised >= campaign.goal {
        release_successful_campaign(env, &mut campaign, merchant_addr);
    } else {
        campaign.status = CampaignStatus::Failed;
        campaign.finalized_at = Some(env.ledger().timestamp());
        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);
        events::publish_campaign_failed_event(
            env,
            campaign_id,
            campaign.merchant_id,
            campaign.raised,
            campaign.goal,
            campaign.token.clone(),
            env.ledger().timestamp(),
        );
    }
}

pub fn claim_campaign_refund(env: &Env, backer: &Address, campaign_id: u64) -> i128 {
    backer.require_auth();
    refund_backer(env, campaign_id, backer)
}

pub fn process_failed_campaign_refunds(env: &Env, campaign_id: u64, limit: u32) -> (i128, u32) {
    if limit == 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let mut campaign = ensure_failed_campaign(env, campaign_id);
    let backers: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignBackers(campaign_id))
        .unwrap_or_else(|| Vec::new(env));
    let mut cursor: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::CampaignRefundCursor(campaign_id))
        .unwrap_or(0u32);
    let mut processed = 0u32;
    let mut total_refunded = 0i128;
    let token_client = token::TokenClient::new(env, &campaign.token);
    let contract = env.current_contract_address();

    while cursor < backers.len() && processed < limit {
        let backer = backers
            .get(cursor)
            .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound));
        let pledge_key = DataKey::CampaignPledge(campaign_id, backer.clone());
        let pledge = env.storage().persistent().get(&pledge_key).unwrap_or(0i128);

        if pledge > 0 {
            env.storage().persistent().set(&pledge_key, &0i128);
            token_client.transfer(&contract, &backer, &pledge);
            campaign.total_refunded = campaign.total_refunded.saturating_add(pledge);
            campaign.refund_count = campaign.refund_count.saturating_add(1);
            total_refunded = total_refunded.saturating_add(pledge);
            processed = processed.saturating_add(1);

            events::publish_campaign_refund_claimed_event(
                env,
                campaign_id,
                campaign.merchant_id,
                backer,
                pledge,
                campaign.total_refunded,
                campaign.token.clone(),
                env.ledger().timestamp(),
            );
        }

        cursor = cursor.saturating_add(1);
    }

    if cursor >= backers.len() {
        campaign.status = CampaignStatus::Refunded;
    }

    env.storage()
        .persistent()
        .set(&DataKey::CampaignRefundCursor(campaign_id), &cursor);
    env.storage()
        .persistent()
        .set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_failed_campaign_refund_batch_event(
        env,
        campaign_id,
        campaign.merchant_id,
        total_refunded,
        processed,
        cursor,
        campaign.status,
        campaign.token.clone(),
        env.ledger().timestamp(),
    );

    (total_refunded, processed)
}

pub fn get_campaign(env: &Env, campaign_id: u64) -> Campaign {
    env.storage()
        .persistent()
        .get(&DataKey::Campaign(campaign_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound))
}

pub fn get_campaign_pledge(env: &Env, campaign_id: u64, backer: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::CampaignPledge(campaign_id, backer.clone()))
        .unwrap_or(0)
}

fn refund_backer(env: &Env, campaign_id: u64, backer: &Address) -> i128 {
    let mut campaign = ensure_failed_campaign(env, campaign_id);
    let pledge_key = DataKey::CampaignPledge(campaign_id, backer.clone());
    let pledge = env.storage().persistent().get(&pledge_key).unwrap_or(0i128);
    if pledge <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    env.storage().persistent().set(&pledge_key, &0i128);
    token::TokenClient::new(env, &campaign.token).transfer(
        &env.current_contract_address(),
        backer,
        &pledge,
    );

    campaign.total_refunded = campaign.total_refunded.saturating_add(pledge);
    campaign.refund_count = campaign.refund_count.saturating_add(1);
    env.storage()
        .persistent()
        .set(&DataKey::Campaign(campaign_id), &campaign);

    events::publish_campaign_refund_claimed_event(
        env,
        campaign_id,
        campaign.merchant_id,
        backer.clone(),
        pledge,
        campaign.total_refunded,
        campaign.token.clone(),
        env.ledger().timestamp(),
    );

    pledge
}

fn ensure_failed_campaign(env: &Env, campaign_id: u64) -> Campaign {
    let mut campaign = get_campaign(env, campaign_id);

    if campaign.status == CampaignStatus::Refunded {
        return campaign;
    }
    if campaign.status == CampaignStatus::Successful {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }
    if env.ledger().timestamp() <= campaign.deadline {
        panic_with_error!(env, ContractError::InvoiceNotPaid);
    }
    if campaign.raised >= campaign.goal {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    if campaign.status == CampaignStatus::Active {
        campaign.status = CampaignStatus::Failed;
        campaign.finalized_at = Some(env.ledger().timestamp());
        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);
        events::publish_campaign_failed_event(
            env,
            campaign_id,
            campaign.merchant_id,
            campaign.raised,
            campaign.goal,
            campaign.token.clone(),
            env.ledger().timestamp(),
        );
    }

    campaign
}

fn release_successful_campaign(env: &Env, campaign: &mut Campaign, merchant_addr: Address) {
    let merchant_account = merchant::get_merchant_account(env, campaign.merchant_id);
    let platform_account = admin::get_platform_account(env);
    let fee = admin::calculate_fee(env, &merchant_addr, &campaign.token, campaign.raised);
    if fee < 0 || fee >= campaign.raised {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let merchant_amount = campaign.raised - fee;
    let token_client = token::TokenClient::new(env, &campaign.token);
    let contract = env.current_contract_address();

    if merchant_amount > 0 {
        token_client.transfer(&contract, &merchant_account, &merchant_amount);
    }
    if fee > 0 {
        token_client.transfer(&contract, &platform_account, &fee);
    }

    admin::record_merchant_payment(env, &merchant_addr, &campaign.token, campaign.raised, fee);

    campaign.status = CampaignStatus::Successful;
    campaign.finalized_at = Some(env.ledger().timestamp());
    env.storage()
        .persistent()
        .set(&DataKey::Campaign(campaign.id), &*campaign);

    events::publish_campaign_succeeded_event(
        env,
        campaign.id,
        campaign.merchant_id,
        merchant_account,
        campaign.raised,
        fee,
        merchant_amount,
        campaign.token.clone(),
        env.ledger().timestamp(),
    );
}

fn merchant_id_to_address(env: &Env, merchant_id: u64) -> Address {
    let merchant_record = merchant::get_merchant(env, merchant_id);
    merchant_record.address
}
