#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, Symbol,
    Vec,
};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractStatus {
    Created = 0,
    Funded = 1,
    Completed = 2,
    Disputed = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub amount: i128,
    pub released: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowContract {
    pub client: Address,
    pub freelancer: Address,
    pub milestones: Vec<Milestone>,
    pub total_amount: i128,
    pub funded_amount: i128,
    pub released_amount: i128,
    pub status: ContractStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientMigrationRequest {
    pub current_client: Address,
    pub proposed_client: Address,
    pub proposed_client_confirmed: bool,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    NextContractId,
    Contract(u32),
    PendingClientMigration(u32),
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum EscrowError {
    ContractNotFound = 1,
    InvalidMilestones = 2,
    InvalidAmount = 3,
    UnauthorizedRoleOverlap = 4,
    Overflow = 5,
    Overfunding = 6,
    InvalidMilestone = 7,
    MilestoneAlreadyReleased = 8,
    ContractNotFunded = 9,
    PendingMigrationExists = 10,
    PendingMigrationNotFound = 11,
    InvalidMigrationTarget = 12,
    MigrationNotConfirmed = 13,
    MigrationUnavailable = 14,
}

#[contract]
pub struct Escrow;

#[contractimpl]
impl Escrow {
    /// Create a new escrow contract with immutable freelancer identity, mutable
    /// client identity, and milestone-based payment obligations.
    ///
    /// Requirements:
    /// - `client` must authorize contract creation.
    /// - `client` and `freelancer` must be distinct addresses.
    /// - `milestone_amounts` must be non-empty and contain only positive values.
    ///
    /// Security:
    /// - The stored client address is the only authority allowed to fund,
    ///   release milestones, or manage identity migration.
    pub fn create_contract(
        env: Env,
        client: Address,
        freelancer: Address,
        milestone_amounts: Vec<i128>,
    ) -> u32 {
        client.require_auth();
        validate_distinct_roles(&env, &client, &freelancer);

        let (milestones, total_amount) = build_milestones(&env, &milestone_amounts);
        let contract_id = allocate_contract_id(&env);
        let contract = EscrowContract {
            client,
            freelancer,
            milestones,
            total_amount,
            funded_amount: 0,
            released_amount: 0,
            status: ContractStatus::Created,
        };

        save_contract(&env, contract_id, &contract);
        contract_id
    }

    /// Deposit funds into escrow.
    ///
    /// Requirements:
    /// - Only the current client may authorize the deposit.
    /// - Deposits must be positive and may not exceed the contract total.
    ///
    /// Effects:
    /// - Contract status becomes `Funded` once the full milestone total is held.
    pub fn deposit_funds(env: Env, contract_id: u32, amount: i128) -> bool {
        let mut contract = load_contract(&env, contract_id);
        contract.client.require_auth();
        require_positive_amount(&env, amount);

        let next_funded = checked_add(&env, contract.funded_amount, amount);
        if next_funded > contract.total_amount {
            panic_with_error!(&env, EscrowError::Overfunding);
        }

        contract.funded_amount = next_funded;
        if contract.funded_amount == contract.total_amount {
            contract.status = ContractStatus::Funded;
        }

        save_contract(&env, contract_id, &contract);
        true
    }

    /// Release a milestone payment after successful delivery verification.
    ///
    /// Requirements:
    /// - Only the current client may authorize release.
    /// - The contract must be fully funded before any release occurs.
    /// - The selected milestone must exist and be unreleased.
    pub fn release_milestone(env: Env, contract_id: u32, milestone_id: u32) -> bool {
        let mut contract = load_contract(&env, contract_id);
        contract.client.require_auth();

        if contract.funded_amount != contract.total_amount {
            panic_with_error!(&env, EscrowError::ContractNotFunded);
        }

        let mut milestone = match contract.milestones.get(milestone_id) {
            Some(milestone) => milestone,
            None => panic_with_error!(&env, EscrowError::InvalidMilestone),
        };

        if milestone.released {
            panic_with_error!(&env, EscrowError::MilestoneAlreadyReleased);
        }

        milestone.released = true;
        contract.milestones.set(milestone_id, milestone.clone());
        contract.released_amount = checked_add(&env, contract.released_amount, milestone.amount);
        contract.status = if all_milestones_released(&contract.milestones) {
            ContractStatus::Completed
        } else {
            ContractStatus::Funded
        };

        save_contract(&env, contract_id, &contract);
        true
    }

    /// Request migration of the client identity to a new address.
    ///
    /// Flow:
    /// 1. Current client requests migration to `proposed_client`.
    /// 2. Proposed client explicitly confirms the handover.
    /// 3. Current client explicitly finalizes the migration.
    ///
    /// Security:
    /// - Active migrations cannot be overwritten; they must be finalized or
    ///   cancelled first, preventing stale approvals from being reused.
    pub fn request_client_migration(env: Env, contract_id: u32, proposed_client: Address) -> bool {
        let contract = load_contract(&env, contract_id);
        contract.client.require_auth();
        ensure_migration_allowed(&env, &contract);

        if has_pending_migration(&env, contract_id) {
            panic_with_error!(&env, EscrowError::PendingMigrationExists);
        }

        if proposed_client == contract.client || proposed_client == contract.freelancer {
            panic_with_error!(&env, EscrowError::InvalidMigrationTarget);
        }

        let migration = ClientMigrationRequest {
            current_client: contract.client,
            proposed_client,
            proposed_client_confirmed: false,
        };

        save_pending_migration(&env, contract_id, &migration);
        true
    }

    /// Confirm willingness to assume the client role for a pending migration.
    ///
    /// Requirements:
    /// - Only the proposed client may confirm the request.
    /// - Confirmation does not transfer authority by itself; finalization by the
    ///   current client is still required.
    pub fn confirm_client_migration(env: Env, contract_id: u32) -> bool {
        let mut migration = load_pending_migration(&env, contract_id);
        migration.proposed_client.require_auth();
        migration.proposed_client_confirmed = true;
        save_pending_migration(&env, contract_id, &migration);
        true
    }

    /// Finalize a previously confirmed client migration.
    ///
    /// Requirements:
    /// - Only the current client may finalize.
    /// - The proposed client must have confirmed first.
    ///
    /// Effects:
    /// - The stored client authority is replaced.
    /// - The pending migration record is deleted.
    pub fn finalize_client_migration(env: Env, contract_id: u32) -> bool {
        let mut contract = load_contract(&env, contract_id);
        let migration = load_pending_migration(&env, contract_id);
        contract.client.require_auth();
        ensure_migration_allowed(&env, &contract);

        if migration.current_client != contract.client || !migration.proposed_client_confirmed {
            panic_with_error!(&env, EscrowError::MigrationNotConfirmed);
        }

        contract.client = migration.proposed_client;
        save_contract(&env, contract_id, &contract);
        clear_pending_migration(&env, contract_id);
        true
    }

    /// Cancel an in-flight client migration request before finalization.
    ///
    /// Requirements:
    /// - Only the current client may cancel.
    pub fn cancel_client_migration(env: Env, contract_id: u32) -> bool {
        let contract = load_contract(&env, contract_id);
        load_pending_migration(&env, contract_id);
        contract.client.require_auth();
        clear_pending_migration(&env, contract_id);
        true
    }

    /// Fetch the current escrow contract state for a contract id.
    pub fn get_contract(env: Env, contract_id: u32) -> EscrowContract {
        load_contract(&env, contract_id)
    }

    /// Returns `true` when a client migration request is awaiting cancellation
    /// or finalization.
    pub fn has_pending_client_migration(env: Env, contract_id: u32) -> bool {
        has_pending_migration(&env, contract_id)
    }

    /// Fetch the active pending migration request.
    pub fn get_pending_client_migration(env: Env, contract_id: u32) -> ClientMigrationRequest {
        load_pending_migration(&env, contract_id)
    }

    /// Issue a reputation credential for the freelancer after contract
    /// completion. This remains a placeholder for downstream integration.
    pub fn issue_reputation(_env: Env, _freelancer: Address, rating: i128) -> bool {
        rating > 0
    }

    /// Hello-world style function for testing and CI.
    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }
}

fn allocate_contract_id(env: &Env) -> u32 {
    let key = DataKey::NextContractId;
    let next_id = env.storage().persistent().get::<_, u32>(&key).unwrap_or(1);
    let following_id = next_id.checked_add(1).unwrap_or_else(|| {
        panic_with_error!(env, EscrowError::Overflow);
    });

    env.storage().persistent().set(&key, &following_id);
    next_id
}

fn build_milestones(env: &Env, milestone_amounts: &Vec<i128>) -> (Vec<Milestone>, i128) {
    if milestone_amounts.is_empty() {
        panic_with_error!(env, EscrowError::InvalidMilestones);
    }

    let mut milestones = Vec::new(env);
    let mut total_amount = 0_i128;

    for amount in milestone_amounts.iter() {
        require_positive_amount(env, amount);
        total_amount = checked_add(env, total_amount, amount);
        milestones.push_back(Milestone {
            amount,
            released: false,
        });
    }

    (milestones, total_amount)
}

fn require_positive_amount(env: &Env, amount: i128) {
    if amount <= 0 {
        panic_with_error!(env, EscrowError::InvalidAmount);
    }
}

fn checked_add(env: &Env, lhs: i128, rhs: i128) -> i128 {
    lhs.checked_add(rhs)
        .unwrap_or_else(|| panic_with_error!(env, EscrowError::Overflow))
}

fn validate_distinct_roles(env: &Env, client: &Address, freelancer: &Address) {
    if client == freelancer {
        panic_with_error!(env, EscrowError::UnauthorizedRoleOverlap);
    }
}

fn ensure_migration_allowed(env: &Env, contract: &EscrowContract) {
    if contract.status == ContractStatus::Completed || contract.status == ContractStatus::Disputed {
        panic_with_error!(env, EscrowError::MigrationUnavailable);
    }
}

fn all_milestones_released(milestones: &Vec<Milestone>) -> bool {
    for milestone in milestones.iter() {
        if !milestone.released {
            return false;
        }
    }

    true
}

fn load_contract(env: &Env, contract_id: u32) -> EscrowContract {
    let key = DataKey::Contract(contract_id);
    match env.storage().persistent().get(&key) {
        Some(contract) => contract,
        None => panic_with_error!(env, EscrowError::ContractNotFound),
    }
}

fn save_contract(env: &Env, contract_id: u32, contract: &EscrowContract) {
    env.storage()
        .persistent()
        .set(&DataKey::Contract(contract_id), contract);
}

fn has_pending_migration(env: &Env, contract_id: u32) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::PendingClientMigration(contract_id))
}

fn load_pending_migration(env: &Env, contract_id: u32) -> ClientMigrationRequest {
    let key = DataKey::PendingClientMigration(contract_id);
    match env.storage().persistent().get(&key) {
        Some(migration) => migration,
        None => panic_with_error!(env, EscrowError::PendingMigrationNotFound),
    }
}

fn save_pending_migration(env: &Env, contract_id: u32, migration: &ClientMigrationRequest) {
    env.storage()
        .persistent()
        .set(&DataKey::PendingClientMigration(contract_id), migration);
}

fn clear_pending_migration(env: &Env, contract_id: u32) {
    env.storage()
        .persistent()
        .remove(&DataKey::PendingClientMigration(contract_id));
}

#[cfg(test)]
mod test;
