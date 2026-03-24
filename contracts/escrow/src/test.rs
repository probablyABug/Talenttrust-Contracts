use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env, String};

use crate::{Escrow, EscrowClient};

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
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 200_0000000_i128, 400_0000000_i128];

    client.create_contract(&client_addr, &freelancer_addr, &milestones);
    
    let stored_milestones = client.get_milestones();
    assert_eq!(stored_milestones.len(), 2);
    assert_eq!(stored_milestones.get(0).unwrap().amount, 200_0000000_i128);
}

#[test]
fn test_deposit_funds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    client.create_contract(&client_addr, &freelancer_addr, &vec![&env, 1000]);

    client.deposit_funds(&1000);
}

#[test]
fn test_release_milestone_and_idempotency() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 200_i128];
    client.create_contract(&client_addr, &freelancer_addr, &milestones);

    let evidence = String::from_str(&env, "ipfs://work-evidence-hash");
    
    // First release
    client.release_milestone(&0, &evidence);
    
    let stored_milestones = client.get_milestones();
    let milestone = stored_milestones.get(0).unwrap();
    assert!(milestone.released);
    assert_eq!(milestone.work_evidence, Some(evidence.clone()));

    // Idempotency: Second release should fail
    let result = client.try_release_milestone(&0, &evidence);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_prevent_reinitialization() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 100];

    client.create_contract(&client_addr, &freelancer_addr, &milestones);
    // Should panic
    client.create_contract(&client_addr, &freelancer_addr, &milestones);
}
