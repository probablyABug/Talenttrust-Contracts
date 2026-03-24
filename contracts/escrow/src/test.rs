extern crate std;

use std::panic::{catch_unwind, AssertUnwindSafe};

use soroban_sdk::{testutils::Address as _, vec, Address, Env, Vec};

use crate::{Escrow, EscrowClient};

mod client_migration;
mod hello;
mod lifecycle;
mod security;

pub(super) struct Parties {
    pub client: Address,
    pub freelancer: Address,
    pub replacement_client: Address,
}

pub(super) fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

pub(super) fn register_escrow<'a>(env: &'a Env) -> EscrowClient<'a> {
    let contract_id = env.register(Escrow, ());
    EscrowClient::new(env, &contract_id)
}

pub(super) fn sample_parties(env: &Env) -> Parties {
    Parties {
        client: Address::generate(env),
        freelancer: Address::generate(env),
        replacement_client: Address::generate(env),
    }
}

pub(super) fn sample_milestones(env: &Env) -> Vec<i128> {
    vec![env, 200_0000000_i128, 400_0000000_i128, 600_0000000_i128]
}

pub(super) fn create_sample_contract(env: &Env, client: &EscrowClient<'_>) -> (Parties, u32) {
    let parties = sample_parties(env);
    let milestones = sample_milestones(env);
    let contract_id = client.create_contract(&parties.client, &parties.freelancer, &milestones);
    (parties, contract_id)
}

pub(super) fn full_funding_amount() -> i128 {
    1_200_0000000
}

pub(super) fn assert_panics<F>(f: F)
where
    F: FnOnce(),
{
    assert!(catch_unwind(AssertUnwindSafe(f)).is_err());
}
