use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::soroban_client::{
    SorobanClient, SorobanConfig, ContractCall, BatchOperation, 
    EventFilter, ContractEvent, SorobanError, SigningConfig
};
use super::event_monitor_service::EventMonitorService;
use std::sync::Arc;

// Type alias to avoid conflicts with loco_rs::Result
type SorobanResult<T> = std::result::Result<T, SorobanError>;

/// Bitcoin deposit request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinDepositRequest {
    pub user_address: String,
    pub btc_amount: u64,
    pub btc_tx_hash: String,
    pub confirmations: u32,
}

/// Token withdrawal request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenWithdrawalRequest {
    pub user_address: String,
    pub token_amount: u64,
    pub btc_address: String,
}

/// Cross-token exchange request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossTokenExchangeRequest {
    pub user_address: String,
    pub from_token: String,
    pub to_token: String,
    pub amount: u64,
}

/// Operation status for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

/// Integration operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationOperationResult {
    pub operation_id: String,
    pub status: OperationStatus,
    pub transaction_hash: Option<String>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub gas_used: u64,
}

/// System overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    pub total_btc_reserves: u64,
    pub total_token_supply: u64,
    pub reserve_ratio: f64,
    pub active_operations: u64,
    pub system_health: String,
    pub last_reconciliation: u64,
}

/// Integration service for coordinating blockchain operations
pub struct IntegrationService {
    soroban_client: Arc<SorobanClient>,
    event_monitor: Option<Arc<EventMonitorService>>,
}

impl IntegrationService {
    /// Create a new integration service with Soroban client
    pub fn new(config: SorobanConfig) -> SorobanResult<Self> {
        let soroban_client = Arc::new(SorobanClient::new(config)?);
        
        Ok(Self {
            soroban_client,
            event_monitor: None,
        })
    }

    /// Create a new integration service with signing configuration
    pub fn new_with_signing(config: SorobanConfig, signing_config: SigningConfig) -> SorobanResult<Self> {
        let soroban_client = Arc::new(SorobanClient::new_with_signing(config, signing_config)?);
        
        Ok(Self {
            soroban_client,
            event_monitor: None,
        })
    }

    /// Initialize event monitoring
    pub async fn initialize_event_monitoring(&mut self) -> SorobanResult<()> {
        let event_monitor = Arc::new(EventMonitorService::new(Arc::clone(&self.soroban_client)));
        
        event_monitor.initialize().await
            .map_err(|e| SorobanError::EventMonitoringError(e.to_string()))?;
        
        event_monitor.start_monitoring().await
            .map_err(|e| SorobanError::EventMonitoringError(e.to_string()))?;
        
        self.event_monitor = Some(event_monitor);
        
        info!("Event monitoring initialized and started");
        Ok(())
    }

    /// Get event monitoring statistics
    pub async fn get_event_statistics(&self) -> Option<super::event_monitor_service::EventStatistics> {
        if let Some(monitor) = &self.event_monitor {
            Some(monitor.get_statistics().await)
        } else {
            None
        }
    }
    
    /// Process a Bitcoin deposit through the integration router
    pub async fn execute_bitcoin_deposit(
        &self,
        request: BitcoinDepositRequest,
    ) -> SorobanResult<IntegrationOperationResult> {
        let operation_id = Uuid::new_v4().to_string();
        
        info!("Processing Bitcoin deposit: {} BTC for user {}", 
              request.btc_amount, request.user_address);
        
        // Validate minimum confirmations
        if request.confirmations < 3 {
            warn!("Insufficient confirmations: {} < 3", request.confirmations);
            return Ok(IntegrationOperationResult {
                operation_id,
                status: OperationStatus::Failed,
                transaction_hash: None,
                error_message: Some("Insufficient Bitcoin confirmations".to_string()),
                execution_time_ms: 0,
                gas_used: 0,
            });
        }
        
        // Execute the deposit through Soroban
        let result = self.soroban_client.execute_bitcoin_deposit(
            &request.user_address,
            request.btc_amount,
            &request.btc_tx_hash,
        ).await?;
        
        let status = if result.success {
            OperationStatus::Completed
        } else {
            OperationStatus::Failed
        };
        
        Ok(IntegrationOperationResult {
            operation_id,
            status,
            transaction_hash: result.transaction_hash,
            error_message: result.error_message,
            execution_time_ms: result.execution_time_ms,
            gas_used: result.gas_used,
        })
    }
    
    /// Process a token withdrawal through the integration router
    pub async fn execute_token_withdrawal(
        &self,
        request: TokenWithdrawalRequest,
    ) -> SorobanResult<IntegrationOperationResult> {
        let operation_id = Uuid::new_v4().to_string();
        
        info!("Processing token withdrawal: {} tokens for user {} to BTC address {}", 
              request.token_amount, request.user_address, request.btc_address);
        
        // Validate Bitcoin address format (basic validation)
        if !self.is_valid_btc_address(&request.btc_address) {
            return Ok(IntegrationOperationResult {
                operation_id,
                status: OperationStatus::Failed,
                transaction_hash: None,
                error_message: Some("Invalid Bitcoin address format".to_string()),
                execution_time_ms: 0,
                gas_used: 0,
            });
        }
        
        // Execute the withdrawal through Soroban
        let result = self.soroban_client.execute_token_withdrawal(
            &request.user_address,
            request.token_amount,
            &request.btc_address,
        ).await?;
        
        let status = if result.success {
            OperationStatus::Completed
        } else {
            OperationStatus::Failed
        };
        
        Ok(IntegrationOperationResult {
            operation_id,
            status,
            transaction_hash: result.transaction_hash,
            error_message: result.error_message,
            execution_time_ms: result.execution_time_ms,
            gas_used: result.gas_used,
        })
    }
    
    /// Process a cross-token exchange
    pub async fn process_cross_token_exchange(
        &self,
        request: CrossTokenExchangeRequest,
    ) -> SorobanResult<IntegrationOperationResult> {
        let operation_id = Uuid::new_v4().to_string();
        
        info!("Processing cross-token exchange: {} from {} to {} for user {}", 
              request.amount, request.from_token, request.to_token, request.user_address);
        
        // Get contract addresses for tokens
        let _from_token_address = self.soroban_client.get_contract_address(&request.from_token)?;
        let _to_token_address = self.soroban_client.get_contract_address(&request.to_token)?;
        
        // Create batch operation for atomic exchange
        let calls = vec![
            ContractCall {
                contract_address: self.soroban_client.get_contract_address("integration_router")?,
                function_name: "execute_cross_token_exchange".to_string(),
                parameters: vec![], // Parameters would be properly constructed here
                timeout_seconds: 30,
                retry_count: 3,
            }
        ];
        
        let batch_op = BatchOperation {
            operation_id: operation_id.clone(),
            calls,
            rollback_calls: vec![], // Rollback calls would be defined here
            atomic: true,
            timeout_seconds: 60,
        };
        
        let batch_result = self.soroban_client.execute_batch_operation(batch_op).await?;
        
        let status = if batch_result.overall_success {
            OperationStatus::Completed
        } else if batch_result.rollback_executed {
            OperationStatus::RolledBack
        } else {
            OperationStatus::Failed
        };
        
        Ok(IntegrationOperationResult {
            operation_id,
            status,
            transaction_hash: None, // Would extract from batch result
            error_message: None,
            execution_time_ms: batch_result.total_execution_time_ms,
            gas_used: batch_result.call_results.iter().map(|r| r.gas_used).sum(),
        })
    }
    
    /// Get system overview from all contracts
    pub async fn get_system_overview(&self) -> SorobanResult<SystemOverview> {
        debug!("Fetching system overview");
        
        let _result = self.soroban_client.get_system_overview().await?;
        
        // Parse the system health data from contract response
        // This would normally parse the actual contract response
        Ok(SystemOverview {
            total_btc_reserves: 0, // Would be parsed from contract response
            total_token_supply: 0,
            reserve_ratio: 1.0,
            active_operations: 0,
            system_health: "Healthy".to_string(),
            last_reconciliation: 0,
        })
    }
    
    /// Start monitoring contract events
    pub async fn start_event_monitoring(&self) -> SorobanResult<()> {
        info!("Starting contract event monitoring");
        
        let _filter = EventFilter {
            contract_addresses: vec![
                self.soroban_client.get_contract_address("integration_router")?,
                self.soroban_client.get_contract_address("kyc_registry")?,
                self.soroban_client.get_contract_address("istsi_token")?,
                self.soroban_client.get_contract_address("reserve_manager")?,
            ],
            event_types: vec![
                "BitcoinDepositCompleted".to_string(),
                "TokenWithdrawalCompleted".to_string(),
                "CrossTokenExchangeCompleted".to_string(),
                "ComplianceViolation".to_string(),
                "SystemAlert".to_string(),
            ],
            from_ledger: None,
            to_ledger: None,
        };
        
        // For now, just return Ok - event monitoring would be implemented
        // in a separate background service
        Ok(())
    }
    
    /// Handle incoming contract events
    fn handle_contract_event(&self, event: ContractEvent) {
        info!("Received contract event: {} from {}", event.event_type, event.contract_address);
        
        match event.event_type.as_str() {
            "BitcoinDepositCompleted" => {
                debug!("Bitcoin deposit completed: {:?}", event.data);
                // Handle deposit completion logic
            }
            "TokenWithdrawalCompleted" => {
                debug!("Token withdrawal completed: {:?}", event.data);
                // Handle withdrawal completion logic
            }
            "CrossTokenExchangeCompleted" => {
                debug!("Cross-token exchange completed: {:?}", event.data);
                // Handle exchange completion logic
            }
            "ComplianceViolation" => {
                warn!("Compliance violation detected: {:?}", event.data);
                // Handle compliance violation
            }
            "SystemAlert" => {
                error!("System alert: {:?}", event.data);
                // Handle system alert
            }
            _ => {
                debug!("Unknown event type: {}", event.event_type);
            }
        }
    }
    
    /// Get recent contract events
    pub async fn get_recent_events(
        &self,
        contract_name: Option<String>,
        event_type: Option<String>,
        limit: u32,
    ) -> SorobanResult<Vec<ContractEvent>> {
        let contract_addresses = if let Some(name) = contract_name {
            vec![self.soroban_client.get_contract_address(&name)?]
        } else {
            vec![
                self.soroban_client.get_contract_address("integration_router")?,
                self.soroban_client.get_contract_address("kyc_registry")?,
                self.soroban_client.get_contract_address("istsi_token")?,
                self.soroban_client.get_contract_address("reserve_manager")?,
            ]
        };
        
        let event_types = if let Some(event_type) = event_type {
            vec![event_type]
        } else {
            vec![]
        };
        
        let filter = EventFilter {
            contract_addresses,
            event_types,
            from_ledger: None,
            to_ledger: None,
        };
        
        self.soroban_client.get_recent_events(filter, limit).await
    }
    
    /// Validate Bitcoin address format (basic validation)
    fn is_valid_btc_address(&self, address: &str) -> bool {
        // Basic validation - in production this would be more comprehensive
        address.len() >= 26 && address.len() <= 35 && 
        (address.starts_with('1') || address.starts_with('3') || address.starts_with("bc1"))
    }
}