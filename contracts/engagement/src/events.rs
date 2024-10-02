use soroban_sdk::{Env, vec, IntoVal, Val, Address, Vec, symbol_short};
use crate::storage_types::{DataKey, Engagement};

pub (crate) fn engagement_created(e: &Env, engagement_id: DataKey, client: Address, service_provider: Address, prices: Vec<u128>) {
    let topics = (symbol_short!("p_created"),);

    let engagement_id_val: Val = engagement_id.into_val(e);
    let client_val: Val = client.into_val(e);
    let service_provider_val: Val = service_provider.into_val(e);
    let prices_val: Val = prices.into_val(e);

    let event_payload = vec![e, engagement_id_val, client_val, service_provider_val, prices_val];
    e.events().publish(topics, event_payload);
}

pub (crate) fn engagement_completed(e: &Env, engagement_id: DataKey) {
    let topics = (symbol_short!("p_c"),); // c -> completed

    let engagement_id_val: Val = engagement_id.into_val(e);
    e.events().publish(topics, engagement_id_val);
}

pub (crate) fn engagement_cancelled(e: &Env, engagement_id: DataKey) {
    let topics = (symbol_short!("p_cd"),); // cd -> cancelled

    let engagement_id_val: Val = engagement_id.into_val(e);
    e.events().publish(topics, engagement_id_val);
}

pub (crate) fn engagement_refunded(e: &Env, engagement_id: DataKey, client: Address, price: u128) {
    let topics = (symbol_short!("p_rd"),); // rd -> refunded

    let engagement_id_val: Val = engagement_id.into_val(e);
    let client_val: Val = client.into_val(e);
    let price_val: Val = price.into_val(e);

    let event_payload = vec![e, engagement_id_val, client_val, price_val];

    e.events().publish(topics, event_payload);
}

pub (crate) fn engagements_by_address(e: &Env, address: Address, engagements: Vec<Engagement>) {
    let topics = (symbol_short!("p_by_spdr"),);
    
    let address_val: Val = address.into_val(e);
    let engagements_val: Val = engagements.into_val(e);

    let event_payload = vec![e, address_val, engagements_val];
    e.events().publish(topics, event_payload);
}


// ------ Escrows

pub (crate) fn escrow_added(e: &Env, engagement_id: &DataKey, escrow_id: u128, price: u128) {
    let topics = (symbol_short!("ob_added"),);

    let engagement_id_val: Val = engagement_id.into_val(e);
    let escrow_id_val: Val = escrow_id.into_val(e);
    let price_val: Val = price.into_val(e);

    let event_payload = vec![e, engagement_id_val, escrow_id_val, price_val];
    e.events().publish(topics, event_payload);
}

pub (crate) fn escrow_funded(e: &Env, engagement_id: DataKey, escrow_id: u128, half_price: u128) {
    let topics = (symbol_short!("ob_funded"),);

    let engagement_id_val: Val = engagement_id.into_val(e);
    let escrow_id_val: Val = escrow_id.into_val(e);
    let half_price_val: Val = half_price.into_val(e);

    let event_payload = vec![e, engagement_id_val, escrow_id_val, half_price_val];
    e.events().publish(topics, event_payload);
}

pub (crate) fn escrow_completed(e: &Env, engagement_id: DataKey, escrow_id: u128, full_price: u128) {
    let topics = (symbol_short!("ob_c"),); // c -> completed

    let engagement_id_val: Val = engagement_id.into_val(e);
    let escrow_id_val: Val = escrow_id.into_val(e);
    let full_price_val: Val = full_price.into_val(e);

    let event_payload = vec![e, engagement_id_val, escrow_id_val, full_price_val];
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