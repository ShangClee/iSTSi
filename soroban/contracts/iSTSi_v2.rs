// SPDX-License-Identifier: MIT
// iSaToShi (iSTSi) v2 - Bitcoin Anchor Service Contract
// Compatible with OpenZeppelin Stellar Soroban Contracts ^0.4.1

//! # iSaToShi (iSTSi) - Bitcoin Anchor Token Contract
//!
//! This contract implements a Bitcoin-backed token where 1 BTC = 100,000,000 iSTSi
//! Features:
//! - Full BTC reserve backing with Proof-of-Reserves
//! - Lightning Network integration
//! - KYC/AML compliance framework
//! - Role-based access control
//! - Fee management and treasury operations
//!
//! # Security
//!
//! For security issues, please contact: isatoshixlm@gmail.com

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, symbol_short, token, panic_with_error,
    Address, Env, String, Symbol, Vec, Map, Bytes, IntoVal
};

use stellar_access::ownable::{self as ownable, Ownable};
use stellar_contract_utils::pausable::{self as pausable, Pausable};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::{default_impl, only_owner, Upgradeable, when_not_paused};
use stellar_tokens::fungible::{Base, burnable::FungibleBurnable, FungibleToken};

//
// Data Types and Events
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,    // Full contract control and governance
    Minter,   // Can mint tokens after BTC deposit verification
    Burner,   // Can burn tokens for BTC withdrawal processing  
    Pauser,   // Can pause/unpause contract in emergencies
    Oracle,   // Can update price feeds and reserve data
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintRequest {
    pub btc_txid: String,                    // Bitcoin transaction ID
    pub ln_payment_hash: Option<String>,     // Lightning payment hash (if LN deposit)
    pub recipient: Address,                  // iSTSi recipient address
    pub amount: i128,                        // Amount in smallest units (satoshi equivalent)
    pub confirmations: u32,                  // Bitcoin confirmations received
    pub timestamp: u64,                      // Request timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BurnRequest {
    pub request_id: String,                  // Unique burn request ID
    pub btc_address: Option<String>,         // Bitcoin withdrawal address
    pub ln_invoice: Option<String>,          // Lightning invoice (if LN withdrawal)
    pub from_address: Address,               // iSTSi holder address
    pub amount: i128,                        // Amount to burn
    pub fee_amount: i128,                    // Withdrawal fee
    pub timestamp: u64,                      // Request timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofOfReserves {
    pub btc_balance: i128,                   // Total BTC in custody (satoshis)
    pub istsi_supply: i128,                  // Total iSTSi in circulation
    pub reserve_ratio: u32,                  // Backing ratio (should be >= 100%)
    pub cold_storage_balance: i128,          // BTC in cold storage
    pub warm_storage_balance: i128,          // BTC in warm/hot storage
    pub ln_channel_balance: i128,            // BTC in Lightning channels
    pub timestamp: u64,                      // Last update timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeConfig {
    pub mint_fee_bps: u32,                   // Default mint fee in basis points (0.01% = 1 bps)
    pub burn_fee_bps: u32,                   // Default burn fee in basis points
    pub min_mint_amount: i128,               // Minimum mint amount (1000 sats)
    pub max_mint_amount: i128,               // Maximum mint amount (1M sats)
    pub treasury_address: Address,           // Fee collection address
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TieredFeeConfig {
    pub mint_none: u32,
    pub mint_basic: u32,
    pub mint_verified: u32,
    pub mint_enhanced: u32,
    pub mint_institutional: u32,
    pub burn_none: u32,
    pub burn_basic: u32,
    pub burn_verified: u32,
    pub burn_enhanced: u32,
    pub burn_institutional: u32,
}

//
// Contract Implementation
//

#[derive(Upgradeable)]
#[contract]
pub struct ISaToShi;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ComplianceError {
    MissingRegistry = 1,
    NotApproved = 2,
}

#[contractimpl]
impl ISaToShi {
    
    /// Initialize the contract with Bitcoin anchor capabilities
    pub fn __constructor(
        e: &Env, 
        recipient: Address, 
        owner: Address,
        kyc_registry: Address,
        treasury: Address
    ) {
        // Set token metadata with new branding
        Base::set_metadata(e, 8, String::from_str(e, "iSaToShi"), String::from_str(e, "iSTSi"));
        
        // Initial supply: 100M tokens (10^8 * 10^8 units for 8 decimals)
        Base::mint(e, &recipient, 10000000000000000);
        
        // Set up ownership and roles
        ownable::set_owner(e, &owner);
        Self::set_role(e, &owner, Role::Admin);
        
        // Initialize compliance registry
        e.storage().persistent().set(&symbol_short!("KYC_REG"), &kyc_registry);
        
        // Set up initial fee configuration
        let fee_config = FeeConfig {
            mint_fee_bps: 10,        // 0.1% mint fee
            burn_fee_bps: 25,        // 0.25% burn fee  
            min_mint_amount: 100000, // 1,000 sats minimum
            max_mint_amount: 100000000000, // 1M sats maximum
            treasury_address: treasury,
        };
        e.storage().persistent().set(&symbol_short!("FEE_CFG"), &fee_config);
        
        // Initialize proof-of-reserves
        let por = ProofOfReserves {
            btc_balance: 1000000000000000,  // Initial BTC backing (10 BTC in sats)
            istsi_supply: 10000000000000000, // Initial iSTSi supply
            reserve_ratio: 100,              // 100% backing
            cold_storage_balance: 800000000000000,  // 8 BTC in cold
            warm_storage_balance: 200000000000000,  // 2 BTC in warm
            ln_channel_balance: 0,                  // No LN channels initially
            timestamp: e.ledger().timestamp(),
        };
        e.storage().persistent().set(&symbol_short!("POR"), &por);
        
        // Emit initialization event
        e.events().publish(
            (symbol_short!("INIT"), symbol_short!("ISTSI")),
            (owner.clone(), kyc_registry, treasury)
        );
    }

    //
    // Role Management
    //

    /// Set role for an address (Admin only)
    #[only_owner]
    pub fn set_role(e: &Env, address: &Address, role: Role) {
        let key = (symbol_short!("ROLE"), address.clone());
        e.storage().persistent().set(&key, &role);
        
        e.events().publish(
            (symbol_short!("ROLE_SET"), address.clone()),
            role
        );
    }

    /// Check if address has specific role
    pub fn has_role(e: &Env, address: &Address, role: Role) -> bool {
        let key = (symbol_short!("ROLE"), address.clone());
        if let Some(user_role) = e.storage().persistent().get::<_, Role>(&key) {
            user_role == role || user_role == Role::Admin
        } else {
            false
        }
    }

    /// Require specific role for caller
    fn require_role(e: &Env, role: Role) {
        let caller = e.current_contract_address(); // In practice, use proper auth
        assert!(Self::has_role(e, &caller, role), "Insufficient permissions");
    }

    //
    // Bitcoin Integration - Minting
    //

    /// Mint iSTSi tokens after BTC deposit verification (Minter role required)
    pub fn mint_with_btc(e: &Env, request: MintRequest) {
        Self::require_role(e, Role::Minter);
        assert!(!pausable::paused(e), "Contract is paused");
        
        // Validate mint request
        Self::validate_mint_request(e, &request);
        
        // Check compliance (KYC/AML) - Mint
        Self::check_compliance_op(e, &request.recipient, request.amount, 1);
        
        // Calculate tiered fees
        let tier_code = Self::get_tier_code_by_address(e, &request.recipient);
        let fee_bps = Self::fee_bps_for(e, tier_code, true);
        let mint_fee = (request.amount * fee_bps as i128) / 10000;
        let net_amount = request.amount - mint_fee;
        
        // Mint tokens to recipient
        Base::mint(e, &request.recipient, net_amount);
        
        // Collect fee to treasury
        if mint_fee > 0 {
            Base::mint(e, &fee_config.treasury_address, mint_fee);
        }
        
        // Store mint record for audit trail
        let key = (symbol_short!("MINT"), request.btc_txid.clone());
        e.storage().persistent().set(&key, &request);
        
        // Emit mint event
        e.events().publish(
            (symbol_short!("MINT"), symbol_short!("BTC")),
            (request.btc_txid.clone(), request.recipient.clone(), net_amount, mint_fee)
        );
    }

    /// Validate mint request parameters
    fn validate_mint_request(e: &Env, request: &MintRequest) {
        let fee_config = Self::get_fee_config(e);
        
        // Check amount limits
        assert!(request.amount >= fee_config.min_mint_amount, "Amount below minimum");
        assert!(request.amount <= fee_config.max_mint_amount, "Amount exceeds maximum");
        
        // Check confirmations (require at least 1 for on-chain, 0 for Lightning)
        if request.ln_payment_hash.is_none() {
            assert!(request.confirmations >= 1, "Insufficient confirmations");
        }
        
        // Check for duplicate requests
        let key = (symbol_short!("MINT"), request.btc_txid.clone());
        assert!(!e.storage().persistent().has(&key), "Duplicate mint request");
    }

    //
    // Bitcoin Integration - Burning/Withdrawal
    //

    /// Burn iSTSi tokens for BTC withdrawal (Burner role required)
    pub fn burn_for_btc(e: &Env, request: BurnRequest) {
        Self::require_role(e, Role::Burner);
        assert!(!pausable::paused(e), "Contract is paused");
        
        // Validate burn request
        Self::validate_burn_request(e, &request);
        
        // Check compliance - Burn
        Self::check_compliance_op(e, &request.from_address, request.amount, 2);
        
        // Calculate tiered fee for burn
        let tier_code = Self::get_tier_code_by_address(e, &request.from_address);
        let fee_bps = Self::fee_bps_for(e, tier_code, false);
        let fee_amount = (request.amount * fee_bps as i128) / 10000;

        // Burn tokens from user
        Base::burn(e, &request.from_address, request.amount + fee_amount);
        
        // Store burn record for audit trail (persist computed fee)
        let mut stored = request.clone();
        stored.fee_amount = fee_amount;
        let key = (symbol_short!("BURN"), request.request_id.clone());
        e.storage().persistent().set(&key, &stored);
        
        // Emit burn event
        e.events().publish(
            (symbol_short!("BURN"), symbol_short!("BTC")),
            (request.request_id.clone(), request.from_address.clone(), request.amount, request.fee_amount)
        );
    }

    /// Validate burn request parameters
    fn validate_burn_request(e: &Env, request: &BurnRequest) {
        let fee_config = Self::get_fee_config(e);
        
        // Check minimum amount
        assert!(request.amount >= fee_config.min_mint_amount, "Amount below minimum");
        
        // Check that either BTC address or LN invoice is provided
        assert!(
            request.btc_address.is_some() || request.ln_invoice.is_some(),
            "Must provide BTC address or LN invoice"
        );
        
        // Check for duplicate requests
        let key = (symbol_short!("BURN"), request.request_id.clone());
        assert!(!e.storage().persistent().has(&key), "Duplicate burn request");
        
        // Verify user has sufficient balance (fee will be computed later, conservatively check default max)
        let balance = Base::balance(e, &request.from_address);
        assert!(balance >= request.amount, "Insufficient balance");
    }

    //
    // Compliance Integration
    //

    /// Check KYC/AML compliance for address and amount for a given operation code
    /// op_code mapping (aligned with KYC registry):
    /// 0 = Transfer, 1 = Mint, 2 = Burn
    fn check_compliance_op(e: &Env, address: &Address, amount: i128, op_code: u32) {
        // Compliance enabled flag (default true)
        let comp_enabled: bool = e
            .storage()
            .persistent()
            .get(&symbol_short!("COMP_EN"))
            .unwrap_or(true);
        if !comp_enabled { return; }

        // Fetch KYC registry; deny if missing while compliance enabled
        let maybe_reg: Option<Address> = e.storage().persistent().get(&symbol_short!("KYC_REG"));
        let kyc_registry = match maybe_reg {
            Some(a) => a,
            None => panic_with_error!(e, ComplianceError::MissingRegistry),
        };

        let approved: bool = e.invoke_contract(
            &kyc_registry,
            &symbol_short!("is_approved_simple"),
            (address.clone(), op_code, amount).into_val(e),
        );

        if !approved {
            panic_with_error!(e, ComplianceError::NotApproved);
        }

        e.events().publish(
            (symbol_short!("COMPLY"), symbol_short!("CHECK")),
            (address.clone(), amount)
        );
    }

    /// Backward-compatible helper that defaults to Transfer op_code
    fn check_compliance(e: &Env, address: &Address, amount: i128) {
        Self::check_compliance_op(e, address, amount, 0);
    }

    /// Update KYC registry address (Admin only)
    #[only_owner]
    pub fn set_kyc_registry(e: &Env, registry: Address) {
        e.storage().persistent().set(&symbol_short!("KYC_REG"), &registry);
        
        e.events().publish(
            (symbol_short!("KYC_REG"), symbol_short!("UPDATE")),
            registry
        );
    }

    /// Get current KYC registry address (if set)
    pub fn get_kyc_registry(e: &Env) -> Option<Address> {
        e.storage().persistent().get(&symbol_short!("KYC_REG"))
    }

    /// Enable or disable on-chain compliance checks (owner only)
    #[only_owner]
    pub fn set_compliance_enabled(e: &Env, enabled: bool) {
        e.storage().persistent().set(&symbol_short!("COMP_EN"), &enabled);
        e.events().publish((symbol_short!("COMPLY"), symbol_short!("ENABLED")), enabled);
    }

    /// Returns whether on-chain compliance checks are enabled
    pub fn get_compliance_enabled(e: &Env) -> bool {
        e.storage().persistent().get(&symbol_short!("COMP_EN")).unwrap_or(true)
    }

    /// Read-only passthrough: check compliance status for an address
    pub fn check_compliance_status(e: &Env, address: Address, op_code: u32, amount: i128) -> bool {
        let Some(reg) = e.storage().persistent().get::<_, Address>(&symbol_short!("KYC_REG")) else {
            return false;
        };
        e.invoke_contract(&reg, &symbol_short!("is_approved_simple"), (address, op_code, amount).into_val(e))
    }

    /// Set tiered fee configuration (owner only)
    #[only_owner]
    pub fn set_tiered_fee_config(e: &Env, cfg: TieredFeeConfig) {
        // basic validation: each bps <= 10000 (<=100%)
        let fields = [
            cfg.mint_none, cfg.mint_basic, cfg.mint_verified, cfg.mint_enhanced, cfg.mint_institutional,
            cfg.burn_none, cfg.burn_basic, cfg.burn_verified, cfg.burn_enhanced, cfg.burn_institutional,
        ];
        for b in fields { assert!(b <= 10_000, "fee too high"); }
        e.storage().persistent().set(&symbol_short!("FEE_TIER"), &cfg);
        e.events().publish((symbol_short!("FEE"), symbol_short!("TIERSET")), 1u32);
    }

    /// Get tiered fee configuration if set
    pub fn get_tiered_fee_config(e: &Env) -> Option<TieredFeeConfig> {
        e.storage().persistent().get(&symbol_short!("FEE_TIER"))
    }

    /// Helper: resolve tier code for address via KYC registry
    fn get_tier_code_by_address(e: &Env, addr: &Address) -> u32 {
        let Some(reg) = e.storage().persistent().get::<_, Address>(&symbol_short!("KYC_REG")) else {
            return 0; // default None tier
        };
        e.invoke_contract(&reg, &symbol_short!("get_tier_code_by_address"), (addr.clone(),).into_val(e))
    }

    /// Helper: pick fee bps from tiered config or default FeeConfig
    fn fee_bps_for(e: &Env, tier_code: u32, is_mint: bool) -> u32 {
        if let Some(cfg) = Self::get_tiered_fee_config(e) {
            return match (is_mint, tier_code) {
                (true, 0) => cfg.mint_none,
                (true, 1) => cfg.mint_basic,
                (true, 2) => cfg.mint_verified,
                (true, 3) => cfg.mint_enhanced,
                (true, 4) => cfg.mint_institutional,
                (false, 0) => cfg.burn_none,
                (false, 1) => cfg.burn_basic,
                (false, 2) => cfg.burn_verified,
                (false, 3) => cfg.burn_enhanced,
                (false, 4) => cfg.burn_institutional,
                _ => 0,
            };
        }
        let def = Self::get_fee_config(e);
        if is_mint { def.mint_fee_bps } else { def.burn_fee_bps }
    }

    //
    // Proof-of-Reserves
    //

    /// Update proof-of-reserves data (Oracle role required)
    pub fn update_proof_of_reserves(e: &Env, por: ProofOfReserves) {
        Self::require_role(e, Role::Oracle);
        
        // Validate reserve data
        assert!(por.reserve_ratio >= 100, "Reserve ratio must be >= 100%");
        assert!(
            por.btc_balance == por.cold_storage_balance + por.warm_storage_balance + por.ln_channel_balance,
            "BTC balance components don't match total"
        );
        
        // Store updated data
        e.storage().persistent().set(&symbol_short!("POR"), &por);
        
        // Emit proof-of-reserves event
        e.events().publish(
            (symbol_short!("POR"), symbol_short!("UPDATE")),
            (por.btc_balance, por.istsi_supply, por.reserve_ratio, por.timestamp)
        );
    }

    /// Get current proof-of-reserves data
    pub fn get_proof_of_reserves(e: &Env) -> ProofOfReserves {
        e.storage().persistent()
            .get(&symbol_short!("POR"))
            .unwrap_or(ProofOfReserves {
                btc_balance: 0,
                istsi_supply: Base::total_supply(e),
                reserve_ratio: 0,
                cold_storage_balance: 0,
                warm_storage_balance: 0,
                ln_channel_balance: 0,
                timestamp: e.ledger().timestamp(),
            })
    }

    //
    // Fee Management
    //

    /// Update fee configuration (Admin only)
    #[only_owner]
    pub fn set_fee_config(e: &Env, config: FeeConfig) {
        // Validate fee parameters
        assert!(config.mint_fee_bps <= 100, "Mint fee too high"); // Max 1%
        assert!(config.burn_fee_bps <= 200, "Burn fee too high"); // Max 2%
        assert!(config.min_mint_amount >= 100000, "Minimum too low"); // At least 1000 sats
        
        e.storage().persistent().set(&symbol_short!("FEE_CFG"), &config);
        
        e.events().publish(
            (symbol_short!("FEE_CFG"), symbol_short!("UPDATE")),
            (config.mint_fee_bps, config.burn_fee_bps)
        );
    }

    /// Get current fee configuration
    pub fn get_fee_config(e: &Env) -> FeeConfig {
        e.storage().persistent()
            .get(&symbol_short!("FEE_CFG"))
            .unwrap()
    }

    //
    // Enhanced Minting (Owner role for direct minting)
    //

    #[only_owner]
    #[when_not_paused]
    pub fn mint(e: &Env, account: Address, amount: i128) {
        // Check compliance for direct mints
        Self::check_compliance_op(e, &account, amount, 1);
        
        Base::mint(e, &account, amount);
        
        e.events().publish(
            (symbol_short!("MINT"), symbol_short!("DIRECT")),
            (account, amount)
        );
    }

    //
    // Emergency Functions
    //

    /// Emergency pause (Pauser role required)
    pub fn emergency_pause(e: &Env, caller: Address) {
        assert!(
            Self::has_role(e, &caller, Role::Pauser) || Self::has_role(e, &caller, Role::Admin),
            "Insufficient permissions"
        );
        
        pausable::pause(e);
        
        e.events().publish(
            (symbol_short!("EMERGENCY"), symbol_short!("PAUSE")),
            caller
        );
    }

    /// Emergency unpause (Admin only)
    #[only_owner]
    pub fn emergency_unpause(e: &Env, caller: Address) {
        pausable::unpause(e);
        
        e.events().publish(
            (symbol_short!("EMERGENCY"), symbol_short!("UNPAUSE")),
            caller
        );
    }

    //
    // View Functions
    //

    /// Get mint record by BTC transaction ID
    pub fn get_mint_record(e: &Env, btc_txid: String) -> Option<MintRequest> {
        let key = (symbol_short!("MINT"), btc_txid);
        e.storage().persistent().get(&key)
    }

    /// Get burn record by request ID
    pub fn get_burn_record(e: &Env, request_id: String) -> Option<BurnRequest> {
        let key = (symbol_short!("BURN"), request_id);
        e.storage().persistent().get(&key)
    }

    /// Get contract version
    pub fn version(e: &Env) -> String {
        String::from_str(e, "2.0.0")
    }
}

//
// Standard Token Implementation with Compliance
//

#[default_impl]
#[contractimpl]
impl FungibleToken for ISaToShi {
    type ContractType = Base;

    #[when_not_paused]
    fn transfer(e: &Env, from: Address, to: Address, amount: i128) {
        // Check compliance for both sender and recipient - Transfer
        ISaToShi::check_compliance_op(e, &from, amount, 0);
        ISaToShi::check_compliance_op(e, &to, amount, 0);
        
        Self::ContractType::transfer(e, &from, &to, amount);
    }

    #[when_not_paused]
    fn transfer_from(e: &Env, spender: Address, from: Address, to: Address, amount: i128) {
        // Check compliance for sender and recipient - Transfer
        ISaToShi::check_compliance_op(e, &from, amount, 0);
        ISaToShi::check_compliance_op(e, &to, amount, 0);
        
        Self::ContractType::transfer_from(e, &spender, &from, &to, amount);
    }
}

//
// Burnable Implementation
//

#[contractimpl]
impl FungibleBurnable for ISaToShi {
    #[when_not_paused]
    fn burn(e: &Env, from: Address, amount: i128) {
        ISaToShi::check_compliance_op(e, &from, amount, 2);
        Base::burn(e, &from, amount);
    }

    #[when_not_paused]
    fn burn_from(e: &Env, spender: Address, from: Address, amount: i128) {
        ISaToShi::check_compliance(e, &from, amount);
        Base::burn_from(e, &spender, &from, amount);
    }
}

//
// Utility Implementations
//

impl UpgradeableInternal for ISaToShi {
    fn _require_auth(e: &Env, _operator: &Address) {
        ownable::enforce_owner_auth(e);
    }
}

#[contractimpl]
impl Pausable for ISaToShi {
    fn paused(e: &Env) -> bool {
        pausable::paused(e)
    }

    #[only_owner]
    fn pause(e: &Env, _caller: Address) {
        pausable::pause(e);
    }

    #[only_owner]
    fn unpause(e: &Env, _caller: Address) {
        pausable::unpause(e);
    }
}

#[default_impl]
#[contractimpl]
impl Ownable for ISaToShi {}

//
// Unit Tests
//

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_initialization() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ISaToShi);
        let client = ISaToShiClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let recipient = Address::generate(&env);
        let kyc_registry = Address::generate(&env);
        let treasury = Address::generate(&env);
        
        client.__constructor(&recipient, &admin, &kyc_registry, &treasury);
        
        // Check token metadata
        assert_eq!(client.name(), String::from_str(&env, "iSaToShi"));
        assert_eq!(client.symbol(), String::from_str(&env, "iSTSi"));
        assert_eq!(client.decimals(), 8);
        
        // Check initial supply
        assert_eq!(client.balance(&recipient), 10000000000000000);
    }
    
    #[test]
    fn test_proof_of_reserves() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ISaToShi);
        let client = ISaToShiClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let recipient = Address::generate(&env);
        let kyc_registry = Address::generate(&env);
        let treasury = Address::generate(&env);
        
        client.__constructor(&recipient, &admin, &kyc_registry, &treasury);
        
        let por = client.get_proof_of_reserves();
        assert_eq!(por.reserve_ratio, 100);
        assert!(por.btc_balance > 0);
    }
}
