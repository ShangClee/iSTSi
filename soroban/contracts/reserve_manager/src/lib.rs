#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, symbol_short, panic_with_error,
    Address, Env, String, BytesN
};

/// Reserve Manager Contract for Bitcoin-backed Token System
/// 
/// This contract manages Bitcoin reserves, tracks deposits/withdrawals,
/// calculates reserve ratios, and generates proof-of-reserves.

#[contract]
pub struct ReserveManager;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReserveError {
    Unauthorized = 1,
    NotFound = 2,
    InvalidInput = 3,
    InsufficientReserves = 4,
    InvalidTransaction = 5,
    ThresholdBreach = 6,
    AlreadyProcessed = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    IntegrationRouter,
    TotalReserves,
    TotalTokenSupply,
    BitcoinDeposit(BytesN<32>),     // tx_hash -> BitcoinTransaction
    WithdrawalRequest(BytesN<32>),  // withdrawal_id -> WithdrawalRequest
    ReserveThresholds,
    ProofOfReserves,
    OperationHistory(u64),          // timestamp -> OperationRecord
    ReserveRatioHistory(u64),       // timestamp -> u64 (ratio in basis points)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitcoinTransaction {
    pub tx_hash: BytesN<32>,
    pub amount: u64,
    pub confirmations: u32,
    pub timestamp: u64,
    pub processed: bool,
    pub user: Address,
    pub block_height: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalRequest {
    pub withdrawal_id: BytesN<32>,
    pub user: Address,
    pub amount: u64,
    pub btc_address: String,
    pub timestamp: u64,
    pub processed: bool,
    pub btc_tx_hash: Option<BytesN<32>>,
    pub status: WithdrawalStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WithdrawalStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveThresholds {
    pub minimum_ratio: u64,     // Basis points (e.g., 10000 = 100%)
    pub warning_ratio: u64,     // Basis points
    pub critical_ratio: u64,    // Basis points
    pub emergency_halt: bool,   // Auto-halt on critical breach
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofOfReserves {
    pub total_btc_reserves: u64,
    pub total_token_supply: u64,
    pub reserve_ratio: u64,      // Basis points
    pub timestamp: u64,
    pub merkle_root: BytesN<32>, // Merkle root of all deposits
    pub signature: BytesN<64>,   // Cryptographic proof
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationType {
    Deposit,
    Withdrawal,
    ReserveUpdate,
    ProofGeneration,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationRecord {
    pub operation_type: OperationType,
    pub amount: u64,
    pub timestamp: u64,
    pub tx_hash: Option<BytesN<32>>,
    pub user: Option<Address>,
    pub notes: String,
}

#[contractimpl]
impl ReserveManager {
    
    /// Initialize the reserve manager contract
    pub fn initialize(env: Env, admin: Address, integration_router: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, ReserveError::AlreadyProcessed);
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::IntegrationRouter, &integration_router);
        
        // Initialize reserves and supply to zero
        env.storage().persistent().set(&DataKey::TotalReserves, &0u64);
        env.storage().persistent().set(&DataKey::TotalTokenSupply, &0u64);
        
        // Set default thresholds
        let thresholds = ReserveThresholds {
            minimum_ratio: 10000,    // 100% - fully backed
            warning_ratio: 11000,    // 110% - warning level
            critical_ratio: 10500,   // 105% - critical level
            emergency_halt: true,
        };
        env.storage().instance().set(&DataKey::ReserveThresholds, &thresholds);
        
        env.events().publish(
            (symbol_short!("init"), admin.clone()),
            (symbol_short!("reserve"), symbol_short!("mgr"))
        );
    }
    
    /// Register a Bitcoin deposit transaction
    pub fn register_bitcoin_deposit(
        env: Env,
        caller: Address,
        tx_hash: BytesN<32>,
        amount: u64,
        confirmations: u32,
        user: Address,
        block_height: u64
    ) {
        Self::require_authorized(&env, &caller);
        
        // Check if transaction already exists
        if env.storage().persistent().has(&DataKey::BitcoinDeposit(tx_hash.clone())) {
            panic_with_error!(&env, ReserveError::AlreadyProcessed);
        }
        
        if amount == 0 {
            panic_with_error!(&env, ReserveError::InvalidInput);
        }
        
        let deposit = BitcoinTransaction {
            tx_hash: tx_hash.clone(),
            amount,
            confirmations,
            timestamp: env.ledger().timestamp(),
            processed: false,
            user: user.clone(),
            block_height,
        };
        
        env.storage().persistent().set(&DataKey::BitcoinDeposit(tx_hash.clone()), &deposit);
        
        // Log operation
        Self::log_operation(&env, OperationRecord {
            operation_type: OperationType::Deposit,
            amount,
            timestamp: env.ledger().timestamp(),
            tx_hash: Some(tx_hash.clone()),
            user: Some(user.clone()),
            notes: String::from_str(&env, "Bitcoin deposit registered"),
        });
        
        env.events().publish(
            (symbol_short!("btc_dep"), tx_hash, user),
            (amount, confirmations, block_height)
        );
    }
    
    /// Process a confirmed Bitcoin deposit (add to reserves)
    pub fn process_bitcoin_deposit(
        env: Env,
        caller: Address,
        tx_hash: BytesN<32>
    ) {
        Self::require_authorized(&env, &caller);
        
        let mut deposit: BitcoinTransaction = env.storage().persistent()
            .get(&DataKey::BitcoinDeposit(tx_hash.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ReserveError::NotFound));
        
        if deposit.processed {
            panic_with_error!(&env, ReserveError::AlreadyProcessed);
        }
        
        // Mark as processed
        deposit.processed = true;
        env.storage().persistent().set(&DataKey::BitcoinDeposit(tx_hash.clone()), &deposit);
        
        // Update total reserves
        let current_reserves: u64 = env.storage().persistent()
            .get(&DataKey::TotalReserves)
            .unwrap_or(0);
        
        let new_reserves = current_reserves + deposit.amount;
        env.storage().persistent().set(&DataKey::TotalReserves, &new_reserves);
        
        // Update reserve ratio and check thresholds
        Self::update_reserve_ratio(&env);
        Self::check_reserve_thresholds(&env);
        
        env.events().publish(
            (symbol_short!("dep_proc"), tx_hash, deposit.user),
            (deposit.amount, new_reserves)
        );
    }
    
    /// Create a Bitcoin withdrawal request
    pub fn create_withdrawal_request(
        env: Env,
        caller: Address,
        user: Address,
        amount: u64,
        btc_address: String
    ) -> BytesN<32> {
        Self::require_authorized(&env, &caller);
        
        if amount == 0 {
            panic_with_error!(&env, ReserveError::InvalidInput);
        }
        
        // Check if sufficient reserves
        let current_reserves: u64 = env.storage().persistent()
            .get(&DataKey::TotalReserves)
            .unwrap_or(0);
        
        if amount > current_reserves {
            panic_with_error!(&env, ReserveError::InsufficientReserves);
        }
        
        // Generate withdrawal ID
        let withdrawal_id = Self::generate_withdrawal_id(&env, &user, amount);
        
        let withdrawal = WithdrawalRequest {
            withdrawal_id: withdrawal_id.clone(),
            user: user.clone(),
            amount,
            btc_address: btc_address.clone(),
            timestamp: env.ledger().timestamp(),
            processed: false,
            btc_tx_hash: None,
            status: WithdrawalStatus::Pending,
        };
        
        env.storage().persistent().set(&DataKey::WithdrawalRequest(withdrawal_id.clone()), &withdrawal);
        
        // Log operation
        Self::log_operation(&env, OperationRecord {
            operation_type: OperationType::Withdrawal,
            amount,
            timestamp: env.ledger().timestamp(),
            tx_hash: Some(withdrawal_id.clone()),
            user: Some(user.clone()),
            notes: String::from_str(&env, "Withdrawal request created"),
        });
        
        env.events().publish(
            (symbol_short!("with_req"), withdrawal_id.clone(), user),
            (amount, btc_address)
        );
        
        withdrawal_id
    }
    
    /// Process a Bitcoin withdrawal (deduct from reserves)
    pub fn process_bitcoin_withdrawal(
        env: Env,
        caller: Address,
        withdrawal_id: BytesN<32>,
        btc_tx_hash: BytesN<32>
    ) {
        Self::require_authorized(&env, &caller);
        
        let mut withdrawal: WithdrawalRequest = env.storage().persistent()
            .get(&DataKey::WithdrawalRequest(withdrawal_id.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ReserveError::NotFound));
        
        if withdrawal.processed {
            panic_with_error!(&env, ReserveError::AlreadyProcessed);
        }
        
        // Update withdrawal status
        withdrawal.processed = true;
        withdrawal.btc_tx_hash = Some(btc_tx_hash.clone());
        withdrawal.status = WithdrawalStatus::Completed;
        env.storage().persistent().set(&DataKey::WithdrawalRequest(withdrawal_id.clone()), &withdrawal);
        
        // Update total reserves
        let current_reserves: u64 = env.storage().persistent()
            .get(&DataKey::TotalReserves)
            .unwrap_or(0);
        
        if withdrawal.amount > current_reserves {
            panic_with_error!(&env, ReserveError::InsufficientReserves);
        }
        
        let new_reserves = current_reserves - withdrawal.amount;
        env.storage().persistent().set(&DataKey::TotalReserves, &new_reserves);
        
        // Update reserve ratio and check thresholds
        Self::update_reserve_ratio(&env);
        Self::check_reserve_thresholds(&env);
        
        env.events().publish(
            (symbol_short!("with_proc"), withdrawal_id, btc_tx_hash),
            (withdrawal.amount, new_reserves)
        );
    }
    
    /// Update token supply (called by token contract)
    pub fn update_token_supply(
        env: Env,
        caller: Address,
        new_supply: u64
    ) {
        Self::require_authorized(&env, &caller);
        
        let old_supply: u64 = env.storage().persistent()
            .get(&DataKey::TotalTokenSupply)
            .unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::TotalTokenSupply, &new_supply);
        
        // Update reserve ratio and check thresholds
        Self::update_reserve_ratio(&env);
        Self::check_reserve_thresholds(&env);
        
        // Log operation
        Self::log_operation(&env, OperationRecord {
            operation_type: OperationType::ReserveUpdate,
            amount: new_supply,
            timestamp: env.ledger().timestamp(),
            tx_hash: None,
            user: None,
            notes: String::from_str(&env, "Token supply updated"),
        });
        
        env.events().publish(
            (symbol_short!("supply"), caller),
            (old_supply, new_supply)
        );
    }
    
    /// Get current reserve ratio in basis points
    pub fn get_reserve_ratio(env: Env) -> u64 {
        let reserves: u64 = env.storage().persistent()
            .get(&DataKey::TotalReserves)
            .unwrap_or(0);
        
        let supply: u64 = env.storage().persistent()
            .get(&DataKey::TotalTokenSupply)
            .unwrap_or(0);
        
        if supply == 0 {
            return 0; // No tokens issued yet
        }
        
        // Calculate ratio in basis points (10000 = 100%)
        (reserves * 10000) / supply
    }
    
    /// Generate proof of reserves
    pub fn generate_proof_of_reserves(env: Env, caller: Address) -> ProofOfReserves {
        Self::require_authorized(&env, &caller);
        
        let reserves: u64 = env.storage().persistent()
            .get(&DataKey::TotalReserves)
            .unwrap_or(0);
        
        let supply: u64 = env.storage().persistent()
            .get(&DataKey::TotalTokenSupply)
            .unwrap_or(0);
        
        let ratio = Self::get_reserve_ratio(env.clone());
        
        // Generate merkle root of all deposits (simplified)
        let merkle_root = Self::calculate_merkle_root(&env);
        
        // Generate cryptographic signature (simplified)
        let signature = Self::generate_proof_signature(&env, reserves, supply, ratio);
        
        let proof = ProofOfReserves {
            total_btc_reserves: reserves,
            total_token_supply: supply,
            reserve_ratio: ratio,
            timestamp: env.ledger().timestamp(),
            merkle_root,
            signature,
        };
        
        env.storage().instance().set(&DataKey::ProofOfReserves, &proof);
        
        // Log operation
        Self::log_operation(&env, OperationRecord {
            operation_type: OperationType::ProofGeneration,
            amount: reserves,
            timestamp: env.ledger().timestamp(),
            tx_hash: None,
            user: None,
            notes: String::from_str(&env, "Proof of reserves generated"),
        });
        
        env.events().publish(
            (symbol_short!("proof"), caller),
            (reserves, supply, ratio)
        );
        
        proof
    }
    
    /// Get current proof of reserves
    pub fn get_proof_of_reserves(env: Env) -> Option<ProofOfReserves> {
        env.storage().instance().get(&DataKey::ProofOfReserves)
    }
    
    /// Get total reserves
    pub fn get_total_reserves(env: Env) -> u64 {
        env.storage().persistent()
            .get(&DataKey::TotalReserves)
            .unwrap_or(0)
    }
    
    /// Get total token supply
    pub fn get_total_token_supply(env: Env) -> u64 {
        env.storage().persistent()
            .get(&DataKey::TotalTokenSupply)
            .unwrap_or(0)
    }
    
    /// Get Bitcoin deposit information
    pub fn get_bitcoin_deposit(env: Env, tx_hash: BytesN<32>) -> Option<BitcoinTransaction> {
        env.storage().persistent().get(&DataKey::BitcoinDeposit(tx_hash))
    }
    
    /// Get withdrawal request information
    pub fn get_withdrawal_request(env: Env, withdrawal_id: BytesN<32>) -> Option<WithdrawalRequest> {
        env.storage().persistent().get(&DataKey::WithdrawalRequest(withdrawal_id))
    }
    
    /// Set reserve thresholds (admin only)
    pub fn set_reserve_thresholds(
        env: Env,
        caller: Address,
        thresholds: ReserveThresholds
    ) {
        Self::require_admin(&env, &caller);
        
        env.storage().instance().set(&DataKey::ReserveThresholds, &thresholds);
        
        env.events().publish(
            (symbol_short!("thresh"), caller),
            (thresholds.minimum_ratio, thresholds.warning_ratio, thresholds.critical_ratio)
        );
    }
    
    /// Get reserve thresholds
    pub fn get_reserve_thresholds(env: Env) -> ReserveThresholds {
        env.storage().instance()
            .get(&DataKey::ReserveThresholds)
            .unwrap_or(ReserveThresholds {
                minimum_ratio: 10000,
                warning_ratio: 11000,
                critical_ratio: 10500,
                emergency_halt: true,
            })
    }
    
    // =====================
    // Helper Functions
    // =====================
    
    /// Require caller to be admin
    fn require_admin(env: &Env, caller: &Address) {
        caller.require_auth();
        
        let admin: Address = env.storage().instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, ReserveError::Unauthorized));
        
        if *caller != admin {
            panic_with_error!(env, ReserveError::Unauthorized);
        }
    }
    
    /// Require caller to be authorized (admin or integration router)
    fn require_authorized(env: &Env, caller: &Address) {
        caller.require_auth();
        
        // Check if caller is admin
        if let Some(admin) = env.storage().instance().get::<DataKey, Address>(&DataKey::Admin) {
            if admin == *caller {
                return;
            }
        }
        
        // Check if caller is integration router
        if let Some(router) = env.storage().instance().get::<DataKey, Address>(&DataKey::IntegrationRouter) {
            if router == *caller {
                return;
            }
        }
        
        panic_with_error!(env, ReserveError::Unauthorized);
    }
    
    /// Generate withdrawal ID
    fn generate_withdrawal_id(env: &Env, user: &Address, amount: u64) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        
        let mut id_bytes = [0u8; 32];
        id_bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
        id_bytes[8..12].copy_from_slice(&sequence.to_be_bytes());
        id_bytes[12..20].copy_from_slice(&amount.to_be_bytes());
        
        // Add some user address bytes for uniqueness (simplified)
        // In a real implementation, you'd use proper address serialization
        let user_hash = user.clone().to_string().len() as u64;
        id_bytes[20..28].copy_from_slice(&user_hash.to_be_bytes());
        
        BytesN::from_array(env, &id_bytes)
    }
    
    /// Update reserve ratio and store in history
    fn update_reserve_ratio(env: &Env) {
        let ratio = Self::get_reserve_ratio(env.clone());
        let timestamp = env.ledger().timestamp();
        
        env.storage().persistent().set(&DataKey::ReserveRatioHistory(timestamp), &ratio);
    }
    
    /// Check reserve thresholds and emit alerts
    fn check_reserve_thresholds(env: &Env) {
        let thresholds = Self::get_reserve_thresholds(env.clone());
        let current_ratio = Self::get_reserve_ratio(env.clone());
        
        if current_ratio < thresholds.critical_ratio {
            // Critical threshold breach
            env.events().publish(
                (symbol_short!("alert"), symbol_short!("critical")),
                (current_ratio, thresholds.critical_ratio)
            );
            
            if thresholds.emergency_halt {
                // In a real implementation, this would trigger emergency procedures
                env.events().publish(
                    (symbol_short!("emergency"), symbol_short!("halt")),
                    current_ratio
                );
            }
        } else if current_ratio < thresholds.warning_ratio {
            // Warning threshold breach
            env.events().publish(
                (symbol_short!("alert"), symbol_short!("warning")),
                (current_ratio, thresholds.warning_ratio)
            );
        }
    }
    
    /// Calculate merkle root of all deposits (simplified implementation)
    fn calculate_merkle_root(env: &Env) -> BytesN<32> {
        // In a real implementation, this would calculate the actual merkle root
        // of all Bitcoin deposits. For now, we'll create a simple hash.
        let reserves: u64 = env.storage().persistent()
            .get(&DataKey::TotalReserves)
            .unwrap_or(0);
        
        let timestamp = env.ledger().timestamp();
        
        let mut hash_input = [0u8; 32];
        hash_input[0..8].copy_from_slice(&reserves.to_be_bytes());
        hash_input[8..16].copy_from_slice(&timestamp.to_be_bytes());
        
        BytesN::from_array(env, &hash_input)
    }
    
    /// Generate cryptographic proof signature (simplified implementation)
    fn generate_proof_signature(env: &Env, reserves: u64, supply: u64, ratio: u64) -> BytesN<64> {
        // In a real implementation, this would use proper cryptographic signatures
        // For now, we'll create a deterministic signature-like value
        let timestamp = env.ledger().timestamp();
        
        let mut signature = [0u8; 64];
        signature[0..8].copy_from_slice(&reserves.to_be_bytes());
        signature[8..16].copy_from_slice(&supply.to_be_bytes());
        signature[16..24].copy_from_slice(&ratio.to_be_bytes());
        signature[24..32].copy_from_slice(&timestamp.to_be_bytes());
        
        // Fill the rest with a pattern
        for i in 32..64 {
            signature[i] = ((i * 7) % 256) as u8;
        }
        
        BytesN::from_array(env, &signature)
    }
    
    /// Log operation for audit trail
    fn log_operation(env: &Env, operation: OperationRecord) {
        let timestamp = operation.timestamp;
        env.storage().persistent().set(&DataKey::OperationHistory(timestamp), &operation);
        
        env.events().publish(
            (symbol_short!("op_log"), operation.operation_type),
            (operation.amount, timestamp)
        );
    }
}

// Unit tests
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as AddressTestUtils, Address, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ReserveManager, ());
        let client = ReserveManagerClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        
        client.initialize(&admin, &router);
        
        // Test initial state
        assert_eq!(client.get_total_reserves(), 0);
        assert_eq!(client.get_total_token_supply(), 0);
        assert_eq!(client.get_reserve_ratio(), 0);
        
        let thresholds = client.get_reserve_thresholds();
        assert_eq!(thresholds.minimum_ratio, 10000);
        assert_eq!(thresholds.emergency_halt, true);
    }
    
    #[test]
    fn test_bitcoin_deposit_flow() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ReserveManager, ());
        let client = ReserveManagerClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        let user = Address::generate(&env);
        
        client.initialize(&admin, &router);
        
        // Create a Bitcoin transaction hash
        let tx_hash = BytesN::from_array(&env, &[1u8; 32]);
        let amount = 100_000_000u64; // 1 BTC in satoshis
        
        // Register Bitcoin deposit
        client.register_bitcoin_deposit(
            &router,
            &tx_hash,
            &amount,
            &6u32, // confirmations
            &user,
            &800000u64 // block height
        );
        
        // Check deposit was registered
        let deposit = client.get_bitcoin_deposit(&tx_hash).unwrap();
        assert_eq!(deposit.amount, amount);
        assert_eq!(deposit.user, user);
        assert_eq!(deposit.processed, false);
        
        // Process the deposit
        client.process_bitcoin_deposit(&router, &tx_hash);
        
        // Check reserves were updated
        assert_eq!(client.get_total_reserves(), amount);
        
        // Check deposit was marked as processed
        let processed_deposit = client.get_bitcoin_deposit(&tx_hash).unwrap();
        assert_eq!(processed_deposit.processed, true);
    }
    
    #[test]
    fn test_withdrawal_flow() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ReserveManager, ());
        let client = ReserveManagerClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        let user = Address::generate(&env);
        
        client.initialize(&admin, &router);
        
        // First add some reserves
        let deposit_hash = BytesN::from_array(&env, &[1u8; 32]);
        let deposit_amount = 200_000_000u64; // 2 BTC
        
        client.register_bitcoin_deposit(&router, &deposit_hash, &deposit_amount, &6u32, &user, &800000u64);
        client.process_bitcoin_deposit(&router, &deposit_hash);
        
        // Create withdrawal request
        let withdrawal_amount = 50_000_000u64; // 0.5 BTC
        let btc_address = String::from_str(&env, "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh");
        
        let withdrawal_id = client.create_withdrawal_request(
            &router,
            &user,
            &withdrawal_amount,
            &btc_address
        );
        
        // Check withdrawal request
        let withdrawal = client.get_withdrawal_request(&withdrawal_id).unwrap();
        assert_eq!(withdrawal.amount, withdrawal_amount);
        assert_eq!(withdrawal.user, user);
        assert_eq!(withdrawal.status, WithdrawalStatus::Pending);
        
        // Process withdrawal
        let btc_tx_hash = BytesN::from_array(&env, &[2u8; 32]);
        client.process_bitcoin_withdrawal(&router, &withdrawal_id, &btc_tx_hash);
        
        // Check reserves were updated
        assert_eq!(client.get_total_reserves(), deposit_amount - withdrawal_amount);
        
        // Check withdrawal was processed
        let processed_withdrawal = client.get_withdrawal_request(&withdrawal_id).unwrap();
        assert_eq!(processed_withdrawal.processed, true);
        assert_eq!(processed_withdrawal.status, WithdrawalStatus::Completed);
        assert_eq!(processed_withdrawal.btc_tx_hash, Some(btc_tx_hash));
    }
    
    #[test]
    fn test_reserve_ratio_calculation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ReserveManager, ());
        let client = ReserveManagerClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        
        client.initialize(&admin, &router);
        
        // Add reserves
        let deposit_hash = BytesN::from_array(&env, &[1u8; 32]);
        let deposit_amount = 100_000_000u64; // 1 BTC
        let user = Address::generate(&env);
        
        client.register_bitcoin_deposit(&router, &deposit_hash, &deposit_amount, &6u32, &user, &800000u64);
        client.process_bitcoin_deposit(&router, &deposit_hash);
        
        // Update token supply
        let token_supply = 100_000_000u64; // 1 BTC worth of tokens
        client.update_token_supply(&router, &token_supply);
        
        // Check reserve ratio (should be 100% = 10000 basis points)
        let ratio = client.get_reserve_ratio();
        assert_eq!(ratio, 10000);
        
        // Add more reserves
        let deposit_hash2 = BytesN::from_array(&env, &[2u8; 32]);
        client.register_bitcoin_deposit(&router, &deposit_hash2, &deposit_amount, &6u32, &user, &800001u64);
        client.process_bitcoin_deposit(&router, &deposit_hash2);
        
        // Check reserve ratio (should be 200% = 20000 basis points)
        let new_ratio = client.get_reserve_ratio();
        assert_eq!(new_ratio, 20000);
    }
    
    #[test]
    fn test_proof_of_reserves() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ReserveManager, ());
        let client = ReserveManagerClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        
        client.initialize(&admin, &router);
        
        // Add some reserves and token supply
        let deposit_hash = BytesN::from_array(&env, &[1u8; 32]);
        let deposit_amount = 150_000_000u64; // 1.5 BTC
        let user = Address::generate(&env);
        
        client.register_bitcoin_deposit(&router, &deposit_hash, &deposit_amount, &6u32, &user, &800000u64);
        client.process_bitcoin_deposit(&router, &deposit_hash);
        client.update_token_supply(&router, &100_000_000u64); // 1 BTC worth of tokens
        
        // Generate proof of reserves
        let proof = client.generate_proof_of_reserves(&admin);
        
        assert_eq!(proof.total_btc_reserves, deposit_amount);
        assert_eq!(proof.total_token_supply, 100_000_000u64);
        assert_eq!(proof.reserve_ratio, 15000); // 150% backing
        // Timestamp is set by the ledger
        
        // Check proof can be retrieved
        let stored_proof = client.get_proof_of_reserves().unwrap();
        assert_eq!(stored_proof.total_btc_reserves, proof.total_btc_reserves);
        assert_eq!(stored_proof.reserve_ratio, proof.reserve_ratio);
    }
    
    #[test]
    fn test_reserve_thresholds() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ReserveManager, ());
        let client = ReserveManagerClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        
        client.initialize(&admin, &router);
        
        // Set custom thresholds
        let new_thresholds = ReserveThresholds {
            minimum_ratio: 9000,    // 90%
            warning_ratio: 10500,   // 105%
            critical_ratio: 9500,   // 95%
            emergency_halt: false,
        };
        
        client.set_reserve_thresholds(&admin, &new_thresholds);
        
        let stored_thresholds = client.get_reserve_thresholds();
        assert_eq!(stored_thresholds.minimum_ratio, 9000);
        assert_eq!(stored_thresholds.warning_ratio, 10500);
        assert_eq!(stored_thresholds.critical_ratio, 9500);
        assert_eq!(stored_thresholds.emergency_halt, false);
    }
    
    #[test]
    #[should_panic(expected = "Error(Contract, #4)")]
    fn test_insufficient_reserves_withdrawal() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ReserveManager, ());
        let client = ReserveManagerClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        let user = Address::generate(&env);
        
        client.initialize(&admin, &router);
        
        // Try to create withdrawal without sufficient reserves
        let btc_address = String::from_str(&env, "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh");
        client.create_withdrawal_request(&router, &user, &100_000_000u64, &btc_address);
    }
    
    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn test_duplicate_deposit_registration() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ReserveManager, ());
        let client = ReserveManagerClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        let user = Address::generate(&env);
        
        client.initialize(&admin, &router);
        
        let tx_hash = BytesN::from_array(&env, &[1u8; 32]);
        
        // Register deposit twice
        client.register_bitcoin_deposit(&router, &tx_hash, &100_000_000u64, &6u32, &user, &800000u64);
        client.register_bitcoin_deposit(&router, &tx_hash, &100_000_000u64, &6u32, &user, &800000u64);
    }
}