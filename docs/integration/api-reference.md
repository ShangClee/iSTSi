# API Reference

Complete API reference for all contracts in the iSHS-XLM token ecosystem.

## KYC Registry Contract

### Data Types

```rust
pub struct UserInfo {
    pub tier: u32,
    pub daily_spent: i128,
    pub monthly_spent: i128,
    pub last_daily_reset: u64,
    pub last_monthly_reset: u64,
    pub is_active: bool,
}

pub struct TierLimits {
    pub daily_limit: i128,
    pub monthly_limit: i128,
}
```

### Functions

#### Administrative Functions

**initialize**
```rust
fn initialize(env: Env, admin: Address)
```
Initialize the KYC registry with an admin address.

**set_admin**
```rust
fn set_admin(env: Env, new_admin: Address)
```
Update the admin address (admin only).

**get_admin**
```rust
fn get_admin(env: Env) -> Address
```
Get the current admin address.

#### User Management

**register_user**
```rust
fn register_user(env: Env, user: Address, tier: u32)
```
Register a new user with specified tier (admin only).

**update_user_tier**
```rust
fn update_user_tier(env: Env, user: Address, new_tier: u32)
```
Update user's KYC tier (admin only).

**deactivate_user**
```rust
fn deactivate_user(env: Env, user: Address)
```
Deactivate a user account (admin only).

**reactivate_user**
```rust
fn reactivate_user(env: Env, user: Address)
```
Reactivate a user account (admin only).

#### User Information

**get_user_info**
```rust
fn get_user_info(env: Env, user: Address) -> UserInfo
```
Get complete user information.

**get_user_tier**
```rust
fn get_user_tier(env: Env, user: Address) -> u32
```
Get user's KYC tier.

**is_user_active**
```rust
fn is_user_active(env: Env, user: Address) -> bool
```
Check if user account is active.

#### Tier Management

**set_tier_limits**
```rust
fn set_tier_limits(env: Env, tier: u32, daily_limit: i128, monthly_limit: i128)
```
Set transaction limits for a tier (admin only).

**get_tier_limits**
```rust
fn get_tier_limits(env: Env, tier: u32) -> TierLimits
```
Get transaction limits for a tier.

#### Transaction Validation

**validate_transaction**
```rust
fn validate_transaction(env: Env, user: Address, amount: i128) -> bool
```
Validate if user can perform transaction of given amount.

**record_transaction**
```rust
fn record_transaction(env: Env, user: Address, amount: i128)
```
Record a transaction against user's limits.

**get_remaining_daily_limit**
```rust
fn get_remaining_daily_limit(env: Env, user: Address) -> i128
```
Get user's remaining daily transaction limit.

**get_remaining_monthly_limit**
```rust
fn get_remaining_monthly_limit(env: Env, user: Address) -> i128
```
Get user's remaining monthly transaction limit.

## Reserve Manager Contract

### Data Types

```rust
pub struct ReserveInfo {
    pub total_reserves: i128,
    pub total_issued: i128,
    pub reserve_ratio: u32,
    pub minimum_ratio: u32,
}
```

### Functions

#### Administrative Functions

**initialize**
```rust
fn initialize(env: Env, admin: Address, kyc_registry: Address)
```
Initialize the reserve manager.

**set_admin**
```rust
fn set_admin(env: Env, new_admin: Address)
```
Update the admin address (admin only).

**get_admin**
```rust
fn get_admin(env: Env) -> Address
```
Get the current admin address.

#### Reserve Management

**add_reserves**
```rust
fn add_reserves(env: Env, amount: i128)
```
Add reserves to the system (admin only).

**remove_reserves**
```rust
fn remove_reserves(env: Env, amount: i128)
```
Remove reserves from the system (admin only).

**get_total_reserves**
```rust
fn get_total_reserves(env: Env) -> i128
```
Get total reserves in the system.

#### Issuance Tracking

**record_issuance**
```rust
fn record_issuance(env: Env, amount: i128)
```
Record token issuance (token contract only).

**record_burn**
```rust
fn record_burn(env: Env, amount: i128)
```
Record token burn (token contract only).

**get_total_issued**
```rust
fn get_total_issued(env: Env) -> i128
```
Get total tokens issued.

#### Reserve Ratio Management

**get_reserve_ratio**
```rust
fn get_reserve_ratio(env: Env) -> u32
```
Get current reserve ratio (basis points).

**set_reserve_threshold**
```rust
fn set_reserve_threshold(env: Env, threshold: u32)
```
Set minimum reserve ratio threshold (admin only).

**get_reserve_threshold**
```rust
fn get_reserve_threshold(env: Env) -> u32
```
Get minimum reserve ratio threshold.

**can_issue**
```rust
fn can_issue(env: Env, amount: i128) -> bool
```
Check if tokens can be issued without violating reserve ratio.

**get_reserve_info**
```rust
fn get_reserve_info(env: Env) -> ReserveInfo
```
Get complete reserve information.

## iSTSi Token Contract

### Data Types

```rust
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub total_supply: i128,
}

pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}
```

### Functions

#### Administrative Functions

**initialize**
```rust
fn initialize(
    env: Env,
    admin: Address,
    kyc_registry: Address,
    reserve_manager: Address
)
```
Initialize the iSTSi token contract.

**set_admin**
```rust
fn set_admin(env: Env, new_admin: Address)
```
Update the admin address (admin only).

**get_admin**
```rust
fn get_admin(env: Env) -> Address
```
Get the current admin address.

#### Token Information

**name**
```rust
fn name(env: Env) -> String
```
Get token name.

**symbol**
```rust
fn symbol(env: Env) -> String
```
Get token symbol.

**decimals**
```rust
fn decimals(env: Env) -> u32
```
Get token decimals.

**total_supply**
```rust
fn total_supply(env: Env) -> i128
```
Get total token supply.

#### Balance Operations

**balance**
```rust
fn balance(env: Env, id: Address) -> i128
```
Get account balance.

**spendable_balance**
```rust
fn spendable_balance(env: Env, id: Address) -> i128
```
Get spendable balance (considering allowances).

#### Transfer Operations

**transfer**
```rust
fn transfer(env: Env, from: Address, to: Address, amount: i128)
```
Transfer tokens between accounts.

**transfer_from**
```rust
fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128)
```
Transfer tokens using allowance.

#### Allowance Operations

**approve**
```rust
fn approve(
    env: Env,
    from: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32
)
```
Approve spending allowance.

**allowance**
```rust
fn allowance(env: Env, from: Address, spender: Address) -> AllowanceValue
```
Get spending allowance.

#### Mint and Burn Operations

**mint**
```rust
fn mint(env: Env, to: Address, amount: i128)
```
Mint new tokens (admin only, with KYC and reserve checks).

**burn**
```rust
fn burn(env: Env, from: Address, amount: i128)
```
Burn tokens (admin only).

**burn_from**
```rust
fn burn_from(env: Env, spender: Address, from: Address, amount: i128)
```
Burn tokens using allowance (admin only).

#### Integration Functions

**get_kyc_registry**
```rust
fn get_kyc_registry(env: Env) -> Address
```
Get KYC registry contract address.

**get_reserve_manager**
```rust
fn get_reserve_manager(env: Env) -> Address
```
Get reserve manager contract address.

**set_kyc_registry**
```rust
fn set_kyc_registry(env: Env, kyc_registry: Address)
```
Update KYC registry address (admin only).

**set_reserve_manager**
```rust
fn set_reserve_manager(env: Env, reserve_manager: Address)
```
Update reserve manager address (admin only).

## Fungible Token Contract

### Functions

#### Standard Token Functions

**initialize**
```rust
fn initialize(env: Env, admin: Address, name: String, symbol: String, decimals: u32)
```
Initialize the fungible token.

**name**
```rust
fn name(env: Env) -> String
```
Get token name.

**symbol**
```rust
fn symbol(env: Env) -> String
```
Get token symbol.

**decimals**
```rust
fn decimals(env: Env) -> u32
```
Get token decimals.

**total_supply**
```rust
fn total_supply(env: Env) -> i128
```
Get total supply.

**balance**
```rust
fn balance(env: Env, id: Address) -> i128
```
Get account balance.

**transfer**
```rust
fn transfer(env: Env, from: Address, to: Address, amount: i128)
```
Transfer tokens.

**approve**
```rust
fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32)
```
Approve spending allowance.

**allowance**
```rust
fn allowance(env: Env, from: Address, spender: Address) -> AllowanceValue
```
Get allowance.

**transfer_from**
```rust
fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128)
```
Transfer using allowance.

**mint**
```rust
fn mint(env: Env, to: Address, amount: i128)
```
Mint tokens (admin only).

**burn**
```rust
fn burn(env: Env, from: Address, amount: i128)
```
Burn tokens (admin only).

## Error Codes

### KYC Registry Errors

- `KYC_NOT_INITIALIZED`: Contract not initialized
- `KYC_UNAUTHORIZED`: Caller not authorized
- `KYC_USER_NOT_FOUND`: User not registered
- `KYC_USER_INACTIVE`: User account deactivated
- `KYC_INVALID_TIER`: Invalid tier specified
- `KYC_DAILY_LIMIT_EXCEEDED`: Daily transaction limit exceeded
- `KYC_MONTHLY_LIMIT_EXCEEDED`: Monthly transaction limit exceeded

### Reserve Manager Errors

- `RESERVE_NOT_INITIALIZED`: Contract not initialized
- `RESERVE_UNAUTHORIZED`: Caller not authorized
- `RESERVE_INSUFFICIENT`: Insufficient reserves
- `RESERVE_RATIO_TOO_LOW`: Reserve ratio below threshold
- `RESERVE_INVALID_AMOUNT`: Invalid amount specified

### Token Errors

- `TOKEN_NOT_INITIALIZED`: Contract not initialized
- `TOKEN_UNAUTHORIZED`: Caller not authorized
- `TOKEN_INSUFFICIENT_BALANCE`: Insufficient balance
- `TOKEN_INSUFFICIENT_ALLOWANCE`: Insufficient allowance
- `TOKEN_INVALID_AMOUNT`: Invalid amount
- `TOKEN_EXPIRED_ALLOWANCE`: Allowance expired
- `TOKEN_KYC_FAILED`: KYC validation failed
- `TOKEN_RESERVE_CHECK_FAILED`: Reserve check failed

## Usage Examples

### Complete Integration Flow

```bash
#!/bin/bash
# Complete integration example

# Set contract addresses
KYC_REGISTRY="CKYC123..."
RESERVE_MANAGER="CRESERVE123..."
ISTSI_TOKEN="CISTSI123..."

# User addresses
ADMIN="GADMIN123..."
USER="GUSER123..."

# 1. Register user
stellar contract invoke \
  --id $KYC_REGISTRY \
  --network testnet \
  -- register_user \
  --user $USER \
  --tier 1

# 2. Set tier limits
stellar contract invoke \
  --id $KYC_REGISTRY \
  --network testnet \
  -- set_tier_limits \
  --tier 1 \
  --daily_limit 1000000000 \
  --monthly_limit 10000000000

# 3. Add reserves
stellar contract invoke \
  --id $RESERVE_MANAGER \
  --network testnet \
  -- add_reserves \
  --amount 10000000000

# 4. Mint tokens
stellar contract invoke \
  --id $ISTSI_TOKEN \
  --network testnet \
  -- mint \
  --to $USER \
  --amount 1000000000

# 5. Check balance
stellar contract invoke \
  --id $ISTSI_TOKEN \
  --network testnet \
  -- balance \
  --id $USER
```