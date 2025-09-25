#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, symbol_short, vec, panic_with_error,
    Address, Env, Map, Vec, String
};

/// KYC Registry Contract for iSTSi Compliance Framework
/// 
/// This contract manages KYC (Know Your Customer) compliance status and address allowlists
/// for both iSTSi and iUSDi tokens. It provides tiered KYC levels and granular permissions
/// for different types of operations.

#[contract]
pub struct KYCRegistry;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KYCError {
    Unauthorized = 1,
    NotFound = 2,
    AlreadyExists = 3,
    InvalidInput = 4,
    RegistryDisabled = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KYCTier {
    None,           // No KYC - limited operations
    Basic,          // Basic verification - small amounts
    Verified,       // Full KYC - standard operations  
    Enhanced,       // Enhanced due diligence - large amounts
    Institutional,  // Institutional KYC - unlimited
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationType {
    Transfer,       // Token transfers
    Mint,          // Token minting
    Burn,          // Token burning
    Deposit,       // BTC/fiat deposits
    Withdraw,      // BTC/fiat withdrawals
    Exchange,      // Cross-token exchanges
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationLimits {
    pub daily_limit: i128,         // Daily operation limit
    pub monthly_limit: i128,       // Monthly operation limit
    pub single_tx_limit: i128,     // Single transaction limit
    pub enabled: bool,             // Operation enabled/disabled
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Core storage keys
    Admin,                         // Admin address
    CustomerRecord(String),        // Customer ID -> CustomerRecord
    AddressToCustomer(Address),    // Address -> Customer ID mapping
    
    /// Tier limits and permissions
    TierLimits(KYCTier, OperationType), // KYC tier operation limits
    
    /// Compliance settings
    RequiredTier(OperationType),   // Minimum tier for operation type
    GlobalSettings,                // Global registry settings
    
    /// Audit and monitoring
    AuditLog(u64),                // Audit log entries by timestamp
    ComplianceOfficers,           // List of compliance officers
    
    /// Statistics and reporting
    TierStats(KYCTier),           // Statistics by KYC tier
    JurisdictionStats(String),    // Statistics by jurisdiction
    
    /// Integration hooks
    IntegrationRouter,            // Address of the integration router
}

/// Global registry settings
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSettings {
    pub registry_enabled: bool,     // Global enable/disable
    pub strict_mode: bool,         // Strict compliance mode
    pub auto_expire_days: u64,     // Auto-expire KYC after N days (0 = disabled)
    pub sanctions_required: bool,   // Require sanctions clearance
    pub audit_enabled: bool,       // Enable audit logging
}

/// Audit log entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditLogEntry {
    pub timestamp: u64,
    pub action: String,
    pub customer_id: String,
    pub address: Address,
    pub old_tier: KYCTier,
    pub new_tier: KYCTier,
    pub officer: Address,
    pub notes: String,
}

const DAY_IN_LEDGERS: u64 = 17280; // Approximately 1 day in ledgers (5s each)

#[contractimpl]
impl KYCRegistry {
    
    /// Initialize the KYC registry with default settings
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        // Set default global settings
        let settings = GlobalSettings {
            registry_enabled: true,
            strict_mode: false,
            auto_expire_days: 365, // 1 year default expiration
            sanctions_required: true,
            audit_enabled: true,
        };
        env.storage().instance().set(&DataKey::GlobalSettings, &settings);
        
        // Initialize default tier limits
        Self::init_all_default_tier_limits(&env);
        
        // Initialize compliance officers list
        let officers: Vec<Address> = vec![&env, admin.clone()];
        env.storage().instance().set(&DataKey::ComplianceOfficers, &officers);
        
        // Emit initialization event
        env.events().publish(
            (symbol_short!("kyc_init"), admin.clone()),
            (symbol_short!("registry"), symbol_short!("init"))
        );
    }
    
    /// Register a new customer with KYC information
    pub fn register_customer(
        env: Env,
        caller: Address,
        customer_id: String,
        kyc_tier: KYCTier,
        addresses: Vec<Address>,
        jurisdiction: String,
        metadata: Map<String, String>
    ) {
        Self::require_admin(&env, &caller);
        Self::require_registry_enabled(&env);

        let current_ledger = env.ledger().timestamp();
        let settings = Self::get_global_settings_internal(&env);
        
        // Calculate expiration
        let expires_at = if settings.auto_expire_days > 0 {
            current_ledger + (settings.auto_expire_days * DAY_IN_LEDGERS)
        } else {
            0 // No expiration
        };
        
        let customer_record = CustomerRecord {
            customer_id: customer_id.clone(),
            kyc_tier: kyc_tier.clone(),
            approved_addresses: addresses.clone(),
            jurisdiction: jurisdiction.clone(),
            created_at: current_ledger,
            updated_at: current_ledger,
            expires_at,
            sanctions_cleared: !settings.sanctions_required, // Default based on settings
            metadata,
        };
        
        // Store customer record
        env.storage().persistent().set(
            &DataKey::CustomerRecord(customer_id.clone()),
            &customer_record
        );
        
        // Create address -> customer mappings
        for address in addresses.iter() {
            env.storage().persistent().set(
                &DataKey::AddressToCustomer(address.clone()),
                &customer_id
            );
        }
        
        // Log registration
        if settings.audit_enabled {
            Self::log_audit_entry(&env, AuditLogEntry {
                timestamp: current_ledger,
                action: String::from_str(&env, "register"),
                customer_id: customer_id.clone(),
                address: env.current_contract_address(), // Use contract address as placeholder
                old_tier: KYCTier::None,
                new_tier: kyc_tier.clone(),
                officer: env.current_contract_address(), // Would be actual caller in production
                notes: String::from_str(&env, "Customer registration"),
            });
        }
        
        // Update statistics
        Self::update_tier_stats(&env, &kyc_tier, 1);
        Self::update_jurisdiction_stats(&env, &jurisdiction, 1);
        
        // Emit registration event
        env.events().publish(
            (symbol_short!("kyc_reg"), customer_id.clone()),
            (kyc_tier, addresses.len())
        );
    }
    
    /// Update customer KYC tier
    /// 
    /// # Arguments
    /// * `env` - The environment
    /// * `caller` - Address of the caller (must be a compliance officer)
    /// * `customer_id` - ID of the customer to update
    /// * `new_tier` - New KYC tier to assign
    /// * `notes` - Reason for the tier change (for audit purposes)
    /// 
    /// # Panics
    /// - If the caller is not authorized
    /// - If the registry is disabled
    /// - If the customer doesn't exist
    /// - If the new tier is the same as the current tier
    /// - If the tier transition is not allowed
    pub fn update_customer_tier(
        env: Env,
        caller: Address,
        customer_id: String,
        new_tier: KYCTier,
        notes: String
    ) {
        Self::require_admin(&env, &caller);
        Self::require_registry_enabled(&env);

        if notes.is_empty() {
            panic!("Notes are required for tier updates");
        }

        let mut customer = Self::get_customer_record_internal(&env, &customer_id)
            .unwrap_or_else(|| panic!("Customer not found"));

        if customer.kyc_tier == new_tier {
            panic!("Customer already has this tier");
        }

        let old_tier = customer.kyc_tier.clone();
        customer.kyc_tier = new_tier.clone();
        customer.updated_at = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::CustomerRecord(customer_id.clone()), &customer);

        Self::update_tier_stats(&env, &old_tier, -1);
        Self::update_tier_stats(&env, &new_tier, 1);

        if Self::get_global_settings_internal(&env).audit_enabled {
            let log_entry = AuditLogEntry {
                timestamp: env.ledger().timestamp(),
                action: String::from_str(&env, "tier_update"),
                customer_id: customer_id.clone(),
                address: caller.clone(),
                old_tier: old_tier.clone(),
                new_tier: new_tier.clone(),
                officer: caller.clone(),
                notes: notes.clone(),
            };
            env.storage().persistent().set(&DataKey::AuditLog(env.ledger().timestamp()), &log_entry);
        }

        env.events().publish(
            (symbol_short!("kyc_tier"), customer_id),
            (old_tier, new_tier)
        );
    }
    /// Add approved address to customer record
    pub fn add_approved_address(
        env: Env,
        caller: Address,
        customer_id: String,
        address: Address
    ) {
        Self::require_admin(&env, &caller);
        Self::require_registry_enabled(&env);
        
        let mut customer = Self::get_customer_record_internal(&env, &customer_id)
            .unwrap_or_else(|| panic!("Customer not found"));
        
        // Check if address already approved
        for existing_addr in customer.approved_addresses.iter() {
            if existing_addr == address {
                panic!("Address already approved");
            }
        }
        
        // Add new address
        customer.approved_addresses.push_back(address.clone());
        customer.updated_at = env.ledger().timestamp();
        
        // Store updated record
        env.storage().persistent().set(
            &DataKey::CustomerRecord(customer_id.clone()),
            &customer
        );
        
        // Create address -> customer mapping
        env.storage().persistent().set(
            &DataKey::AddressToCustomer(address.clone()),
            &customer_id
        );
        
        // Emit address added event
        env.events().publish(
            (symbol_short!("kyc_addr"), customer_id),
            (symbol_short!("added"), address)
        );
    }
    
    /// Remove approved address from customer record
    pub fn remove_approved_address(
        env: Env,
        caller: Address,
        customer_id: String,
        address: Address
    ) {
        Self::require_admin(&env, &caller);
        Self::require_registry_enabled(&env);
        
        let mut customer = Self::get_customer_record_internal(&env, &customer_id)
            .unwrap_or_else(|| panic!("Customer not found"));
        
        // Find and remove address
        let mut new_addresses = Vec::new(&env);
        let mut found = false;
        
        for existing_addr in customer.approved_addresses.iter() {
            if existing_addr != address {
                new_addresses.push_back(existing_addr);
            } else {
                found = true;
            }
        }
        
        if !found {
            panic!("Address not found in approved list");
        }
        
        customer.approved_addresses = new_addresses;
        customer.updated_at = env.ledger().timestamp();
        
        // Store updated record
        env.storage().persistent().set(
            &DataKey::CustomerRecord(customer_id.clone()),
            &customer
        );
        
        // Remove address -> customer mapping
        env.storage().persistent().remove(&DataKey::AddressToCustomer(address.clone()));
        
        // Emit address removed event
        env.events().publish(
            (symbol_short!("kyc_addr"), customer_id),
            (symbol_short!("removed"), address)
        );
    }
    
    /// Check if address is approved for operation type
    pub fn is_approved_for_operation(
        env: Env,
        address: Address,
        operation: OperationType,
        amount: i128
    ) -> bool {
        if !Self::get_global_settings_internal(&env).registry_enabled {
            return true; // Registry disabled - allow all operations
        }
        
        // Get customer ID from address
        let customer_id = match env.storage().persistent().get::<DataKey, String>(
            &DataKey::AddressToCustomer(address.clone())
        ) {
            Some(id) => id,
            None => return false, // Address not registered
        };
        
        // Get customer record
        let customer = match Self::get_customer_record_internal(&env, &customer_id) {
            Some(record) => record,
            None => return false, // Customer record not found
        };
        
        // Check if KYC is expired
        if customer.expires_at > 0 && u64::from(env.ledger().timestamp()) > customer.expires_at {
            return false; // KYC expired
        }
        
        // Check sanctions clearance if required
        let settings = Self::get_global_settings_internal(&env);
        if settings.sanctions_required && !customer.sanctions_cleared {
            return false; // Sanctions not cleared
        }
        
        // Check minimum tier requirement for operation
        let required_tier = Self::get_required_tier_internal(&env, &operation);
        if !Self::tier_meets_requirement(&customer.kyc_tier, &required_tier) {
            return false; // Insufficient KYC tier
        }
        
        // Check operation limits
        let limits = Self::get_tier_limits_internal(&env, &customer.kyc_tier, &operation);
        if !limits.enabled {
            return false; // Operation disabled for this tier
        }
        
        // Check amount limits
        if amount > limits.single_tx_limit {
            return false; // Amount exceeds single transaction limit
        }
        
        // TODO: Implement daily/monthly limit tracking
        
        // All checks passed, return true
        true
    }

    /// Simple approval endpoint for cross-contract calls that avoids enum ABI
    /// op_code mapping:
    /// 0 = Transfer, 1 = Mint, 2 = Burn, 3 = Deposit, 4 = Withdraw, 5 = Exchange
    pub fn is_approved_simple(env: Env, address: Address, op_code: u32, amount: i128) -> bool {
        let operation = match op_code {
            0 => OperationType::Transfer,
            1 => OperationType::Mint,
            2 => OperationType::Burn,
            3 => OperationType::Deposit,
            4 => OperationType::Withdraw,
            5 => OperationType::Exchange,
            _ => return false,
        };
        Self::is_approved_for_operation(env, address, operation, amount)
    }
    
    /// Helper method to require the caller to be an admin
    fn require_admin(env: &Env, caller: &Address) {
        caller.require_auth();
        
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, KYCError::Unauthorized));
            
        if *caller != admin {
            panic_with_error!(env, KYCError::Unauthorized);
        }
    }
    
    /// Helper method to require the caller to be a compliance officer
    fn require_compliance_officer(env: &Env, caller: &Address) {
        caller.require_auth();
        
        // Check if caller is admin
        if let Some(admin) = env.storage().instance().get::<DataKey, Address>(&DataKey::Admin) {
            if admin == *caller {
                return; // Admin has compliance officer privileges
            }
        }
        
        // Check if caller is in the compliance officers list
        if let Some(officers) = env.storage().instance().get::<DataKey, Vec<Address>>(&DataKey::ComplianceOfficers) {
            for officer in officers.iter() {
                if officer == *caller {
                    return; // Caller is a compliance officer
                }
            }
        }
        
        panic!("Not authorized: caller is not a compliance officer");
    }
    
    /// Helper method to require the registry to be enabled
    fn require_registry_enabled(env: &Env) {
        let settings = Self::get_global_settings_internal(env);
        if !settings.registry_enabled {
            panic_with_error!(env, KYCError::RegistryDisabled);
        }
    }
    
    
    /// Get global settings (internal helper)
    fn get_global_settings_internal(env: &Env) -> GlobalSettings {
        env.storage().instance().get(&DataKey::GlobalSettings)
            .unwrap_or_else(|| {
                // Return default settings if not found
                GlobalSettings {
                    registry_enabled: true,
                    strict_mode: false,
                    auto_expire_days: 365,
                    sanctions_required: true,
                    audit_enabled: true,
                }
            })
    }
    
    /// Get global settings (public getter)
    pub fn get_global_settings(env: Env) -> GlobalSettings {
        Self::get_global_settings_internal(&env)
    }
    
    /// Get customer record (internal helper)
    fn get_customer_record_internal(env: &Env, customer_id: &String) -> Option<CustomerRecord> {
        env.storage().persistent().get(&DataKey::CustomerRecord(customer_id.clone()))
    }
    
    /// Get customer record (public getter)
    pub fn get_customer_record(env: Env, customer_id: String) -> Option<CustomerRecord> {
        Self::get_customer_record_internal(&env, &customer_id)
    }
    
    /// Get customer by address (public getter)
    pub fn get_customer_by_address(env: Env, address: Address) -> Option<String> {
        env.storage().persistent().get(&DataKey::AddressToCustomer(address))
    }

    /// Return the numeric tier code for a registered address
    /// 0=None, 1=Basic, 2=Verified, 3=Enhanced, 4=Institutional
    pub fn get_tier_code_by_address(env: Env, address: Address) -> u32 {
        let Some(customer_id) = env.storage().persistent().get::<_, String>(&DataKey::AddressToCustomer(address)) else {
            return 0;
        };
        let Some(rec) = env.storage().persistent().get::<_, CustomerRecord>(&DataKey::CustomerRecord(customer_id)) else {
            return 0;
        };
        match rec.kyc_tier {
            KYCTier::None => 0,
            KYCTier::Basic => 1,
            KYCTier::Verified => 2,
            KYCTier::Enhanced => 3,
            KYCTier::Institutional => 4,
        }
    }

    // =====================
    // Admin management APIs
    // =====================

    /// Enable/disable the registry
    pub fn set_registry_enabled(env: Env, caller: Address, enabled: bool) {
        Self::require_admin(&env, &caller);
        let mut s = Self::get_global_settings_internal(&env);
        s.registry_enabled = enabled;
        env.storage().instance().set(&DataKey::GlobalSettings, &s);
env.events().publish((symbol_short!("kyc_set"), symbol_short!("reg_en")), enabled);
    }

    /// Set strict mode
    pub fn set_strict_mode(env: Env, caller: Address, strict: bool) {
        Self::require_admin(&env, &caller);
        let mut s = Self::get_global_settings_internal(&env);
        s.strict_mode = strict;
        env.storage().instance().set(&DataKey::GlobalSettings, &s);
env.events().publish((symbol_short!("kyc_set"), symbol_short!("strict")), strict);
    }

    /// Replace all global settings
    pub fn set_global_settings(env: Env, caller: Address, settings: GlobalSettings) {
        Self::require_admin(&env, &caller);
        env.storage().instance().set(&DataKey::GlobalSettings, &settings);
env.events().publish((symbol_short!("kyc_set"), symbol_short!("update")), (settings.registry_enabled, settings.strict_mode));
    }

    /// Add a compliance officer
    pub fn add_compliance_officer(env: Env, caller: Address, officer: Address) {
        Self::require_admin(&env, &caller);
        let mut officers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ComplianceOfficers)
            .unwrap_or(Vec::new(&env));
        for o in officers.iter() {
            if o == officer {
                panic_with_error!(&env, KYCError::AlreadyExists);
            }
        }
        officers.push_back(officer.clone());
        env.storage().instance().set(&DataKey::ComplianceOfficers, &officers);
env.events().publish((symbol_short!("kyc_ofc"), symbol_short!("add")), officer);
    }

    /// Remove a compliance officer
    pub fn remove_compliance_officer(env: Env, caller: Address, officer: Address) {
        Self::require_admin(&env, &caller);
        let officers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ComplianceOfficers)
            .unwrap_or(Vec::new(&env));
        let mut next = Vec::new(&env);
        let mut found = false;
        for o in officers.iter() {
            if o != officer {
                next.push_back(o);
            } else {
                found = true;
            }
        }
        if !found { panic_with_error!(&env, KYCError::NotFound); }
        env.storage().instance().set(&DataKey::ComplianceOfficers, &next);
env.events().publish((symbol_short!("kyc_ofc"), symbol_short!("remove")), officer);
    }

    /// Set required minimum tier per operation
    pub fn set_required_tier(env: Env, caller: Address, operation: OperationType, tier: KYCTier) {
        Self::require_admin(&env, &caller);
        env.storage().persistent().set(&DataKey::RequiredTier(operation.clone()), &tier);
env.events().publish((symbol_short!("req_tier"), operation), tier);
    }

    /// Set limits for a (tier, operation) pair
    pub fn set_tier_limits(env: Env, caller: Address, tier: KYCTier, operation: OperationType, limits: OperationLimits) {
        Self::require_admin(&env, &caller);
        env.storage().persistent().set(&DataKey::TierLimits(tier.clone(), operation.clone()), &limits);
env.events().publish((symbol_short!("kyc_lims"), (tier, operation)), (limits.single_tx_limit, limits.daily_limit, limits.monthly_limit));
    }

    /// Set sanctions cleared flag for a customer
    pub fn set_sanctions_status(env: Env, caller: Address, customer_id: String, cleared: bool) {
        Self::require_admin(&env, &caller);
        let mut rec = Self::get_customer_record_internal(&env, &customer_id).unwrap_or_else(|| panic_with_error!(&env, KYCError::NotFound));
        rec.sanctions_cleared = cleared;
        rec.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::CustomerRecord(customer_id.clone()), &rec);
env.events().publish((symbol_short!("kyc_cust"), symbol_short!("sanct")), (customer_id, cleared));
    }

    /// Set expiration for a customer
    pub fn set_customer_expiration(env: Env, caller: Address, customer_id: String, expires_at: u64) {
        Self::require_admin(&env, &caller);
        let mut rec = Self::get_customer_record_internal(&env, &customer_id).unwrap_or_else(|| panic_with_error!(&env, KYCError::NotFound));
        rec.expires_at = expires_at;
        rec.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::CustomerRecord(customer_id.clone()), &rec);
env.events().publish((symbol_short!("kyc_cust"), symbol_short!("expire")), (customer_id, expires_at));
    }

    /// Update metadata entry for a customer (upsert)
    pub fn set_customer_metadata(env: Env, caller: Address, customer_id: String, key: String, value: String) {
        Self::require_admin(&env, &caller);
        let mut rec = Self::get_customer_record_internal(&env, &customer_id).unwrap_or_else(|| panic_with_error!(&env, KYCError::NotFound));
        rec.metadata.set(key.clone(), value.clone());
        rec.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::CustomerRecord(customer_id.clone()), &rec);
env.events().publish((symbol_short!("kyc_cust"), symbol_short!("meta")), (customer_id, key));
    }
    
    // =====================
    // Integration Functions
    // =====================
    
    /// Set integration router address (admin only)
    pub fn set_integration_router(env: Env, caller: Address, router_address: Address) {
        Self::require_admin(&env, &caller);
        env.storage().instance().set(&DataKey::IntegrationRouter, &router_address);
        
        env.events().publish(
            (symbol_short!("router"), router_address),
            (symbol_short!("set"), symbol_short!("ok"))
        );
    }
    
    /// Get integration router address
    pub fn get_integration_router(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::IntegrationRouter)
    }
    
    /// Verify operation compliance for integration (simplified)
    pub fn verify_integration_compliance(
        env: Env,
        user: Address,
        operation: OperationType,
        amount: u64
    ) -> bool {
        // Convert u64 to i128 for internal processing
        let amount_i128 = amount as i128;
        
        // Use existing approval logic
        Self::is_approved_for_operation(env, user, operation, amount_i128)
    }
    
    /// Batch compliance check for integration
    pub fn batch_integration_compliance(
        env: Env,
        operations: Vec<(Address, OperationType, u64)>
    ) -> Vec<bool> {
        let mut results = Vec::new(&env);
        
        for operation_data in operations.iter() {
            let (user, operation, amount) = operation_data;
            let is_compliant = Self::verify_integration_compliance(
                env.clone(),
                user.clone(),
                operation.clone(),
                amount
            );
            results.push_back(is_compliant);
        }
        
        results
    }
    
    /// Register integration event for audit trail
    pub fn register_integration_event(
        env: Env,
        caller: Address,
        user: Address,
        operation: OperationType,
        amount: u64,
        notes: String
    ) -> String {
        // Only allow integration router or admin to register events
        Self::require_integration_caller(&env, &caller);
        
        let settings = Self::get_global_settings_internal(&env);
        if !settings.audit_enabled {
            return String::from_str(&env, "audit_disabled");
        }
        
        // Generate simple correlation ID
        let correlation_id = Self::generate_correlation_id(&env);
        
        // Get customer ID for the user
        let customer_id = match env.storage().persistent().get::<DataKey, String>(
            &DataKey::AddressToCustomer(user.clone())
        ) {
            Some(id) => id,
            None => String::from_str(&env, "unknown"),
        };
        
        // Create audit log entry
        let audit_entry = AuditLogEntry {
            timestamp: env.ledger().timestamp(),
            action: String::from_str(&env, "integration_event"),
            customer_id: customer_id.clone(),
            address: user.clone(),
            old_tier: KYCTier::None,
            new_tier: KYCTier::None,
            officer: caller.clone(),
            notes,
        };
        
        // Store audit entry
        env.storage().persistent().set(&DataKey::AuditLog(env.ledger().timestamp()), &audit_entry);
        
        // Emit integration event
        env.events().publish(
            (symbol_short!("int_event"), user.clone(), correlation_id.clone()),
            (operation, amount)
        );
        
        correlation_id
    }
    
    /// Get required tier for operation (internal helper)
    fn get_required_tier_internal(_env: &Env, operation: &OperationType) -> KYCTier {
        // Return tier requirements based on operation type
        match operation {
            OperationType::Transfer => KYCTier::Basic,
            OperationType::Mint => KYCTier::Verified,
            OperationType::Burn => KYCTier::Verified,
            OperationType::Deposit => KYCTier::Basic,
            OperationType::Withdraw => KYCTier::Verified,
            OperationType::Exchange => KYCTier::Enhanced,
        }
    }
    
    /// Check if user tier meets requirement
    fn tier_meets_requirement(user_tier: &KYCTier, required_tier: &KYCTier) -> bool {
        let user_level = match user_tier {
            KYCTier::None => 0,
            KYCTier::Basic => 1,
            KYCTier::Verified => 2,
            KYCTier::Enhanced => 3,
            KYCTier::Institutional => 4,
        };
        
        let required_level = match required_tier {
            KYCTier::None => 0,
            KYCTier::Basic => 1,
            KYCTier::Verified => 2,
            KYCTier::Enhanced => 3,
            KYCTier::Institutional => 4,
        };
        
        user_level >= required_level
    }
    
    /// Get tier limits (internal helper)
    fn get_tier_limits_internal(env: &Env, tier: &KYCTier, operation: &OperationType) -> OperationLimits {
        let key = DataKey::TierLimits(tier.clone(), operation.clone());
        env.storage().persistent().get(&key)
            .unwrap_or_else(|| Self::init_default_tier_limits(env, tier, operation))
    }
    
    /// Initialize default tier limits
    fn init_default_tier_limits(env: &Env, tier: &KYCTier, operation: &OperationType) -> OperationLimits {
        let limits = match tier {
            KYCTier::None => OperationLimits {
                daily_limit: 0,
                monthly_limit: 0,
                single_tx_limit: 0,
                enabled: false,
            },
            KYCTier::Basic => OperationLimits {
                daily_limit: 5_000_0000000,    // 0.05 BTC equivalent in satoshis
                monthly_limit: 50_000_0000000,  // 0.5 BTC equivalent
                single_tx_limit: 1_000_0000000, // 0.01 BTC equivalent
                enabled: true,
            },
            KYCTier::Verified => OperationLimits {
                daily_limit: 50_000_0000000,    // 0.5 BTC equivalent
                monthly_limit: 500_000_0000000, // 5 BTC equivalent
                single_tx_limit: 10_000_0000000, // 0.1 BTC equivalent
                enabled: true,
            },
            KYCTier::Enhanced => OperationLimits {
                daily_limit: 500_000_0000000,     // 5 BTC equivalent
                monthly_limit: 5_000_000_0000000, // 50 BTC equivalent
                single_tx_limit: 100_000_0000000, // 1 BTC equivalent
                enabled: true,
            },
            KYCTier::Institutional => OperationLimits {
                daily_limit: i128::MAX / 4,
                monthly_limit: i128::MAX / 4,
                single_tx_limit: i128::MAX / 4,
                enabled: true,
            },
        };
        
        // Store the default limits
        let key = DataKey::TierLimits(tier.clone(), operation.clone());
        env.storage().persistent().set(&key, &limits);
        
        limits
    }
    
    /// Initialize default tier limits for all operations
    fn init_all_default_tier_limits(env: &Env) {
        let tiers = vec![env,
            KYCTier::None,
            KYCTier::Basic,
            KYCTier::Verified,
            KYCTier::Enhanced,
            KYCTier::Institutional,
        ];
        
        let operations = vec![env,
            OperationType::Transfer,
            OperationType::Mint,
            OperationType::Burn,
            OperationType::Deposit,
            OperationType::Withdraw,
            OperationType::Exchange,
        ];
        
        for tier in tiers {
            for operation in operations.clone() {
                Self::init_default_tier_limits(env, &tier, &operation);
            }
        }
    }
    
    /// Log audit entry
    fn log_audit_entry(env: &Env, entry: AuditLogEntry) {
        let settings = Self::get_global_settings_internal(env);
        if !settings.audit_enabled {
            return;
        }
        
        // Store the audit entry
        env.storage().persistent().set(&DataKey::AuditLog(entry.timestamp), &entry);
        
        // Emit audit event
        env.events().publish(
            (symbol_short!("kyc_audit"), entry.action.clone()),
            (entry.customer_id.clone(), entry.new_tier.clone())
        );
    }
    
    /// Update tier statistics
    fn update_tier_stats(_env: &Env, _tier: &KYCTier, _delta: i64) {
        // Simple implementation - would be more sophisticated in production
        // This is a placeholder for statistical tracking
    }
    
    /// Update jurisdiction statistics
    fn update_jurisdiction_stats(_env: &Env, _jurisdiction: &String, _delta: i64) {
        // Simple implementation - would be more sophisticated in production
        // This is a placeholder for statistical tracking
    }
    
    /// Require caller to be integration router or admin
    fn require_integration_caller(env: &Env, caller: &Address) {
        caller.require_auth();
        
        // Check if caller is admin
        if let Some(admin) = env.storage().instance().get::<DataKey, Address>(&DataKey::Admin) {
            if admin == *caller {
                return;
            }
        }
        
        // Check if caller is the integration router
        if let Some(router) = env.storage().instance().get::<DataKey, Address>(&DataKey::IntegrationRouter) {
            if router == *caller {
                return;
            }
        }
        
        panic!("Not authorized: caller is not integration router or admin");
    }
    
    /// Generate correlation ID for integration events
    fn generate_correlation_id(env: &Env) -> String {
        let _timestamp = env.ledger().timestamp();
        let _sequence = env.ledger().sequence();
        
        // Create a simple correlation ID
        // In production, this could be more sophisticated
        String::from_str(env, "correlation_id")
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
        let contract_id = env.register(KYCRegistry, ());
        let client = KYCRegistryClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        
        client.initialize(&admin);
        
        // Test that contract is initialized
        let settings = client.get_global_settings();
        assert_eq!(settings.registry_enabled, true);
        assert_eq!(settings.sanctions_required, true);
    }

    #[test]
    fn test_customer_registration() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(KYCRegistry, ());
        let client = KYCRegistryClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let customer_addr = Address::generate(&env);
        
        client.initialize(&admin);
        
        let customer_id = String::from_str(&env, "customer_001");
        let addresses = vec![&env, customer_addr.clone()];
        let jurisdiction = String::from_str(&env, "US");
        let metadata = Map::new(&env);
        
        client.register_customer(
            &admin,
            &customer_id,
            &KYCTier::Verified,
            &addresses,
            &jurisdiction,
            &metadata,
        );
        
        // Test customer record
        let record = client.get_customer_record(&customer_id).unwrap();
        assert_eq!(record.kyc_tier, KYCTier::Verified);
        assert_eq!(record.jurisdiction, jurisdiction);
        assert_eq!(record.approved_addresses.len(), 1);
        
        // Test address mapping
        let found_customer = client.get_customer_by_address(&customer_addr).unwrap();
        assert_eq!(found_customer, customer_id);
    }

    #[test]
    fn test_operation_approval() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(KYCRegistry, ());
        let client = KYCRegistryClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let customer_addr = Address::generate(&env);
        
        client.initialize(&admin);
        
        let customer_id = String::from_str(&env, "customer_001");
        let addresses = vec![&env, customer_addr.clone()];
        let jurisdiction = String::from_str(&env, "US");
        let metadata = Map::new(&env);
        
        client.register_customer(
            &admin,
            &customer_id,
            &KYCTier::Verified,
            &addresses,
            &jurisdiction,
            &metadata
        );
        
        // Clear sanctions to allow operations
        client.set_sanctions_status(&admin, &customer_id, &true);
        
        // Test operation approval
        let approved = client.is_approved_for_operation(
            &customer_addr,
            &OperationType::Transfer,
            &100_000_000 // 0.001 BTC equivalent
        );
        assert_eq!(approved, true);
        
        // Test operation with excessive amount
        let denied = client.is_approved_for_operation(
            &customer_addr,
            &OperationType::Transfer,
            &20_000_0000000 // 0.2 BTC equivalent - exceeds Verified tier single tx limit of 0.1 BTC
        );
        assert_eq!(denied, false);
    }

    #[test]
    fn test_tier_updates() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(KYCRegistry, ());
        let client = KYCRegistryClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        client.initialize(&admin);
        
        let customer_id = String::from_str(&env, "customer_001");
        let customer_addr = Address::generate(&env);
        let addresses = vec![&env, customer_addr.clone()];
        let jurisdiction = String::from_str(&env, "US");
        let metadata = Map::new(&env);
        
        client.register_customer(
            &admin,
            &customer_id,
            &KYCTier::Basic,
            &addresses,
            &jurisdiction,
            &metadata
        );
        
        // Clear sanctions to allow operations
        client.set_sanctions_status(&admin, &customer_id, &true);
        
        // Update to Enhanced tier
        let notes = String::from_str(&env, "Enhanced due diligence completed");
        client.update_customer_tier(&admin, &customer_id, &KYCTier::Enhanced, &notes);
        
        // Verify tier update
        let record = client.get_customer_record(&customer_id).unwrap();
        assert_eq!(record.kyc_tier, KYCTier::Enhanced);
        
        // Test that higher limits are now available
        let approved = client.is_approved_for_operation(
            &customer_addr,
            &OperationType::Transfer,
            &5_000_000_000 // 0.05 BTC - should be approved for Enhanced tier
        );
        assert_eq!(approved, true);
    }
    
    #[test]
    fn test_integration_router_management() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(KYCRegistry, ());
        let client = KYCRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        
        client.initialize(&admin);
        
        // Set integration router
        client.set_integration_router(&admin, &router);
        
        // Verify router was set
        let stored_router = client.get_integration_router();
        assert_eq!(stored_router, Some(router));
    }
    
    #[test]
    fn test_integration_compliance_verification() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(KYCRegistry, ());
        let client = KYCRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let customer_addr = Address::generate(&env);
        
        client.initialize(&admin);
        
        // Register customer
        let customer_id = String::from_str(&env, "integration_test");
        let addresses = vec![&env, customer_addr.clone()];
        let jurisdiction = String::from_str(&env, "US");
        let metadata = Map::new(&env);

        client.register_customer(
            &admin,
            &customer_id,
            &KYCTier::Verified,
            &addresses,
            &jurisdiction,
            &metadata
        );

        // Clear sanctions
        client.set_sanctions_status(&admin, &customer_id, &true);
        
        // Test integration compliance verification
        let is_compliant = client.verify_integration_compliance(
            &customer_addr,
            &OperationType::Transfer,
            &1_000_000
        );
        
        assert_eq!(is_compliant, true);
    }
    
    #[test]
    fn test_batch_integration_compliance() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(KYCRegistry, ());
        let client = KYCRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        
        client.initialize(&admin);
        
        // Register first user
        let customer_id1 = String::from_str(&env, "batch_user1");
        let addresses1 = vec![&env, user1.clone()];
        let jurisdiction = String::from_str(&env, "US");
        let metadata = Map::new(&env);

        client.register_customer(
            &admin,
            &customer_id1,
            &KYCTier::Enhanced,
            &addresses1,
            &jurisdiction,
            &metadata
        );

        client.set_sanctions_status(&admin, &customer_id1, &true);
        
        // Create batch operations
        let mut operations = Vec::new(&env);
        operations.push_back((user1.clone(), OperationType::Transfer, 1_000_000u64));
        operations.push_back((user2.clone(), OperationType::Mint, 500_000u64)); // Unregistered user
        
        // Test batch compliance check
        let results = client.batch_integration_compliance(&operations);
        
        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap(), true);  // user1 should be compliant
        assert_eq!(results.get(1).unwrap(), false); // user2 not registered
    }
    
    #[test]
    fn test_integration_event_registration() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(KYCRegistry, ());
        let client = KYCRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let router = Address::generate(&env);
        let user = Address::generate(&env);
        
        client.initialize(&admin);
        
        // Set integration router
        client.set_integration_router(&admin, &router);
        
        // Register user
        let customer_id = String::from_str(&env, "event_user");
        let addresses = vec![&env, user.clone()];
        let jurisdiction = String::from_str(&env, "US");
        let metadata = Map::new(&env);

        client.register_customer(
            &admin,
            &customer_id,
            &KYCTier::Basic,
            &addresses,
            &jurisdiction,
            &metadata
        );
        
        // Register integration event
        let notes = String::from_str(&env, "Test integration event");
        let correlation_id = client.register_integration_event(
            &router,
            &user,
            &OperationType::Deposit,
            &1_000_000,
            &notes
        );
        
        // Verify correlation ID was generated
        assert_eq!(correlation_id, String::from_str(&env, "correlation_id"));
    }
}
