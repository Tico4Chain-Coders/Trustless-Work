use soroban_sdk::{contracttype, Address, String};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone, Debug)]
pub struct Escrow {
    pub engagement_id: String,
    pub description: String,
    pub issuer: Address,
    pub signer: Address,
    pub service_provider: Address,
    pub amount: u128,
    pub balance: u128,
    pub completed: bool,
    pub cancelled: bool,
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
    Escrow(String),
    Balance(Address),
    Allowance(AllowanceDataKey),
    Admin,

    // User storage
    User(Address),
    UserRegId(Address),
    UserCounter,
}