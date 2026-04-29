#![no_std]

mod errors;

use crate::errors::TicketingError;
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, BytesN, Env, String, Vec,
};

const HASH_LENGTH: usize = 32;

// ── Data Structures ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub event_id: u64,
    pub organizer: Address,
    pub name: String,
    pub description: String,
    pub start_time: u64,
    pub end_time: u64,
    pub max_capacity: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ticket {
    pub ticket_id: u64,
    pub event_id: u64,
    pub holder: Address,
    pub qr_hash: BytesN<32>,
    pub checked_in: bool,
    pub check_in_time: Option<u64>,
    pub refunded: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckInRecord {
    pub ticket_id: u64,
    pub checked_in_by: Address,
    pub check_in_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TicketVerification {
    pub ticket_id: u64,
    pub event_id: u64,
    pub holder: Address,
    pub valid: bool,
    pub already_checked_in: bool,
}

/// An entry in the per-event FIFO waitlist queue.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitlistEntry {
    pub event_id: u64,
    pub applicant: Address,
    pub joined_at: u64,
}

// ── Storage Keys ───────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Event(u64),
    Ticket(u64),
    EventCount,
    TicketCount,
    EventTickets(u64),  // Vec<u64> – active ticket IDs for an event
    CheckInRecord(u64), // CheckInRecord by ticket_id
    Waitlist(u64),      // Vec<WaitlistEntry> – FIFO waitlist per event
}

// ── Events ─────────────────────────────────────────────────────────────────────

#[contractevent]
pub struct EventCreatedEvent {
    pub event_id: u64,
    pub organizer: Address,
    pub name: String,
    pub timestamp: u64,
}

pub fn publish_event_created_event(
    env: &Env,
    event_id: u64,
    organizer: Address,
    name: String,
    timestamp: u64,
) {
    EventCreatedEvent {
        event_id,
        organizer,
        name,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct TicketIssuedEvent {
    pub ticket_id: u64,
    pub event_id: u64,
    pub holder: Address,
    pub qr_hash: BytesN<32>,
    pub timestamp: u64,
}

pub fn publish_ticket_issued_event(
    env: &Env,
    ticket_id: u64,
    event_id: u64,
    holder: Address,
    qr_hash: BytesN<32>,
    timestamp: u64,
) {
    TicketIssuedEvent {
        ticket_id,
        event_id,
        holder,
        qr_hash,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct TicketCheckedInEvent {
    pub ticket_id: u64,
    pub event_id: u64,
    pub holder: Address,
    pub checked_in_by: Address,
    pub check_in_time: u64,
}

pub fn publish_ticket_checked_in_event(
    env: &Env,
    ticket_id: u64,
    event_id: u64,
    holder: Address,
    checked_in_by: Address,
    check_in_time: u64,
) {
    TicketCheckedInEvent {
        ticket_id,
        event_id,
        holder,
        checked_in_by,
        check_in_time,
    }
    .publish(env);
}

#[contractevent]
pub struct TicketTransferedEvent {
    pub ticket_id: u64,
    pub event_id: u64,
    pub old_holder: Address,
    pub new_holder: Address,
    pub timestamp: u64,
}

pub fn publish_ticket_transferred_event(
    env: &Env,
    ticket_id: u64,
    event_id: u64,
    old_holder: Address,
    new_holder: Address,
    timestamp: u64,
) {
    TicketTransferedEvent {
        ticket_id,
        event_id,
        old_holder,
        new_holder,
        timestamp,
    }
    .publish(env);
}

/// Emitted when a ticket is refunded / cancelled by its holder.
#[contractevent]
pub struct TicketRefundedEvent {
    pub ticket_id: u64,
    pub event_id: u64,
    pub old_holder: Address,
    pub timestamp: u64,
}

pub fn publish_ticket_refunded_event(
    env: &Env,
    ticket_id: u64,
    event_id: u64,
    old_holder: Address,
    timestamp: u64,
) {
    TicketRefundedEvent {
        ticket_id,
        event_id,
        old_holder,
        timestamp,
    }
    .publish(env);
}

/// Emitted when a user joins the waitlist.
#[contractevent]
pub struct WaitlistJoinedEvent {
    pub event_id: u64,
    pub applicant: Address,
    pub position: u32,
    pub timestamp: u64,
}

pub fn publish_waitlist_joined_event(
    env: &Env,
    event_id: u64,
    applicant: Address,
    position: u32,
    timestamp: u64,
) {
    WaitlistJoinedEvent {
        event_id,
        applicant,
        position,
        timestamp,
    }
    .publish(env);
}

/// Emitted when a waitlisted user is automatically assigned a ticket after a refund.
#[contractevent]
pub struct WaitlistAssignedEvent {
    pub event_id: u64,
    pub applicant: Address,
    pub ticket_id: u64,
    pub timestamp: u64,
}

pub fn publish_waitlist_assigned_event(
    env: &Env,
    event_id: u64,
    applicant: Address,
    ticket_id: u64,
    timestamp: u64,
) {
    WaitlistAssignedEvent {
        event_id,
        applicant,
        ticket_id,
        timestamp,
    }
    .publish(env);
}

// ── Helper Functions ───────────────────────────────────────────────────────────

fn get_event_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::EventCount)
        .unwrap_or(0)
}

fn get_ticket_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::TicketCount)
        .unwrap_or(0)
}

fn increment_event_count(env: &Env, count: u64) {
    env.storage().persistent().set(&DataKey::EventCount, &count);
}

fn increment_ticket_count(env: &Env, count: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::TicketCount, &count);
}

fn add_ticket_to_event(env: &Env, event_id: u64, ticket_id: u64) {
    let key = DataKey::EventTickets(event_id);
    let mut tickets: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    tickets.push_back(ticket_id);
    env.storage().persistent().set(&key, &tickets);
}

/// Remove a ticket_id from the event's active ticket list (used on refund).
fn remove_ticket_from_event(env: &Env, event_id: u64, ticket_id: u64) {
    let key = DataKey::EventTickets(event_id);
    let tickets: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    let mut updated = Vec::new(env);
    for tid in tickets.iter() {
        if tid != ticket_id {
            updated.push_back(tid);
        }
    }
    env.storage().persistent().set(&key, &updated);
}

fn get_waitlist(env: &Env, event_id: u64) -> Vec<WaitlistEntry> {
    env.storage()
        .persistent()
        .get(&DataKey::Waitlist(event_id))
        .unwrap_or_else(|| Vec::new(env))
}

fn save_waitlist(env: &Env, event_id: u64, waitlist: &Vec<WaitlistEntry>) {
    env.storage()
        .persistent()
        .set(&DataKey::Waitlist(event_id), waitlist);
}

/// Returns the number of active (non-refunded) tickets for an event.
fn active_ticket_count(env: &Env, event_id: u64) -> u64 {
    let ticket_ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::EventTickets(event_id))
        .unwrap_or_else(|| Vec::new(env));
    ticket_ids.len() as u64
}

/// Internal: mint a new ticket without the QR-uniqueness check used at the
/// outer `issue_ticket` call. Used for waitlist auto-assignment where the
/// organizer is no longer the signer.
fn mint_ticket(env: &Env, event_id: u64, holder: Address, qr_hash: BytesN<32>) -> u64 {
    let ticket_count = get_ticket_count(env);
    let new_ticket_id = ticket_count + 1;

    let ticket = Ticket {
        ticket_id: new_ticket_id,
        event_id,
        holder: holder.clone(),
        qr_hash: qr_hash.clone(),
        checked_in: false,
        check_in_time: None,
        refunded: false,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Ticket(new_ticket_id), &ticket);

    add_ticket_to_event(env, event_id, new_ticket_id);
    increment_ticket_count(env, new_ticket_id);

    publish_ticket_issued_event(
        env,
        new_ticket_id,
        event_id,
        holder,
        qr_hash,
        env.ledger().timestamp(),
    );

    new_ticket_id
}

// ── Contract ───────────────────────────────────────────────────────────────────

#[contract]
pub struct TicketingContract;

#[contractimpl]
impl TicketingContract {
    /// Creates a new event. Only the organizer can later issue tickets for this event.
    pub fn create_event(
        env: Env,
        organizer: Address,
        name: String,
        description: String,
        start_time: u64,
        end_time: u64,
        max_capacity: Option<u64>,
    ) -> u64 {
        organizer.require_auth();

        if end_time < start_time {
            panic_with_error!(env, TicketingError::InvalidTimeRange);
        }

        let event_count = get_event_count(&env);
        let new_event_id = event_count + 1;

        let event = Event {
            event_id: new_event_id,
            organizer: organizer.clone(),
            name,
            description,
            start_time,
            end_time,
            max_capacity,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Event(new_event_id), &event);

        increment_event_count(&env, new_event_id);

        publish_event_created_event(
            &env,
            new_event_id,
            organizer,
            event.name,
            env.ledger().timestamp(),
        );

        new_event_id
    }

    /// Get event details by ID.
    pub fn get_event(env: Env, event_id: u64) -> Event {
        env.storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .unwrap_or_else(|| panic_with_error!(env, TicketingError::EventNotFound))
    }

    /// Issue a ticket for an event.
    /// The qr_hash must be a secure unique hash (SHA256) generated off-chain.
    pub fn issue_ticket(
        env: Env,
        organizer: Address,
        event_id: u64,
        holder: Address,
        qr_hash: BytesN<32>,
    ) -> u64 {
        organizer.require_auth();

        // Verify event exists and organizer owns it
        let event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .unwrap_or_else(|| panic_with_error!(env, TicketingError::EventNotFound));

        if event.organizer != organizer {
            panic_with_error!(env, TicketingError::NotAuthorized);
        }

        // Check capacity if set
        if let Some(max_cap) = event.max_capacity {
            let active = active_ticket_count(&env, event_id);
            if active >= max_cap {
                panic_with_error!(env, TicketingError::EventAtCapacity);
            }
        }

        // Ensure QR hash uniqueness (no duplicate hashes across all tickets)
        let ticket_count = get_ticket_count(&env);
        for i in 1..=ticket_count {
            if let Some(ticket) = env
                .storage()
                .persistent()
                .get::<_, Ticket>(&DataKey::Ticket(i))
            {
                if ticket.qr_hash == qr_hash {
                    panic_with_error!(env, TicketingError::DuplicateQRHash);
                }
            }
        }

        mint_ticket(&env, event_id, holder, qr_hash)
    }

    /// Get ticket details by ID.
    pub fn get_ticket(env: Env, ticket_id: u64) -> Ticket {
        env.storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic_with_error!(env, TicketingError::TicketNotFound))
    }

    /// Get all tickets for an event.
    pub fn get_event_tickets(env: Env, event_id: u64) -> Vec<Ticket> {
        let ticket_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::EventTickets(event_id))
            .unwrap_or_else(|| Vec::new(&env));

        let mut tickets = Vec::new(&env);
        for ticket_id in ticket_ids.iter() {
            let ticket: Ticket = env
                .storage()
                .persistent()
                .get(&DataKey::Ticket(ticket_id))
                .unwrap();
            tickets.push_back(ticket);
        }
        tickets
    }

    /// Verify a ticket by comparing the provided QR hash with stored hash.
    /// Returns ticket verification status without marking as checked in.
    pub fn verify_ticket(env: Env, ticket_id: u64, qr_hash: BytesN<32>) -> TicketVerification {
        let ticket: Ticket = env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic_with_error!(env, TicketingError::TicketNotFound));

        let event_id = ticket.event_id;

        let valid = ticket.qr_hash == qr_hash;
        let already_checked_in = ticket.checked_in;

        TicketVerification {
            ticket_id,
            event_id,
            holder: ticket.holder,
            valid,
            already_checked_in,
        }
    }

    /// Check in a ticket. This is a state transition: pending → checked_in.
    /// Cannot be undone. Duplicate check-ins are rejected.
    pub fn check_in(env: Env, operator: Address, ticket_id: u64) {
        operator.require_auth();

        // Get the ticket
        let mut ticket: Ticket = env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic_with_error!(env, TicketingError::TicketNotFound));

        // Prevent double check-in (idempotency)
        if ticket.checked_in {
            panic_with_error!(env, TicketingError::AlreadyCheckedIn);
        }

        let check_in_time = env.ledger().timestamp();

        ticket.checked_in = true;
        ticket.check_in_time = Some(check_in_time);

        // Save updated ticket
        env.storage()
            .persistent()
            .set(&DataKey::Ticket(ticket_id), &ticket);

        // Record check-in metadata (who scanned, when)
        let check_in_record = CheckInRecord {
            ticket_id,
            checked_in_by: operator.clone(),
            check_in_time,
        };
        env.storage()
            .persistent()
            .set(&DataKey::CheckInRecord(ticket_id), &check_in_record);

        // Emit event
        publish_ticket_checked_in_event(
            &env,
            ticket_id,
            ticket.event_id,
            ticket.holder,
            operator,
            check_in_time,
        );
    }

    /// Transfer a ticket from current holder to a new holder.
    /// Cannot transfer a checked-in ticket.
    pub fn transfer_ticket(env: Env, current_holder: Address, ticket_id: u64, new_holder: Address) {
        current_holder.require_auth();

        let mut ticket: Ticket = env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic_with_error!(env, TicketingError::TicketNotFound));

        if ticket.holder != current_holder {
            panic_with_error!(env, TicketingError::NotAuthorized);
        }

        if ticket.checked_in {
            panic_with_error!(env, TicketingError::TicketAlreadyCheckedIn);
        }

        let old_holder = ticket.holder.clone();
        ticket.holder = new_holder.clone();

        env.storage()
            .persistent()
            .set(&DataKey::Ticket(ticket_id), &ticket);

        publish_ticket_transferred_event(
            &env,
            ticket_id,
            ticket.event_id,
            old_holder,
            new_holder,
            env.ledger().timestamp(),
        );
    }

    /// Refund / cancel a ticket.
    ///
    /// - The ticket is marked `refunded = true` and removed from the event's
    ///   active ticket list, freeing one capacity slot.
    /// - If anyone is on the waitlist the **first** entry is automatically
    ///   issued a new ticket (auto-assignment) and removed from the queue.
    ///
    /// Panics if: ticket not found, caller is not the holder, ticket is
    /// already checked-in, or ticket is already refunded.
    pub fn refund_ticket(env: Env, holder: Address, ticket_id: u64) {
        holder.require_auth();

        let mut ticket: Ticket = env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .unwrap_or_else(|| panic_with_error!(env, TicketingError::TicketNotFound));

        if ticket.holder != holder {
            panic_with_error!(env, TicketingError::NotAuthorized);
        }

        if ticket.checked_in {
            panic_with_error!(env, TicketingError::AlreadyCheckedIn);
        }

        if ticket.refunded {
            panic_with_error!(env, TicketingError::TicketAlreadyRefunded);
        }

        let event_id = ticket.event_id;
        let timestamp = env.ledger().timestamp();

        // Mark as refunded
        ticket.refunded = true;
        env.storage()
            .persistent()
            .set(&DataKey::Ticket(ticket_id), &ticket);

        // Remove from the event's active list (frees one capacity slot)
        remove_ticket_from_event(&env, event_id, ticket_id);

        publish_ticket_refunded_event(&env, ticket_id, event_id, holder, timestamp);

        // ── Auto-assign waitlist ──────────────────────────────────────────────
        let mut waitlist = get_waitlist(&env, event_id);
        if !waitlist.is_empty() {
            // Pop the front of the FIFO queue
            let first = waitlist.get(0).unwrap();
            let assignee = first.applicant.clone();

            // Build new waitlist without the first entry
            let mut new_waitlist: Vec<WaitlistEntry> = Vec::new(&env);
            for idx in 1..waitlist.len() {
                new_waitlist.push_back(waitlist.get(idx).unwrap());
            }
            save_waitlist(&env, event_id, &new_waitlist);

            // Generate a deterministic placeholder QR hash for the new ticket.
            // The assignee should replace this off-chain before the event.
            let mut placeholder = [0u8; 32];
            let id_bytes = ticket_id.to_be_bytes();
            let ts_bytes = timestamp.to_be_bytes();
            for i in 0..8 {
                placeholder[i] = id_bytes[i];
                placeholder[i + 8] = ts_bytes[i];
            }
            let qr_hash = BytesN::from_slice(&env, &placeholder);

            let new_ticket_id = mint_ticket(&env, event_id, assignee.clone(), qr_hash);

            publish_waitlist_assigned_event(&env, event_id, assignee, new_ticket_id, timestamp);
        }
    }

    /// Join the waitlist for a sold-out event.
    ///
    /// Returns the caller's 1-based position in the queue.
    ///
    /// Panics if: event not found, event has no capacity limit set,
    /// event is not yet at capacity, or caller is already on the list.
    pub fn join_waitlist(env: Env, event_id: u64, applicant: Address) -> u32 {
        applicant.require_auth();

        let event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .unwrap_or_else(|| panic_with_error!(env, TicketingError::EventNotFound));

        // Waitlists only make sense for capacity-limited events
        let max_cap = match event.max_capacity {
            Some(c) => c,
            None => panic_with_error!(env, TicketingError::NotAtCapacity),
        };

        // Must be actually full before joining the waitlist
        let active = active_ticket_count(&env, event_id);
        if active < max_cap {
            panic_with_error!(env, TicketingError::NotAtCapacity);
        }

        // Duplicate check
        let mut waitlist = get_waitlist(&env, event_id);
        for entry in waitlist.iter() {
            if entry.applicant == applicant {
                panic_with_error!(env, TicketingError::AlreadyOnWaitlist);
            }
        }

        let timestamp = env.ledger().timestamp();
        waitlist.push_back(WaitlistEntry {
            event_id,
            applicant: applicant.clone(),
            joined_at: timestamp,
        });

        let position = waitlist.len(); // 1-based position
        save_waitlist(&env, event_id, &waitlist);

        publish_waitlist_joined_event(&env, event_id, applicant, position, timestamp);

        position
    }

    /// Return the current waitlist queue for an event (FIFO order).
    pub fn get_waitlist(env: Env, event_id: u64) -> Vec<WaitlistEntry> {
        get_waitlist(&env, event_id)
    }

    /// Get the check-in record for a ticket.
    pub fn get_check_in_record(env: Env, ticket_id: u64) -> Option<CheckInRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::CheckInRecord(ticket_id))
    }

    /// Get total number of tickets issued for an event.
    pub fn get_event_ticket_count(env: Env, event_id: u64) -> u64 {
        active_ticket_count(&env, event_id)
    }

    /// Get total number of checked-in tickets for an event.
    pub fn get_event_checked_in_count(env: Env, event_id: u64) -> u64 {
        let ticket_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::EventTickets(event_id))
            .unwrap_or_else(|| Vec::new(&env));
        let mut count = 0;
        for ticket_id in ticket_ids.iter() {
            let ticket: Ticket = env
                .storage()
                .persistent()
                .get(&DataKey::Ticket(ticket_id))
                .unwrap();
            if ticket.checked_in {
                count += 1;
            }
        }
        count
    }
}
