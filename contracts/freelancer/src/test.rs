#![cfg(test)]

extern crate std;

use crate::storage_types::{Escrow, DataKey};
use crate::{contract::FreelanceContract, FreelanceContractClient};
use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Address, Env, Vec, IntoVal, symbol_short};
use crate::token::{ Token, TokenClient };
use crate::utils::u128_to_bytes;

fn create_token<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    let token = TokenClient::new(e, &e.register_contract(None, Token {}));
    token.initialize(admin, &7, &"name".into_val(e), &"symbol".into_val(e));
    token
}

#[test]
fn test_create_fund_complete_objectives() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let freelancer_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let freelance_contract_address = env.register_contract(None, FreelanceContract); 
    let freelance_client = FreelanceContractClient::new(&env, &freelance_contract_address);

    token.mint(&client_address, &1000);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&client_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&client_address), 1000);

    let usdc_contract_address = token.address.clone();

    let expiration_ledger = env.ledger().sequence() + 1000;
    let full_price = 100;
    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    assert_eq!(token.allowance(&client_address, &freelance_contract_address), 100);

    let prices: Vec<u128> = Vec::from_array(&env, [100_u128, 100_u128]);
    let project_id = freelance_client.initialize_escrow(&freelancer_address, &prices, &client_address);
    let project_id_in_bytes = u128_to_bytes(&env, project_id);

    freelance_client.fund_objective(&project_id_in_bytes, &0, &client_address, &usdc_contract_address, &freelance_contract_address);
    env.as_contract(&freelance_contract_address, || {
        let project_key = DataKey::Escrow(project_id_in_bytes.clone());
        let project: Escrow = env.storage().instance().get(&project_key).unwrap();
        let first_objective = project.parties.get(0).unwrap();
        assert_eq!(first_objective.half_paid, 50);
    });

    freelance_client.complete_objective(&project_id_in_bytes, &0, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);
    
    env.as_contract(&freelance_contract_address, || {
        let project_key = DataKey::Escrow(project_id_in_bytes.clone());
        let project: Escrow = env.storage().instance().get(&project_key).unwrap();
        let first_objective = project.parties.get(0).unwrap();
        assert_eq!(first_objective.completed, true);
    });
    
    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &1, &client_address, &usdc_contract_address, &freelance_contract_address);
    freelance_client.complete_objective(&project_id_in_bytes, &1, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);
    
    env.as_contract(&freelance_contract_address, || {
        let project_key = DataKey::Escrow(project_id_in_bytes.clone());
        let project: Escrow = env.storage().instance().get(&project_key).unwrap();
        let second_objective = project.parties.get(1).unwrap();
        assert_eq!(second_objective.completed, true);
    });
}

#[test]
fn test_client_can_recover_funds_if_freelancer_does_not_complete_all_objectives() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let freelancer_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let freelance_contract_address = env.register_contract(None, FreelanceContract);
    let freelance_client = FreelanceContractClient::new(&env, &freelance_contract_address);

    token.mint(&client_address, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&client_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&client_address), 1000);

    let usdc_contract_address = token.address.clone();
    let expiration_ledger = env.ledger().sequence() + 1000;
    let full_price = 100;

    let prices: Vec<u128> = Vec::from_array(&env, [100_u128, 100_u128, 100_u128]);
    let project_id = freelance_client.initialize_escrow(&freelancer_address, &prices, &client_address);
    let project_id_in_bytes = u128_to_bytes(&env, project_id);

    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &0, &client_address, &usdc_contract_address, &freelance_contract_address);
    env.as_contract(&freelance_contract_address, || {
        let project_key = DataKey::Escrow(project_id_in_bytes.clone());
        let project: Escrow = env.storage().instance().get(&project_key).unwrap();
        let first_objective = project.parties.get(0).unwrap();
        assert_eq!(first_objective.half_paid, 50);
    });

    freelance_client.complete_objective(&project_id_in_bytes, &0, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);

    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &1, &client_address, &usdc_contract_address, &freelance_contract_address);

    freelance_client.complete_objective(&project_id_in_bytes, &1, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);

    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &2, &client_address, &usdc_contract_address, &freelance_contract_address);

    freelance_client.cancel_project(&project_id_in_bytes, &client_address);

    env.as_contract(&freelance_contract_address, || {
        let balance = token.balance(&freelance_contract_address);
        assert_eq!(balance, 50);
    });
    
    freelance_client.refund_remaining_funds(&project_id_in_bytes, &2, &client_address, &usdc_contract_address, &freelance_contract_address);

    env.as_contract(&freelance_contract_address, || {
        let balance = token.balance(&freelance_contract_address);
        assert_eq!(balance, 0);
    });

    let client_balance = token.balance(&client_address);
    let freelancer_balance = token.balance(&freelancer_address);

    assert_eq!(client_balance, 800);
    assert_eq!(freelancer_balance, 200);
}

#[test]
fn test_add_new_objectives_and_complete_them() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let freelancer_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let freelance_contract_address = env.register_contract(None, FreelanceContract);
    let freelance_client = FreelanceContractClient::new(&env, &freelance_contract_address);

    token.mint(&client_address, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&client_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&client_address), 1000);

    let usdc_contract_address = token.address.clone();
    let expiration_ledger = env.ledger().sequence() + 1000;
    let full_price = 100;

    let prices: Vec<u128> = Vec::from_array(&env, [100_u128, 100_u128]);
    let project_id = freelance_client.initialize_escrow(&freelancer_address, &prices, &client_address);
    let project_id_in_bytes = u128_to_bytes(&env, project_id);

    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &0, &client_address, &usdc_contract_address, &freelance_contract_address);
    env.as_contract(&freelance_contract_address, || {
        let project_key = DataKey::Escrow(project_id_in_bytes.clone());
        let project: Escrow = env.storage().instance().get(&project_key).unwrap();
        let first_objective = project.parties.get(0).unwrap();
        assert_eq!(first_objective.half_paid, 50);
    });

    freelance_client.complete_objective(&project_id_in_bytes, &0, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);

    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &1, &client_address, &usdc_contract_address, &freelance_contract_address);

    freelance_client.complete_objective(&project_id_in_bytes, &1, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);

    let new_prices: Vec<u128> = Vec::from_array(&env, [100_u128]);
    freelance_client.add_objective(&project_id_in_bytes, &new_prices, &client_address);

    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &2, &client_address, &usdc_contract_address, &freelance_contract_address);

    freelance_client.complete_objective(&project_id_in_bytes, &2, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);

    let client_balance = token.balance(&client_address);
    let freelancer_balance = token.balance(&freelancer_address);

    assert_eq!(client_balance, 700);
    assert_eq!(freelancer_balance, 300);
}

#[test]
fn test_complete_project_after_all_objectives_completed() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let freelancer_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let freelance_contract_address = env.register_contract(None, FreelanceContract);
    let freelance_client = FreelanceContractClient::new(&env, &freelance_contract_address);

    token.mint(&client_address, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&client_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&client_address), 1000);

    let usdc_contract_address = token.address.clone();
    let expiration_ledger = env.ledger().sequence() + 1000;
    let full_price = 100;

    let prices: Vec<u128> = Vec::from_array(&env, [100_u128, 100_u128]);
    let project_id = freelance_client.initialize_escrow(&freelancer_address, &prices, &client_address);
    let project_id_in_bytes = u128_to_bytes(&env, project_id);

    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &0, &client_address, &usdc_contract_address, &freelance_contract_address);
    env.as_contract(&freelance_contract_address, || {
        let project_key = DataKey::Escrow(project_id_in_bytes.clone());
        let project: Escrow = env.storage().instance().get(&project_key).unwrap();
        let first_objective = project.parties.get(0).unwrap();
        assert_eq!(first_objective.half_paid, 50);
    });

    freelance_client.complete_objective(&project_id_in_bytes, &0, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);

    token.approve(&client_address, &freelance_contract_address, &full_price, &expiration_ledger);
    freelance_client.fund_objective(&project_id_in_bytes, &1, &client_address, &usdc_contract_address, &freelance_contract_address);

    freelance_client.complete_objective(&project_id_in_bytes, &1, &client_address, &usdc_contract_address, &freelance_contract_address, &freelancer_address);

    freelance_client.complete_project(&project_id_in_bytes, &client_address);

    env.as_contract(&freelance_contract_address, || {
        let project_key = DataKey::Escrow(project_id_in_bytes);
        let project: Escrow = env.storage().instance().get(&project_key).unwrap();
        assert_eq!(project.completed, true);
    });
}

#[test]
fn test_get_projects_by_freelancer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let another_client_address = Address::generate(&env);
    let freelancer_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let freelance_contract_address = env.register_contract(None, FreelanceContract);
    let freelance_client = FreelanceContractClient::new(&env, &freelance_contract_address);

    token.mint(&client_address, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&client_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&client_address), 1000);

    let prices: Vec<u128> = Vec::from_array(&env, [100_u128, 100_u128]);
    freelance_client.initialize_escrow(&freelancer_address, &prices, &client_address);
    freelance_client.initialize_escrow(&freelancer_address, &prices, &another_client_address);

    let projects = freelance_client.get_projects_by_freelancer(&freelancer_address);

    assert_eq!(projects.len(), 2);
}

#[test]
fn test_get_projects_by_client() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let freelancer_address = Address::generate(&env);
    let another_freelancer_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let freelance_contract_address = env.register_contract(None, FreelanceContract);
    let freelance_client = FreelanceContractClient::new(&env, &freelance_contract_address);

    token.mint(&client_address, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&client_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&client_address), 1000);

    let prices: Vec<u128> = Vec::from_array(&env, [100_u128, 100_u128]);
    freelance_client.initialize_escrow(&freelancer_address, &prices, &client_address);
    freelance_client.initialize_escrow(&another_freelancer_address, &prices, &client_address);

    let projects = freelance_client.get_projects_by_spender(&client_address);
    assert_eq!(projects.len(), 2);
}