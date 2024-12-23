use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, String, Symbol, Val, Vec
};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::{Escrow, Milestone};
use crate::error::ContractError;
use crate::events::{balance_retrieved_event};
use crate::core::{EscrowManager, MilestoneManager, DisputeManager, UserManager};

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

    ////////////////////////
    // Escrow /////
    ////////////////////////

    pub fn initialize_escrow(
        e: Env,
        engagement_id: String,
        client: Address,
        service_provider: Address,
        platform_address: Address,
        amount: i128,
        platform_fee: i128,
        milestones: Vec<Milestone>,
        release_signer: Address,
        dispute_resolver: Address,
    ) -> Result<String, ContractError> {
        EscrowManager::initialize_escrow(
            e, 
            engagement_id, 
            client, 
            service_provider, 
            platform_address, 
            amount, 
            platform_fee, 
            milestones, 
            release_signer, 
            dispute_resolver
        )
    }
    
    pub fn fund_escrow(
        e: Env, 
        engagement_id: String, 
        signer: Address, 
        usdc_contract: Address, 
        amount_to_deposit: i128
    ) -> Result<(), ContractError> {
        EscrowManager::fund_escrow(
            e, 
            engagement_id, 
            signer, 
            usdc_contract, 
            amount_to_deposit
        )
    }

    pub fn distribute_escrow_earnings(
        e: Env, 
        engagement_id: String, 
        release_signer: Address, 
        usdc_contract: Address,
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        EscrowManager::distribute_escrow_earnings(
            e, 
            engagement_id, 
            release_signer, 
            usdc_contract,
            trustless_work_address
        )
    }

    pub fn change_escrow_properties(
        e: Env,
        engagement_id: String,
        client: Address,
        service_provider: Address,
        platform_address: Address,
        amount: i128,
        platform_fee: i128,
        milestones: Vec<Milestone>,
        release_signer: Address,
        dispute_resolver: Address,
    ) -> Result<(), ContractError> {
        EscrowManager::change_escrow_properties(
            e,
            engagement_id,
            client,
            service_provider,
            platform_address,
            amount,
            platform_fee,
            milestones,
            release_signer,
            dispute_resolver
        )
    }

    pub fn get_escrow_by_id(e: Env, engagement_id: String) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow_by_id(e, engagement_id)
    }

    ////////////////////////
    // Milestones /////
    ////////////////////////

    pub fn change_milestone_status(
        e: Env,
        engagement_id: String,
        milestone_index: i128,
        new_status: String,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_status(
            e,
            engagement_id,
            milestone_index,
            new_status,
            service_provider
        )
    }
    
    pub fn change_milestone_flag(
        e: Env,
        engagement_id: String,
        milestone_index: i128,
        new_flag: bool,
        client: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_flag(
            e,
            engagement_id,
            milestone_index,
            new_flag,
            client
        )
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolving_disputes(
        e: Env,
        engagement_id: String,
        dispute_resolver: Address,
        usdc_contract: Address,
        client_funds: i128,
        service_provider_funds: i128
    ) -> Result<(), ContractError> {
        DisputeManager::resolving_disputes(
            e,
            engagement_id,
            dispute_resolver,
            usdc_contract,
            client_funds,
            service_provider_funds
        )
    }
    
    pub fn change_dispute_flag(
        e: Env, 
        engagement_id: String,
    ) -> Result<(), ContractError> {
        DisputeManager::change_dispute_flag(e, engagement_id)
    }

    ////////////////////////
    // User /////
    ////////////////////////

    pub fn register_user(
        e: Env, 
        user_address: Address, 
        name: String, 
        email: String
    ) -> bool {
        UserManager::register(e, user_address, name, email)
    }

    pub fn login(e: Env, user_address: Address) -> String {
        UserManager::login(&e, user_address)
    }

    pub fn get_balance(e: Env, address: Address, usdc_token_address: Address) {
        let usdc_token = TokenClient::new(&e, &usdc_token_address);
        let balance = usdc_token.balance(&address);
        balance_retrieved_event(&e, address, usdc_token_address, balance);
    }
}