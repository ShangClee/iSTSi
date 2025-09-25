use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

/// Common types used across Bitcoin custody contracts

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterConfig {
    pub kyc_registry: Address,
    pub istsi_token: Address,
    pub fungible_token: Address,
    pub reserve_manager: Address,
    pub admin: Address,
    pub paused: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntegrationOperation {
    BitcoinDeposit(Address, u64, BytesN<32>),    // user, btc_amount, btc_tx_hash
    TokenWithdrawal(Address, u64, String),       // user, token_amount, btc_address
    CrossTokenExchange(Address, Address, Address, u64), // user, from_token, to_token, amount
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationContext {
    pub operation_id: BytesN<32>,
    pub user: Address,
    pub operation_type: OperationType,
    pub amount: u64,
    pub timestamp: u64,
    pub status: OperationStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationType {
    BitcoinDeposit,
    TokenWithdrawal,
    CrossTokenExchange,
    ComplianceCheck,
    ReserveUpdate,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceResult {
    pub user: Address,
    pub operation_type: OperationType,
    pub approved: bool,
    pub reason: String,
    pub tier_required: u32,
    pub user_tier: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_processing_time: u64,
    pub current_reserve_ratio: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCall {
    pub target_contract: Address,
    pub function_name: String,
    pub parameters: Vec<soroban_sdk::Val>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchOperation {
    pub operation_id: BytesN<32>,
    pub calls: Vec<ContractCall>,
    pub rollback_calls: Vec<ContractCall>,
    pub timeout: u64,
}