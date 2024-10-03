use soroban_sdk::{Env, vec, IntoVal, Val, Address, String, symbol_short};
use crate::storage_types::{DataKey, Escrow};

// ------ Escrows

pub (crate) fn escrow_created(e: &Env, escrow_id: String, client: Address, service_provider: Address, prices: u128) {
    let topics = (symbol_short!("p_created"),);

    let escrow_id_val: Val = escrow_id.into_val(e);
    let client_val: Val = client.into_val(e);
    let service_provider_val: Val = service_provider.into_val(e);
    let prices_val: Val = prices.into_val(e);

    let event_payload = vec![e, escrow_id_val, client_val, service_provider_val, prices_val];
    e.events().publish(topics, event_payload);
}

pub (crate) fn escrow_cancelled(e: &Env, escrow_id: DataKey) {
    let topics = (symbol_short!("p_cd"),); // cd -> cancelled

    let escrow_id_val: Val = escrow_id.into_val(e);
    e.events().publish(topics, escrow_id_val);
}

pub (crate) fn escrow_refunded(e: &Env, engagement_id: DataKey, client: Address, price: u128) {
    let topics = (symbol_short!("p_rd"),); // rd -> refunded

    let engagement_id_val: Val = engagement_id.into_val(e);
    let client_val: Val = client.into_val(e);
    let price_val: Val = price.into_val(e);

    let event_payload = vec![e, engagement_id_val, client_val, price_val];

    e.events().publish(topics, event_payload);
}

pub (crate) fn escrows_by_engagement_id(e: &Env, engagement_id: String, escrow: Escrow) {
    let topics = (symbol_short!("p_by_spdr"),);
    
    let engagement_id_val: Val = engagement_id.into_val(e);
    let escrow_val: Val = escrow.into_val(e);

    let event_payload = vec![e, engagement_id_val, escrow_val];
    e.events().publish(topics, event_payload);
}


pub (crate) fn escrow_funded(e: &Env, engagement_id: String, half_price: u128) {
    let topics = (symbol_short!("ob_funded"),);

    let escrow_id_val: Val = engagement_id.into_val(e);
    let half_price_val: Val = half_price.into_val(e);

    let event_payload = vec![e, escrow_id_val, escrow_id_val, half_price_val];
    e.events().publish(topics, event_payload);
}

pub (crate) fn escrow_completed(e: &Env, escrow_id: String, full_price: u128) {
    let topics = (symbol_short!("ob_c"),); // c -> completed

    let escrow_id_val: Val = escrow_id.into_val(e);
    let full_price_val: Val = full_price.into_val(e);

    let event_payload = vec![e, escrow_id_val, escrow_id_val, full_price_val];
    e.events().publish(topics, event_payload);
}

// ------ Token

pub (crate) fn balance_retrieved_event(e: &Env, address: Address, usdc_token_address: Address, balance: i128) {
    let topics = (symbol_short!("blnc"),);
    let address_val: Val = address.into_val(e);
    let token_address_val: Val = usdc_token_address.into_val(e);
    let balance_val: Val = balance.into_val(e);

    let event_payload = vec![e, address_val, token_address_val, balance_val];
    e.events().publish(topics, event_payload);
}

pub (crate) fn allowance_retrieved_event(e: &Env, from: Address, spender: Address, balance: i128) {
    let topics = (symbol_short!("blnc"),);
    let from_val: Val = from.into_val(e);
    let spender_address_val: Val = spender.into_val(e);
    let balance_val: Val = balance.into_val(e);

    let event_payload = vec![e, from_val, spender_address_val, balance_val];
    e.events().publish(topics, event_payload);
}