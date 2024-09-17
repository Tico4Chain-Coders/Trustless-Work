use soroban_sdk::{contracttype, Address, Bytes, Map, String};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone)]
pub struct Objective {
    pub price: u128,
    pub half_paid: u128,
    pub completed: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct Escrow {
    pub escrow_id: Bytes,
    pub spender: Address,
    pub from: Address,
    pub parties_count: u128,
    pub parties: Map<u128, Objective>,
    pub completed_parties: u128,
    pub earned_amount: u128,
    pub contract_balance: u128,
    pub cancelled: bool,
    pub completed: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct User {
    pub id: u64,
    pub user: Address,
    pub name: String,
    pub email: String,
    pub registered: bool,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow(Bytes),
    Balance(Address),
    Allowance(AllowanceDataKey),
    Admin,

    // User storage
    User(Address),
    UserRegId(Address),
    UserCounter,
}