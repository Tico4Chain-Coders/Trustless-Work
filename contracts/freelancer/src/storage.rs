use soroban_sdk::{contracttype, Env, symbol_short, Vec, Bytes};

use crate::storage_types::{ Escrow, DataKey };
use crate::utils::u128_to_bytes;

#[derive(Clone)]
#[contracttype]

enum DataKeyAddress {
    Initialized,
    TotalAddress,
    Shares(u32),
    Addresses(u32),
}

pub fn get_escrow(e: &Env, escrow_id: Bytes) -> (Escrow, DataKey) {
    let escrow_key = DataKey::Escrow(escrow_id);
    let escrow: Escrow = e.storage().instance().get(&escrow_key).unwrap();
    (escrow, escrow_key)
}

pub fn get_all_escrows(e: Env) -> Vec<Escrow> {
    let escrow_count: u128 = e
        .storage()
        .instance()
        .get(&symbol_short!("pk"))
        .unwrap_or(0);

    let mut escrows: Vec<Escrow> = Vec::new(&e);

    for id in 1..=escrow_count {
        let escrow_id_in_bytes = u128_to_bytes(&e, id);
        let escrow_key = DataKey::Escrow(escrow_id_in_bytes);
        if let Some(escrow) = e.storage().instance().get(&escrow_key) {
            escrows.push_back(escrow);
        }
    }

    escrows
}