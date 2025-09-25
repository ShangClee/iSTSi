# Bitcoin Custody Soroban Contracts

A comprehensive suite of Soroban smart contracts for Bitcoin custody operations, including integration routing, KYC compliance, token management, and reserve operations on the Stellar network.

## 🏗️ Architecture

This contract suite provides:
- **Integration Router** - Central coordination for all custody operations
- **KYC Registry** - Compliance and identity verification management
- **iSTSi Token** - Bitcoin-backed token with integrated compliance
- **Reserve Manager** - Bitcoin reserve tracking and proof generation
- **Fungible Token** - Standard token implementation for additional assets

## 📁 Project Structure

```
soroban/
├── contracts/                    # Smart contract implementations
│   ├── integration-router/       # Main integration coordination contract
│   │   ├── src/
│   │   │   ├── lib.rs           # Main contract logic
│   │   │   ├── storage.rs       # Contract storage definitions
│   │   │   ├── events.rs        # Contract events
│   │   │   └── operations.rs    # Core operations
│   │   └── Cargo.toml
│   ├── kyc-registry/            # KYC compliance contract
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   ├── istsi-token/             # Bitcoin-backed token contract
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   ├── reserve-manager/         # Reserve management contract
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   └── fungible-token/          # Standard fungible token
│       ├── src/
│       │   ├── lib.rs
│       │   ├── contract.rs
│       │   └── test.rs
│       └── Cargo.toml
├── shared/                      # Shared utilities and types
│   ├── src/
│   │   ├── lib.rs              # Shared library exports
│   │   ├── types.rs            # Common data types
│   │   ├── events.rs           # Shared event definitions
│   │   ├── errors.rs           # Error types and handling
│   │   └── utils.rs            # Utility functions
│   └── Cargo.toml
├── client/                      # Contract client interfaces
│   ├── lib.rs                  # Client library exports
│   ├── integration_router_client.rs
│   ├── kyc_registry_client.rs
│   ├── istsi_token_client.rs
│   ├── reserve_manager_client.rs
│   └── Cargo.toml
├── tests/                       # Integration tests
│   ├── integration_test.rs      # Full integration test suite
│   ├── bitcoin_deposit_test.rs  # Bitcoin deposit flow tests
│   └── cross_token_test.rs      # Cross-contract interaction tests
├── scripts/                     # Deployment and utility scripts
│   ├── build.sh                # Build all contracts
│   ├── deploy-testnet.sh       # Deploy to testnet
│   ├── deploy-mainnet.sh       # Deploy to mainnet
│   ├── test.sh                 # Run all tests
│   └── config-manager.sh       # Contract configuration management
├── config/                      # Configuration files
│   ├── contract_addresses.json  # Deployed contract addresses
│   ├── network_config.toml     # Network configurations
│   └── environments/           # Environment-specific configs
├── docs/                        # Additional documentation
│   ├── contract_abis.md        # Contract ABI documentation
│   └── integration_guide.md    # Integration guide for developers
├── Cargo.toml                   # Workspace configuration
├── .env.example                 # Environment variables template
└── soroban-project.toml        # Soroban project configuration
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Soroban CLI 21.0+
- Stellar account with testnet XLM (for testing)
- Node.js 18+ (for some utility scripts)

### Installation

1. **Install Soroban CLI:**
   ```bash
   cargo install --locked soroban-cli
   ```

2. **Add WebAssembly target:**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Navigate to soroban directory:**
   ```bash
   cd soroban
   ```

4. **Set up environment variables:**
   ```bash
   cp .env.example .env
   ```
   
   Edit `.env` with your configuration:
   ```env
   SOROBAN_NETWORK=testnet
   SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
   SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
   SOROBAN_ACCOUNT=your-stellar-secret-key
   ```

5. **Build all contracts:**
   ```bash
   ./scripts/build.sh
   ```

6. **Run tests:**
   ```bash
   ./scripts/test.sh
   ```

## 🛠️ Development

### Building Contracts

**Build all contracts:**
```bash
# Using provided script
./scripts/build.sh

# Or manually
cargo build --target wasm32-unknown-unknown --release
```

**Build specific contract:**
```bash
# Build integration router
cd contracts/integration-router
cargo build --target wasm32-unknown-unknown --release

# Build KYC registry
cd contracts/kyc-registry
cargo build --target wasm32-unknown-unknown --release
```

### Testing Contracts

**Run all tests:**
```bash
# Using provided script
./scripts/test.sh

# Or manually
cargo test
```

**Run specific contract tests:**
```bash
# Test integration router
cd contracts/integration-router
cargo test

# Test with output
cargo test -- --nocapture
```

**Run integration tests:**
```bash
# Full integration test suite
cargo test --test integration_test

# Bitcoin deposit flow tests
cargo test --test bitcoin_deposit_test
```

### Development Workflow

1. **Contract Development:**
   - Implement contract logic in `src/lib.rs`
   - Define storage structures in `storage.rs`
   - Add events in `events.rs`
   - Write comprehensive tests

2. **Testing Strategy:**
   - Unit tests within each contract
   - Integration tests in `/tests` directory
   - End-to-end testing with full deployment

3. **Code Quality:**
   - Use `cargo fmt` for consistent formatting
   - Run `cargo clippy` for linting
   - Follow Soroban best practices

## 📋 Contract Specifications

### 1. Integration Router Contract

**Purpose:** Central coordination hub for all Bitcoin custody operations.

**Key Functions:**
```rust
// Initialize the integration system
pub fn initialize(env: Env, admin: Address, config: IntegrationConfig) -> Result<(), Error>

// Execute Bitcoin deposit with KYC compliance
pub fn execute_bitcoin_deposit(
    env: Env,
    user: Address,
    btc_amount: u64,
    btc_tx_hash: String,
) -> Result<String, Error>

// Execute token withdrawal to Bitcoin
pub fn execute_token_withdrawal(
    env: Env,
    user: Address,
    token_amount: u64,
    btc_address: String,
) -> Result<String, Error>

// Get system status and metrics
pub fn get_system_status(env: Env) -> SystemStatus
```

**Events:**
```rust
#[contractevent]
pub struct BitcoinDepositExecuted {
    pub user: Address,
    pub btc_amount: u64,
    pub token_amount: u64,
    pub btc_tx_hash: String,
    pub stellar_tx_id: String,
}

#[contractevent]
pub struct TokenWithdrawalExecuted {
    pub user: Address,
    pub token_amount: u64,
    pub btc_amount: u64,
    pub btc_address: String,
    pub withdrawal_id: String,
}
```

### 2. KYC Registry Contract

**Purpose:** Manage user identity verification and compliance tiers.

**Key Functions:**
```rust
// Register new user with KYC information
pub fn register_user(
    env: Env,
    user: Address,
    kyc_data: KycData,
) -> Result<(), Error>

// Update user KYC tier
pub fn update_tier(
    env: Env,
    user: Address,
    new_tier: u32,
) -> Result<(), Error>

// Check if user is approved for operation
pub fn is_approved(
    env: Env,
    user: Address,
    operation_type: OperationType,
    amount: u64,
) -> bool

// Get user KYC status
pub fn get_kyc_status(env: Env, user: Address) -> KycStatus
```

**Compliance Tiers:**
- **Tier 0:** Unverified (no operations allowed)
- **Tier 1:** Basic verification (limited amounts)
- **Tier 2:** Enhanced verification (higher limits)
- **Tier 3:** Institutional verification (unlimited)

### 3. iSTSi Token Contract

**Purpose:** Bitcoin-backed token with integrated compliance checks.

**Key Functions:**
```rust
// Mint tokens (only via integration router)
pub fn mint(env: Env, to: Address, amount: u64) -> Result<(), Error>

// Burn tokens for Bitcoin withdrawal
pub fn burn(env: Env, from: Address, amount: u64) -> Result<(), Error>

// Transfer with compliance checks
pub fn transfer(
    env: Env,
    from: Address,
    to: Address,
    amount: u64,
) -> Result<(), Error>

// Get token balance
pub fn balance(env: Env, id: Address) -> u64

// Get total supply
pub fn total_supply(env: Env) -> u64
```

**Compliance Integration:**
- All transfers check KYC status
- Minting requires Bitcoin deposit verification
- Burning initiates Bitcoin withdrawal process

### 4. Reserve Manager Contract

**Purpose:** Track Bitcoin reserves and generate proofs of reserves.

**Key Functions:**
```rust
// Register Bitcoin deposit
pub fn register_deposit(
    env: Env,
    btc_tx_hash: String,
    amount: u64,
    confirmations: u32,
) -> Result<(), Error>

// Register Bitcoin withdrawal
pub fn register_withdrawal(
    env: Env,
    btc_tx_hash: String,
    amount: u64,
) -> Result<(), Error>

// Get current reserve ratio
pub fn get_reserve_ratio(env: Env) -> u64

// Generate proof of reserves
pub fn generate_proof(env: Env) -> ProofOfReserves

// Get reserve status
pub fn get_reserve_status(env: Env) -> ReserveStatus
```

**Reserve Monitoring:**
- Real-time reserve ratio calculation
- Automated alerts for low reserves
- Cryptographic proof generation
- Historical reserve tracking

## 🚀 Deployment

### Testnet Deployment

**Deploy all contracts to testnet:**
```bash
./scripts/deploy-testnet.sh
```

**Manual deployment steps:**
```bash
# 1. Configure Soroban for testnet
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# 2. Configure identity
soroban config identity generate deployer
soroban config identity fund deployer --network testnet

# 3. Deploy contracts in order
# Deploy KYC Registry first
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/kyc_registry.wasm \
  --source deployer \
  --network testnet

# Deploy Reserve Manager
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/reserve_manager.wasm \
  --source deployer \
  --network testnet

# Deploy iSTSi Token
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/istsi_token.wasm \
  --source deployer \
  --network testnet

# Deploy Integration Router (requires other contract addresses)
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/integration_router.wasm \
  --source deployer \
  --network testnet
```

**Initialize contracts:**
```bash
# Initialize KYC Registry
soroban contract invoke \
  --id $KYC_REGISTRY_ID \
  --source deployer \
  --network testnet \
  -- initialize \
  --admin $ADMIN_ADDRESS

# Initialize other contracts similarly...
```

### Mainnet Deployment

**Deploy to mainnet (production):**
```bash
# IMPORTANT: Test thoroughly on testnet first!
./scripts/deploy-mainnet.sh
```

**Mainnet deployment checklist:**
- [ ] All contracts tested on testnet
- [ ] Security audit completed
- [ ] Admin keys secured with multi-signature
- [ ] Backup and recovery procedures in place
- [ ] Monitoring and alerting configured

### Contract Addresses Management

**Update contract addresses:**
```bash
# After deployment, update config/contract_addresses.json
{
  "testnet": {
    "integration_router": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "kyc_registry": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "istsi_token": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "reserve_manager": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  },
  "mainnet": {
    "integration_router": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "kyc_registry": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "istsi_token": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "reserve_manager": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  }
}
```

## 🔗 Contract Interactions

### Using Soroban CLI

**Query contract state:**
```bash
# Get system status
soroban contract invoke \
  --id $INTEGRATION_ROUTER_ID \
  --source user \
  --network testnet \
  -- get_system_status

# Check user KYC status
soroban contract invoke \
  --id $KYC_REGISTRY_ID \
  --source user \
  --network testnet \
  -- get_kyc_status \
  --user $USER_ADDRESS

# Get token balance
soroban contract invoke \
  --id $ISTSI_TOKEN_ID \
  --source user \
  --network testnet \
  -- balance \
  --id $USER_ADDRESS
```

**Execute operations:**
```bash
# Execute Bitcoin deposit
soroban contract invoke \
  --id $INTEGRATION_ROUTER_ID \
  --source user \
  --network testnet \
  -- execute_bitcoin_deposit \
  --user $USER_ADDRESS \
  --btc_amount 100000000 \
  --btc_tx_hash "abc123..."

# Transfer tokens
soroban contract invoke \
  --id $ISTSI_TOKEN_ID \
  --source user \
  --network testnet \
  -- transfer \
  --from $FROM_ADDRESS \
  --to $TO_ADDRESS \
  --amount 50000000
```

### Using Client Libraries

**Rust client example:**
```rust
use soroban_client::integration_router_client::IntegrationRouterClient;

// Initialize client
let client = IntegrationRouterClient::new(
    &network_config,
    &contract_address,
)?;

// Execute Bitcoin deposit
let result = client
    .execute_bitcoin_deposit(
        &user_address,
        100_000_000, // 1 BTC in satoshis
        "bitcoin_tx_hash_here",
    )
    .await?;

println!("Deposit executed: {}", result);
```

**JavaScript client example:**
```javascript
import { SorobanRpc, Contract, Keypair } from '@stellar/stellar-sdk';

// Initialize contract
const contract = new Contract(contractAddress);
const server = new SorobanRpc.Server(rpcUrl);

// Execute operation
const operation = contract.call(
  'execute_bitcoin_deposit',
  userAddress,
  100000000,
  'bitcoin_tx_hash_here'
);

const transaction = new TransactionBuilder(account, { fee: '100' })
  .addOperation(operation)
  .setTimeout(30)
  .build();

// Sign and submit transaction
transaction.sign(userKeypair);
const result = await server.sendTransaction(transaction);
```

## 🧪 Testing

### Unit Testing

**Run contract unit tests:**
```bash
# Test all contracts
cargo test

# Test specific contract
cd contracts/integration-router
cargo test

# Test with coverage
cargo test --coverage
```

**Example unit test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_bitcoin_deposit() {
        let env = Env::default();
        let contract_id = env.register_contract(None, IntegrationRouter);
        let client = IntegrationRouterClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let admin = Address::generate(&env);

        // Initialize contract
        client.initialize(&admin, &default_config());

        // Execute deposit
        let result = client.execute_bitcoin_deposit(
            &user,
            &100_000_000,
            &String::from_str(&env, "test_tx_hash"),
        );

        assert!(result.is_ok());
    }
}
```

### Integration Testing

**Run integration tests:**
```bash
# Full integration test suite
cargo test --test integration_test

# Specific integration test
cargo test --test bitcoin_deposit_test -- --nocapture
```

**Example integration test:**
```rust
// tests/integration_test.rs
use soroban_sdk::{Env, Address};

#[test]
fn test_full_bitcoin_deposit_flow() {
    let env = Env::default();
    
    // Deploy all contracts
    let kyc_registry = deploy_kyc_registry(&env);
    let reserve_manager = deploy_reserve_manager(&env);
    let istsi_token = deploy_istsi_token(&env);
    let integration_router = deploy_integration_router(&env);

    // Initialize contracts with proper configuration
    initialize_contracts(&env, &kyc_registry, &reserve_manager, &istsi_token, &integration_router);

    // Register user with KYC
    let user = Address::generate(&env);
    register_user_kyc(&env, &kyc_registry, &user);

    // Execute Bitcoin deposit
    let deposit_result = execute_bitcoin_deposit(&env, &integration_router, &user);
    
    // Verify results
    assert!(deposit_result.is_ok());
    verify_token_balance(&env, &istsi_token, &user, 100_000_000);
    verify_reserve_update(&env, &reserve_manager, 100_000_000);
}
```

### End-to-End Testing

**Test with live network:**
```bash
# Deploy to testnet and run E2E tests
./scripts/deploy-testnet.sh
./scripts/test-e2e.sh
```

## 🔧 Troubleshooting

### Common Issues

**1. Build Failures**
```bash
# Clear build cache
cargo clean

# Update dependencies
cargo update

# Check Rust toolchain
rustup show

# Ensure wasm target is installed
rustup target add wasm32-unknown-unknown
```

**2. Deployment Issues**
```bash
# Check Soroban CLI version
soroban --version

# Verify network configuration
soroban config network ls

# Check account funding
soroban config identity address deployer
# Fund account at https://laboratory.stellar.org/#account-creator
```

**3. Contract Interaction Issues**
```bash
# Verify contract is deployed
soroban contract id wasm-hash $WASM_HASH --network testnet

# Check contract state
soroban contract invoke --id $CONTRACT_ID --source user --network testnet -- get_admin

# Verify transaction submission
soroban contract invoke --id $CONTRACT_ID --source user --network testnet --simulate-only -- function_name
```

**4. Testing Issues**
```bash
# Run tests with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Check test dependencies
cargo tree
```

### Performance Optimization

**Contract Size Optimization:**
```bash
# Build with optimizations
cargo build --target wasm32-unknown-unknown --release

# Check contract size
ls -la target/wasm32-unknown-unknown/release/*.wasm

# Use wasm-opt for further optimization (if available)
wasm-opt -Oz input.wasm -o output.wasm
```

**Gas Optimization:**
- Minimize storage operations
- Use efficient data structures
- Batch operations when possible
- Optimize contract call patterns

### Debugging

**Contract Debugging:**
```rust
// Use debug prints in tests
use soroban_sdk::log;

log!(&env, "Debug message: {}", value);
```

**Transaction Debugging:**
```bash
# Simulate transaction to check for errors
soroban contract invoke \
  --id $CONTRACT_ID \
  --source user \
  --network testnet \
  --simulate-only \
  -- function_name --param value
```

## 📚 Additional Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Developer Portal](https://developers.stellar.org/)
- [Soroban Examples](https://github.com/stellar/soroban-examples)
- [Rust Programming Language](https://doc.rust-lang.org/book/)
- [WebAssembly Documentation](https://webassembly.org/)

## 🔐 Security Considerations

### Smart Contract Security

**Access Control:**
- Implement proper admin controls
- Use multi-signature for critical operations
- Validate all input parameters
- Implement emergency pause functionality

**State Management:**
- Protect against reentrancy attacks
- Validate state transitions
- Use atomic operations where necessary
- Implement proper error handling

**Upgrade Strategy:**
- Plan for contract upgrades
- Implement proxy patterns if needed
- Maintain backward compatibility
- Test upgrade procedures thoroughly

### Audit Checklist

- [ ] All functions have proper access controls
- [ ] Input validation is comprehensive
- [ ] State changes are atomic and consistent
- [ ] Error handling covers all edge cases
- [ ] Events are emitted for all state changes
- [ ] Gas usage is optimized
- [ ] Contract size is within limits
- [ ] Integration tests cover all scenarios

## 🤝 Contributing

1. Follow Rust and Soroban coding conventions
2. Write comprehensive tests for all new functionality
3. Update documentation for contract changes
4. Use proper error handling and validation
5. Follow the established project structure
6. Test on testnet before proposing mainnet changes

For questions or issues, refer to the main project documentation or create an issue in the project repository.