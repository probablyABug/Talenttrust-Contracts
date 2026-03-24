use soroban_sdk::testutils::Address as _;

use super::{
    assert_panics, create_sample_contract, full_funding_amount, register_escrow, sample_parties,
    setup_env,
};

#[test]
fn create_contract_rejects_role_overlap() {
    let env = setup_env();
    let client = register_escrow(&env);
    let parties = sample_parties(&env);
    let milestones = soroban_sdk::vec![&env, 100_i128];

    assert_panics(|| {
        client.create_contract(&parties.client, &parties.client, &milestones);
    });
}

#[test]
fn create_contract_rejects_empty_or_non_positive_milestones() {
    let env = setup_env();
    let client = register_escrow(&env);
    let parties = sample_parties(&env);
    let empty = soroban_sdk::Vec::<i128>::new(&env);
    let invalid = soroban_sdk::vec![&env, 100_i128, 0_i128];

    assert_panics(|| {
        client.create_contract(&parties.client, &parties.freelancer, &empty);
    });
    assert_panics(|| {
        client.create_contract(&parties.client, &parties.freelancer, &invalid);
    });
}

#[test]
fn deposit_rejects_invalid_amounts_and_overfunding() {
    let env = setup_env();
    let client = register_escrow(&env);
    let (_, contract_id) = create_sample_contract(&env, &client);

    assert_panics(|| {
        client.deposit_funds(&contract_id, &0);
    });
    assert_panics(|| {
        client.deposit_funds(&contract_id, &(full_funding_amount() + 1));
    });
}

#[test]
fn release_rejects_unfunded_invalid_and_duplicate_milestones() {
    let env = setup_env();
    let client = register_escrow(&env);
    let (_, contract_id) = create_sample_contract(&env, &client);

    assert_panics(|| {
        client.release_milestone(&contract_id, &0);
    });

    assert!(client.deposit_funds(&contract_id, &full_funding_amount()));

    assert_panics(|| {
        client.release_milestone(&contract_id, &99);
    });

    assert!(client.release_milestone(&contract_id, &0));
    assert_panics(|| {
        client.release_milestone(&contract_id, &0);
    });
}

#[test]
fn migration_rejects_invalid_targets_and_duplicate_requests() {
    let env = setup_env();
    let client = register_escrow(&env);
    let (parties, contract_id) = create_sample_contract(&env, &client);

    assert_panics(|| {
        client.request_client_migration(&contract_id, &parties.client);
    });
    assert_panics(|| {
        client.request_client_migration(&contract_id, &parties.freelancer);
    });

    assert!(client.request_client_migration(&contract_id, &parties.replacement_client));
    assert_panics(|| {
        client.request_client_migration(&contract_id, &soroban_sdk::Address::generate(&env));
    });
}

#[test]
fn completed_contracts_cannot_be_migrated() {
    let env = setup_env();
    let client = register_escrow(&env);
    let (parties, contract_id) = create_sample_contract(&env, &client);

    assert!(client.deposit_funds(&contract_id, &full_funding_amount()));
    assert!(client.release_milestone(&contract_id, &0));
    assert!(client.release_milestone(&contract_id, &1));
    assert!(client.release_milestone(&contract_id, &2));

    assert_panics(|| {
        client.request_client_migration(&contract_id, &parties.replacement_client);
    });
}
