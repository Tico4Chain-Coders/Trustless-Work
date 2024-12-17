
use soroban_sdk::{Address, Env, String};
use crate::storage::types::{DataKey, User};

pub struct UserManager;

impl UserManager {
    pub fn register(
        e: Env,
        user_address: Address,
        name: String,
        email: String
    ) -> bool {
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
    
        true
    }
    
    pub fn login(e: &Env, user_address: Address) -> String {
        user_address.require_auth();
    
        let key = DataKey::User(user_address.clone());
    
        if let Some(user) = e.storage().persistent().get::<_, User>(&key) {
            user.name
        } else {
            String::from_str(&e, "User not found")
        }
    }
    
}