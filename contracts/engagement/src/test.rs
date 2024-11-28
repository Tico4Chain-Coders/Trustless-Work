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


#[test]
fn test_change_escrow_properties() {

    let env = Env::default();
    env.mock_all_auths();
    
    let client_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    
    let amount: u128 = 100_000_000;
    let platform_fee = (0.3 * 10u128.pow(18) as f64) as u128;

    let initial_milestones = vec![
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
    let initialized_id = engagement_client.initialize_escrow(
        &engagement_id.clone(), 
        &client_address, 
        &service_provider_address, 
        &platform_address, 
        &amount, 
        &platform_fee,
        &initial_milestones,
        &release_signer_address,
        &dispute_resolver_address
    );

    // Verify escrow was initialized
    let initial_escrow = engagement_client.get_escrow_by_id(&initialized_id);
    assert_eq!(initial_escrow.engagement_id, initialized_id);

    // Create new values for updating the escrow
    let new_client_address = Address::generate(&env);
    let new_service_provider = Address::generate(&env);
    let new_release_signer = Address::generate(&env);
    let new_dispute_resolver = Address::generate(&env);
    let new_amount: u128 = 200_000_000;
    let new_platform_fee = (0.5 * 10u128.pow(18) as f64) as u128;
    
    let new_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Updated first milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Updated second milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
        Milestone {
            description: String::from_str(&env, "New third milestone"),
            status: String::from_str(&env, "Pending"),
            flag: false,
        },
    ];

    // Test unauthorized access (should fail)
    let unauthorized_address = Address::generate(&env);
    env.mock_all_auths();
    let result = engagement_client.try_change_escrow_properties(
        &engagement_id,
        &new_client_address,
        &new_service_provider,
        &unauthorized_address, // Using unauthorized address
        &new_amount,
        &new_platform_fee,
        &new_milestones,
        &new_release_signer,
        &new_dispute_resolver
    );
    assert!(result.is_err());

    // Update escrow with authorized platform_address
    env.mock_all_auths();
    engagement_client.change_escrow_properties(
        &engagement_id,
        &new_client_address,
        &new_service_provider,
        &platform_address, // Using original platform_address
        &new_amount,
        &new_platform_fee,
        &new_milestones,
        &new_release_signer,
        &new_dispute_resolver
    );

    // Verify updated escrow properties
    let updated_escrow = engagement_client.get_escrow_by_id(&engagement_id);
    assert_eq!(updated_escrow.engagement_id, engagement_id);
    assert_eq!(updated_escrow.client, new_client_address);
    assert_eq!(updated_escrow.service_provider, new_service_provider);
    assert_eq!(updated_escrow.platform_address, platform_address);
    assert_eq!(updated_escrow.amount, new_amount);
    assert_eq!(updated_escrow.platform_fee, new_platform_fee);
    assert_eq!(updated_escrow.milestones, new_milestones);
    assert_eq!(updated_escrow.release_signer, new_release_signer);
    assert_eq!(updated_escrow.dispute_resolver, new_dispute_resolver);

    // Test with non-existent escrow (should fail)
    let non_existent_id = String::from_str(&env, "99999");
    env.mock_all_auths();
    let result = engagement_client.try_change_escrow_properties(
        &non_existent_id,
        &new_client_address,
        &new_service_provider,
        &platform_address,
        &new_amount,
        &new_platform_fee,
        &new_milestones,
        &new_release_signer,
        &new_dispute_resolver
    );
    assert!(result.is_err());
}

#[test]
fn test_change_milestone_status_and_flag() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: u128 = 100_000_000;
    let platform_fee = (0.3 * 10u128.pow(18) as f64) as u128;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            flag: false,
        },
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_client = EngagementContractClient::new(&env, &engagement_contract_address);

    let engagement_id = String::from_str(&env, "test_engagement");
    engagement_client.initialize_escrow(
        &engagement_id.clone(),
        &client_address,
        &service_provider_address,
        &platform_address,
        &amount,
        &platform_fee,
        &initial_milestones,
        &release_signer_address,
        &dispute_resolver_address,
    );

    // Change milestone status (valid case)
    let new_status = String::from_str(&env, "completed");
    engagement_client.change_milestone_status(
        &engagement_id.clone(),
        &(0 as i128), // Milestone index
        &new_status,
        &service_provider_address,
    );

    // Verify milestone status change
    let updated_escrow = engagement_client.get_escrow_by_id(&engagement_id);
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, new_status);

    // Change milestone flag (valid case)
    engagement_client.change_milestone_flag(
        &engagement_id,
        &(0 as i128),
        &true,
        &client_address,
    );

    // Verify milestone flag change
    let final_escrow = engagement_client.get_escrow_by_id(&engagement_id);
    assert!(final_escrow.milestones.get(0).unwrap().flag);

    // Invalid index test
    let invalid_index = 10 as i128;
    let new_status = String::from_str(&env, "completed");

    // Test for `change_status` with invalid index
    let result = engagement_client.try_change_milestone_status(
        &engagement_id,
        &invalid_index,
        &new_status,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test for `change_flag` with invalid index
    let result = engagement_client.try_change_milestone_flag(
        &engagement_id,
        &invalid_index,
        &true,
        &client_address,
    );
    assert!(result.is_err());

    // Invalid Engagement ID test
    let invalid_engagement_id = String::from_str(&env, "invalid_engagement");

    // Test for `change_status` with invalid engagement ID
    let result = engagement_client.try_change_milestone_status(
        &invalid_engagement_id,
        &(0 as i128),
        &new_status,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test for `change_flag` with invalid engagement ID
    let result = engagement_client.try_change_milestone_flag(
        &invalid_engagement_id,
        &(0 as i128),
        &true,
        &client_address,
    );
    assert!(result.is_err());

    // Test only authorized party can perform the function
    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let result = engagement_client.try_change_milestone_status(
        &engagement_id,
        &(0 as i128),
        &new_status,
        &unauthorized_address,
    );
    assert!(result.is_err());

    // Test for `change_flag` by invalid client
    let result = engagement_client.try_change_milestone_flag(
        &engagement_id,
        &(0 as i128),
        &true,
        &unauthorized_address,
    );
    assert!(result.is_err());

    //Escrow Test with no milestone
    engagement_client.change_escrow_properties(
        &engagement_id,
        &client_address,
        &service_provider_address,
        &platform_address,
        &amount,
        &platform_fee,
        &vec![&env],
        &release_signer_address,
        &dispute_resolver_address
    );
    // Test for `change_status` on escrow with no milestones
    let result = engagement_client.try_change_milestone_status(
    &engagement_id,
        &(0 as i128),
        &new_status,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test for `change_flag` on escrow with no milestones
    let result = engagement_client.try_change_milestone_flag(
        &engagement_id,
        &(0 as i128),
        &true,
        &client_address,
    );
    assert!(result.is_err());

}

