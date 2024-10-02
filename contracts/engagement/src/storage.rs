use soroban_sdk::{contracttype, Env, symbol_short, Vec, Bytes};

use crate::storage_types::{ Engagement, DataKey };
use crate::utils::u128_to_bytes;

#[derive(Clone)]
#[contracttype]

enum DataKeyAddress {
    Initialized,
    TotalAddress,
    Shares(u32),
    Addresses(u32),
}

pub fn get_engagement(e: &Env, engagement_id: Bytes) -> (Engagement, DataKey) {
    let engagement_key = DataKey::Engagement(engagement_id);
    let engagement: Engagement = e.storage().instance().get(&engagement_key).unwrap();
    (engagement, engagement_key)
}

pub fn get_all_engagements(e: Env) -> Vec<Engagement> {
    let engagement_count: u128 = e
        .storage()
        .instance()
        .get(&symbol_short!("pk"))
        .unwrap_or(0);

    let mut engagements: Vec<Engagement> = Vec::new(&e);

    for id in 1..=engagement_count {
        let engagement_id_in_bytes = u128_to_bytes(&e, id);
        let engagement_key = DataKey::Engagement(engagement_id_in_bytes);
        if let Some(engagement) = e.storage().instance().get(&engagement_key) {
            engagements.push_back(engagement);
        }
    }

    engagements
}