# Implementation Update

Date: 2025-09-06
Repository: iSHS-XLM-OpenZepplin
Branch: fix/kyc-registry-completion

## Summary

This update implements a production-ready KYC compliance layer and integrates it with the iSTSi token contract. It also introduces tiered feeing based on a user's KYC tier. The changes include:

- KYC Registry
  - Typed error handling with KYCError
  - Admin controls for settings, officers, per-op tier requirements, per-tier operation limits
  - Cross-contract endpoints for approvals and tier queries
  - Compact, audit-friendly events
- iSTSi Token
  - Cross-contract KYC integration with typed ComplianceError
  - Compliance enable/disable toggle, getters and passthrough status queries
  - Tiered feeing via TieredFeeConfig driven by KYC tier

---

## KYC Registry Changes (contracts/kyc_registry/src/lib.rs)

### Typed Errors
- KYCError (#[contracterror])
  - Unauthorized = 1
  - NotFound = 2
  - AlreadyExists = 3
  - InvalidInput = 4
  - RegistryDisabled = 5

All critical access and state-precondition failures use panic_with_error!(env, KYCError::...).

### Admin Management APIs (owner-only)
- set_registry_enabled(env, caller, enabled: bool)
- set_strict_mode(env, caller, strict: bool)
- set_global_settings(env, caller, settings: GlobalSettings)
- add_compliance_officer(env, caller, officer: Address)
- remove_compliance_officer(env, caller, officer: Address)
- set_required_tier(env, caller, operation: OperationType, tier: KYCTier)
- set_tier_limits(env, caller, tier: KYCTier, operation: OperationType, limits: OperationLimits)
- set_sanctions_status(env, caller, customer_id: String, cleared: bool)
- set_customer_expiration(env, caller, customer_id: String, expires_at: u64)
- set_customer_metadata(env, caller, customer_id: String, key: String, value: String)

### Cross-Contract Endpoints
- is_approved_simple(address: Address, op_code: u32, amount: i128) -> bool
  - op_code: 0=Transfer, 1=Mint, 2=Burn, 3=Deposit, 4=Withdraw, 5=Exchange
  - ABI-friendly to avoid cross-contract enum serialization issues
- get_tier_code_by_address(address: Address) -> u32
  - 0=None, 1=Basic, 2=Verified, 3=Enhanced, 4=Institutional

### Events (short symbols <= 9 chars)
- Settings: ("kyc_set", "reg_en" | "strict" | "update")
- Officers: ("kyc_ofc", "add" | "remove")
- Required tier: ("req_tier", OperationType)
- Limits: ("kyc_lims", (KYCTier, OperationType))
- Customer changes: ("kyc_cust", "sanct" | "expire" | "meta")
- Audit framework and previous customer/tier events retained as implemented.

Build status: cargo check OK.

---

## iSTSi Token Changes (contracts/iSTSi_v2.rs)

### Typed Errors
- ComplianceError (#[contracterror])
  - MissingRegistry = 1
  - NotApproved = 2

### Compliance Integration
- Cross-contract call to KYC Registry
  - Uses is_approved_simple(address, op_code, amount)
- Behavior defaults
  - Compliance enabled by default
  - With compliance enabled, missing KYC registry DENIES operations (MissingRegistry)
  - KYC denial produces NotApproved
- Events: ("COMPLY", "CHECK") on approval checks

### Integration Controls
- get_kyc_registry() -> Option<Address>
- set_kyc_registry(registry: Address) [owner-only]
- set_compliance_enabled(enabled: bool) [owner-only]
- get_compliance_enabled() -> bool (default true)
- check_compliance_status(address: Address, op_code: u32, amount: i128) -> bool

### Tiered Fees
- New data type: TieredFeeConfig
  - mint_none/basic/verified/enhanced/institutional (u32 bps)
  - burn_none/basic/verified/enhanced/institutional (u32 bps)
- Storage key: FEE_TIER (optional)
- Owner APIs
  - set_tiered_fee_config(cfg: TieredFeeConfig) [owner-only]
  - get_tiered_fee_config() -> Option<TieredFeeConfig>
- Fee selection logic
  - Resolve user tier via KYC get_tier_code_by_address(address) -> u32
  - If TieredFeeConfig is set, pick fee from tier; otherwise fallback to FeeConfig defaults
- Minting (mint_with_btc)
  - Computes per-tier mint_fee; mints net to recipient, mints fee to treasury
- Burning (burn_for_btc)
  - Computes per-tier fee; burns (amount + fee); persists computed fee in stored burn record
- Owner direct mint (mint)
  - Still compliance-checked; no fee applied by design

---

## Defaults & Operational Considerations

- Compliance enforcement is ON by default (COMP_EN=true)
  - Missing KYC registry will DENY operations
  - To bypass in emergencies: set_compliance_enabled(false) [owner-only]
- KYC Registry can be globally enabled/disabled via set_registry_enabled
- All admin changes emit compact events for auditability
- No PII stored on-chain; customer IDs expected to be hashed or opaque

---

## Tiered Fee Configuration Examples

Below are concrete examples on how to configure and reason about tiered mint/burn fees using TieredFeeConfig on the iSTSi token contract.

1) Set Tiered Fees (owner-only)

```rust path=null start=null
// Example values in basis points (bps). 100 bps = 1%.
let cfg = TieredFeeConfig {
  // Mint fees by tier
  mint_none: 100,          // 1.00%
  mint_basic: 50,          // 0.50%
  mint_verified: 25,       // 0.25%
  mint_enhanced: 10,       // 0.10%
  mint_institutional: 5,   // 0.05%

  // Burn fees by tier
  burn_none: 200,          // 2.00%
  burn_basic: 100,         // 1.00%
  burn_verified: 50,       // 0.50%
  burn_enhanced: 25,       // 0.25%
  burn_institutional: 10,  // 0.10%
};

// Owner address must be authorized. Example client call:
istsi_client.set_tiered_fee_config(&owner, &cfg);
```

2) Expected Mint Fee Outcomes (example)

Assume a mint amount of 1,000,000 sats (0.01 BTC). With the above tiered config:
- None (0): fee = 1,000,000 * 1.00% = 10,000 sats, net = 990,000 sats
- Basic (1): fee = 1,000,000 * 0.50% = 5,000 sats, net = 995,000 sats
- Verified (2): fee = 1,000,000 * 0.25% = 2,500 sats, net = 997,500 sats
- Enhanced (3): fee = 1,000,000 * 0.10% = 1,000 sats, net = 999,000 sats
- Institutional (4): fee = 1,000,000 * 0.05% = 500 sats, net = 999,500 sats

3) Expected Burn Fee Outcomes (example)

Assume a burn amount of 2,000,000 sats (0.02 BTC). With the above tiered config:
- None (0): fee = 2,000,000 * 2.00% = 40,000 sats, total burned = 2,040,000 sats
- Basic (1): fee = 2,000,000 * 1.00% = 20,000 sats, total burned = 2,020,000 sats
- Verified (2): fee = 2,000,000 * 0.50% = 10,000 sats, total burned = 2,010,000 sats
- Enhanced (3): fee = 2,000,000 * 0.25% = 5,000 sats, total burned = 2,005,000 sats
- Institutional (4): fee = 2,000,000 * 0.10% = 2,000 sats, total burned = 2,002,000 sats

Notes:
- Minting: the net minted amount to recipient is (amount - fee) and fee is minted to the treasury.
- Burning: the contract burns (amount + fee) from the user and records the fee in the stored burn record.
- If TieredFeeConfig is not set, default FeeConfig is used.

4) Mapping Address -> Tier -> Fee Selection

```rust path=null start=null
// iSTSi internally asks the KYC registry for the numeric tier code:
let tier_code: u32 = kyc_registry.get_tier_code_by_address(user_addr);

// iSTSi selects fee basis points from either TieredFeeConfig or FeeConfig fallback
let bps = fee_bps_for(tier_code, is_mint /* true for mint, false for burn */);
let fee_amount = (amount * bps as i128) / 10_000;
```

5) Pre-requisites in KYC for Tiered Fees to apply
- The user address must be registered in KYC and mapped to a customer ID.
- The customer's KYC tier should be set appropriately (Basic/Verified/Enhanced/Institutional).
- Sanctions must be cleared (if required by settings) and KYC not expired.

---

## Usage Examples (Conceptual)

- Enable KYC registry strict mode:
  - set_strict_mode(admin, true)
- Require Verified for Transfer:
  - set_required_tier(admin, OperationType::Transfer, KYCTier::Verified)
- Set per-tier Transfer limits for Verified:
  - set_tier_limits(admin, KYCTier::Verified, OperationType::Transfer, OperationLimits { single_tx_limit, daily_limit, monthly_limit, enabled: true })
- Configure tiered fees (bps):
  - set_tiered_fee_config(TieredFeeConfig {
      mint_none: 100, mint_basic: 50, mint_verified: 25, mint_enhanced: 10, mint_institutional: 5,
      burn_none: 200, burn_basic: 100, burn_verified: 50, burn_enhanced: 25, burn_institutional: 10,
    })

Note: Replace "admin" with a properly authorized owner address in-contract. When using CLI, ensure signatures/auth are provided.

---

## Next Steps

- Tests
  - Unit tests for admin APIs and error paths (Unauthorized, NotFound, etc.)
  - Cross-contract tests for compliance checks and tiered fee computation
- Documentation
  - Expand KYC_registry.md to include the new admin endpoints
  - Add examples for configuring tiers/limits/fees
- Deployment
  - Build WASM artifacts and prepare deployment scripts
  - Stage on testnet, verify end-to-end flows with PWA/backend

---

For questions or follow-up tasks, ping: isatoshixlm@gmail.com

