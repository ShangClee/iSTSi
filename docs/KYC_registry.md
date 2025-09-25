# iSHSi KYC Registry Documentation

**Version**: 1.0  
**Date**: January 6, 2025  
**Contract Path**: `contracts/kyc_registry/src/lib.rs`  
**Purpose**: Compliance and KYC management for iSHSi Bitcoin Anchor Service  

## Overview

The KYC Registry is a Soroban smart contract that manages Know Your Customer (KYC) compliance for the iSHSi Bitcoin Anchor Service. It provides tiered access control, operation limits, and comprehensive audit trails for regulatory compliance.

## Architecture

### Core Components

1. **Customer Management** - Registration and tier management
2. **Access Control** - Admin and compliance officer authentication  
3. **Operation Approval** - Transaction compliance checking
4. **Limit Enforcement** - Tiered transaction limits
5. **Audit Framework** - Compliance logging and tracking

### Data Models

#### KYC Tiers
```rust
pub enum KYCTier {
    None,           // No KYC - limited operations
    Basic,          // Basic verification - small amounts
    Verified,       // Full KYC - standard operations  
    Enhanced,       // Enhanced due diligence - large amounts
    Institutional,  // Institutional KYC - unlimited
}
```

#### Operation Types
```rust
pub enum OperationType {
    Transfer,       // Token transfers
    Mint,          // Token minting
    Burn,          // Token burning
    Deposit,       // BTC/fiat deposits
    Withdraw,      // BTC/fiat withdrawals
    Exchange,      // Cross-token exchanges
}
```

#### Customer Record
```rust
pub struct CustomerRecord {
    pub customer_id: String,        // Hashed customer ID
    pub kyc_tier: KYCTier,         // Current KYC tier
    pub approved_addresses: Vec<Address>, // Approved wallet addresses
    pub jurisdiction: String,       // Customer jurisdiction
    pub created_at: u64,           // Registration timestamp
    pub updated_at: u64,           // Last update timestamp
    pub expires_at: u64,           // KYC expiration (0 = no expiration)
    pub sanctions_cleared: bool,    // Sanctions screening status
    pub metadata: Map<String, String>, // Additional metadata
}
```

#### Operation Limits
```rust
pub struct OperationLimits {
    pub daily_limit: i128,         // Daily operation limit (satoshis)
    pub monthly_limit: i128,       // Monthly operation limit (satoshis)
    pub single_tx_limit: i128,     // Single transaction limit (satoshis)
    pub enabled: bool,             // Operation enabled/disabled
}
```

## Default Tier Limits

All limits are in satoshis (1 BTC = 100,000,000 satoshis):

| Tier | Single TX | Daily | Monthly | Notes |
|------|-----------|-------|---------|-------|
| None | 0 | 0 | 0 | All operations disabled |
| Basic | 0.01 BTC | 0.05 BTC | 0.5 BTC | Small amounts only |
| Verified | 0.1 BTC | 0.5 BTC | 5 BTC | Standard operations |
| Enhanced | 1 BTC | 5 BTC | 50 BTC | Large amounts |
| Institutional | Unlimited | Unlimited | Unlimited | No limits |

## Core Functions

### Administrative Functions

#### `initialize(admin: Address)`
- Initializes the KYC registry
- Sets the admin address
- Creates default settings and tier limits
- **Access**: Public (one-time only)

#### `register_customer(caller, customer_id, kyc_tier, addresses, jurisdiction, metadata)`
- Registers a new customer with KYC information
- Creates address-to-customer mappings
- **Access**: Admin only
- **Events**: Emits `kyc_reg` event

#### `update_customer_tier(caller, customer_id, new_tier, notes)`
- Updates a customer's KYC tier
- Requires audit notes for compliance
- **Access**: Admin only
- **Events**: Emits `kyc_tier` event

### Address Management

#### `add_approved_address(caller, customer_id, address)`
- Adds a new approved address to a customer
- Creates address-to-customer mapping
- **Access**: Admin only
- **Events**: Emits `kyc_addr` event

#### `remove_approved_address(caller, customer_id, address)`
- Removes an approved address from a customer
- Removes address-to-customer mapping
- **Access**: Admin only
- **Events**: Emits `kyc_addr` event

### Compliance Functions

#### `is_approved_for_operation(address, operation, amount) -> bool`
- Core compliance checking function
- Validates KYC status, tier requirements, and limits
- **Access**: Public (read-only)
- **Returns**: `true` if operation is approved

### Query Functions

#### `get_customer_record(customer_id) -> Option<CustomerRecord>`
- Retrieves customer KYC record
- **Access**: Public

#### `get_customer_by_address(address) -> Option<String>`
- Gets customer ID by address
- **Access**: Public

#### `get_global_settings() -> GlobalSettings`
- Returns registry configuration
- **Access**: Public

## Operation Approval Logic

The `is_approved_for_operation` function performs these checks:

1. **Registry Status**: Ensure KYC registry is enabled
2. **Address Mapping**: Verify address is registered to a customer
3. **Customer Exists**: Confirm customer record exists
4. **KYC Expiration**: Check if KYC has expired
5. **Sanctions Check**: Verify sanctions clearance (if required)
6. **Tier Requirements**: Ensure customer tier meets operation requirements
7. **Operation Limits**: Validate single transaction, daily, and monthly limits
8. **Amount Validation**: Check transaction amount against limits

### Tier Requirements by Operation

| Operation | Minimum Tier |
|-----------|--------------|
| Transfer | Basic |
| Deposit | Basic |
| Mint | Verified |
| Burn | Verified |
| Withdraw | Verified |
| Exchange | Enhanced |

## Access Control

### Admin Functions
- Initialize registry
- Register customers
- Update customer tiers
- Manage approved addresses
- Modify global settings

### Compliance Officer Functions
- Same as Admin (currently)
- Future: Limited to customer management only

### Public Functions
- Query customer records
- Check operation approval
- View global settings

## Integration Guide

### For Token Contracts

```rust
// Example integration in iSTSi token contract
use kyc_registry::KYCRegistryClient;

pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    // ... existing logic ...
    
    // KYC compliance check
    let kyc_registry = KYCRegistryClient::new(&env, &KYC_REGISTRY_ADDRESS);
    
    if !kyc_registry.is_approved_for_operation(&from, &OperationType::Transfer, &amount) {
        panic!("Transfer not approved by KYC registry");
    }
    
    // ... continue with transfer ...
}
```

### For Frontend Applications

```typescript
// Example frontend integration
const kycRegistry = new Contract({
  contractId: KYC_REGISTRY_CONTRACT_ID,
  networkPassphrase: Networks.TESTNET,
  rpc: new SorobanRpc.Server(RPC_URL),
});

// Check if user can perform operation
const isApproved = await kycRegistry.call(
  'is_approved_for_operation',
  userAddress,
  'Transfer',
  amountInSatoshis
);

if (!isApproved.result) {
  throw new Error('Transaction not approved by KYC system');
}
```

## Events

### Customer Registration
```
Topic: ["kyc_reg", customer_id]
Data: [kyc_tier, address_count]
```

### Tier Updates
```
Topic: ["kyc_tier", customer_id]
Data: [old_tier, new_tier]
```

### Address Management
```
Topic: ["kyc_addr", customer_id]
Data: ["added"/"removed", address]
```

### Audit Events
```
Topic: ["kyc_audit", action]
Data: [customer_id, new_tier]
```

## Security Considerations

### Access Control
- All administrative functions require caller authentication
- Admin address is stored securely in contract storage
- Compliance officers have same privileges as admin (future: to be restricted)

### Data Privacy
- Customer IDs should be hashed for privacy
- No PII is stored directly in the contract
- Jurisdiction stored as ISO country codes only

### Audit Trail
- All tier changes require audit notes
- Timestamps are recorded for all operations
- Events provide comprehensive activity log

## Testing

### Unit Tests
The contract includes comprehensive unit tests:

```bash
cd contracts/kyc_registry
cargo test
```

### Test Coverage
- Contract initialization
- Customer registration and management
- Tier updates and validation
- Address management
- Operation approval logic

## Deployment

### Prerequisites
1. Soroban CLI installed
2. Stellar account with XLM for fees
3. Network configuration (testnet/mainnet)

### Build Contract
```bash
cd contracts/kyc_registry
cargo build --target wasm32-unknown-unknown --release
```

### Deploy Contract
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/kyc_registry.wasm \
  --source ADMIN_SECRET_KEY \
  --network testnet
```

### Initialize Contract
```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source ADMIN_SECRET_KEY \
  --network testnet \
  -- initialize \
  --admin ADMIN_ADDRESS
```

## Configuration

### Global Settings
```rust
pub struct GlobalSettings {
    pub registry_enabled: bool,     // Global enable/disable
    pub strict_mode: bool,         // Strict compliance mode
    pub auto_expire_days: u64,     // Auto-expire KYC after N days
    pub sanctions_required: bool,   // Require sanctions clearance
    pub audit_enabled: bool,       // Enable audit logging
}
```

### Default Configuration
- Registry enabled: `true`
- Strict mode: `false`
- Auto-expire: `365` days
- Sanctions required: `true`
- Audit enabled: `true`

## Compliance Features

### Regulatory Support
- Multi-jurisdiction customer tracking
- Tiered KYC levels matching regulatory requirements
- Sanctions screening integration points
- Comprehensive audit trails

### AML Integration
- Transaction limit enforcement
- Pattern recognition support (future)
- Risk scoring framework (future)
- Regulatory reporting (future)

### Privacy Protection
- No direct PII storage
- Hashed customer identifiers
- Minimal metadata collection
- GDPR-compliant design patterns

## Admin API Reference (owner-only)

These functions require admin authorization (the admin address set at initialize). Ensure the caller is properly authorized; unauthorized calls will revert with KYCError::Unauthorized.

- set_registry_enabled(caller: Address, enabled: bool)
  - Globally enable/disable the KYC registry enforcement.
  - Events: ("kyc_set", "reg_en"), enabled

- set_strict_mode(caller: Address, strict: bool)
  - Toggles strict compliance behavior. Projects may use this to tighten rules.
  - Events: ("kyc_set", "strict"), strict

- set_global_settings(caller: Address, settings: GlobalSettings)
  - Replace all global settings atomically.
  - Events: ("kyc_set", "update"), (registry_enabled, strict_mode)

- add_compliance_officer(caller: Address, officer: Address)
  - Adds an officer with compliance privileges.
  - Events: ("kyc_ofc", "add"), officer

- remove_compliance_officer(caller: Address, officer: Address)
  - Removes an officer.
  - Events: ("kyc_ofc", "remove"), officer

- set_required_tier(caller: Address, operation: OperationType, tier: KYCTier)
  - Sets the minimum KYC tier required per operation (e.g., Transfer requires Verified).
  - Events: ("req_tier", operation), tier

- set_tier_limits(caller: Address, tier: KYCTier, operation: OperationType, limits: OperationLimits)
  - Sets transaction limits for a (tier, operation) pair.
  - Events: ("kyc_lims", (tier, operation)), (single_tx_limit, daily_limit, monthly_limit)

- set_sanctions_status(caller: Address, customer_id: String, cleared: bool)
  - Updates a customer's sanctions-screening status.
  - Events: ("kyc_cust", "sanct"), (customer_id, cleared)

- set_customer_expiration(caller: Address, customer_id: String, expires_at: u64)
  - Sets KYC expiration timestamp (0 = no expiration).
  - Events: ("kyc_cust", "expire"), (customer_id, expires_at)

- set_customer_metadata(caller: Address, customer_id: String, key: String, value: String)
  - Upserts an arbitrary metadata entry.
  - Events: ("kyc_cust", "meta"), (customer_id, key)


## Cross-Contract Endpoints

- is_approved_simple(address: Address, op_code: u32, amount: i128) -> bool
  - ABI-friendly wrapper around is_approved_for_operation.
  - op_code mapping:
    - 0 = Transfer
    - 1 = Mint
    - 2 = Burn
    - 3 = Deposit
    - 4 = Withdraw
    - 5 = Exchange

- get_tier_code_by_address(address: Address) -> u32
  - Returns numeric KYC tier code for fee-tiering and UI logic:
    - 0=None, 1=Basic, 2=Verified, 3=Enhanced, 4=Institutional


## Event Topics (short symbols)

- Settings: ("kyc_set", "reg_en"|"strict"|"update")
- Officers: ("kyc_ofc", "add"|"remove")
- Required Tier: ("req_tier", operation)
- Limits: ("kyc_lims", (tier, operation))
- Customer: ("kyc_cust", "sanct"|"expire"|"meta")
- Existing customer/tier events described earlier remain applicable.


## Sample Admin Flows

1) Turn on strict mode and require Verified for Transfers
- set_strict_mode(admin, true)
- set_required_tier(admin, OperationType::Transfer, KYCTier::Verified)

2) Set limits for Verified Transfers
- set_tier_limits(admin, KYCTier::Verified, OperationType::Transfer, OperationLimits {
  single_tx_limit: 10_000_0000000, // 0.1 BTC
  daily_limit: 50_000_0000000,     // 0.5 BTC
  monthly_limit: 500_000_0000000,  // 5 BTC
  enabled: true,
})

3) Add officer and update a customer's sanctions status
- add_compliance_officer(admin, officer_addr)
- set_sanctions_status(admin, customer_id_hash, true)

4) Extend a customer's KYC expiration and add metadata
- set_customer_expiration(admin, customer_id_hash, new_expires_at)
- set_customer_metadata(admin, customer_id_hash, "jurisdiction_note", "US-EDD complete")

5) Query cross-contract status from another contract (pseudocode)
- approved = kyc.is_approved_simple(user_addr, 0 /* Transfer */, amount)
- fee_tier = kyc.get_tier_code_by_address(user_addr)


## Future Enhancements

### Planned Features
1. **Daily/Monthly Limit Tracking** - Rolling time windows
2. **Enhanced Audit System** - Detailed event logging
3. **Statistics Dashboard** - KYC tier and jurisdiction analytics
4. **Risk Scoring** - Dynamic risk assessment
5. **Automated Compliance** - ML-driven decision making

### Integration Roadmap
1. **iSTSi Token Integration** - Direct compliance checking
2. **iUSDi Stablecoin Integration** - Cross-token compliance
3. **Lightning Network Support** - Micro-transaction compliance
4. **Remittance Corridors** - Geographic compliance rules
5. **Third-party KYC Providers** - Oracle integration

## Support and Maintenance

### Contact Information
- **Email**: isatoshixlm@gmail.com
- **Repository**: `/Users/shang/Prj2025/MintToken/iSTSi/`
- **Documentation**: `/docs/KYC_registry.md`

### Version History
- **v1.0** (Jan 2025): Initial implementation with core features
- **Future**: Enhanced audit, statistics, and automation features

---

**Note**: This KYC Registry is designed to support the iSHSi Bitcoin Anchor Service implementation plan and provides the compliance foundation for Bitcoin-backed token operations on the Stellar network.
