#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

/// Persistent storage keys used by the Escrow contract.
///
/// Each variant corresponds to a distinct piece of contract state:
/// - [`DataKey::Contract`] stores the full [`EscrowContract`] keyed by its numeric ID.
/// - [`DataKey::ReputationIssued`] is a boolean flag that prevents double-issuance of
///   reputation credentials for a given contract.
/// - [`DataKey::NextId`] is a monotonically increasing counter for assigning contract IDs.
#[contracttype]
pub enum DataKey {
    /// Full escrow contract state, keyed by the numeric contract ID.
    Contract(u32),
    /// Whether a reputation credential has already been issued for the given contract ID.
    /// Immutably set to `true` on first issuance; prevents replay and double-issuance.
    ReputationIssued(u32),
    /// Auto-incrementing counter; incremented on every [`Escrow::create_contract`] call.
    NextId,
}

/// The lifecycle status of an escrow contract.
///
/// Valid transitions:
/// ```text
/// Created -> Funded -> Completed
/// Funded  -> Disputed
/// ```
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractStatus {
    /// Contract created, awaiting client deposit.
    Created = 0,
    /// Funds deposited by client; work is in progress.
    Funded = 1,
    /// All milestones released and contract finalised by the client.
    Completed = 2,
    /// A dispute has been raised; milestone payments are paused.
    Disputed = 3,
}

/// A single deliverable and its associated payment within an escrow contract.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Milestone {
    /// Payment amount in stroops (1 XLM = 10_000_000 stroops).
    pub amount: i128,
    /// Whether the client has released this milestone's funds to the freelancer.
    pub released: bool,
}

/// Complete persisted state for one escrow engagement between a client and a freelancer.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowContract {
    /// The party commissioning work and funding the escrow.
    pub client: Address,
    /// The party performing work and receiving milestone payments.
    pub freelancer: Address,
    /// Ordered list of payment milestones for this engagement.
    pub milestones: Vec<Milestone>,
    /// Current phase of the contract lifecycle.
    pub status: ContractStatus,
}

/// TalentTrust Escrow contract.
///
/// Manages milestone-based escrow payments between a client and a freelancer on Stellar.
/// Reputation credentials may only be issued after the contract has been marked
/// `Completed` **and** all milestone funds have been released to the freelancer
/// (final settlement). See [`Escrow::issue_reputation`] for the full constraint set.
#[contract]
pub struct Escrow;

#[contractimpl]
impl Escrow {
    /// Create a new escrow contract.
    ///
    /// Stores the contract in persistent storage with status [`ContractStatus::Created`]
    /// and returns a unique numeric ID (starts at 1, increments by 1 per call).
    ///
    /// # Arguments
    /// * `client` - Address of the party commissioning work (funds the escrow).
    /// * `freelancer` - Address of the party performing work (receives milestones).
    /// * `milestone_amounts` - Ordered payment amounts (in stroops) for each milestone.
    ///
    /// # Panics
    /// Panics if `milestone_amounts` is empty.
    pub fn create_contract(
        env: Env,
        client: Address,
        freelancer: Address,
        milestone_amounts: Vec<i128>,
    ) -> u32 {
        assert!(
            !milestone_amounts.is_empty(),
            "at least one milestone required"
        );

        let mut milestones: Vec<Milestone> = Vec::new(&env);
        for i in 0..milestone_amounts.len() {
            let amount = milestone_amounts.get(i).unwrap();
            milestones.push_back(Milestone {
                amount,
                released: false,
            });
        }

        let next_id: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::NextId)
            .unwrap_or(0u32)
            + 1;

        let escrow = EscrowContract {
            client,
            freelancer,
            milestones,
            status: ContractStatus::Created,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Contract(next_id), &escrow);
        env.storage().persistent().set(&DataKey::NextId, &next_id);

        next_id
    }

    /// Deposit funds into escrow and transition status to [`ContractStatus::Funded`].
    ///
    /// Only the client may call this function. The contract must be in
    /// [`ContractStatus::Created`] state; calling again on an already-funded contract panics.
    ///
    /// # Arguments
    /// * `contract_id` - Numeric ID returned by [`Escrow::create_contract`].
    /// * `amount` - Deposit amount in stroops; must be positive.
    ///
    /// # Panics
    /// * If the contract does not exist.
    /// * If the contract is not in [`ContractStatus::Created`] state.
    /// * If `amount` is not positive.
    /// * If the caller has not authorised this call.
    pub fn deposit_funds(env: Env, contract_id: u32, amount: i128) -> bool {
        let mut escrow: EscrowContract = env
            .storage()
            .persistent()
            .get(&DataKey::Contract(contract_id))
            .expect("contract not found");

        assert!(
            escrow.status == ContractStatus::Created,
            "contract not in Created status"
        );
        assert!(amount > 0, "deposit amount must be positive");

        escrow.client.require_auth();
        escrow.status = ContractStatus::Funded;

        env.storage()
            .persistent()
            .set(&DataKey::Contract(contract_id), &escrow);
        true
    }

    /// Release a single milestone payment to the freelancer.
    ///
    /// Only the client may call this. The contract must be [`ContractStatus::Funded`].
    /// Each milestone can only be released once; releasing the same milestone twice panics.
    ///
    /// # Arguments
    /// * `contract_id` - Numeric ID of the escrow contract.
    /// * `milestone_id` - Zero-based index of the milestone to release.
    ///
    /// # Panics
    /// * If the contract does not exist.
    /// * If the contract is not in [`ContractStatus::Funded`] state.
    /// * If `milestone_id` is out of range.
    /// * If the milestone has already been released.
    /// * If the caller has not authorised this call.
    pub fn release_milestone(env: Env, contract_id: u32, milestone_id: u32) -> bool {
        let mut escrow: EscrowContract = env
            .storage()
            .persistent()
            .get(&DataKey::Contract(contract_id))
            .expect("contract not found");

        assert!(
            escrow.status == ContractStatus::Funded,
            "contract not in Funded status"
        );
        assert!(
            milestone_id < escrow.milestones.len(),
            "milestone_id out of range"
        );

        escrow.client.require_auth();

        let mut milestone = escrow.milestones.get(milestone_id).unwrap();
        assert!(!milestone.released, "milestone already released");

        milestone.released = true;
        escrow.milestones.set(milestone_id, milestone);

        env.storage()
            .persistent()
            .set(&DataKey::Contract(contract_id), &escrow);
        true
    }

    /// Mark the contract as [`ContractStatus::Completed`], enabling reputation issuance.
    ///
    /// Only the client may call this. All milestones **must** be released before the
    /// contract can transition to `Completed`; this enforces the final-settlement gate
    /// required by [`Escrow::issue_reputation`].
    ///
    /// # Arguments
    /// * `contract_id` - Numeric ID of the escrow contract.
    ///
    /// # Panics
    /// * If the contract does not exist.
    /// * If the contract is not in [`ContractStatus::Funded`] state.
    /// * If any milestone has not yet been released.
    /// * If the caller has not authorised this call.
    pub fn complete_contract(env: Env, contract_id: u32) -> bool {
        let mut escrow: EscrowContract = env
            .storage()
            .persistent()
            .get(&DataKey::Contract(contract_id))
            .expect("contract not found");

        assert!(
            escrow.status == ContractStatus::Funded,
            "contract not in Funded status"
        );

        escrow.client.require_auth();

        // Enforce final-settlement gate: every milestone must be released.
        for i in 0..escrow.milestones.len() {
            let m = escrow.milestones.get(i).unwrap();
            assert!(
                m.released,
                "all milestones must be released before completing"
            );
        }

        escrow.status = ContractStatus::Completed;
        env.storage()
            .persistent()
            .set(&DataKey::Contract(contract_id), &escrow);
        true
    }

    /// Issue a reputation credential for the freelancer of a completed escrow contract.
    ///
    /// # Reputation Issuance Constraints
    ///
    /// This function enforces the following ordered constraints:
    ///
    /// 1. **Contract existence** - The contract identified by `contract_id` must exist in
    ///    persistent storage.
    /// 2. **Completion gate** - `status` must be [`ContractStatus::Completed`]. Reputation
    ///    cannot be issued for contracts that are still `Created`, `Funded`, or `Disputed`.
    /// 3. **Final settlement** - Every milestone must have `released == true`. This ensures
    ///    no outstanding payment obligations remain before a credential is recorded.
    /// 4. **Single issuance** - A credential can be issued at most once per contract.
    ///    The [`DataKey::ReputationIssued`] flag is set atomically before the event is
    ///    emitted, preventing replay attacks and double-issuance.
    /// 5. **Valid rating** - `rating` must be in the inclusive range `[1, 5]`.
    ///
    /// On success, a `reputation_issued` event is published for off-chain indexers:
    /// ```text
    /// topic:  (Symbol("reputation_issued"),)
    /// data:   (contract_id: u32, freelancer: Address, rating: u32)
    /// ```
    ///
    /// # Arguments
    /// * `contract_id` - Numeric ID of the completed escrow contract.
    /// * `rating` - Reputation score in `[1, 5]`.
    ///
    /// # Panics
    /// Panics with a descriptive message for each violated constraint (see above).
    pub fn issue_reputation(env: Env, contract_id: u32, rating: u32) -> bool {
        // Constraint 1: contract must exist.
        let escrow: EscrowContract = env
            .storage()
            .persistent()
            .get(&DataKey::Contract(contract_id))
            .expect("contract not found");

        // Constraint 2: must be Completed.
        assert!(
            escrow.status == ContractStatus::Completed,
            "reputation can only be issued after contract completion"
        );

        // Constraint 3: all milestones released (final settlement).
        for i in 0..escrow.milestones.len() {
            let m = escrow.milestones.get(i).unwrap();
            assert!(
                m.released,
                "reputation can only be issued after final settlement of all milestones"
            );
        }

        // Constraint 4: no double issuance.
        let already_issued: bool = env
            .storage()
            .persistent()
            .get(&DataKey::ReputationIssued(contract_id))
            .unwrap_or(false);
        assert!(
            !already_issued,
            "reputation already issued for this contract"
        );

        // Constraint 5: rating must be in [1, 5].
        assert!(
            rating >= 1 && rating <= 5,
            "rating must be between 1 and 5"
        );

        // Set the issued flag before emitting the event (checks-effects-interactions).
        env.storage()
            .persistent()
            .set(&DataKey::ReputationIssued(contract_id), &true);

        // Emit an observable event for off-chain indexers and auditors.
        env.events().publish(
            (Symbol::new(&env, "reputation_issued"),),
            (contract_id, escrow.freelancer.clone(), rating),
        );

        true
    }

    /// Echo function used for smoke-testing connectivity and CI health checks.
    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }
}

#[cfg(test)]
mod test;
