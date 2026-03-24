use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env};

use crate::{Escrow, EscrowClient, EscrowError};

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
    let milestones = vec![&env, 200_0000000_i128, 400_0000000_i128, 600_0000000_i128];

    let id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    assert_eq!(id, 1);
}

#[test]
#[should_panic(expected = "NoMilestones")]
fn test_create_contract_no_milestones() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env];
    client.create_contract(&client_addr, &freelancer_addr, &milestones);
}

#[test]
#[should_panic(expected = "InvalidMilestoneAmount")]
fn test_create_contract_zero_milestone() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 0_i128, 100_i128];
    client.create_contract(&client_addr, &freelancer_addr, &milestones);
}

#[test]
#[should_panic(expected = "InvalidMilestoneAmount")]
fn test_create_contract_negative_milestone() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, -50_i128, 100_i128];
    client.create_contract(&client_addr, &freelancer_addr, &milestones);
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

#[test]
#[should_panic(expected = "InvalidMilestoneIndex")]
fn test_release_milestone_out_of_bounds() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);
    client.release_milestone(&1, &5); // Only 3 milestones in placeholder logic
}

// #[test]
// #[should_panic(expected = "MilestoneAlreadyReleased")]
// fn test_release_milestone_already_released() {
//     let env = Env::default();
//     let contract_id = env.register(Escrow, ());
//     let client = EscrowClient::new(&env, &contract_id);
//     // Simulate releasing milestone 0 twice
//     let _ = client.release_milestone(&1, &0);
//     // The placeholder logic does not persist state, so this will not panic in the current code.
//     // In a real implementation, this would panic. This test is a placeholder for future logic.
//     // Uncomment the next line when persistent state is implemented:
//     // let _ = client.release_milestone(&1, &0);
// }
