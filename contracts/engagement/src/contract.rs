use soroban_sdk::{
    contract, contractimpl, Address, Env, String
};
use soroban_sdk::token::Client as TokenClient;

use crate::storage_types::{Escrow, DataKey, User};
use crate::events::{
    escrow_created, escrow_completed, escrow_funded, escrow_cancelled,
    escrow_refunded, escrows_by_engagement_id, balance_retrieved_event, allowance_retrieved_event
};

#[contract]
pub struct EngagementContract;

#[contractimpl]
impl EngagementContract {
    pub fn initialize_escrow(
        e: Env,
        engagement_id: String,
        issuer: Address,
        service_provider: Address,
        amount: u128,
        signer: Address,
    ) -> String {
        signer.require_auth(); 

        if amount == 0 {
            panic!("Prices cannot be zero");
        }

        let engagement_id_copy = engagement_id.clone();
        let escrow = Escrow {
            engagement_id: engagement_id.clone(),
            issuer: issuer,
            signer: signer.clone(),
            service_provider: service_provider.clone(),
            amount: amount,
            balance: 0,
            cancelled: false,
            completed: false,
        };
        
        e.storage().instance().set(&DataKey::Escrow(engagement_id.clone().into()), &escrow);
        escrow_created(&e, engagement_id, signer.clone(), service_provider.clone(), amount);

        engagement_id_copy
    }
    
    pub fn fund_escrow(e: Env, engagement_id: String, signer: Address, usdc_contract: Address, contract_address: Address) {
        signer.require_auth();
    
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let mut escrow: Escrow = e.storage().instance().get(&escrow_key).unwrap();
    
        if signer != escrow.signer {
            panic!("Only the signer can fund escrows");
        }

        if escrow.balance > 0 {
            panic!("Escrow already funded");
        }

        if escrow.balance == escrow.amount {
            panic!("This escrow has already been fully funded.");
        }

        let half_price = (escrow.amount / 2) as i128;
        let usdc_client = TokenClient::new(&e, &usdc_contract);

        usdc_client.approve(&signer, &contract_address, &half_price, &e.ledger().sequence());
        
        let allowance = usdc_client.allowance(&signer, &contract_address);

        if allowance < half_price {
            panic!("Not enough allowance to fund this escrow. Please approve the amount first.");
        }

        usdc_client.transfer(&signer, &contract_address, &half_price);

        escrow.balance = half_price as u128;
        e.storage().instance().set(&escrow_key, &escrow);
        escrow_funded(&e, engagement_id, half_price as u128);
    }

    pub fn complete_escrow(
        e: Env,
        engagement_id: String,
        signer: Address,
        usdc_contract: Address,
        contract_address: Address,
        service_provider: Address
    ) {
        signer.require_auth();
    
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let mut escrow: Escrow = e.storage().instance().get(&escrow_key).unwrap();
    
        if signer != escrow.signer {
            panic!("Only the signer can complete the escrow");
        }
    
        if escrow.balance == 0 {
            panic!("Escrow not funded");
        }
    
        if escrow.completed {
            panic!("Escrow already completed");
        }
    
        let remaining_price = (escrow.amount - escrow.balance) as i128;
        let full_price = escrow.amount;
    
        let usdc_client = TokenClient::new(&e, &usdc_contract);
        let expiration_ledger = e.ledger().sequence() + 1000;

        usdc_client.approve(&signer, &contract_address, &remaining_price, &expiration_ledger);
        usdc_client.transfer(
            &signer,              
            &contract_address,
            &remaining_price
        );

        usdc_client.approve(&contract_address, &service_provider, &(escrow.amount as i128), &expiration_ledger);
        usdc_client.transfer(
            &contract_address,
            &service_provider,
            &(escrow.amount as i128)
        );

        escrow.completed = true;
        escrow.balance = escrow.amount;
    
        e.storage().instance().set(&escrow_key, &escrow);
        escrow_completed(&e, engagement_id, full_price);
    }

    pub fn cancel_escrow(e: Env, engagement_id: String, signer: Address) {
        signer.require_auth();
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let mut escrow: Escrow = e.storage().instance().get(&escrow_key).unwrap();

        let invoker = signer;
        if invoker != escrow.signer {
            panic!("Only the signer can mark the escrow as completed");
        }

        if escrow.completed {
            panic!("The escrow is completed");
        }

        if escrow.cancelled {
            panic!("The escrow is cancelled");
        }

        escrow.cancelled = true;

        e.storage().instance().set(&escrow_key, &escrow);
        escrow_cancelled(&e, escrow_key);
    }

    pub fn refund_remaining_funds(e: Env, engagement_id: String, signer: Address, usdc_contract: Address, contract_address: Address) {
        signer.require_auth();

        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow: Escrow = e.storage().instance().get(&escrow_key).unwrap();
        
        let invoker = signer.clone();
        if invoker != escrow.signer {
            panic!("Only the client can mark the engagement as completed");
        }
        if escrow.cancelled {
            panic!("Engagement is cancelled");
        }

        let usdc_client = TokenClient::new(&e, &usdc_contract);
        let contract_balance = usdc_client.balance(&contract_address);

        if  contract_balance == 0 {
            panic!("The contract has no balance to repay");
        }

        usdc_client.approve(&signer, &contract_address, &contract_balance, &e.ledger().sequence());
        usdc_client.transfer(
            &e.current_contract_address(),
            &escrow.signer,
            &(contract_balance as i128) 
        );

        escrow_refunded(&e, escrow_key, signer.clone(), contract_balance as u128);
    }

    pub fn get_escrow_by_id(e: Env, engagement_id: String) -> Escrow {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow: Escrow = e.storage().instance().get(&escrow_key).unwrap();
        
        escrows_by_engagement_id(&e, engagement_id.clone(), escrow.clone());
        escrow
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