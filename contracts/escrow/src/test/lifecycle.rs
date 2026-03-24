use crate::ContractStatus;

use super::{
    create_sample_contract, full_funding_amount, register_escrow, sample_milestones,
    sample_parties, setup_env,
};

#[test]
fn create_contract_persists_state() {
    let env = setup_env();
    let client = register_escrow(&env);
    let parties = sample_parties(&env);
    let milestones = sample_milestones(&env);

    let contract_id = client.create_contract(&parties.client, &parties.freelancer, &milestones);
    let contract = client.get_contract(&contract_id);

    assert_eq!(contract_id, 1);
    assert_eq!(contract.client, parties.client);
    assert_eq!(contract.freelancer, parties.freelancer);
    assert_eq!(contract.total_amount, full_funding_amount());
    assert_eq!(contract.funded_amount, 0);
    assert_eq!(contract.released_amount, 0);
    assert_eq!(contract.status, ContractStatus::Created);
    assert_eq!(contract.milestones.len(), 3);
    assert!(!contract.milestones.get(0).unwrap().released);
}

#[test]
fn create_contract_allocates_incrementing_ids() {
    let env = setup_env();
    let client = register_escrow(&env);

    let (_, first_id) = create_sample_contract(&env, &client);
    let (_, second_id) = create_sample_contract(&env, &client);

    assert_eq!(first_id, 1);
    assert_eq!(second_id, 2);
}

#[test]
fn deposit_funds_tracks_partial_and_full_funding() {
    let env = setup_env();
    let client = register_escrow(&env);
    let (_, contract_id) = create_sample_contract(&env, &client);

    assert!(client.deposit_funds(&contract_id, &200_0000000));
    let after_partial = client.get_contract(&contract_id);
    assert_eq!(after_partial.funded_amount, 200_0000000);
    assert_eq!(after_partial.status, ContractStatus::Created);

    assert!(client.deposit_funds(&contract_id, &1_000_0000000));
    let after_full = client.get_contract(&contract_id);
    assert_eq!(after_full.funded_amount, full_funding_amount());
    assert_eq!(after_full.status, ContractStatus::Funded);
}

#[test]
fn release_milestone_updates_status_and_amounts() {
    let env = setup_env();
    let client = register_escrow(&env);
    let (_, contract_id) = create_sample_contract(&env, &client);

    assert!(client.deposit_funds(&contract_id, &full_funding_amount()));
    assert!(client.release_milestone(&contract_id, &0));

    let after_first_release = client.get_contract(&contract_id);
    assert_eq!(after_first_release.released_amount, 200_0000000);
    assert!(after_first_release.milestones.get(0).unwrap().released);
    assert_eq!(after_first_release.status, ContractStatus::Funded);

    assert!(client.release_milestone(&contract_id, &1));
    assert!(client.release_milestone(&contract_id, &2));

    let completed = client.get_contract(&contract_id);
    assert_eq!(completed.released_amount, full_funding_amount());
    assert_eq!(completed.status, ContractStatus::Completed);
}
