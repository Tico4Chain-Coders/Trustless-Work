use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, Env, Map, String, Vec
};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::{get_engagement, get_all_engagements};
use crate::storage_types::{Escrow, Engagement, DataKey, User};
use crate::events::{
    engagement_created, escrow_added, escrow_completed, escrow_funded, engagement_cancelled, engagement_completed, 
    engagement_refunded, engagements_by_address, balance_retrieved_event, allowance_retrieved_event
};

#[contract]
pub struct EngagementContract;

#[contractimpl]
impl EngagementContract {
    pub fn initialize_escrow(
        e: Env,
        service_provider: Address,
        prices: Vec<u128>,
        client: Address,
    ) -> u128 {
        client.require_auth(); 

        if prices.is_empty() {
            panic!("Prices cannot be empty");
        }

        let contract_key = symbol_short!("pk");
        let mut engagement_count: u128 = e
            .storage()
            .instance()
            .get(&contract_key)
            .unwrap_or(0);
    
        engagement_count += 1;
        e.storage().instance().set(&contract_key, &engagement_count);
        let engagement_id = Bytes::from_slice(&e, &engagement_count.to_be_bytes());
        let mut parties: Map<u128, Escrow> = Map::new(&e);
        for (i, price) in prices.iter().enumerate() {
            parties.set(i as u128, Escrow {
                price: price as u128,
                amount_paid: 0,
                completed: false,
            });
        }
        let engagement = Engagement {
            engagement_id,
            client: client.clone(),
            service_provider: service_provider.clone(),
            parties_count: prices.len() as u128,
            parties,
            completed_parties: 0,
            earned_amount: 0,
            contract_balance: 0,
            cancelled: false,
            completed: false,
        };
        
        let engagement_key = DataKey::Engagement(Bytes::from_slice(&e, &engagement_count.to_be_bytes()));
        e.storage().instance().set(&engagement_key, &engagement);
        engagement_created(&e, engagement_key, client.clone(), service_provider.clone(), prices);

        u128::from_be_bytes(engagement_count.to_be_bytes())
    }

    pub fn complete_engagement(e: Env, engagement_id: Bytes, client: Address) {
        let (mut engagement, engagement_key) = get_engagement(&e, engagement_id);

        let invoker = client;
        if invoker != engagement.client {
            panic!("Only the client can mark the engagement as completed");
        }

        if engagement.completed {
            panic!("Engagement is completed");
        }

        if engagement.cancelled {
            panic!("Engagement is cancelled");
        }

        if engagement.completed_parties != engagement.parties_count {
            panic!("Not all escrows completed");
        }

        engagement.completed = true;
        e.storage().instance().set(&engagement_key, &engagement);
        engagement_completed(&e, engagement_key);
    }
    

    pub fn complete_escrow(
        e: Env,
        engagement_id: Bytes,
        escrow_id: u128,
        client: Address,
        usdc_contract: Address,
        contract_address: Address,
        service_provider: Address
    ) {
        client.require_auth();
    
        let engagement_key = DataKey::Engagement(engagement_id);
        let mut engagement: Engagement = e.storage().instance().get(&engagement_key).unwrap();
    
        if service_provider != engagement.service_provider {
            panic!("Only the service provider can complete escrows");
        }
    
        let mut escrow = engagement.parties.get(escrow_id).unwrap();
    
        if escrow.amount_paid == 0 {
            panic!("Escrow not funded");
        }
    
        if escrow.completed {
            panic!("Escrow already completed");
        }
    
        let remaining_price = (escrow.price - escrow.amount_paid) as i128;
        let full_price = escrow.price;
    
        let usdc_client = TokenClient::new(&e, &usdc_contract);
        usdc_client.transfer(
            &client,              
            &contract_address,
            &remaining_price
        );

        let expiration_ledger = e.ledger().sequence() + 1000;
        usdc_client.approve(&contract_address, &service_provider, &remaining_price, &expiration_ledger);
        usdc_client.transfer(
            &contract_address,
            &service_provider,
            &(escrow.price as i128)
        );
    
        escrow.completed = true;
        engagement.completed_parties += 1;
        engagement.earned_amount += escrow.price;
    
        engagement.parties.set(escrow_id, escrow);
        e.storage().instance().set(&engagement_key, &engagement);
    
        escrow_completed(&e, engagement_key, escrow_id, full_price);
    }

    pub fn cancel_engagement(e: Env, engagement_id: Bytes, client: Address) {
        client.require_auth();
        let (mut engagement, engagement_key) = get_engagement(&e, engagement_id);

        let invoker = client;
        if invoker != engagement.client {
            panic!("Only the client can mark the engagement as completed");
        }

        if engagement.completed {
            panic!("Engagement is completed");
        }

        if engagement.cancelled {
            panic!("Engagement is cancelled");
        }

        engagement.cancelled = true;

        e.storage().instance().set(&engagement_key, &engagement);
        engagement_cancelled(&e, engagement_key);
    }

    pub fn add_escrow(e: Env, engagement_id: Bytes, prices: Vec<u128>, client: Address) {
        client.require_auth();
        let (mut engagement, engagement_key) = get_engagement(&e, engagement_id);

        let invoker = client;
        if invoker != engagement.client {
            panic!("Only the client can add escrows");
        }

        if engagement.completed {
            panic!("Engagement is completed");
        }

        if engagement.cancelled {
            panic!("Engagement is cancelled");
        }
        
        for (i, price) in prices.iter().enumerate() {
            let escrow_id = engagement.parties_count + i as u128;

            engagement.parties.set(escrow_id, Escrow {
                price: price,
                amount_paid: 0,
                completed: false,
            });

            escrow_added(&e, &engagement_key, escrow_id, price);
        }

        engagement.parties_count += prices.len() as u128;
        e.storage().instance().set(&engagement_key, &engagement);
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

    pub fn fund_escrow(e: Env, engagement_id: Bytes, escrow_id: u128, client: Address, usdc_contract: Address, contract_address: Address) {
        client.require_auth();
    
        let engagement_key = DataKey::Engagement(engagement_id);
        let mut engagement: Engagement = e.storage().instance().get(&engagement_key).unwrap();
    
        if client != engagement.client {
            panic!("Only the client can fund escrows");
        }
    
        let mut escrow = engagement.parties.get(escrow_id).unwrap();
        if escrow.amount_paid > 0 {
            panic!("Escrow already funded");
        }
    
        let half_price = (escrow.price / 2) as i128;
        let usdc_client = TokenClient::new(&e, &usdc_contract);

        let allowance = usdc_client.allowance(&client, &contract_address);

        if allowance < half_price {
            panic!("Not enough allowance to fund this escrow. Please approve the amount first.");
        }
    
        usdc_client.transfer(
            &client,              
            &contract_address,
            &half_price       
        );

        usdc_client.approve(&client, &contract_address, &0, &e.ledger().sequence());
    
        escrow.amount_paid = half_price as u128;
        engagement.parties.set(escrow_id, escrow);
        e.storage().instance().set(&engagement_key, &engagement);
    
        escrow_funded(&e, engagement_key, escrow_id, half_price as u128);
    }

    pub fn refund_remaining_funds(e: Env, engagement_id: Bytes, escrow_id: u128, client: Address, usdc_contract: Address, contract_address: Address) {
        client.require_auth();
        let (engagement, engagement_key) = get_engagement(&e, engagement_id);

        let invoker = client.clone();
        if invoker != engagement.client {
            panic!("Only the client can mark the engagement as completed");
        }

        if !engagement.cancelled {
            panic!("Engagement is cancelled");
        }


        let mut refundable_amount : i128 = 0;
        for _i in 0..engagement.parties_count {
            let mut escrow = engagement.parties.get(escrow_id).unwrap(); 
            
            if !escrow.completed && escrow.amount_paid > 0 {
                refundable_amount += escrow.amount_paid as i128;
                escrow.amount_paid = 0; 
            }
        }
        
        let usdc_client = TokenClient::new(&e, &usdc_contract);
        let contract_balance = usdc_client.balance(&contract_address);
        if  contract_balance == 0 {
            panic!("The contract has no balance to repay");
        }

        usdc_client.transfer(
            &e.current_contract_address(),
            &engagement.client,
            &(contract_balance as i128) 
        );

        engagement_refunded(&e, engagement_key, client.clone(), refundable_amount as u128);

    }
    
    pub fn get_engagements_by_provider(e: Env, service_provider: Address, page: u32, limit: u32) -> Vec<Engagement> {
        let all_engagements: Vec<Engagement> = get_all_engagements(e.clone());
    
        let mut result: Vec<Engagement> = Vec::new(&e);

        let start = (page * limit) as usize;
        let end = start + limit as usize;

        for (i, engagement) in all_engagements.iter().enumerate() {
            if i >= start && i < end && engagement.service_provider == service_provider {
                result.push_back(engagement);
            }
        }
        engagements_by_address(&e, service_provider, result.clone());
        result
    }

    pub fn get_engagements_by_client(e: Env, client: Address, page: u32, limit: u32) -> Vec<Engagement> {
        let all_engagements: Vec<Engagement> = get_all_engagements(e.clone());

        let mut result: Vec<Engagement> = Vec::new(&e);

        let start = (page * limit) as usize;
        let end = start + limit as usize;

        for (i, engagement) in all_engagements.iter().enumerate() {
            if i >= start && i < end && engagement.client == client {
                result.push_back(engagement);
            }
        }
    
        engagements_by_address(&e, client, result.clone());
        result
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