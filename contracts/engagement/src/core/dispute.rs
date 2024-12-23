use soroban_sdk::{Address, Env, String};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::DataKey;
use crate::error::ContractError;
use crate::events::escrows_by_engagement_id;
use crate::core::escrow::EscrowManager;

pub struct DisputeManager;

impl DisputeManager {
    
    pub fn resolving_disputes(
        e: Env,
        engagement_id: String,
        dispute_resolver: Address,
        usdc_contract: Address,
        client_funds: i128,
        service_provider_funds: i128
    ) -> Result<(), ContractError> {
        dispute_resolver.require_auth();
    
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = EscrowManager::get_escrow_by_id(e.clone(), engagement_id.clone());
    
        let escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
    
        if dispute_resolver != escrow.dispute_resolver {
            return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
        }
    
        if !escrow.dispute_flag {
            return Err(ContractError::EscrowNotInDispute);
        }
 
        let usdc_client = TokenClient::new(&e, &usdc_contract);
        let contract_balance = usdc_client.balance(&e.current_contract_address());

        let total_funds = client_funds + service_provider_funds;
        if total_funds > contract_balance {
            return Err(ContractError::InsufficientFundsForResolution);
        }
    
        if client_funds > 0 {
            usdc_client.transfer(
                &e.current_contract_address(),
                &escrow.client,
                &(client_funds as i128)
            );
        }

        if service_provider_funds > 0 {
            usdc_client.transfer(
                &e.current_contract_address(),
                &escrow.service_provider,
                &(service_provider_funds as i128)
            );
        }
    
        e.storage().instance().set(&escrow_key, &escrow);
    
        escrows_by_engagement_id(&e, engagement_id, escrow);
    
        Ok(())
    }

    pub fn change_dispute_flag(
        e: Env, 
        engagement_id: String,
    ) -> Result<(), ContractError> {
    
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = EscrowManager::get_escrow_by_id(e.clone(), engagement_id.clone());
    
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
    
        if escrow.dispute_flag {
            return Err(ContractError::EscrowAlreadyInDispute);
        }
    
        escrow.dispute_flag = true;
        e.storage().instance().set(&escrow_key, &escrow);
    
        escrows_by_engagement_id(&e, engagement_id, escrow);
    
        Ok(())
    }
}