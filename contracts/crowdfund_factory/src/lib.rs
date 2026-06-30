#![no_std]

mod errors;
#[cfg(test)]
mod test;
#[cfg(test)]
mod tests;

use crate::errors::FactoryError;
use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, panic_with_error, Address, Bytes,
    BytesN, Env, IntoVal, String, Symbol, Vec,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CampaignRef {
    pub campaign_id: u64,
    pub contract: Address,
    pub organizer: Address,
    pub deployed_at: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DaoProposalStatus {
    Voting,
    Executed,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DaoProposal {
    pub id: u64,
    pub proposer: Address,
    pub description: String,
    pub new_wasm_hash: BytesN<32>,
    pub votes_for: u32,
    pub votes_against: u32,
    pub status: DaoProposalStatus,
    pub created_at: u64,
    pub voting_deadline: u64,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    CrowdfundWasmHash,
    CampaignRef(u64),
    CampaignRefCount,
    // Address authorised to add/remove DAO members (#359).
    DaoAdmin,
    // Whether a given address is a voting DAO member.
    DaoMember(Address),
    DaoMemberCount,
    // Basis points (1 bp = 0.01%) of the member count required to reach quorum.
    DaoQuorumBps,
    DaoProposal(u64),
    DaoProposalCount,
    // Tracks whether a member already voted on a specific proposal.
    DaoVote(u64, Address),
}

#[contractevent]
pub struct CampaignDeployedEvent {
    pub campaign_id: u64,
    pub contract: Address,
    pub organizer: Address,
    pub deployed_at: u64,
}

#[contractevent]
pub struct DaoMemberAddedEvent {
    pub member: Address,
}

#[contractevent]
pub struct DaoMemberRemovedEvent {
    pub member: Address,
}

#[contractevent]
pub struct DaoProposalCreatedEvent {
    pub proposal_id: u64,
    pub proposer: Address,
    pub description: String,
    pub voting_deadline: u64,
}

#[contractevent]
pub struct DaoVoteCastEvent {
    pub proposal_id: u64,
    pub voter: Address,
    pub support: bool,
    pub votes_for: u32,
    pub votes_against: u32,
}

#[contractevent]
pub struct DaoProposalExecutedEvent {
    pub proposal_id: u64,
    pub new_wasm_hash: BytesN<32>,
}

#[contractevent]
pub struct DaoProposalRejectedEvent {
    pub proposal_id: u64,
}

fn get_campaign_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::CampaignRefCount)
        .unwrap_or(0)
}

fn get_dao_admin(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::DaoAdmin)
        .unwrap_or_else(|| panic_with_error!(env, FactoryError::DaoNotInitialized))
}

fn get_dao_member_count(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::DaoMemberCount)
        .unwrap_or(0)
}

fn get_dao_proposal_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::DaoProposalCount)
        .unwrap_or(0)
}

fn is_dao_member(env: &Env, address: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::DaoMember(address.clone()))
        .unwrap_or(false)
}

#[contract]
pub struct CrowdfundFactory;

#[contractimpl]
impl CrowdfundFactory {
    pub fn initialize(env: Env, crowdfund_wasm_hash: BytesN<32>) {
        if env.storage().persistent().has(&DataKey::CrowdfundWasmHash) {
            panic_with_error!(&env, FactoryError::AlreadyInitialized);
        }
        env.storage()
            .persistent()
            .set(&DataKey::CrowdfundWasmHash, &crowdfund_wasm_hash);
        env.storage()
            .persistent()
            .set(&DataKey::CampaignRefCount, &0_u64);
    }

    pub fn set_crowdfund_wasm_hash(env: Env, crowdfund_wasm_hash: BytesN<32>) {
        if !env.storage().persistent().has(&DataKey::CrowdfundWasmHash) {
            panic_with_error!(&env, FactoryError::NotInitialized);
        }
        env.storage()
            .persistent()
            .set(&DataKey::CrowdfundWasmHash, &crowdfund_wasm_hash);
    }

    /// Deploy and initialize an independent crowdfund campaign (#316).
    pub fn deploy_campaign(
        env: Env,
        organizer: Address,
        token: Address,
        goal: i128,
        deadline: u64,
    ) -> CampaignRef {
        organizer.require_auth();

        let wasm_hash: BytesN<32> = env
            .storage()
            .persistent()
            .get(&DataKey::CrowdfundWasmHash)
            .unwrap_or_else(|| panic_with_error!(&env, FactoryError::WasmHashNotSet));

        let random: BytesN<32> = env.prng().gen();
        let salt = env
            .crypto()
            .keccak256(&Bytes::from_slice(&env, &random.to_array()));

        let campaign_addr = env.deployer().with_current_contract(salt).deploy_v2(wasm_hash, ());
        env.invoke_contract::<()>(
            &campaign_addr,
            &Symbol::new(&env, "init_campaign"),
            (organizer.clone(), token, goal, deadline).into_val(&env),
        );

        let campaign_id = get_campaign_count(&env) + 1;
        let deployed_at = env.ledger().timestamp();
        let campaign_ref = CampaignRef {
            campaign_id,
            contract: campaign_addr.clone(),
            organizer: organizer.clone(),
            deployed_at,
        };

        env.storage()
            .persistent()
            .set(&DataKey::CampaignRef(campaign_id), &campaign_ref);
        env.storage()
            .persistent()
            .set(&DataKey::CampaignRefCount, &campaign_id);

        CampaignDeployedEvent {
            campaign_id,
            contract: campaign_addr,
            organizer,
            deployed_at,
        }
        .publish(&env);

        campaign_ref
    }

    pub fn get_campaign_ref(env: Env, campaign_id: u64) -> CampaignRef {
        env.storage()
            .persistent()
            .get(&DataKey::CampaignRef(campaign_id))
            .unwrap_or_else(|| panic_with_error!(&env, FactoryError::CampaignNotFound))
    }

    pub fn get_campaign_count(env: Env) -> u64 {
        get_campaign_count(&env)
    }

    pub fn get_all_campaigns(env: Env) -> Vec<CampaignRef> {
        let count = get_campaign_count(&env);
        let mut campaigns = Vec::new(&env);
        for i in 1..=count {
            if let Some(campaign_ref) = env.storage().persistent().get(&DataKey::CampaignRef(i)) {
                campaigns.push_back(campaign_ref);
            }
        }
        campaigns
    }

    // ── DAO governance (#359) ─────────────────────────────────────────────────

    /// One-time setup of the DAO admin and quorum. The admin manages
    /// membership; quorum is expressed in basis points of the current
    /// member count (e.g. 5000 = 50%).
    pub fn init_dao(env: Env, admin: Address, quorum_bps: u32) {
        admin.require_auth();
        if env.storage().persistent().has(&DataKey::DaoAdmin) {
            panic_with_error!(&env, FactoryError::DaoAlreadyInitialized);
        }
        if quorum_bps == 0 || quorum_bps > 10_000 {
            panic_with_error!(&env, FactoryError::InvalidQuorum);
        }
        env.storage().persistent().set(&DataKey::DaoAdmin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::DaoQuorumBps, &quorum_bps);
        env.storage()
            .persistent()
            .set(&DataKey::DaoMemberCount, &0_u32);
        env.storage()
            .persistent()
            .set(&DataKey::DaoProposalCount, &0_u64);
    }

    pub fn add_dao_member(env: Env, admin: Address, member: Address) {
        admin.require_auth();
        if admin != get_dao_admin(&env) {
            panic_with_error!(&env, FactoryError::NotDaoAdmin);
        }
        if is_dao_member(&env, &member) {
            panic_with_error!(&env, FactoryError::AlreadyDaoMember);
        }
        env.storage()
            .persistent()
            .set(&DataKey::DaoMember(member.clone()), &true);
        let count = get_dao_member_count(&env) + 1;
        env.storage()
            .persistent()
            .set(&DataKey::DaoMemberCount, &count);
        DaoMemberAddedEvent { member }.publish(&env);
    }

    pub fn remove_dao_member(env: Env, admin: Address, member: Address) {
        admin.require_auth();
        if admin != get_dao_admin(&env) {
            panic_with_error!(&env, FactoryError::NotDaoAdmin);
        }
        if !is_dao_member(&env, &member) {
            panic_with_error!(&env, FactoryError::NotDaoMember);
        }
        env.storage()
            .persistent()
            .set(&DataKey::DaoMember(member.clone()), &false);
        let count = get_dao_member_count(&env).saturating_sub(1);
        env.storage()
            .persistent()
            .set(&DataKey::DaoMemberCount, &count);
        DaoMemberRemovedEvent { member }.publish(&env);
    }

    pub fn is_dao_member(env: Env, address: Address) -> bool {
        is_dao_member(&env, &address)
    }

    pub fn get_dao_member_count(env: Env) -> u32 {
        get_dao_member_count(&env)
    }

    /// A DAO member proposes updating the platform's crowdfund wasm hash.
    /// `voting_period` is in seconds and must be > 0.
    pub fn create_dao_proposal(
        env: Env,
        proposer: Address,
        description: String,
        new_wasm_hash: BytesN<32>,
        voting_period: u64,
    ) -> u64 {
        proposer.require_auth();
        if !is_dao_member(&env, &proposer) {
            panic_with_error!(&env, FactoryError::NotDaoMember);
        }
        if voting_period == 0 {
            panic_with_error!(&env, FactoryError::InvalidVotingPeriod);
        }

        let proposal_id = get_dao_proposal_count(&env) + 1;
        let created_at = env.ledger().timestamp();
        let voting_deadline = created_at + voting_period;

        let proposal = DaoProposal {
            id: proposal_id,
            proposer: proposer.clone(),
            description: description.clone(),
            new_wasm_hash,
            votes_for: 0,
            votes_against: 0,
            status: DaoProposalStatus::Voting,
            created_at,
            voting_deadline,
        };

        env.storage()
            .persistent()
            .set(&DataKey::DaoProposal(proposal_id), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::DaoProposalCount, &proposal_id);

        DaoProposalCreatedEvent {
            proposal_id,
            proposer,
            description,
            voting_deadline,
        }
        .publish(&env);

        proposal_id
    }

    pub fn get_dao_proposal(env: Env, proposal_id: u64) -> DaoProposal {
        env.storage()
            .persistent()
            .get(&DataKey::DaoProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, FactoryError::DaoProposalNotFound))
    }

    pub fn get_dao_proposal_count(env: Env) -> u64 {
        get_dao_proposal_count(&env)
    }

    /// Cast a single-weight vote on a proposal still in its voting window.
    pub fn cast_dao_vote(env: Env, voter: Address, proposal_id: u64, support: bool) {
        voter.require_auth();
        if !is_dao_member(&env, &voter) {
            panic_with_error!(&env, FactoryError::NotDaoMember);
        }

        let mut proposal: DaoProposal = env
            .storage()
            .persistent()
            .get(&DataKey::DaoProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, FactoryError::DaoProposalNotFound));

        if proposal.status != DaoProposalStatus::Voting {
            panic_with_error!(&env, FactoryError::VotingClosed);
        }
        if env.ledger().timestamp() > proposal.voting_deadline {
            panic_with_error!(&env, FactoryError::VotingClosed);
        }

        let vote_key = DataKey::DaoVote(proposal_id, voter.clone());
        if env.storage().persistent().has(&vote_key) {
            panic_with_error!(&env, FactoryError::AlreadyVoted);
        }
        env.storage().persistent().set(&vote_key, &support);

        if support {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }
        env.storage()
            .persistent()
            .set(&DataKey::DaoProposal(proposal_id), &proposal);

        DaoVoteCastEvent {
            proposal_id,
            voter,
            support,
            votes_for: proposal.votes_for,
            votes_against: proposal.votes_against,
        }
        .publish(&env);
    }

    /// Finalise a proposal after its voting window closes: executes (updates
    /// the platform wasm hash) if quorum was met and votes_for is a strict
    /// majority of votes cast, otherwise rejects it. Callable by anyone.
    pub fn execute_dao_proposal(env: Env, proposal_id: u64) {
        let mut proposal: DaoProposal = env
            .storage()
            .persistent()
            .get(&DataKey::DaoProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, FactoryError::DaoProposalNotFound));

        if proposal.status != DaoProposalStatus::Voting {
            panic_with_error!(&env, FactoryError::DaoProposalAlreadyExecuted);
        }
        if env.ledger().timestamp() <= proposal.voting_deadline {
            panic_with_error!(&env, FactoryError::VotingNotClosed);
        }

        let total_votes = proposal.votes_for + proposal.votes_against;
        let member_count = get_dao_member_count(&env);
        let quorum_bps: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::DaoQuorumBps)
            .unwrap_or_else(|| panic_with_error!(&env, FactoryError::DaoNotInitialized));

        let quorum_met = (total_votes as u64) * 10_000 >= (member_count as u64) * (quorum_bps as u64);

        if quorum_met && proposal.votes_for > proposal.votes_against {
            proposal.status = DaoProposalStatus::Executed;
            env.storage()
                .persistent()
                .set(&DataKey::DaoProposal(proposal_id), &proposal);
            env.storage()
                .persistent()
                .set(&DataKey::CrowdfundWasmHash, &proposal.new_wasm_hash);

            DaoProposalExecutedEvent {
                proposal_id,
                new_wasm_hash: proposal.new_wasm_hash,
            }
            .publish(&env);
        } else {
            proposal.status = DaoProposalStatus::Rejected;
            env.storage()
                .persistent()
                .set(&DataKey::DaoProposal(proposal_id), &proposal);

            DaoProposalRejectedEvent { proposal_id }.publish(&env);
        }
    }
}
