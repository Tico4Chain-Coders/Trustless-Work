use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, IntoVal, Map, String, TryFromVal, Vec
};

use crate::storage::{get_project, get_all_projects};
use crate::storage_types::{Objective, Project, DataKey, User};
use crate::token::TokenClient;
use crate::events::{project_created, objective_added, objective_completed, objective_funded, project_cancelled, project_completed, project_refunded};

#[contract]
pub struct FreelanceContract;

#[contractimpl]
impl FreelanceContract {

    pub fn create_project(
        e: Env,
        freelancer: Address,
        prices: Vec<u128>,
        user: Address,
    ) -> u128 {
        user.require_auth(); 

        if prices.is_empty() {
            panic!("Prices cannot be empty");
        }

        let contract_key = symbol_short!("pk");
        let mut project_count: u128 = e
            .storage()
            .instance()
            .get(&contract_key)
            .unwrap_or(0);
    
        project_count += 1;
        e.storage().instance().set(&contract_key, &project_count);
        let project_id = Bytes::from_slice(&e, &project_count.to_be_bytes());
        let mut objectives: Map<u128, Objective> = Map::new(&e);
        for (i, price) in prices.iter().enumerate() {
            objectives.set(i as u128, Objective {
                price: price as u128,
                half_paid: 0,
                completed: false,
            });
        }
        let project = Project {
            project_id,
            client: user.clone(),
            freelancer: freelancer.clone(),
            objectives_count: prices.len() as u128,
            objectives,
            completed_objectives: 0,
            earned_amount: 0,
            contract_balance: 0,
            cancelled: false,
            completed: false,
        };
        
        let project_key = DataKey::Project(Bytes::from_slice(&e, &project_count.to_be_bytes()));
        e.storage().instance().set(&project_key, &project);
        project_created(&e, project_key, user.clone(), freelancer.clone(), prices);

        u128::from_be_bytes(project_count.to_be_bytes())
    }

    pub fn complete_project(e: Env, project_id: Bytes, user: Address) {
        let (mut project, project_key) = get_project(&e, project_id);

        let invoker = user;
        if invoker != project.client {
            panic!("Only the client can mark the project as completed");
        }

        if project.completed {
            panic!("Project is completed");
        }

        if project.cancelled {
            panic!("Project is cancelled");
        }

        if project.completed_objectives != project.objectives_count {
            panic!("Not all objectives completed");
        }

        project.completed = true;
        e.storage().instance().set(&project_key, &project);
        project_completed(&e, project_key);

    }
    

    pub fn complete_objective(
        e: Env,
        project_id: Bytes,
        objective_id: u128,
        user: Address,
        usdc_contract: Address,
        freelance_contract_address: Address,
        freelancer_address: Address
    ) {
        user.require_auth();
    
        let project_key = DataKey::Project(project_id);
        let mut project: Project = e.storage().instance().get(&project_key).unwrap();
    
        if freelancer_address != project.freelancer {
            panic!("Only the freelancer can complete objectives");
        }
    
        let mut objective = project.objectives.get(objective_id).unwrap();
    
        if objective.half_paid == 0 {
            panic!("Objective not funded");
        }
    
        if objective.completed {
            panic!("Objective already completed");
        }
    
        let remaining_price = (objective.price - objective.half_paid) as i128;
        let full_price = objective.price;
    
        let usdc_client = TokenClient::new(&e, &usdc_contract);
        usdc_client.transfer(
            &user,              
            &freelance_contract_address,
            &remaining_price
        );

        let expiration_ledger = e.ledger().sequence() + 1000;
        usdc_client.approve(&freelance_contract_address, &freelancer_address, &remaining_price, &expiration_ledger);
        usdc_client.transfer(
            &freelance_contract_address,
            &freelancer_address,
            &(objective.price as i128)
        );
    
        objective.completed = true;
        project.completed_objectives += 1;
        project.earned_amount += objective.price;
    
        project.objectives.set(objective_id, objective);
        e.storage().instance().set(&project_key, &project);
    
        objective_completed(&e, project_key, objective_id, full_price);
    }

    pub fn cancel_project(e: Env, project_id: Bytes, user: Address) {
        user.require_auth();
        let (mut project, project_key) = get_project(&e, project_id);

        let invoker = user;
        if invoker != project.client {
            panic!("Only the client can mark the project as completed");
        }

        if project.completed {
            panic!("Project is completed");
        }

        if project.cancelled {
            panic!("Project is cancelled");
        }

        project.cancelled = true;

        e.storage().instance().set(&project_key, &project);
         project_cancelled(&e, project_key);
    }

    pub fn add_objective(e: Env, project_id: Bytes, prices: Vec<u128>, user: Address) {
        user.require_auth();
        let (mut project, project_key) = get_project(&e, project_id);

        let invoker = user;
        if invoker != project.client {
            panic!("Only the client can add objectives");
        }

        if project.completed {
            panic!("Project is completed");
        }

        if project.cancelled {
            panic!("Project is cancelled");
        }
        
        for (i, price) in prices.iter().enumerate() {
            let objective_id = project.objectives_count + i as u128;

            project.objectives.set(objective_id, Objective {
                price: price,
                half_paid: 0,
                completed: false,
            });

            objective_added(&e, &project_key, objective_id, price);
        }

        project.objectives_count += prices.len() as u128;
        e.storage().instance().set(&project_key, &project);
    }

    pub fn fund_objective(e: Env, project_id: Bytes, objective_id: u128, user: Address, usdc_contract: Address, freelance_contract_address: Address) {
        user.require_auth();
    
        let project_key = DataKey::Project(project_id);
        let mut project: Project = e.storage().instance().get(&project_key).unwrap();
    
        if user != project.client {
            panic!("Only the client can fund objectives");
        }
    
        let mut objective = project.objectives.get(objective_id).unwrap();
        if objective.half_paid > 0 {
            panic!("Objective already funded");
        }
    
        let half_price = (objective.price / 2) as i128;
        let usdc_client = TokenClient::new(&e, &usdc_contract);

        let allowance = usdc_client.allowance(&user, &freelance_contract_address);
        if allowance < half_price {
            panic!("Not enough allowance to fund this objective. Please approve the amount first.");
        }
    
        usdc_client.transfer(
            &user,              
            &freelance_contract_address,
            &half_price       
        );

        usdc_client.approve(&user, &freelance_contract_address, &0, &e.ledger().sequence());
    
        objective.half_paid = half_price as u128;
        project.objectives.set(objective_id, objective);
        e.storage().instance().set(&project_key, &project);
    
        objective_funded(&e, project_key, objective_id, half_price as u128);
    }

    pub fn refund_remaining_funds(e: Env, project_id: Bytes, objective_id: u128, user: Address, usdc_contract: Address, freelance_contract_address: Address) {
        user.require_auth();
        let (project, project_key) = get_project(&e, project_id);

        let invoker = user.clone();
        if invoker != project.client {
            panic!("Only the client can mark the project as completed");
        }

        if !project.cancelled {
            panic!("Project is cancelled");
        }


        let mut refundable_amount : i128 = 0;
        for _i in 0..project.objectives_count {
            let mut objective = project.objectives.get(objective_id).unwrap(); 
            
            if !objective.completed && objective.half_paid > 0 {
                refundable_amount += objective.half_paid as i128;
                objective.half_paid = 0; 
            }
        }
        
        let usdc_client = TokenClient::new(&e, &usdc_contract);
        let contract_balance = usdc_client.balance(&freelance_contract_address);
        if  contract_balance == 0 {
            panic!("The contract has no balance to repay");
        }

        usdc_client.transfer(
            &e.current_contract_address(),
            &project.client,
            &(contract_balance as i128) 
        );

        project_refunded(&e, project_key, user.clone(), refundable_amount as u128);

    }
    
    pub fn get_projects_by_freelancer(e: Env, freelancer: Address) -> Vec<Project> {
        let all_projects: Vec<Project> = get_all_projects(e.clone());
    
        let mut result: Vec<Project> = Vec::new(&e);
    
        for project in all_projects.iter() {
            if project.freelancer == freelancer {
                result.push_back(project);
            }
        }
    
        result
    }

    pub fn get_projects_by_client(e: Env, client: Address) -> Vec<Project> {
        let all_projects: Vec<Project> = get_all_projects(e.clone());
    
        let mut result: Vec<Project> = Vec::new(&e);
    
        for project in all_projects.iter() {
            if project.client == client {
                result.push_back(project);
            }
        }
    
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