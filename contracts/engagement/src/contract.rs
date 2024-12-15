use soroban_sdk::{
    contract, contractimpl, Address, Env, String, BytesN, Val, Vec, Symbol
};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::{Escrow};
use crate::error::ContractError;
use crate::events::{balance_retrieved_event, allowance_retrieved_event};

use crate::core::{EscrowManager, UserManager};

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
        EscrowManager::initialize_escrow(
            e, 
            engagement_id, 
            description, 
            issuer, 
            service_provider, 
            amount, 
            signer
        )
    }

    pub fn fund_escrow(
        e: Env, 
        engagement_id: String, 
        signer: Address, 
        usdc_contract: Address, 
        contract_address: Address
    ) -> Result<(), ContractError> {
        EscrowManager::fund_escrow(
            e, 
            engagement_id, 
            signer, 
            usdc_contract, 
            contract_address
        )
    }
  
    pub fn claim_escrow_earnings(
        e: Env, 
        engagement_id: String, 
        service_provider: Address, 
        usdc_contract: Address, 
        contract_address: Address
    ) -> Result<(), ContractError> {
        EscrowManager::claim_escrow_earnings(
            e, 
            engagement_id, 
            service_provider, 
            usdc_contract, 
            contract_address
        )
    }

    pub fn cancel_escrow(
        e: Env, 
        engagement_id: String, 
        service_provider:Address
    ) -> Result<(), ContractError> {
        EscrowManager::cancel_escrow(
            e, 
            engagement_id, 
            service_provider
        )
    }

    pub fn refund_remaining_funds(
        e: Env, 
        engagement_id: String, 
        signer: Address, 
        usdc_contract: Address, 
        contract_address: Address
    ) -> Result<(), ContractError> {
        EscrowManager::refund_remaining_funds(
            e, 
            engagement_id, 
            signer, 
            usdc_contract, 
            contract_address
        )
    }

    pub fn complete_escrow(
        e: Env,
        engagement_id: String,
        signer: Address,
        usdc_contract: Address,
        contract_address: Address,
    ) -> Result<(), ContractError> {
        EscrowManager::complete_escrow(
            e,
            engagement_id,
            signer,
            usdc_contract,
            contract_address
        )
    }

    pub fn get_escrow_by_id(e: Env, engagement_id: String) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow_by_id(&e, engagement_id)
    }

    pub fn register(e: Env, user_address: Address, name: String, email: String) -> bool {
        UserManager::register(e, user_address, name, email)
    }

    pub fn login(e: Env, user_address: Address) -> String {
        UserManager::login(&e, user_address)
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
}