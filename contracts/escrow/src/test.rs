use soroban_sdk::{
    symbol_short, testutils::Address as _, testutils::Ledger as _, vec, Address, Env, String,
};

use crate::{ContractStatus, DataKey, DisputeError, Escrow, EscrowClient};

// ---------------------------------------------------------------------------
// Helper: patch escrow status via env.as_contract
// ---------------------------------------------------------------------------

fn set_escrow_status(env: &Env, contract_id: &Address, status: ContractStatus) {
    env.as_contract(contract_id, || {
        let mut state: crate::EscrowState = env
            .storage()
            .persistent()
            .get(&DataKey::EscrowState(1_u32))
            .unwrap();
        state.status = status;
        env.storage()
            .persistent()
            .set(&DataKey::EscrowState(1_u32), &state);
    });
}

// ---------------------------------------------------------------------------
// Existing tests (kept passing)
// ---------------------------------------------------------------------------

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("World"));
    assert_eq!(result, symbol_short!("World"));
}

#[test]
fn test_create_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 200_0000000_i128, 400_0000000_i128, 600_0000000_i128];

    let id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    assert_eq!(id, 1);
}

#[test]
fn test_deposit_funds() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let result = client.deposit_funds(&1, &1_000_0000000);
    assert!(result);
}

#[test]
fn test_release_milestone() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let result = client.release_milestone(&1, &0);
    assert!(result);
}

// ---------------------------------------------------------------------------
// Dispute unit tests
// ---------------------------------------------------------------------------

/// Client can initiate a dispute on a Funded escrow; status becomes Disputed.
#[test]
fn test_initiate_dispute_from_client() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Escrow, ());
    let escrow = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    escrow.create_contract(&client_addr, &freelancer_addr, &vec![&env, 100_i128]);

    set_escrow_status(&env, &contract_id, ContractStatus::Funded);

    let reason = String::from_str(&env, "Work not delivered");
    escrow.initiate_dispute(&1_u32, &client_addr, &reason);

    let updated_status = env.as_contract(&contract_id, || {
        let state: crate::EscrowState = env
            .storage()
            .persistent()
            .get(&DataKey::EscrowState(1_u32))
            .unwrap();
        state.status
    });
    assert_eq!(updated_status, ContractStatus::Disputed);
}

/// Freelancer can initiate a dispute on a Funded escrow.
#[test]
fn test_initiate_dispute_from_freelancer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Escrow, ());
    let escrow = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    escrow.create_contract(&client_addr, &freelancer_addr, &vec![&env, 100_i128]);

    set_escrow_status(&env, &contract_id, ContractStatus::Funded);

    let reason = String::from_str(&env, "Payment withheld");
    escrow.initiate_dispute(&1_u32, &freelancer_addr, &reason);

    let updated_status = env.as_contract(&contract_id, || {
        let state: crate::EscrowState = env
            .storage()
            .persistent()
            .get(&DataKey::EscrowState(1_u32))
            .unwrap();
        state.status
    });
    assert_eq!(updated_status, ContractStatus::Disputed);
}

/// Disputing a Created escrow returns InvalidStatus.
#[test]
fn test_dispute_on_created_escrow_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Escrow, ());
    let escrow = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    escrow.create_contract(&client_addr, &freelancer_addr, &vec![&env, 100_i128]);

    let reason = String::from_str(&env, "Too early");
    let result = escrow.try_initiate_dispute(&1_u32, &client_addr, &reason);
    assert_eq!(result, Err(Ok(DisputeError::InvalidStatus)));
}

/// Disputing an already-Disputed escrow returns AlreadyDisputed.
#[test]
fn test_dispute_already_disputed_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Escrow, ());
    let escrow = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    escrow.create_contract(&client_addr, &freelancer_addr, &vec![&env, 100_i128]);

    set_escrow_status(&env, &contract_id, ContractStatus::Funded);

    let reason = String::from_str(&env, "First dispute");
    escrow.initiate_dispute(&1_u32, &client_addr, &reason);

    let reason2 = String::from_str(&env, "Second dispute");
    let result = escrow.try_initiate_dispute(&1_u32, &client_addr, &reason2);
    assert_eq!(result, Err(Ok(DisputeError::AlreadyDisputed)));
}

/// get_dispute returns None when no dispute has been initiated.
#[test]
fn test_get_dispute_no_record() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Escrow, ());
    let escrow = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    escrow.create_contract(&client_addr, &freelancer_addr, &vec![&env, 100_i128]);

    let record = escrow.get_dispute(&1_u32);
    assert!(record.is_none());
}

/// get_dispute returns the correct record after a successful dispute.
#[test]
fn test_get_dispute_returns_record() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_700_000_000); // set a realistic timestamp

    let contract_id = env.register(Escrow, ());
    let escrow = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    escrow.create_contract(&client_addr, &freelancer_addr, &vec![&env, 100_i128]);

    set_escrow_status(&env, &contract_id, ContractStatus::Funded);

    let reason = String::from_str(&env, "Milestone not met");
    escrow.initiate_dispute(&1_u32, &client_addr, &reason);

    let record = escrow.get_dispute(&1_u32).expect("record should exist");
    assert_eq!(record.initiator, client_addr);
    assert_eq!(record.reason, reason);
    assert!(record.timestamp > 0);
}

/// A third-party address (not client or freelancer) is rejected with Unauthorized.
#[test]
fn test_dispute_unauthorized_caller() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Escrow, ());
    let escrow = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let third_party = Address::generate(&env);
    escrow.create_contract(&client_addr, &freelancer_addr, &vec![&env, 100_i128]);

    set_escrow_status(&env, &contract_id, ContractStatus::Funded);

    let reason = String::from_str(&env, "Unauthorized attempt");
    let result = escrow.try_initiate_dispute(&1_u32, &third_party, &reason);
    assert_eq!(result, Err(Ok(DisputeError::Unauthorized)));
}

/// Dispute can be initiated on a Completed escrow.
#[test]
fn test_dispute_on_completed_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Escrow, ());
    let escrow = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    escrow.create_contract(&client_addr, &freelancer_addr, &vec![&env, 100_i128]);

    set_escrow_status(&env, &contract_id, ContractStatus::Completed);

    let reason = String::from_str(&env, "Quality issue after completion");
    escrow.initiate_dispute(&1_u32, &client_addr, &reason);

    let updated_status = env.as_contract(&contract_id, || {
        let state: crate::EscrowState = env
            .storage()
            .persistent()
            .get(&DataKey::EscrowState(1_u32))
            .unwrap();
        state.status
    });
    assert_eq!(updated_status, ContractStatus::Disputed);
}
