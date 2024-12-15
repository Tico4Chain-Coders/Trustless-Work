use soroban_sdk::{Address, Env, String};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::{Escrow, DataKey};
use crate::error::ContractError;
use crate::events::escrows_by_engagement_id;

pub struct EscrowManager;

impl EscrowManager {
    pub fn initialize_escrow(
        e: Env,
        engagement_id: String,
        description: String,
        issuer: Address,
        service_provider: Address,
        amount: u128,
        signer: Address,
    ) -> Result<String, ContractError> {
        // if e.storage().instance().has(&DataKey::Admin) {
        //     panic!("An escrow has already been initialized for this contract");
        // }

        if amount == 0 {
            return Err(ContractError::AmountCannotBeZero);
        }

        let engagement_id_copy = engagement_id.clone();
        let escrow = Escrow {
            engagement_id: engagement_id.clone(),
            description,
            issuer,
            signer: signer.clone(),
            service_provider: service_provider.clone(),
            amount,
            balance: 0,
            cancelled: false,
            completed: false,
        };
        
        e.storage().instance().set(&DataKey::Escrow(engagement_id.clone().into()), &escrow);
        // e.storage().instance().set(&DataKey::Admin, &true);

        Ok(engagement_id_copy)
    }
     
    pub fn fund_escrow(
        e: Env, 
        engagement_id: String, 
        signer: Address, 
        usdc_contract: Address, 
        contract_address: Address
    ) -> Result<(), ContractError> {
        signer.require_auth();

        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(&e.clone(), engagement_id);
    
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if escrow.cancelled == true {
            return Err(ContractError::EscrowAlreadyCancelled);
        }

        if escrow.completed == true {
            return Err(ContractError::EscrowAlreadyCompleted);
        }
    
        if signer != escrow.signer {
            return Err(ContractError::OnlySignerCanFundEscrow);
        }
    
        if escrow.balance > 0 {
            return Err(ContractError::EscrowAlreadyFunded);
        }
    
        if escrow.balance == escrow.amount {
            return Err(ContractError::EscrowFullyFunded);
        }
    
        let half_price_in_micro_usdc = (escrow.amount as i128) / 2;
        let usdc_client = TokenClient::new(&e, &usdc_contract);
    
        let signer_balance = usdc_client.balance(&signer);
        if signer_balance < half_price_in_micro_usdc {
            return Err(ContractError::SignerInsufficientFunds);
        }
    
        usdc_client.transfer(&signer, &contract_address, &half_price_in_micro_usdc);
    
        escrow.balance = half_price_in_micro_usdc as u128;
        e.storage().instance().set(&escrow_key, &escrow);
    
        Ok(())
    }

    pub fn complete_escrow(
        e: Env,
        engagement_id: String,
        signer: Address,
        usdc_contract: Address,
        contract_address: Address,
    ) -> Result<(), ContractError> {
        signer.require_auth();
    
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(&e.clone(), engagement_id);
    
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if escrow.cancelled == true {
            return Err(ContractError::EscrowAlreadyCancelled);
        }
    
        if signer != escrow.signer {
            return Err(ContractError::OnlySignerCanCompleteEscrow);
        }
    
        if escrow.balance == 0 {
            return Err(ContractError::EscrowNotFunded);
        }
    
        if escrow.completed {
            return Err(ContractError::EscrowAlreadyCompleted);
        }
    
        let remaining_price = (escrow.amount - escrow.balance) as i128;
    
        let usdc_client = TokenClient::new(&e, &usdc_contract);

        let signer_balance = usdc_client.balance(&escrow.signer);
        if signer_balance < remaining_price {
            return Err(ContractError::SignerInsufficientFunds);
        }

        usdc_client.transfer(
            &signer,              
            &contract_address,
            &remaining_price
        );

        escrow.completed = true;
        escrow.balance = escrow.amount;
    
        e.storage().instance().set(&escrow_key, &escrow);
        Ok(())
    }

    pub fn claim_escrow_earnings(
        e: Env, 
        engagement_id: String, 
        service_provider: Address, 
        usdc_contract: Address, 
        contract_address: Address
    ) -> Result<(), ContractError> {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(&e.clone(), engagement_id);

        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        let invoker = service_provider;
        if invoker != escrow.service_provider {
            return Err(ContractError::OnlyServiceProviderCanClaimEarnings);
        }

        if escrow.cancelled == true {
            return Err(ContractError::EscrowAlreadyCancelled);
        }

        if escrow.completed == false {
            return Err(ContractError::EscrowNotCompleted);
        }

        if escrow.balance != escrow.amount {
            return Err(ContractError::EscrowBalanceNotSufficienteToSendEarnings);
        }

        let usdc_client = TokenClient::new(&e, &usdc_contract);

        let escrow_balance = usdc_client.balance(&contract_address);
        if escrow_balance < escrow.amount as i128 {
            return Err(ContractError::ContractInsufficientFunds);
        }

        usdc_client.transfer(
            &contract_address,
            &escrow.service_provider,
            &(escrow.amount as i128)
        );

        escrow.balance = 0;

        e.storage().instance().set(&escrow_key, &escrow);
        Ok(())
    }

    pub fn cancel_escrow(
        e: Env, 
        engagement_id: String, 
        service_provider:Address
    ) -> Result<(), ContractError> {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(&e.clone(), engagement_id);
    
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        let invoker = service_provider;
        if invoker != escrow.service_provider {
            return Err(ContractError::OnlyServiceProviderCanCancelEscrow);
        }

        if escrow.completed {
            return Err(ContractError::EscrowAlreadyCompleted);
        }

        if escrow.cancelled {
            return Err(ContractError::EscrowAlreadyCancelled);
        }

        escrow.cancelled = true;

        e.storage().instance().set(&escrow_key, &escrow);
        Ok(())
    }

    pub fn refund_remaining_funds(
        e: Env, 
        engagement_id: String, 
        signer: Address, 
        usdc_contract: Address, 
        contract_address: Address
    ) -> Result<(), ContractError> {
        signer.require_auth();

        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(&e.clone(), engagement_id);
    
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
        
        let invoker = signer.clone();
        if invoker != escrow.signer {
            return Err(ContractError::OnlySignerCanRequestRefund);
        }
        if !escrow.cancelled {
            return Err(ContractError::EscrowNotCancelled);
        }

        if escrow.completed {
            return Err(ContractError::EscrowAlreadyCompleted);
        }

        let usdc_client = TokenClient::new(&e, &usdc_contract);
        let contract_balance = usdc_client.balance(&contract_address);

        if  contract_balance == 0 {
            return Err(ContractError::ContractHasInsufficientBalance);
        }

        usdc_client.transfer(
            &e.current_contract_address(),
            &escrow.signer,
            &contract_balance
        );

        escrow.balance = 0;
        e.storage().instance().set(&escrow_key, &escrow);

        Ok(())
    }

    pub fn get_escrow_by_id(e: &Env, engagement_id: String) -> Result<Escrow, ContractError> {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        if let Some(escrow) = e.storage().instance().get::<DataKey, Escrow>(&escrow_key) {
            escrows_by_engagement_id(&e, engagement_id.clone(), escrow.clone());
            Ok(escrow)
        } else {
            return Err(ContractError::EscrowNotFound)
        }
    }
}