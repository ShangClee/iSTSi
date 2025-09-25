use soroban_sdk::{Address, Env};
use alloc::string::ToString;
use crate::{
    ContractResult, ContractError, OperationContext, ContractClient,
    IntegrationRouterClient, KycRegistryClient, IstsiTokenClient, ReserveManagerClient,
    ContractAddresses, NetworkConfig
};

/// Central contract manager for coordinating all contract interactions
/// 
/// This manager provides a unified interface for backend services to interact
/// with all Soroban contracts in the Bitcoin custody system.
pub struct ContractManager {
    env: Env,
    addresses: ContractAddresses,
    network_config: NetworkConfig,
    
    // Contract clients
    integration_router: IntegrationRouterClient,
    kyc_registry: KycRegistryClient,
    istsi_token: IstsiTokenClient,
    reserve_manager: ReserveManagerClient,
}

impl ContractManager {
    /// Create a new contract manager
    /// 
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `addresses` - Contract addresses configuration
    /// * `network_config` - Network configuration
    /// 
    /// # Returns
    /// * `Ok(manager)` - Contract manager instance
    /// * `Err(ContractError)` - Error details
    pub fn new(
        env: Env,
        addresses: ContractAddresses,
        network_config: NetworkConfig,
    ) -> ContractResult<Self> {
        // Validate that all required addresses are provided
        if addresses.integration_router.is_none() {
            return Err(ContractError::ContractNotFound("integration_router".to_string()));
        }
        
        if addresses.kyc_registry.is_none() {
            return Err(ContractError::ContractNotFound("kyc_registry".to_string()));
        }
        
        if addresses.istsi_token.is_none() {
            return Err(ContractError::ContractNotFound("istsi_token".to_string()));
        }
        
        if addresses.reserve_manager.is_none() {
            return Err(ContractError::ContractNotFound("reserve_manager".to_string()));
        }

        // Create contract clients
        let integration_router = IntegrationRouterClient::new(
            env.clone(),
            addresses.integration_router.clone().unwrap(),
        );
        
        let kyc_registry = KycRegistryClient::new(
            env.clone(),
            addresses.kyc_registry.clone().unwrap(),
        );
        
        let istsi_token = IstsiTokenClient::new(
            env.clone(),
            addresses.istsi_token.clone().unwrap(),
        );
        
        let reserve_manager = ReserveManagerClient::new(
            env.clone(),
            addresses.reserve_manager.clone().unwrap(),
        );

        Ok(Self {
            env,
            addresses,
            network_config,
            integration_router,
            kyc_registry,
            istsi_token,
            reserve_manager,
        })
    }

    /// Get the integration router client
    pub fn integration_router(&self) -> &IntegrationRouterClient {
        &self.integration_router
    }

    /// Get the KYC registry client
    pub fn kyc_registry(&self) -> &KycRegistryClient {
        &self.kyc_registry
    }

    /// Get the iSTSi token client
    pub fn istsi_token(&self) -> &IstsiTokenClient {
        &self.istsi_token
    }

    /// Get the reserve manager client
    pub fn reserve_manager(&self) -> &ReserveManagerClient {
        &self.reserve_manager
    }

    /// Execute a complete Bitcoin deposit workflow
    /// 
    /// This method orchestrates the entire Bitcoin deposit process across
    /// multiple contracts with proper error handling and rollback.
    /// 
    /// # Arguments
    /// * `ctx` - Operation context
    /// * `user` - User address
    /// * `btc_amount` - Bitcoin amount in satoshis
    /// * `btc_tx_hash` - Bitcoin transaction hash
    /// * `confirmations` - Number of confirmations
    /// * `block_height` - Bitcoin block height
    /// 
    /// # Returns
    /// * `Ok(operation_id)` - Successful operation ID
    /// * `Err(ContractError)` - Error details
    pub fn execute_bitcoin_deposit_workflow(
        &self,
        ctx: &OperationContext,
        user: &Address,
        btc_amount: u64,
        btc_tx_hash: &soroban_sdk::BytesN<32>,
        confirmations: u32,
        block_height: u64,
    ) -> ContractResult<soroban_sdk::BytesN<32>> {
        // Step 1: Check KYC compliance
        let kyc_approved = self.kyc_registry.is_approved_for_operation(
            user,
            3, // Deposit operation
            btc_amount,
        )?;
        
        if !kyc_approved {
            return Err(ContractError::Integration(
                shared::IntegrationError::ComplianceCheckFailed
            ));
        }

        // Step 2: Register Bitcoin deposit with reserve manager
        self.reserve_manager.register_bitcoin_deposit(
            ctx,
            btc_tx_hash,
            btc_amount,
            confirmations,
            user,
            block_height,
        )?;

        // Step 3: Process deposit if confirmations are sufficient
        if confirmations >= self.network_config.min_confirmations {
            self.reserve_manager.process_bitcoin_deposit(ctx, btc_tx_hash)?;
            
            // Step 4: Mint iSTSi tokens
            let istsi_amount = self.calculate_istsi_amount(btc_amount)?;
            self.istsi_token.mint_with_btc_link(ctx, user, istsi_amount, btc_tx_hash)?;
            
            // Step 5: Update token supply in reserve manager
            let new_supply = self.istsi_token.total_supply()?;
            self.reserve_manager.update_token_supply(ctx, new_supply)?;
        }

        // Step 6: Execute through integration router for coordination
        let operation_id = self.integration_router.execute_bitcoin_deposit(
            ctx,
            user,
            btc_amount,
            btc_tx_hash,
            confirmations,
        )?;

        Ok(operation_id)
    }

    /// Execute a complete token withdrawal workflow
    /// 
    /// This method orchestrates the entire token withdrawal process across
    /// multiple contracts with proper error handling and rollback.
    /// 
    /// # Arguments
    /// * `ctx` - Operation context
    /// * `user` - User address
    /// * `istsi_amount` - iSTSi token amount to withdraw
    /// * `btc_address` - Bitcoin address for withdrawal
    /// 
    /// # Returns
    /// * `Ok(withdrawal_id)` - Successful withdrawal ID
    /// * `Err(ContractError)` - Error details
    pub fn execute_token_withdrawal_workflow(
        &self,
        ctx: &OperationContext,
        user: &Address,
        istsi_amount: u64,
        btc_address: &str,
    ) -> ContractResult<soroban_sdk::BytesN<32>> {
        // Step 1: Check KYC compliance
        let kyc_approved = self.kyc_registry.is_approved_for_operation(
            user,
            4, // Withdrawal operation
            istsi_amount,
        )?;
        
        if !kyc_approved {
            return Err(ContractError::Integration(
                shared::IntegrationError::ComplianceCheckFailed
            ));
        }

        // Step 2: Check token balance
        let balance = self.istsi_token.balance(user)?;
        if balance < istsi_amount {
            return Err(ContractError::Integration(
                shared::IntegrationError::InsufficientReserves
            ));
        }

        // Step 3: Calculate Bitcoin amount
        let btc_amount = self.calculate_btc_amount(istsi_amount)?;

        // Step 4: Check reserve availability
        let total_reserves = self.reserve_manager.get_total_reserves()?;
        if total_reserves < btc_amount {
            return Err(ContractError::Integration(
                shared::IntegrationError::InsufficientReserves
            ));
        }

        // Step 5: Burn iSTSi tokens
        let burn_request_id = self.istsi_token.burn_for_btc_withdrawal(
            ctx,
            user,
            istsi_amount,
            btc_address,
        )?;

        // Step 6: Create withdrawal request
        let withdrawal_id = self.reserve_manager.create_withdrawal_request(
            ctx,
            user,
            btc_amount,
            btc_address,
        )?;

        // Step 7: Update token supply
        let new_supply = self.istsi_token.total_supply()?;
        self.reserve_manager.update_token_supply(ctx, new_supply)?;

        // Step 8: Execute through integration router for coordination
        let _operation_id = self.integration_router.execute_token_withdrawal(
            ctx,
            user,
            istsi_amount,
            btc_address,
        )?;

        Ok(withdrawal_id)
    }

    /// Execute a cross-token exchange workflow
    /// 
    /// # Arguments
    /// * `ctx` - Operation context
    /// * `user` - User address
    /// * `from_token` - Source token address
    /// * `to_token` - Destination token address
    /// * `from_amount` - Amount to exchange
    /// 
    /// # Returns
    /// * `Ok((operation_id, to_amount))` - Operation ID and received amount
    /// * `Err(ContractError)` - Error details
    pub fn execute_cross_token_exchange_workflow(
        &self,
        ctx: &OperationContext,
        user: &Address,
        from_token: &Address,
        to_token: &Address,
        from_amount: u64,
    ) -> ContractResult<(soroban_sdk::BytesN<32>, u64)> {
        // Step 1: Check KYC compliance
        let kyc_approved = self.kyc_registry.is_approved_for_operation(
            user,
            5, // Exchange operation
            from_amount,
        )?;
        
        if !kyc_approved {
            return Err(ContractError::Integration(
                shared::IntegrationError::ComplianceCheckFailed
            ));
        }

        // Step 2: Execute through integration router
        let (operation_id, to_amount) = self.integration_router.execute_cross_token_exchange(
            ctx,
            user,
            from_token,
            to_token,
            from_amount,
        )?;

        Ok((operation_id, to_amount))
    }

    /// Check system health across all contracts
    /// 
    /// # Returns
    /// * `Ok(health)` - System health status
    /// * `Err(ContractError)` - Error details
    pub fn check_system_health(&self) -> ContractResult<SystemHealth> {
        let mut health = SystemHealth {
            integration_router_available: false,
            kyc_registry_available: false,
            istsi_token_available: false,
            reserve_manager_available: false,
            system_paused: false,
            reserve_ratio_healthy: false,
            last_checked: self.env.ledger().timestamp(),
        };

        // Check contract availability
        health.integration_router_available = self.integration_router.is_available();
        health.kyc_registry_available = self.kyc_registry.is_available();
        health.istsi_token_available = self.istsi_token.is_available();
        health.reserve_manager_available = self.reserve_manager.is_available();

        // Check if system is paused
        health.system_paused = self.integration_router.is_paused().unwrap_or(true);

        // Check reserve ratio health
        if let Ok(ratio) = self.reserve_manager.get_reserve_ratio() {
            health.reserve_ratio_healthy = ratio >= 10000; // At least 100% backing
        }

        Ok(health)
    }

    /// Get comprehensive system status
    /// 
    /// # Returns
    /// * `Ok(status)` - System status
    /// * `Err(ContractError)` - Error details
    pub fn get_system_status(&self) -> ContractResult<SystemStatus> {
        let total_reserves = self.reserve_manager.get_total_reserves()?;
        let total_supply = self.reserve_manager.get_total_token_supply()?;
        let reserve_ratio = self.reserve_manager.get_reserve_ratio()?;
        let integration_enabled = self.istsi_token.is_integration_enabled()?;
        let kyc_enabled = self.kyc_registry.is_registry_enabled()?;

        Ok(SystemStatus {
            total_btc_reserves: total_reserves,
            total_istsi_supply: total_supply,
            reserve_ratio_bp: reserve_ratio,
            integration_enabled,
            kyc_enabled,
            system_paused: self.integration_router.is_paused().unwrap_or(false),
            last_updated: self.env.ledger().timestamp(),
        })
    }

    /// Helper function to calculate iSTSi amount from Bitcoin amount
    fn calculate_istsi_amount(&self, btc_amount: u64) -> ContractResult<u64> {
        // Simplified 1:1 conversion for now
        // In a real implementation, this would use exchange rates
        Ok(btc_amount)
    }

    /// Helper function to calculate Bitcoin amount from iSTSi amount
    fn calculate_btc_amount(&self, istsi_amount: u64) -> ContractResult<u64> {
        // Simplified 1:1 conversion for now
        // In a real implementation, this would use exchange rates
        Ok(istsi_amount)
    }
}

/// System health status
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub integration_router_available: bool,
    pub kyc_registry_available: bool,
    pub istsi_token_available: bool,
    pub reserve_manager_available: bool,
    pub system_paused: bool,
    pub reserve_ratio_healthy: bool,
    pub last_checked: u64,
}

/// Comprehensive system status
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub total_btc_reserves: u64,
    pub total_istsi_supply: u64,
    pub reserve_ratio_bp: u64,
    pub integration_enabled: bool,
    pub kyc_enabled: bool,
    pub system_paused: bool,
    pub last_updated: u64,
}