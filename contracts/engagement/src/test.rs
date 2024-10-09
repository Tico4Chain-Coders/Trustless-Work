#![cfg(test)]

extern crate std;

use crate::storage_types::{Escrow, DataKey};
use crate::{contract::EngagementContract, EngagementContractClient};
use crate::error::ContractError;
use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Address, Env, IntoVal, String, symbol_short};
use crate::token::{ Token, TokenClient };

fn create_token<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    let token = TokenClient::new(e, &e.register_contract(None, Token {}));
    token.initialize(admin, &6, &"name".into_val(e), &"symbol".into_val(e));
    token
}

#[test]
fn test_create_fund_complete_escrows() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let signer_address = Address::generate(&env);
    let issuer_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

    token.mint(&signer_address, &1_000_000_000);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&signer_address, 1_000_000_000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&signer_address), 1_000_000_000);

    let usdc_contract_address = token.address.clone();
    let engagement_id = String::from_str(&env, "41431");
    let description = String::from_str(&env, "Any description");

    let amount: u128 = 100_000_000;
    let engagement_id = engagement_client.initialize_escrow(
        &engagement_id.clone(), 
        &description, 
        &issuer_address, 
        &service_provider_address, 
        &amount, 
        &signer_address,
    );
    let engagement_id_copy = engagement_id.clone();
    
    engagement_client.fund_escrow(&engagement_id, &signer_address, &usdc_contract_address, &engagement_contract_address);
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Escrow(engagement_id);
        let engagement: Escrow = env.storage().instance().get(&engagement_key).unwrap();
        assert_eq!(engagement.balance, 50_000_000);

        let signer_new_balance = token.balance(&signer_address);
        assert_eq!(signer_new_balance, 1_000_000_000 - (50 * 1_000_000));
    });

    engagement_client.complete_escrow(
        &engagement_id_copy, 
        &signer_address, 
        &usdc_contract_address, 
        &engagement_contract_address, 
    );
    
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Escrow(engagement_id_copy.clone());
        let engagement: Escrow = env.storage().instance().get(&engagement_key).unwrap();
        
        assert_eq!(engagement.completed, true);
        assert_eq!(engagement.balance, engagement.amount);
    });
}

#[test]
fn test_initialize_escrow_prices_cannot_be_zero() {
    let env = Env::default();
    
    let issuer = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let signer = Address::generate(&env);
    
    let description = String::from_str(&env, "Test escrow");

    let result = EngagementContract::initialize_escrow(
        env.clone(),
        String::from_str(&env, "engagement_1"),
        description,
        issuer,
        service_provider,
        0,
        signer,
    );

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error, ContractError::PricesCannotBeZero);
}

#[test]
fn test_client_can_recover_funds_if_service_provider_does_not_complete_all_escrows() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let signer_address = Address::generate(&env);
    let issuer_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

    token.mint(&signer_address, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&signer_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&signer_address), 1000);

    let usdc_contract_address = token.address.clone();
    let engagement_id = String::from_str(&env, "41431");
    let description = String::from_str(&env, "Any description");

    let amount: u128 = 100_u128;
    let engagement_id = engagement_client.initialize_escrow(
        &engagement_id.clone(), 
        &description, 
        &issuer_address, 
        &service_provider_address, 
        &amount, 
        &signer_address,
    );
    let engagement_id_copy = engagement_id.clone();

    engagement_client.fund_escrow(&engagement_id, &signer_address, &usdc_contract_address, &engagement_contract_address);
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Escrow(engagement_id);
        let engagement: Escrow = env.storage().instance().get(&engagement_key).unwrap();
        assert_eq!(engagement.balance, 50);
    });

    engagement_client.cancel_escrow(&engagement_id_copy, &signer_address);
    engagement_client.refund_remaining_funds(&engagement_id_copy, &signer_address, &usdc_contract_address, &engagement_contract_address);

    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Escrow(engagement_id_copy);
        let engagement: Escrow = env.storage().instance().get(&engagement_key).unwrap();
        let contract_balance = token.balance(&engagement_contract_address);
        let signer_balance = token.balance(&signer_address);
        assert_eq!(contract_balance, 0);
        assert_eq!(signer_balance, 1000);
        assert_eq!(engagement.cancelled, true);
        assert_eq!(engagement.balance, 0);
    });
}  

#[test]
fn test_get_engagements_by_service_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let signer_address = Address::generate(&env);
    let issuer_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let token = create_token(&env, &admin1);

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

    token.mint(&signer_address, &1_000_000_000);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&signer_address, 1_000_000_000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&signer_address), 1_000_000_000);

    let engagement_id = String::from_str(&env, "41431");
    let description = String::from_str(&env, "Any description");

    let amount: u128 = 100_u128;
    let engagement_id = engagement_client.initialize_escrow(
        &engagement_id.clone(), 
        &description, 
        &issuer_address, 
        &service_provider_address, 
        &amount, 
        &signer_address,
    );

    let escrow = engagement_client.get_escrow_by_id(&engagement_id);
    assert_eq!(escrow.engagement_id, engagement_id);
    assert_eq!(escrow.issuer, issuer_address);
    assert_eq!(escrow.signer, signer_address);
    assert_eq!(escrow.service_provider, service_provider_address);
}

#[test]
fn test_get_escrow_by_invalid_id() {
    let env = Env::default();
    env.mock_all_auths();

    let invalid_engagement_id = String::from_str(&env, "99999");

    let engagement_contract_address = env.register_contract(None, EngagementContract);

    env.as_contract(&engagement_contract_address, || {
        let result = EngagementContract::get_escrow_by_id(env.clone(), invalid_engagement_id);
    
        assert!(result.is_err());
        let error_message = result.unwrap_err();
        
        assert_eq!(error_message, ContractError::EscrowNotFound);
    });
}