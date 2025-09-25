use soroban_sdk::{contracttype, Address, BytesN, String};

/// Common event types used across Bitcoin custody contracts

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntegrationEvent {
    BitcoinDeposit(Address, u64, u64, BytesN<32>, u64),    // user, btc_amount, istsi_minted, tx_hash, timestamp
    TokenWithdrawal(Address, u64, u64, BytesN<32>, u64),   // user, istsi_burned, btc_amount, withdrawal_id, timestamp
    CrossTokenExchange(Address, Address, Address, u64, u64, u64), // user, from_token, to_token, from_amount, to_amount, timestamp
    ComplianceAction(Address, String, String, u64),        // user, action, reason, timestamp
    ReserveUpdate(u64, u64, u64, u64),                     // total_btc, total_istsi, reserve_ratio, timestamp
    SystemPause(Address, String, u64),                     // admin, reason, timestamp
    SystemResume(Address, u64),                            // admin, timestamp
    ContractUpgrade(Address, String, String, Address, u64), // contract_address, old_version, new_version, admin, timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventMetadata {
    pub event_id: BytesN<32>,
    pub contract_address: Address,
    pub event_type: String,
    pub timestamp: u64,
    pub block_number: u64,
}

/// Helper function to create event metadata
pub fn create_event_metadata(
    env: &soroban_sdk::Env,
    event_type: &str,
) -> EventMetadata {
    EventMetadata {
        event_id: env.crypto().sha256(&soroban_sdk::Bytes::from_slice(env, event_type.as_bytes())).into(),
        contract_address: env.current_contract_address(),
        event_type: String::from_str(env, event_type),
        timestamp: env.ledger().timestamp(),
        block_number: env.ledger().sequence() as u64,
    }
}