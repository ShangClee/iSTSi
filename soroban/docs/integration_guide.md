# Soroban Contract Integration Guide

This guide provides comprehensive instructions for integrating with the Bitcoin custody system's Soroban smart contracts from backend services.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Setup and Configuration](#setup-and-configuration)
3. [Contract Client Usage](#contract-client-usage)
4. [Event Monitoring](#event-monitoring)
5. [Error Handling](#error-handling)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

## Quick Start

### 1. Add Dependencies

Add the Soroban client library to your `Cargo.toml`:

```toml
[dependencies]
soroban-sdk = "22.0.8"
soroban-client = { path = "../soroban/client" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 2. Initialize Contract Manager

```rust
use soroban_client::{ContractManager, ContractAddresses, NetworkConfig};
use soroban_sdk::Env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Soroban environment
    let env = Env::default();
    
    // Load contract addresses
    let addresses = load_contract_addresses("testnet")?;
    
    // Configure network
    let network_config = NetworkConfig::testnet();
    
    // Create contract manager
    let manager = ContractManager::new(env, addresses, network_config)?;
    
    // Your integration code here...
    
    Ok(())
}

fn load_contract_addresses(network: &str) -> Result<ContractAddresses, Box<dyn std::error::Error>> {
    let config_json = std::fs::read_to_string("soroban/config/contract_addresses.json")?;
    let config: serde_json::Value = serde_json::from_str(&config_json)?;
    
    let network_config = config[network].as_object()
        .ok_or("Network not found in configuration")?;
    
    let mut address_map = std::collections::HashMap::new();
    for (contract_name, address) in network_config {
        if let Some(addr_str) = address.as_str() {
            address_map.insert(contract_name.clone(), addr_str.to_string());
        }
    }
    
    ContractAddresses::from_config(address_map)
        .map_err(|e| e.into())
}
```

### 3. Execute Operations

```rust
use soroban_client::OperationContext;
use soroban_sdk::{Address, BytesN};

// Create operation context
let ctx = OperationContext {
    caller: admin_address,
    operation_id: "deposit_001".to_string(),
    timeout_seconds: 30,
    retry_count: 3,
};

// Execute Bitcoin deposit
let user_address = Address::from_string(&soroban_sdk::String::from_str(&env, "GXXXXXXX..."));
let btc_tx_hash = BytesN::from_array(&env, &[1u8; 32]);

let operation_id = manager.execute_bitcoin_deposit_workflow(
    &ctx,
    &user_address,
    100_000_000, // 1 BTC in satoshis
    &btc_tx_hash,
    6, // confirmations
    800000, // block height
)?;

println!("Bitcoin deposit initiated: {:?}", operation_id);
```

## Setup and Configuration

### Environment Variables

Create a `.env` file with the following variables:

```env
# Network Configuration
SOROBAN_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Contract Addresses
INTEGRATION_ROUTER_ADDRESS=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
KYC_REGISTRY_ADDRESS=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
ISTSI_TOKEN_ADDRESS=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
RESERVE_MANAGER_ADDRESS=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

# Authentication
ADMIN_SECRET_KEY=SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
SERVICE_ACCOUNT_SECRET=SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

# Monitoring
EVENT_MONITORING_ENABLED=true
EVENT_POLL_INTERVAL=5
EVENT_BATCH_SIZE=100

# Timeouts and Retries
DEFAULT_TIMEOUT_SECONDS=30
MAX_RETRY_ATTEMPTS=3
RETRY_BACKOFF_SECONDS=2
```

### Configuration Loading

```rust
use std::env;
use soroban_client::{NetworkConfig, ContractAddresses};

pub struct Config {
    pub network: NetworkConfig,
    pub addresses: ContractAddresses,
    pub admin_secret: String,
    pub monitoring_enabled: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        
        let network_name = env::var("SOROBAN_NETWORK")?;
        let network = match network_name.as_str() {
            "testnet" => NetworkConfig::testnet(),
            "mainnet" => NetworkConfig::mainnet(),
            "local" => NetworkConfig::local(),
            _ => return Err("Invalid network name".into()),
        };
        
        let mut addresses = ContractAddresses::new();
        addresses.integration_router = Some(parse_address(&env::var("INTEGRATION_ROUTER_ADDRESS")?)?);
        addresses.kyc_registry = Some(parse_address(&env::var("KYC_REGISTRY_ADDRESS")?)?);
        addresses.istsi_token = Some(parse_address(&env::var("ISTSI_TOKEN_ADDRESS")?)?);
        addresses.reserve_manager = Some(parse_address(&env::var("RESERVE_MANAGER_ADDRESS")?)?);
        
        Ok(Config {
            network,
            addresses,
            admin_secret: env::var("ADMIN_SECRET_KEY")?,
            monitoring_enabled: env::var("EVENT_MONITORING_ENABLED")?.parse()?,
        })
    }
}

fn parse_address(addr_str: &str) -> Result<Address, Box<dyn std::error::Error>> {
    Ok(Address::from_string(&soroban_sdk::String::from_str(&soroban_sdk::Env::default(), addr_str)))
}
```

## Contract Client Usage

### Bitcoin Deposit Operations

```rust
use soroban_client::ContractManager;

impl BitcoinDepositService {
    pub async fn process_deposit(
        &self,
        user_address: Address,
        btc_tx_hash: BytesN<32>,
        btc_amount: u64,
        confirmations: u32,
        block_height: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        // 1. Validate input parameters
        if btc_amount == 0 {
            return Err("Invalid Bitcoin amount".into());
        }
        
        if confirmations < self.config.network.min_confirmations {
            return Err("Insufficient confirmations".into());
        }
        
        // 2. Check user KYC status
        let kyc_tier = self.manager.kyc_registry().get_tier_code_by_address(&user_address)?;
        if kyc_tier < 2 { // Require at least Verified tier
            return Err("Insufficient KYC tier for deposit".into());
        }
        
        // 3. Execute deposit workflow
        let ctx = OperationContext {
            caller: self.service_address.clone(),
            operation_id: format!("deposit_{}", uuid::Uuid::new_v4()),
            timeout_seconds: 60,
            retry_count: 3,
        };
        
        let operation_id = self.manager.execute_bitcoin_deposit_workflow(
            &ctx,
            &user_address,
            btc_amount,
            &btc_tx_hash,
            confirmations,
            block_height,
        )?;
        
        // 4. Store operation record
        self.store_operation_record(&operation_id, &user_address, btc_amount).await?;
        
        // 5. Return operation ID as string
        Ok(hex::encode(operation_id.to_array()))
    }
    
    async fn store_operation_record(
        &self,
        operation_id: &BytesN<32>,
        user_address: &Address,
        amount: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Store in your database
        let record = OperationRecord {
            id: hex::encode(operation_id.to_array()),
            user_address: user_address.to_string(),
            operation_type: "bitcoin_deposit".to_string(),
            amount,
            status: "pending".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        self.db.insert_operation_record(record).await?;
        Ok(())
    }
}
```

### Token Withdrawal Operations

```rust
impl TokenWithdrawalService {
    pub async fn process_withdrawal(
        &self,
        user_address: Address,
        istsi_amount: u64,
        btc_address: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        // 1. Validate Bitcoin address format
        self.validate_btc_address(&btc_address)?;
        
        // 2. Check user token balance
        let balance = self.manager.istsi_token().balance(&user_address)?;
        if balance < istsi_amount {
            return Err("Insufficient token balance".into());
        }
        
        // 3. Check withdrawal limits
        self.check_withdrawal_limits(&user_address, istsi_amount).await?;
        
        // 4. Execute withdrawal workflow
        let ctx = OperationContext {
            caller: self.service_address.clone(),
            operation_id: format!("withdrawal_{}", uuid::Uuid::new_v4()),
            timeout_seconds: 60,
            retry_count: 3,
        };
        
        let withdrawal_id = self.manager.execute_token_withdrawal_workflow(
            &ctx,
            &user_address,
            istsi_amount,
            &btc_address,
        )?;
        
        // 5. Store withdrawal record
        self.store_withdrawal_record(&withdrawal_id, &user_address, istsi_amount, &btc_address).await?;
        
        Ok(hex::encode(withdrawal_id.to_array()))
    }
    
    fn validate_btc_address(&self, address: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Implement Bitcoin address validation
        if address.len() < 26 || address.len() > 62 {
            return Err("Invalid Bitcoin address length".into());
        }
        
        // Add more validation logic as needed
        Ok(())
    }
    
    async fn check_withdrawal_limits(
        &self,
        user_address: &Address,
        amount: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check daily/monthly limits from your database
        let daily_total = self.db.get_daily_withdrawal_total(user_address).await?;
        let daily_limit = self.get_user_daily_limit(user_address).await?;
        
        if daily_total + amount > daily_limit {
            return Err("Daily withdrawal limit exceeded".into());
        }
        
        Ok(())
    }
}
```

### KYC Compliance Checks

```rust
impl ComplianceService {
    pub async fn check_operation_compliance(
        &self,
        user_address: &Address,
        operation_type: u32,
        amount: u64,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        
        // 1. Check if KYC registry is enabled
        if !self.manager.kyc_registry().is_registry_enabled()? {
            return Ok(true); // Allow all operations if KYC is disabled
        }
        
        // 2. Get user's KYC tier
        let user_tier = self.manager.kyc_registry().get_tier_code_by_address(user_address)?;
        
        // 3. Check operation approval
        let approved = self.manager.kyc_registry().is_approved_for_operation(
            user_address,
            operation_type,
            amount,
        )?;
        
        // 4. Log compliance check
        self.log_compliance_check(user_address, operation_type, amount, approved, user_tier).await?;
        
        Ok(approved)
    }
    
    pub async fn batch_compliance_check(
        &self,
        operations: Vec<(Address, u32, u64)>,
    ) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
        
        let results = self.manager.kyc_registry().batch_compliance_check(&operations)?;
        
        // Log batch results
        for ((address, op_type, amount), approved) in operations.iter().zip(results.iter()) {
            self.log_compliance_check(address, *op_type, *amount, *approved, 0).await?;
        }
        
        Ok(results)
    }
    
    async fn log_compliance_check(
        &self,
        user_address: &Address,
        operation_type: u32,
        amount: u64,
        approved: bool,
        user_tier: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log_entry = ComplianceLog {
            user_address: user_address.to_string(),
            operation_type,
            amount,
            approved,
            user_tier,
            timestamp: chrono::Utc::now(),
        };
        
        self.db.insert_compliance_log(log_entry).await?;
        Ok(())
    }
}
```

## Event Monitoring

### Setting Up Event Monitoring

```rust
use soroban_client::{EventMonitor, EventFilter, ContractEvent, EventData};

pub struct EventMonitoringService {
    monitor: EventMonitor,
    db: Database,
}

impl EventMonitoringService {
    pub fn new(env: Env, db: Database) -> Self {
        let monitor = EventMonitor::new(env);
        Self { monitor, db }
    }
    
    pub async fn start_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Subscribe to Bitcoin deposit events
        self.monitor.subscribe(
            "bitcoin_deposits".to_string(),
            EventFilter::new()
                .for_event_types(vec!["btc_dep".to_string()])
                .limit(100),
            |event| self.handle_bitcoin_deposit_event(event),
        )?;
        
        // Subscribe to token withdrawal events
        self.monitor.subscribe(
            "token_withdrawals".to_string(),
            EventFilter::new()
                .for_event_types(vec!["tok_with".to_string()])
                .limit(100),
            |event| self.handle_token_withdrawal_event(event),
        )?;
        
        // Subscribe to compliance events
        self.monitor.subscribe(
            "compliance_checks".to_string(),
            EventFilter::new()
                .for_event_types(vec!["kyc_chk".to_string()])
                .limit(100),
            |event| self.handle_compliance_event(event),
        )?;
        
        // Start event polling loop
        self.start_event_loop().await
    }
    
    async fn start_event_loop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Fetch events from the network
            let events = self.fetch_recent_events().await?;
            
            // Process events through monitor
            let processed_count = self.monitor.process_events(events)?;
            
            if processed_count > 0 {
                println!("Processed {} events", processed_count);
            }
        }
    }
    
    async fn fetch_recent_events(&self) -> Result<Vec<ContractEvent>, Box<dyn std::error::Error>> {
        // In a real implementation, this would fetch events from the Soroban RPC
        // For now, return empty vector
        Ok(Vec::new())
    }
    
    fn handle_bitcoin_deposit_event(&self, event: &ContractEvent) -> Result<(), Box<dyn std::error::Error>> {
        if let EventData::BitcoinDeposit { user, btc_amount, istsi_amount, btc_tx_hash, confirmations } = &event.data {
            println!("Bitcoin deposit event: {} BTC -> {} iSTSi for user {}", 
                     btc_amount, istsi_amount, user);
            
            // Update database with deposit status
            tokio::spawn({
                let db = self.db.clone();
                let user = user.clone();
                let btc_amount = *btc_amount;
                let istsi_amount = *istsi_amount;
                
                async move {
                    if let Err(e) = db.update_deposit_status(&user, btc_amount, istsi_amount, "completed").await {
                        eprintln!("Failed to update deposit status: {}", e);
                    }
                }
            });
        }
        
        Ok(())
    }
    
    fn handle_token_withdrawal_event(&self, event: &ContractEvent) -> Result<(), Box<dyn std::error::Error>> {
        if let EventData::TokenWithdrawal { user, istsi_amount, btc_amount, withdrawal_id, btc_address } = &event.data {
            println!("Token withdrawal event: {} iSTSi -> {} BTC for user {}", 
                     istsi_amount, btc_amount, user);
            
            // Trigger Bitcoin transaction processing
            tokio::spawn({
                let withdrawal_id = withdrawal_id.clone();
                let btc_amount = *btc_amount;
                let btc_address = btc_address.clone();
                
                async move {
                    // Process Bitcoin withdrawal transaction
                    if let Err(e) = process_bitcoin_withdrawal(withdrawal_id, btc_amount, btc_address).await {
                        eprintln!("Failed to process Bitcoin withdrawal: {}", e);
                    }
                }
            });
        }
        
        Ok(())
    }
    
    fn handle_compliance_event(&self, event: &ContractEvent) -> Result<(), Box<dyn std::error::Error>> {
        if let EventData::ComplianceCheck { user, operation_type, amount, approved, .. } = &event.data {
            println!("Compliance check: user {} operation {} amount {} approved {}", 
                     user, operation_type, amount, approved);
            
            // Log compliance event
            tokio::spawn({
                let db = self.db.clone();
                let user = user.clone();
                let operation_type = *operation_type;
                let amount = *amount;
                let approved = *approved;
                
                async move {
                    if let Err(e) = db.log_compliance_event(&user, operation_type, amount, approved).await {
                        eprintln!("Failed to log compliance event: {}", e);
                    }
                }
            });
        }
        
        Ok(())
    }
}

async fn process_bitcoin_withdrawal(
    withdrawal_id: BytesN<32>,
    amount: u64,
    btc_address: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Implement Bitcoin transaction creation and broadcasting
    println!("Processing Bitcoin withdrawal: {} satoshis to {}", amount, btc_address);
    Ok(())
}
```

## Error Handling

### Comprehensive Error Handling

```rust
use soroban_client::{ContractError, IntegrationError};

#[derive(Debug)]
pub enum ServiceError {
    Contract(ContractError),
    Database(sqlx::Error),
    Network(reqwest::Error),
    Validation(String),
    Configuration(String),
}

impl From<ContractError> for ServiceError {
    fn from(err: ContractError) -> Self {
        ServiceError::Contract(err)
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::Contract(e) => write!(f, "Contract error: {:?}", e),
            ServiceError::Database(e) => write!(f, "Database error: {}", e),
            ServiceError::Network(e) => write!(f, "Network error: {}", e),
            ServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

pub async fn handle_operation_with_retry<F, T>(
    operation: F,
    max_retries: u32,
    backoff_seconds: u64,
) -> Result<T, ServiceError>
where
    F: Fn() -> Result<T, ServiceError>,
{
    let mut attempts = 0;
    
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(ServiceError::Contract(ContractError::NetworkError(_))) if attempts < max_retries => {
                attempts += 1;
                println!("Network error, retrying in {} seconds (attempt {}/{})", 
                         backoff_seconds, attempts, max_retries);
                tokio::time::sleep(tokio::time::Duration::from_secs(backoff_seconds)).await;
            }
            Err(ServiceError::Contract(ContractError::Timeout(_))) if attempts < max_retries => {
                attempts += 1;
                println!("Timeout error, retrying in {} seconds (attempt {}/{})", 
                         backoff_seconds, attempts, max_retries);
                tokio::time::sleep(tokio::time::Duration::from_secs(backoff_seconds)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

// Usage example
pub async fn execute_deposit_with_retry(
    manager: &ContractManager,
    ctx: &OperationContext,
    user: &Address,
    amount: u64,
    tx_hash: &BytesN<32>,
    confirmations: u32,
    block_height: u64,
) -> Result<BytesN<32>, ServiceError> {
    handle_operation_with_retry(
        || {
            manager.execute_bitcoin_deposit_workflow(
                ctx, user, amount, tx_hash, confirmations, block_height
            ).map_err(ServiceError::from)
        },
        3, // max retries
        2, // backoff seconds
    ).await
}
```

## Best Practices

### 1. Connection Management

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ContractConnectionPool {
    managers: Arc<RwLock<Vec<ContractManager>>>,
    config: Config,
}

impl ContractConnectionPool {
    pub fn new(config: Config, pool_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut managers = Vec::with_capacity(pool_size);
        
        for _ in 0..pool_size {
            let env = Env::default();
            let manager = ContractManager::new(
                env,
                config.addresses.clone(),
                config.network.clone(),
            )?;
            managers.push(manager);
        }
        
        Ok(Self {
            managers: Arc::new(RwLock::new(managers)),
            config,
        })
    }
    
    pub async fn get_manager(&self) -> Result<ContractManager, Box<dyn std::error::Error>> {
        let mut managers = self.managers.write().await;
        
        if let Some(manager) = managers.pop() {
            Ok(manager)
        } else {
            // Create new manager if pool is empty
            let env = Env::default();
            ContractManager::new(
                env,
                self.config.addresses.clone(),
                self.config.network.clone(),
            ).map_err(|e| e.into())
        }
    }
    
    pub async fn return_manager(&self, manager: ContractManager) {
        let mut managers = self.managers.write().await;
        managers.push(manager);
    }
}
```

### 2. Rate Limiting

```rust
use tokio::sync::Semaphore;
use std::time::Duration;

pub struct RateLimitedService {
    semaphore: Arc<Semaphore>,
    rate_limit: Duration,
}

impl RateLimitedService {
    pub fn new(max_concurrent: usize, rate_limit_ms: u64) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            rate_limit: Duration::from_millis(rate_limit_ms),
        }
    }
    
    pub async fn execute_with_rate_limit<F, T>(&self, operation: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let _permit = self.semaphore.acquire().await?;
        
        let result = operation.await;
        
        // Rate limiting delay
        tokio::time::sleep(self.rate_limit).await;
        
        result
    }
}
```

### 3. Health Monitoring

```rust
pub struct HealthMonitor {
    manager: ContractManager,
    last_check: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
    health_status: Arc<RwLock<SystemHealth>>,
}

impl HealthMonitor {
    pub async fn start_health_checks(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            match self.manager.check_system_health() {
                Ok(health) => {
                    *self.health_status.write().await = health;
                    *self.last_check.write().await = Some(chrono::Utc::now());
                }
                Err(e) => {
                    eprintln!("Health check failed: {:?}", e);
                }
            }
        }
    }
    
    pub async fn is_healthy(&self) -> bool {
        let health = self.health_status.read().await;
        health.integration_router_available &&
        health.kyc_registry_available &&
        health.istsi_token_available &&
        health.reserve_manager_available &&
        !health.system_paused &&
        health.reserve_ratio_healthy
    }
}
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Contract Not Found Errors

```rust
// Problem: Contract address not found or invalid
// Solution: Verify contract addresses in configuration

match manager.integration_router().is_available() {
    true => println!("Integration router is available"),
    false => {
        eprintln!("Integration router not available. Check contract address.");
        // Verify address in config file
        let addresses = load_contract_addresses("testnet")?;
        println!("Current integration router address: {:?}", 
                 addresses.integration_router);
    }
}
```

#### 2. Network Connection Issues

```rust
// Problem: RPC connection failures
// Solution: Implement connection retry with exponential backoff

async fn connect_with_retry(rpc_url: &str, max_retries: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut backoff = 1;
    
    for attempt in 1..=max_retries {
        match test_connection(rpc_url).await {
            Ok(()) => return Ok(()),
            Err(e) if attempt < max_retries => {
                eprintln!("Connection attempt {} failed: {}. Retrying in {} seconds...", 
                         attempt, e, backoff);
                tokio::time::sleep(Duration::from_secs(backoff)).await;
                backoff *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    
    Err("Max retries exceeded".into())
}
```

#### 3. Event Processing Delays

```rust
// Problem: Events are processed with significant delay
// Solution: Optimize event polling and processing

pub struct OptimizedEventProcessor {
    last_processed_block: Arc<RwLock<u64>>,
    processing_queue: Arc<Mutex<VecDeque<ContractEvent>>>,
}

impl OptimizedEventProcessor {
    pub async fn process_events_efficiently(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Process events in parallel batches
        let events = self.fetch_events_since_last_block().await?;
        
        let batch_size = 50;
        let batches: Vec<_> = events.chunks(batch_size).collect();
        
        let mut handles = Vec::new();
        
        for batch in batches {
            let batch = batch.to_vec();
            let handle = tokio::spawn(async move {
                for event in batch {
                    if let Err(e) = process_single_event(&event).await {
                        eprintln!("Failed to process event: {:?}", e);
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all batches to complete
        for handle in handles {
            handle.await?;
        }
        
        Ok(())
    }
}
```

#### 4. Memory Usage Optimization

```rust
// Problem: High memory usage from event storage
// Solution: Implement event cleanup and archival

pub struct EventArchiver {
    db: Database,
    retention_days: u32,
}

impl EventArchiver {
    pub async fn cleanup_old_events(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(self.retention_days as i64);
        
        // Archive old events to cold storage
        let old_events = self.db.get_events_before(cutoff_date).await?;
        
        if !old_events.is_empty() {
            self.archive_events_to_storage(old_events).await?;
            self.db.delete_events_before(cutoff_date).await?;
            
            println!("Archived and cleaned up events older than {} days", self.retention_days);
        }
        
        Ok(())
    }
}
```

### Debugging Tools

```rust
// Enable debug logging for contract interactions
pub fn enable_debug_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
}

// Contract interaction tracer
pub struct ContractTracer {
    trace_file: std::fs::File,
}

impl ContractTracer {
    pub fn trace_call(&mut self, contract: &str, function: &str, params: &str) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        writeln!(self.trace_file, "[{}] {}::{} - {}", timestamp, contract, function, params).ok();
    }
}
```

This integration guide provides a comprehensive foundation for working with the Soroban contracts. For additional support, refer to the contract ABI documentation and the Soroban SDK documentation.