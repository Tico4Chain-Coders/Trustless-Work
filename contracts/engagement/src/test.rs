#![cfg(test)]

extern crate std;

use crate::storage_types::{DataKey, Escrow, Milestone};
use crate::{contract::EngagementContract, EngagementContractClient};
use crate::error::ContractError;
// use soroban_sdk::vec;
use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Address, Env, IntoVal, String, symbol_short, vec};
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
    let client_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = (0.3 * 10u128.pow(18) as f64) as u128;
    let token = create_token(&env, &admin1);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

    token.mint(&release_signer_address, &1_000_000_000);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&release_signer_address, 1_000_000_000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&release_signer_address), 1_000_000_000);

    let usdc_contract_address = token.address.clone();
    let engagement_id = String::from_str(&env, "41431");

    let amount: u128 = 100_000_000;
    let engagement_id = engagement_client.initialize_escrow(
        &engagement_id.clone(), 
        &client_address, 
        &service_provider_address, 
        &platform_address, 
        &amount, 
        &platform_fee,
        &milestones,
        &release_signer_address,
        &dispute_resolver_address
    );
    let engagement_id_copy = engagement_id.clone();
    
    engagement_client.fund_escrow(&engagement_id, &release_signer_address, &usdc_contract_address, &engagement_contract_address);
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Escrow(engagement_id);
        let engagement: Escrow = env.storage().instance().get(&engagement_key).unwrap();
        assert_eq!(engagement.balance, 50_000_000);

        let signer_new_balance = token.balance(&release_signer_address);
        assert_eq!(signer_new_balance, 1_000_000_000 - (50 * 1_000_000));
    });

    engagement_client.complete_escrow(
        &engagement_id_copy, 
        &release_signer_address, 
        &usdc_contract_address, 
        &engagement_contract_address, 
    );

    engagement_client.claim_escrow_earnings(
        &engagement_id_copy, 
        &service_provider_address, 
        &usdc_contract_address, 
        &engagement_contract_address, 
    );
    
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Escrow(engagement_id_copy.clone());
        let engagement: Escrow = env.storage().instance().get(&engagement_key).unwrap();
        let service_provider_balance = token.balance(&service_provider_address);

        assert_eq!(service_provider_balance, engagement.amount as i128);
        assert_eq!(engagement.balance, 0);
    });
}

#[test]
fn test_initialize_escrow_prices_cannot_be_zero() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EngagementContract);
    
    let client = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
    ];
    
    env.as_contract(&contract_id, || {
        let result = EngagementContract::initialize_escrow(
            env.clone(),
            String::from_str(&env, "engagement_1"),
            client,
            service_provider,
            platform_address,
            0, 
            10,
            milestones,
            release_signer,
            dispute_resolver,
        );
        
        assert!(result.is_err());
    });
}

#[test]
fn test_client_can_recover_funds_if_service_provider_does_not_complete_all_escrows() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = (0.3 * 10u128.pow(18) as f64) as u128;
    let token = create_token(&env, &admin1);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

    token.mint(&release_signer_address, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&release_signer_address, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&release_signer_address), 1000);

    let usdc_contract_address = token.address.clone();
    let engagement_id = String::from_str(&env, "41431");
    let description = String::from_str(&env, "Any description");

    let amount: u128 = 100_u128;
    let engagement_id = engagement_client.initialize_escrow(
        &engagement_id.clone(), 
        &client_address, 
        &service_provider_address, 
        &platform_address, 
        &amount, 
        &platform_fee,
        &milestones,
        &release_signer_address,
        &dispute_resolver_address
    );
    let engagement_id_copy = engagement_id.clone();

    engagement_client.fund_escrow(&engagement_id, &release_signer_address, &usdc_contract_address, &engagement_contract_address);
    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Escrow(engagement_id);
        let engagement: Escrow = env.storage().instance().get(&engagement_key).unwrap();
        assert_eq!(engagement.balance, 50);
    });

    engagement_client.cancel_escrow(&engagement_id_copy, &service_provider_address);
    engagement_client.refund_remaining_funds(&engagement_id_copy, &release_signer_address, &usdc_contract_address, &engagement_contract_address);

    env.as_contract(&engagement_contract_address, || {
        let engagement_key = DataKey::Escrow(engagement_id_copy);
        let engagement: Escrow = env.storage().instance().get(&engagement_key).unwrap();
        let contract_balance = token.balance(&engagement_contract_address);
        let signer_balance = token.balance(&release_signer_address);
        assert_eq!(contract_balance, 0);
        assert_eq!(signer_balance, 1000);
        // assert_eq!(engagement.cancelled, true);
        assert_eq!(engagement.balance, 0);
    });
}  

#[test]
fn test_get_engagements_by_service_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let signer_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = (0.3 * 10u128.pow(18) as f64) as u128;
    let token = create_token(&env, &admin1);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
    ];

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

    let amount: u128 = 100_u128;
    let engagement_id = engagement_client.initialize_escrow(
        &engagement_id.clone(), 
        &client_address, 
        &service_provider_address, 
        &platform_address, 
        &amount, 
        &platform_fee,
        &milestones,
        &release_signer_address,
        &dispute_resolver_address
    );

    let escrow = engagement_client.get_escrow_by_id(&engagement_id);
    assert_eq!(escrow.engagement_id, engagement_id);
    assert_eq!(escrow.platform_address, platform_address);
    assert_eq!(escrow.release_signer, release_signer_address);
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


#[test]
fn test_initialize_excrow() {

    let env = Env::default();
    env.mock_all_auths();
    
    let admin1 = Address::generate(&env);
    let client_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: u128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = (0.3 * 10u128.pow(18) as f64) as u128;
    let token = create_token(&env, &admin1);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract); 
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

    token.mint(&release_signer_address, &1_000_000_000);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&release_signer_address, 1_000_000_000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&release_signer_address), 1_000_000_000);

    let engagement_id = String::from_str(&env, "41431");

    let engagement_id = engagement_client.initialize_escrow(
        &engagement_id.clone(), 
        &client_address, 
        &service_provider_address, 
        &platform_address, 
        &amount, 
        &platform_fee,
        &milestones,
        &release_signer_address,
        &dispute_resolver_address
    );

    let escrow = engagement_client.get_escrow_by_id(&engagement_id);
    assert_eq!(escrow.engagement_id, engagement_id);
    assert_eq!(escrow.client, client_address);
    assert_eq!(escrow.service_provider, service_provider_address);
    assert_eq!(escrow.platform_address, platform_address);
    assert_eq!(escrow.amount, amount);
    assert_eq!(escrow.platform_fee, platform_fee);
    assert_eq!(escrow.milestones, milestones);
    assert_eq!(escrow.release_signer, release_signer_address);
    assert_eq!(escrow.dispute_resolver, dispute_resolver_address);


}