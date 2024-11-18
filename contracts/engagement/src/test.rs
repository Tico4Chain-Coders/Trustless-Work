#![cfg(test)]

extern crate std;

use crate::storage_types::Milestone;
use crate::{contract::EngagementContract, EngagementContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String, vec};

#[test]
fn test_initialize_excrow() {

    let env = Env::default();
    env.mock_all_auths();
    
    let client_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: u128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = (0.3 * 10u128.pow(18) as f64) as u128;

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