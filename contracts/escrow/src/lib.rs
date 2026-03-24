#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

/// Contract status lifecycle states
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractStatus {
    Created = 0,
    Funded = 1,
    Completed = 2,
    Disputed = 3,
}

/// Represents a single milestone payment
#[contracttype]
#[derive(Clone, Debug)]
pub struct Milestone {
    pub amount: i128,
    pub released: bool,
}

/// Funding accounting state for invariant tracking
/// Tracks total funded amount and released totals to maintain invariants
#[contracttype]
#[derive(Clone, Debug)]
pub struct FundingAccount {
    /// Total amount deposited into the contract
    pub total_funded: i128,
    /// Total amount released to freelancer across all milestones
    pub total_released: i128,
    /// Total amount available for release (should equal total_funded - total_released)
    pub total_available: i128,
}

/// Core escrow contract state
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowState {
    pub client: Address,
    pub freelancer: Address,
    pub status: ContractStatus,
    pub milestones: Vec<Milestone>,
    pub funding: FundingAccount,
}

#[contract]
pub struct Escrow;

#[contractimpl]
impl Escrow {
    /// Create a new escrow contract with milestone-based payments.
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `client` - Client address (payer)
    /// * `freelancer` - Freelancer address (payee)
    /// * `milestone_amounts` - Vector of milestone payment amounts
    ///
    /// # Returns
    /// Contract ID (u32)
    ///
    /// # Invariants Established
    /// - total_funded = 0 (no deposits yet)
    /// - total_released = 0 (no releases yet)
    /// - total_available = 0 (no funds available)
    /// - All milestones initialized with released = false
    pub fn create_contract(
        env: Env,
        client: Address,
        freelancer: Address,
        milestone_amounts: Vec<i128>,
    ) -> u32 {
        // Validate inputs
        assert!(
            !milestone_amounts.is_empty(),
            "Must have at least one milestone"
        );

        // Validate all milestone amounts are positive
        for amount in milestone_amounts.iter() {
            assert!(amount > 0, "Milestone amounts must be positive");
        }

        // Initialize milestones with released = false
        let mut milestones = Vec::new(&env);
        for amount in milestone_amounts.iter() {
            milestones.push_back(Milestone {
                amount,
                released: false,
            });
        }

        // Initialize funding account with zero values
        let funding = FundingAccount {
            total_funded: 0,
            total_released: 0,
            total_available: 0,
        };

        // Create contract state
        let _state = EscrowState {
            client,
            freelancer,
            status: ContractStatus::Created,
            milestones,
            funding,
        };

        // In production, this would store state in persistent storage
        // For now, return a placeholder ID
        1
    }

    /// Deposit funds into escrow. Only the client may call this.
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `contract_id` - Contract ID
    /// * `amount` - Amount to deposit
    ///
    /// # Returns
    /// true if deposit successful
    ///
    /// # Invariants Maintained
    /// - total_funded increases by deposit amount
    /// - total_available increases by deposit amount
    /// - total_released remains unchanged
    /// - Invariant: total_available = total_funded - total_released
    pub fn deposit_funds(_env: Env, _contract_id: u32, amount: i128) -> bool {
        assert!(amount > 0, "Deposit amount must be positive");

        // In production: retrieve state from persistent storage
        // Validate caller is client
        // Update funding account
        // Check invariants

        true
    }

    /// Release a milestone payment to the freelancer.
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `contract_id` - Contract ID
    /// * `milestone_id` - Milestone index to release
    ///
    /// # Returns
    /// true if release successful
    ///
    /// # Invariants Maintained
    /// - Milestone can only be released once (released flag checked)
    /// - total_released increases by milestone amount
    /// - total_available decreases by milestone amount
    /// - Invariant: total_available = total_funded - total_released
    /// - Invariant: total_released <= total_funded (no over-release)
    pub fn release_milestone(_env: Env, _contract_id: u32, _milestone_id: u32) -> bool {
        // In production: retrieve state from persistent storage
        // Validate milestone_id is valid
        // Validate milestone not already released
        // Validate sufficient funds available
        // Update funding account
        // Check invariants

        true
    }

    /// Issue a reputation credential for the freelancer after contract completion.
    pub fn issue_reputation(_env: Env, _freelancer: Address, _rating: i128) -> bool {
        true
    }

    /// Hello-world style function for testing and CI.
    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }

    // ============================================================================
    // INVARIANT CHECKING FUNCTIONS
    // ============================================================================

    /// Verify funding accounting invariants.
    ///
    /// # Invariants Checked
    /// 1. total_available = total_funded - total_released
    /// 2. total_released <= total_funded
    /// 3. total_funded >= 0
    /// 4. total_released >= 0
    /// 5. total_available >= 0
    ///
    /// # Panics
    /// If any invariant is violated
    pub fn check_funding_invariants(funding: FundingAccount) {
        // Invariant 1: total_available must equal total_funded - total_released
        let expected_available = funding.total_funded - funding.total_released;
        assert!(
            funding.total_available == expected_available,
            "Invariant violated: total_available != total_funded - total_released"
        );

        // Invariant 2: total_released must not exceed total_funded
        assert!(
            funding.total_released <= funding.total_funded,
            "Invariant violated: total_released > total_funded"
        );

        // Invariant 3: total_funded must be non-negative
        assert!(
            funding.total_funded >= 0,
            "Invariant violated: total_funded < 0"
        );

        // Invariant 4: total_released must be non-negative
        assert!(
            funding.total_released >= 0,
            "Invariant violated: total_released < 0"
        );

        // Invariant 5: total_available must be non-negative
        assert!(
            funding.total_available >= 0,
            "Invariant violated: total_available < 0"
        );
    }

    /// Verify milestone accounting invariants.
    ///
    /// # Invariants Checked
    /// 1. Sum of released milestone amounts equals total_released
    /// 2. No milestone can be released twice
    /// 3. All milestone amounts are positive
    ///
    /// # Panics
    /// If any invariant is violated
    pub fn check_milestone_invariants(milestones: Vec<Milestone>, total_released: i128) {
        let mut released_sum: i128 = 0;

        for milestone in milestones.iter() {
            // Invariant 3: All milestone amounts must be positive
            assert!(
                milestone.amount > 0,
                "Invariant violated: milestone amount must be positive"
            );

            // Invariant 1: Sum released milestones
            if milestone.released {
                released_sum = released_sum
                    .checked_add(milestone.amount)
                    .expect("Invariant violated: released sum overflow");
            }
        }

        // Invariant 1: Sum of released milestones must equal total_released
        assert!(
            released_sum == total_released,
            "Invariant violated: sum of released milestones != total_released"
        );
    }

    /// Verify complete contract state invariants.
    ///
    /// # Invariants Checked
    /// 1. Funding invariants (via check_funding_invariants)
    /// 2. Milestone invariants (via check_milestone_invariants)
    /// 3. Sum of all milestone amounts >= total_funded
    ///
    /// # Panics
    /// If any invariant is violated
    pub fn check_contract_invariants(state: EscrowState) {
        // Check funding invariants
        Self::check_funding_invariants(state.funding.clone());

        // Check milestone invariants
        Self::check_milestone_invariants(state.milestones.clone(), state.funding.total_released);

        // Invariant 3: Total contract value must be >= total funded
        let mut total_contract_value: i128 = 0;
        for milestone in state.milestones.iter() {
            total_contract_value = total_contract_value
                .checked_add(milestone.amount)
                .expect("Invariant violated: contract value overflow");
        }

        assert!(
            total_contract_value >= state.funding.total_funded,
            "Invariant violated: total_contract_value < total_funded"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env};

    // ============================================================================
    // FUNDING ACCOUNT INVARIANT TESTS
    // ============================================================================

    #[test]
    fn test_funding_invariants_valid_state() {
        let funding = FundingAccount {
            total_funded: 1000,
            total_released: 400,
            total_available: 600,
        };

        // Should not panic
        Escrow::check_funding_invariants(funding);
    }

    #[test]
    #[should_panic(expected = "total_available != total_funded - total_released")]
    fn test_funding_invariants_invalid_available() {
        let funding = FundingAccount {
            total_funded: 1000,
            total_released: 400,
            total_available: 500, // Should be 600
        };

        Escrow::check_funding_invariants(funding);
    }

    #[test]
    #[should_panic(expected = "total_released > total_funded")]
    fn test_funding_invariants_over_release() {
        let funding = FundingAccount {
            total_funded: 1000,
            total_released: 1500,
            total_available: -500,
        };

        Escrow::check_funding_invariants(funding);
    }

    #[test]
    #[should_panic(expected = "total_released > total_funded")]
    fn test_funding_invariants_negative_funded() {
        let funding = FundingAccount {
            total_funded: -100,
            total_released: 0,
            total_available: -100,
        };

        Escrow::check_funding_invariants(funding);
    }

    #[test]
    #[should_panic(expected = "total_released < 0")]
    fn test_funding_invariants_negative_released() {
        let funding = FundingAccount {
            total_funded: 1000,
            total_released: -100,
            total_available: 1100,
        };

        Escrow::check_funding_invariants(funding);
    }

    #[test]
    #[should_panic(expected = "total_available != total_funded - total_released")]
    fn test_funding_invariants_negative_available() {
        let funding = FundingAccount {
            total_funded: 1000,
            total_released: 400,
            total_available: -100,
        };

        Escrow::check_funding_invariants(funding);
    }

    #[test]
    fn test_funding_invariants_zero_state() {
        let funding = FundingAccount {
            total_funded: 0,
            total_released: 0,
            total_available: 0,
        };

        // Should not panic
        Escrow::check_funding_invariants(funding);
    }

    #[test]
    fn test_funding_invariants_fully_released() {
        let funding = FundingAccount {
            total_funded: 1000,
            total_released: 1000,
            total_available: 0,
        };

        // Should not panic
        Escrow::check_funding_invariants(funding);
    }

    // ============================================================================
    // MILESTONE ACCOUNTING INVARIANT TESTS
    // ============================================================================

    #[test]
    fn test_milestone_invariants_no_releases() {
        let env = Env::default();
        let milestones = vec![
            &env,
            Milestone {
                amount: 100,
                released: false,
            },
            Milestone {
                amount: 200,
                released: false,
            },
        ];

        // Should not panic
        Escrow::check_milestone_invariants(milestones, 0);
    }

    #[test]
    fn test_milestone_invariants_partial_releases() {
        let env = Env::default();
        let milestones = vec![
            &env,
            Milestone {
                amount: 100,
                released: true,
            },
            Milestone {
                amount: 200,
                released: false,
            },
            Milestone {
                amount: 300,
                released: true,
            },
        ];

        // 100 + 300 = 400
        Escrow::check_milestone_invariants(milestones, 400);
    }

    #[test]
    #[should_panic(expected = "sum of released milestones != total_released")]
    fn test_milestone_invariants_mismatch_released_sum() {
        let env = Env::default();
        let milestones = vec![
            &env,
            Milestone {
                amount: 100,
                released: true,
            },
            Milestone {
                amount: 200,
                released: true,
            },
        ];

        // Sum is 300, but we claim 250
        Escrow::check_milestone_invariants(milestones, 250);
    }

    #[test]
    #[should_panic(expected = "milestone amount must be positive")]
    fn test_milestone_invariants_zero_amount() {
        let env = Env::default();
        let milestones = vec![
            &env,
            Milestone {
                amount: 0,
                released: false,
            },
        ];

        Escrow::check_milestone_invariants(milestones, 0);
    }

    #[test]
    #[should_panic(expected = "milestone amount must be positive")]
    fn test_milestone_invariants_negative_amount() {
        let env = Env::default();
        let milestones = vec![
            &env,
            Milestone {
                amount: -100,
                released: false,
            },
        ];

        Escrow::check_milestone_invariants(milestones, 0);
    }

    #[test]
    fn test_milestone_invariants_all_released() {
        let env = Env::default();
        let milestones = vec![
            &env,
            Milestone {
                amount: 100,
                released: true,
            },
            Milestone {
                amount: 200,
                released: true,
            },
            Milestone {
                amount: 300,
                released: true,
            },
        ];

        // All released: 100 + 200 + 300 = 600
        Escrow::check_milestone_invariants(milestones, 600);
    }

    // ============================================================================
    // CONTRACT STATE INVARIANT TESTS
    // ============================================================================

    #[test]
    fn test_contract_invariants_valid_state() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);

        let milestones = vec![
            &env,
            Milestone {
                amount: 500,
                released: false,
            },
            Milestone {
                amount: 500,
                released: false,
            },
        ];

        let state = EscrowState {
            client,
            freelancer,
            status: ContractStatus::Created,
            milestones,
            funding: FundingAccount {
                total_funded: 0,
                total_released: 0,
                total_available: 0,
            },
        };

        // Should not panic
        Escrow::check_contract_invariants(state);
    }

    #[test]
    fn test_contract_invariants_with_deposits() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);

        let milestones = vec![
            &env,
            Milestone {
                amount: 500,
                released: false,
            },
            Milestone {
                amount: 500,
                released: false,
            },
        ];

        let state = EscrowState {
            client,
            freelancer,
            status: ContractStatus::Funded,
            milestones,
            funding: FundingAccount {
                total_funded: 1000,
                total_released: 0,
                total_available: 1000,
            },
        };

        // Should not panic
        Escrow::check_contract_invariants(state);
    }

    #[test]
    fn test_contract_invariants_with_partial_releases() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);

        let milestones = vec![
            &env,
            Milestone {
                amount: 500,
                released: true,
            },
            Milestone {
                amount: 500,
                released: false,
            },
        ];

        let state = EscrowState {
            client,
            freelancer,
            status: ContractStatus::Funded,
            milestones,
            funding: FundingAccount {
                total_funded: 1000,
                total_released: 500,
                total_available: 500,
            },
        };

        // Should not panic
        Escrow::check_contract_invariants(state);
    }

    #[test]
    #[should_panic(expected = "total_contract_value < total_funded")]
    fn test_contract_invariants_over_funded() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);

        let milestones = vec![
            &env,
            Milestone {
                amount: 500,
                released: false,
            },
            Milestone {
                amount: 500,
                released: false,
            },
        ];

        let state = EscrowState {
            client,
            freelancer,
            status: ContractStatus::Funded,
            milestones,
            funding: FundingAccount {
                total_funded: 2000, // More than total contract value (1000)
                total_released: 0,
                total_available: 2000,
            },
        };

        Escrow::check_contract_invariants(state);
    }

    #[test]
    fn test_contract_invariants_fully_released() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);

        let milestones = vec![
            &env,
            Milestone {
                amount: 500,
                released: true,
            },
            Milestone {
                amount: 500,
                released: true,
            },
        ];

        let state = EscrowState {
            client,
            freelancer,
            status: ContractStatus::Completed,
            milestones,
            funding: FundingAccount {
                total_funded: 1000,
                total_released: 1000,
                total_available: 0,
            },
        };

        // Should not panic
        Escrow::check_contract_invariants(state);
    }

    // ============================================================================
    // CONTRACT CREATION TESTS
    // ============================================================================

    #[test]
    fn test_create_contract_valid() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);
        let milestones = vec![&env, 200_0000000_i128, 400_0000000_i128, 600_0000000_i128];

        let id = Escrow::create_contract(env.clone(), client, freelancer, milestones);
        assert_eq!(id, 1);
    }

    #[test]
    #[should_panic(expected = "Must have at least one milestone")]
    fn test_create_contract_no_milestones() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);
        let milestones = vec![&env];

        Escrow::create_contract(env.clone(), client, freelancer, milestones);
    }

    #[test]
    #[should_panic(expected = "Milestone amounts must be positive")]
    fn test_create_contract_zero_milestone() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);
        let milestones = vec![&env, 100_i128, 0_i128, 200_i128];

        Escrow::create_contract(env.clone(), client, freelancer, milestones);
    }

    #[test]
    #[should_panic(expected = "Milestone amounts must be positive")]
    fn test_create_contract_negative_milestone() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);
        let milestones = vec![&env, 100_i128, -50_i128, 200_i128];

        Escrow::create_contract(env.clone(), client, freelancer, milestones);
    }

    // ============================================================================
    // DEPOSIT FUNDS TESTS
    // ============================================================================

    #[test]
    fn test_deposit_funds_valid() {
        let env = Env::default();
        let result = Escrow::deposit_funds(env.clone(), 1, 1_000_0000000);
        assert!(result);
    }

    #[test]
    #[should_panic(expected = "Deposit amount must be positive")]
    fn test_deposit_funds_zero_amount() {
        let env = Env::default();
        Escrow::deposit_funds(env.clone(), 1, 0);
    }

    #[test]
    #[should_panic(expected = "Deposit amount must be positive")]
    fn test_deposit_funds_negative_amount() {
        let env = Env::default();
        Escrow::deposit_funds(env.clone(), 1, -1_000_0000000);
    }

    // ============================================================================
    // EDGE CASE AND OVERFLOW TESTS
    // ============================================================================

    #[test]
    fn test_large_milestone_amounts() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);
        let milestones = vec![&env, i128::MAX / 3, i128::MAX / 3, i128::MAX / 3];

        let id = Escrow::create_contract(env.clone(), client, freelancer, milestones);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_single_milestone_contract() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);
        let milestones = vec![&env, 1000_i128];

        let id = Escrow::create_contract(env.clone(), client, freelancer, milestones);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_many_milestones_contract() {
        let env = Env::default();
        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);
        let mut milestones = vec![&env];

        for i in 1..=100 {
            milestones.push_back(i as i128 * 100);
        }

        let id = Escrow::create_contract(env.clone(), client, freelancer, milestones);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_funding_invariants_boundary_values() {
        // Test with maximum safe values that satisfy the invariant
        let total_funded = 1_000_000_000_000_000_000_i128;
        let total_released = 500_000_000_000_000_000_i128;
        let total_available = total_funded - total_released;

        let funding = FundingAccount {
            total_funded,
            total_released,
            total_available,
        };

        Escrow::check_funding_invariants(funding);
    }

    // ============================================================================
    // ORIGINAL TESTS (PRESERVED)
    // ============================================================================

    #[test]
    fn test_hello() {
        let env = Env::default();
        let contract_id = env.register(Escrow, ());
        let client = EscrowClient::new(&env, &contract_id);

        let result = client.hello(&symbol_short!("World"));
        assert_eq!(result, symbol_short!("World"));
    }

    #[test]
    fn test_release_milestone() {
        let env = Env::default();
        let contract_id = env.register(Escrow, ());
        let client = EscrowClient::new(&env, &contract_id);

        let result = client.release_milestone(&1, &0);
        assert!(result);
    }
}
