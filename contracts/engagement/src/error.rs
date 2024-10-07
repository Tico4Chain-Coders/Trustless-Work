use soroban_sdk::contracterror;
use core::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum ContractError {
    EscrowNotFunded = 1,
    PricesCannotBeZero = 2,
    EscrowAlreadyInitialized = 3,
    OnlySignerCanFundEscrow = 4,
    EscrowAlreadyFunded = 5,
    EscrowFullyFunded = 6,
    SignerInsufficientFunds = 7,
    NotEnoughAllowance = 8,
    OnlySignerCanCompleteEscrow = 9,
    EscrowAlreadyCompleted = 10,
    SignerInsufficientFundsToComplete = 11,
    OnlySignerCanCancelEscrow = 12,
    EscrowAlreadyCancelled = 13,
    OnlySignerCanRequestRefund = 14,
    EscrowNotCancelled = 15,
    NoFundsToRefund = 16,
    ContractHasInsufficientBalance = 17
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::EscrowNotFunded => write!(f, "Escrow not funded"),
            ContractError::PricesCannotBeZero => write!(f, "Prices cannot be zero"),
            ContractError::EscrowAlreadyInitialized => write!(f, "Escrow already initialized"),
            ContractError::OnlySignerCanFundEscrow => write!(f, "Only the signer can fund the escrow"),
            ContractError::EscrowAlreadyFunded => write!(f, "Escrow already funded"),
            ContractError::EscrowFullyFunded => write!(f, "This escrow is already fully funded"),
            ContractError::SignerInsufficientFunds => write!(f, "The signer does not have sufficient funds"),
            ContractError::NotEnoughAllowance => write!(f, "Not enough allowance to fund this escrow"),
            ContractError::OnlySignerCanCompleteEscrow => write!(f, "Only the signer can complete the escrow"),
            ContractError::EscrowAlreadyCompleted => write!(f, "Escrow already completed"),
            ContractError::SignerInsufficientFundsToComplete => write!(f, "The signer does not have sufficient funds to complete this escrow"),
            ContractError::OnlySignerCanCancelEscrow => write!(f, "Only the signer can cancel the escrow"),
            ContractError::EscrowAlreadyCancelled => write!(f, "The escrow has already been cancelled"),
            ContractError::OnlySignerCanRequestRefund => write!(f, "Only the signer can request a refund"),
            ContractError::EscrowNotCancelled => write!(f, "The escrow must be cancelled to refund the amounts"),
            ContractError::NoFundsToRefund => write!(f, "No funds available to refund"),
            ContractError::ContractHasInsufficientBalance => write!(f, "The contract has no balance to repay")
        }
    }
}