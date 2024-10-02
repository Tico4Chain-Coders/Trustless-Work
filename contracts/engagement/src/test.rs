#![cfg(test)]

extern crate std;

use crate::storage_types::{Engagement, DataKey};
use crate::{contract::EngagementContract, EngagementContractClient};
use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Address, Env, Vec, IntoVal, symbol_short};
use crate::token::{ Token, TokenClient };
use crate::utils::u128_to_bytes;

fn create_token<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    let token = TokenClient::new(e, &e.register_contract(None, Token {}));
    token.initialize(admin, &7, &"name".into_val(e), &"symbol".into_val(e));
    token
}

#[test]
fn test_create_fund_complete_escrows() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

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
    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    assert_eq!(token.allowance(&client_address, &engagement_contract_address), 100);

    let prices: Vec<u128> = Vec::from_array(&env, [100_u128, 100_u128]);
    let engagement_id = engagement_client.initialize_escrow(&service_provider_address, &prices, &client_address);
    let engagement_id_in_bytes = u128_to_bytes(&env, engagement_id);

    engagement_client.fund_escrow(&engagement_id_in_bytes, &0, &client_address, &usdc_contract_address, &engagement_contract_address);
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Engagement(engagement_id_in_bytes.clone());
        let engagement: Engagement = env.storage().instance().get(&engagement_key).unwrap();
        let first_escrow = engagement.parties.get(0).unwrap();
        assert_eq!(first_escrow.amount_paid, 50);
    });

    engagement_client.complete_escrow(&engagement_id_in_bytes, &0, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);
    
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Engagement(engagement_id_in_bytes.clone());
        let engagement: Engagement = env.storage().instance().get(&engagement_key).unwrap();
        let second_escrow = engagement.parties.get(0).unwrap();
        assert_eq!(second_escrow.completed, true);
    });
    
    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &1, &client_address, &usdc_contract_address, &engagement_contract_address);
    engagement_client.complete_escrow(&engagement_id_in_bytes, &1, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);
    
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Engagement(engagement_id_in_bytes.clone());
        let engagement: Engagement = env.storage().instance().get(&engagement_key).unwrap();
        let third_escrow = engagement.parties.get(1).unwrap();
        assert_eq!(third_escrow.completed, true);
    });
}

#[test]
fn test_client_can_recover_funds_if_service_provider_does_not_complete_all_escrows() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

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
    let engagement_id = engagement_client.initialize_escrow(&service_provider_address, &prices, &client_address);
    let engagement_id_in_bytes = u128_to_bytes(&env, engagement_id);

    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &0, &client_address, &usdc_contract_address, &engagement_contract_address);
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Engagement(engagement_id_in_bytes.clone());
        let engagement: Engagement = env.storage().instance().get(&engagement_key).unwrap();
        let first_escrow = engagement.parties.get(0).unwrap();
        assert_eq!(first_escrow.amount_paid, 50);
    });

    engagement_client.complete_escrow(&engagement_id_in_bytes, &0, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);

    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &1, &client_address, &usdc_contract_address, &engagement_contract_address);

    engagement_client.complete_escrow(&engagement_id_in_bytes, &1, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);

    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &2, &client_address, &usdc_contract_address, &engagement_contract_address);

    engagement_client.cancel_engagement(&engagement_id_in_bytes, &client_address);

    env.as_contract(&engagement_contract_address, || {
        let balance = token.balance(&engagement_contract_address);
        assert_eq!(balance, 50);
    });
    
    engagement_client.refund_remaining_funds(&engagement_id_in_bytes, &2, &client_address, &usdc_contract_address, &engagement_contract_address);

    env.as_contract(&engagement_contract_address, || {
        let balance = token.balance(&engagement_contract_address);
        assert_eq!(balance, 0);
    });

    let client_balance = token.balance(&client_address);
    let service_provider_balance = token.balance(&service_provider_address);

    assert_eq!(client_balance, 800);
    assert_eq!(service_provider_balance, 200);
}

#[test]
fn test_add_new_escrows_and_complete_them() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

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
    let engagement_id = engagement_client.initialize_escrow(&service_provider_address, &prices, &client_address);
    let engagement_id_in_bytes = u128_to_bytes(&env, engagement_id);

    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &0, &client_address, &usdc_contract_address, &engagement_contract_address);
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Engagement(engagement_id_in_bytes.clone());
        let engagement: Engagement = env.storage().instance().get(&engagement_key).unwrap();
        let first_escrow = engagement.parties.get(0).unwrap();
        assert_eq!(first_escrow.amount_paid, 50);
    });

    engagement_client.complete_escrow(&engagement_id_in_bytes, &0, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);

    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &1, &client_address, &usdc_contract_address, &engagement_contract_address);

    engagement_client.complete_escrow(&engagement_id_in_bytes, &1, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);

    let new_prices: Vec<u128> = Vec::from_array(&env, [100_u128]);
    engagement_client.add_escrow(&engagement_id_in_bytes, &new_prices, &client_address);

    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &2, &client_address, &usdc_contract_address, &engagement_contract_address);

    engagement_client.complete_escrow(&engagement_id_in_bytes, &2, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);

    let client_balance = token.balance(&client_address);
    let service_provider_balance = token.balance(&service_provider_address);

    assert_eq!(client_balance, 700);
    assert_eq!(service_provider_balance, 300);
}

#[test]
fn test_complete_engagement_after_all_escrows_completed() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

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
    let engagement_id = engagement_client.initialize_escrow(&service_provider_address, &prices, &client_address);
    let engagement_id_in_bytes = u128_to_bytes(&env, engagement_id);

    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &0, &client_address, &usdc_contract_address, &engagement_contract_address);
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Engagement(engagement_id_in_bytes.clone());
        let engagement: Engagement = env.storage().instance().get(&engagement_key).unwrap();
        let first_escrow = engagement.parties.get(0).unwrap();
        assert_eq!(first_escrow.amount_paid, 50);
    });

    engagement_client.complete_escrow(&engagement_id_in_bytes, &0, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);

    token.approve(&client_address, &engagement_contract_address, &full_price, &expiration_ledger);
    engagement_client.fund_escrow(&engagement_id_in_bytes, &1, &client_address, &usdc_contract_address, &engagement_contract_address);

    engagement_client.complete_escrow(&engagement_id_in_bytes, &1, &client_address, &usdc_contract_address, &engagement_contract_address, &service_provider_address);

    engagement_client.complete_engagement(&engagement_id_in_bytes, &client_address);

    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Engagement(engagement_id_in_bytes.clone());
        let engagement: Engagement = env.storage().instance().get(&engagement_key).unwrap();
        assert_eq!(engagement.completed, true);
    });
}

#[test]
fn test_get_engagements_by_service_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let another_client_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

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
    engagement_client.initialize_escrow(&service_provider_address, &prices, &client_address);
    engagement_client.initialize_escrow(&service_provider_address, &prices, &another_client_address);

    let page = 0;
    let limit = 2;
    let engagements = engagement_client.get_engagements_by_provider(&service_provider_address, &page, &limit);
    assert_eq!(engagements.len(), 2);

    let another_client_address2 = Address::generate(&env);
    engagement_client.initialize_escrow(&service_provider_address, &prices, &another_client_address2);

    let engagements_page_2 = engagement_client.get_engagements_by_provider(&service_provider_address, &1, &2);
    assert_eq!(engagements_page_2.len(), 1); 
}

#[test]
fn test_get_engagements_by_client() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let another_service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

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
    engagement_client.initialize_escrow(&service_provider_address, &prices, &client_address);
    engagement_client.initialize_escrow(&another_service_provider_address, &prices, &client_address);

    let page = 0;
    let limit = 2;
    let engagements = engagement_client.get_engagements_by_client(&client_address, &page, &limit);
    assert_eq!(engagements.len(), 2);

    let another_service_provider_address2 = Address::generate(&env);
    engagement_client.initialize_escrow(&another_service_provider_address2, &prices, &client_address);

    let engagements_page_2 = engagement_client.get_engagements_by_client(&client_address, &1, &2);
    assert_eq!(engagements_page_2.len(), 1); 
}