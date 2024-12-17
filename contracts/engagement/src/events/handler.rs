use soroban_sdk::{Env, vec, IntoVal, Val, Address, String, symbol_short};
use crate::storage::types::Escrow;

// ------ Escrows
pub fn escrows_by_engagement_id(e: &Env, engagement_id: String, escrow: Escrow) {
    let topics = (symbol_short!("p_by_spdr"),);
    
    let engagement_id_val: Val = engagement_id.into_val(e);
    let escrow_val: Val = escrow.into_val(e);

    let event_payload = vec![e, engagement_id_val, escrow_val];
    e.events().publish(topics, event_payload);
}

// ------ Token

pub fn balance_retrieved_event(e: &Env, address: Address, usdc_token_address: Address, balance: i128) {
    let topics = (symbol_short!("blnc"),);
    let address_val: Val = address.into_val(e);
    let token_address_val: Val = usdc_token_address.into_val(e);
    let balance_val: Val = balance.into_val(e);

    let event_payload = vec![e, address_val, token_address_val, balance_val];
    e.events().publish(topics, event_payload);
}