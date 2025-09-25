# iSHSi Development Environment Setup

## Prerequisites

1. **Rust Installation**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Add wasm32 target
   rustup target add wasm32-unknown-unknown
   
   # Install Rust nightly (required for Soroban)
   rustup toolchain install nightly
   rustup target add wasm32-unknown-unknown --toolchain nightly
   ```

2. **Soroban CLI Installation**
   ```bash
   # Install Soroban CLI
   cargo install --locked --version 0.9.4 soroban-cli
   
   # Verify installation
   soroban --version
   ```

3. **Stellar Testnet Account**
   - Create a testnet account at [Stellar Laboratory](https://laboratory.stellar.org/#account-creator?network=test)
   - Fund your account with test XLM from the friendbot

4. **Environment Variables**
   Create a `.env` file in the project root:
   ```
   # Testnet
   STELLAR_NETWORK=testnet
   STELLAR_SECRET_KEY=your_testnet_secret_key_here
   
   # Local development
   # STELLAR_NETWORK=standalone
   # STELLAR_SECRET_KEY=your_local_secret_key_here
   ```

## Building the Contracts

1. **Build All Contracts**
   ```bash
   cd soroban
   cargo build --target wasm32-unknown-unknown --release
   ```

2. **Build Individual Contracts**
   ```bash
   # Build specific contract
   cd soroban/contracts/istsi_token
   cargo build --target wasm32-unknown-unknown --release
   
   # Build KYC Registry
   cd ../kyc_registry
   cargo build --target wasm32-unknown-unknown --release
   ```

## Testing

Run the test suite:

```bash
# From soroban directory - run all contract tests
cd soroban
cargo test

# For verbose output
cargo test -- --nocapture

# Test specific contract
cd contracts/istsi_token
cargo test
```

## Deployment

1. **Deploy to Testnet**
   ```bash
   # Deploy iSTSi token (from soroban directory)
   soroban contract deploy \
     --wasm target/wasm32-unknown-unknown/release/istsi_token.wasm \
     --source <YOUR_SECRET_KEY> \
     --network testnet
   ```

2. **Initialize the Contract**
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source <YOUR_SECRET_KEY> \
     --network testnet \
     -- initialize \
     --admin <ADMIN_ADDRESS> \
     --decimal 8 \
     --name "iSatoshi" \
     --symbol "iSHSi"
   ```

## Development Workflow

1. Make code changes
2. Run tests: `cargo test`
3. Build: `cargo build --target wasm32-unknown-unknown --release`
4. Test locally: `soroban contract invoke`
5. Deploy to testnet when ready

## Useful Commands

- Check contract metadata:
  ```bash
  soroban contract meta --wasm target/wasm32-unknown-unknown/release/istsi_token.wasm
  ```

- Get contract details:
  ```bash
  soroban contract read --id <CONTRACT_ID> --network testnet
  ```

- Call contract methods:
  ```bash
  soroban contract invoke \
    --id <CONTRACT_ID> \
    --source <YOUR_SECRET_KEY> \
    --network testnet \
    -- <METHOD_NAME> \
    --<ARG1> <VALUE1> \
    --<ARG2> <VALUE2>
  ```
