use soroban_sdk::{
    contract, contractimpl, Address, Env, String, BytesN, Val, Vec, Symbol
};
use soroban_sdk::token::Client as TokenClient;

use crate::storage_types::{Escrow, DataKey, User};
use crate::error::ContractError;
use crate::events::{
    escrows_by_engagement_id, balance_retrieved_event, allowance_retrieved_event
};

#[contract]
pub struct EngagementContract;

#[contractimpl]
impl EngagementContract {

    pub fn deploy(
        env: Env,
        deployer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> (Address, Val) {
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy(wasm_hash);

        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
        (deployed_address, res)
    }


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
    
    pub fn fund_escrow(e: Env, engagement_id: String, signer: Address, usdc_contract: Address, contract_address: Address) -> Result<(), ContractError> {
        signer.require_auth();

        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(e.clone(), engagement_id);
    
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
        let escrow_result = Self::get_escrow_by_id(e.clone(), engagement_id);
    
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

    pub fn claim_escrow_earnings(e: Env, engagement_id: String, service_provider: Address, usdc_contract: Address, contract_address: Address) -> Result<(), ContractError> {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(e.clone(), engagement_id);

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

    pub fn cancel_escrow(e: Env, engagement_id: String, signer: Address) -> Result<(), ContractError> {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(e.clone(), engagement_id);
    
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        let invoker = signer;
        if invoker != escrow.signer {
            return Err(ContractError::OnlySignerCanCancelEscrow);
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

    pub fn refund_remaining_funds(e: Env, engagement_id: String, signer: Address, usdc_contract: Address, contract_address: Address) -> Result<(), ContractError> {
        signer.require_auth();

        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(e.clone(), engagement_id);
    
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

    pub fn get_escrow_by_id(e: Env, engagement_id: String) -> Result<Escrow, ContractError> {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        if let Some(escrow) = e.storage().instance().get::<DataKey, Escrow>(&escrow_key) {
            escrows_by_engagement_id(&e, engagement_id.clone(), escrow.clone());
            Ok(escrow)
        } else {
            return Err(ContractError::EscrowNotFound)
        }
    }

    pub fn approve_amounts(e: Env, from: Address, spender: Address, amount: i128, usdc_token_address: Address ) {
        from.require_auth();
        let expiration_ledger = e.ledger().sequence() + 1000;
        let usdc_token = TokenClient::new(&e, &usdc_token_address);
        usdc_token.approve(&from, &spender, &amount, &expiration_ledger);
    }

    pub fn get_allowance(e: Env, from: Address, spender: Address, usdc_token_address: Address ) {
        let usdc_token = TokenClient::new(&e, &usdc_token_address);
        let allowance = usdc_token.allowance(&from, &spender);
        allowance_retrieved_event(&e, from, spender, allowance);
    }

    pub fn get_balance(e: Env, address: Address, usdc_token_address: Address) {
        let usdc_token = TokenClient::new(&e, &usdc_token_address);
        let balance = usdc_token.balance(&address);
        balance_retrieved_event(&e, address, usdc_token_address, balance);
    }
      
    pub fn register(e: Env, user_address: Address, name: String, email: String) -> bool {
        user_address.require_auth();

        let key = DataKey::User(user_address.clone());

        if e.storage().persistent().has(&key) {
            return false;
        }

        let user_id = e
            .storage()
            .persistent()
            .get(&DataKey::UserCounter)
            .unwrap_or(0)
            + 1;

        e.storage()
            .persistent()
            .set(&DataKey::UserCounter, &user_id);

        let user = User {
            id: user_id,
            user: user_address.clone(),
            name: name.clone(),
            email: email.clone(),
            registered: true,
            timestamp: e.ledger().timestamp(),
        };

        e.storage()
            .persistent()
            .set(&DataKey::User(user_address.clone()), &user);

        let user_reg_id = e.ledger().sequence();

        e.storage()
            .persistent()
            .set(&DataKey::UserRegId(user_address.clone()), &user_reg_id);

        return true;
    }

    pub fn login(e: Env, user_address: Address) -> String {
        user_address.require_auth();
    
        let key = DataKey::User(user_address.clone());
    
        if let Some(user) = e.storage().persistent().get::<_, User>(&key) {
            user.name
        } else {
            soroban_sdk::String::from_str(&e, "User not found")
        }
    }
}