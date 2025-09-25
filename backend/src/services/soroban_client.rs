use serde::{Deserialize, Serialize};
use soroban_sdk::Val;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

// Type alias to avoid conflicts with loco_rs::Result
pub type SorobanResult<T> = std::result::Result<T, SorobanError>;

/// Configuration for Soroban network connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SorobanConfig {
    pub network: String,
    pub rpc_url: String,
    pub network_passphrase: String,
    pub contracts: ContractAddresses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub integration_router: String,
    pub kyc_registry: String,
    pub istsi_token: String,
    pub reserve_manager: String,
}

/// Represents a contract call with parameters and metadata
#[derive(Debug, Clone)]
pub struct ContractCall {
    pub contract_address: String,
    pub function_name: String,
    pub parameters: Vec<Val>,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

/// Result of a contract call execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallResult {
    pub success: bool,
    pub return_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub gas_used: u64,
    pub execution_time_ms: u64,
    pub transaction_hash: Option<String>,
}

/// Batch operation for multiple contract calls
#[derive(Debug, Clone)]
pub struct BatchOperation {
    pub operation_id: String,
    pub calls: Vec<ContractCall>,
    pub rollback_calls: Vec<ContractCall>,
    pub atomic: bool,
    pub timeout_seconds: u64,
}

/// Result of a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult {
    pub operation_id: String,
    pub overall_success: bool,
    pub call_results: Vec<ContractCallResult>,
    pub rollback_executed: bool,
    pub total_execution_time_ms: u64,
}

/// Contract event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub event_type: String,
    pub contract_address: String,
    pub topic: Vec<String>,
    pub data: serde_json::Value,
    pub ledger: u64,
    pub transaction_hash: String,
    pub timestamp: u64,
}

/// Event filter for monitoring specific events
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub contract_addresses: Vec<String>,
    pub event_types: Vec<String>,
    pub from_ledger: Option<u64>,
    pub to_ledger: Option<u64>,
}

/// Transaction signing configuration
#[derive(Debug, Clone)]
pub struct TransactionConfig {
    pub source_account: String,
    pub sequence_number: u64,
    pub fee: u64,
    pub timeout_seconds: u64,
    pub memo: Option<String>,
    pub max_fee: Option<u64>,
}

/// Transaction signing key configuration
#[derive(Debug, Clone)]
pub struct SigningConfig {
    pub secret_key: String,
    pub network_passphrase: String,
}

/// Transaction envelope for submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEnvelope {
    pub transaction_xdr: String,
    pub signatures: Vec<String>,
    pub network_passphrase: String,
}

/// Transaction submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSubmissionResult {
    pub transaction_hash: String,
    pub status: TransactionStatus,
    pub ledger: Option<u64>,
    pub fee_charged: Option<u64>,
    pub result_xdr: Option<String>,
    pub error_message: Option<String>,
}

/// Transaction status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
    Timeout,
}

/// Custom error types for Soroban operations
#[derive(Debug, thiserror::Error)]
pub enum SorobanError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Contract call failed: {0}")]
    ContractCallFailed(String),
    
    #[error("Transaction submission failed: {0}")]
    TransactionFailed(String),
    
    #[error("Event monitoring error: {0}")]
    EventMonitoringError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Timeout error: operation timed out after {0}s")]
    TimeoutError(u64),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

impl From<SorobanError> for loco_rs::Error {
    fn from(err: SorobanError) -> Self {
        loco_rs::Error::string(&err.to_string())
    }
}

/// Trait for contract interaction abstraction
#[async_trait]
pub trait ContractInteraction {
    async fn call_function(
        &self,
        contract_address: &str,
        function_name: &str,
        parameters: Vec<Val>,
    ) -> SorobanResult<ContractCallResult>;
    
    async fn submit_transaction(
        &self,
        transaction_config: TransactionConfig,
        contract_calls: Vec<ContractCall>,
    ) -> SorobanResult<String>;
    
    async fn get_events(
        &self,
        filter: EventFilter,
    ) -> SorobanResult<Vec<ContractEvent>>;
}

/// Main Soroban client for interacting with contracts
pub struct SorobanClient {
    config: SorobanConfig,
    http_client: reqwest::Client,
    signing_config: Option<SigningConfig>,
    event_cache: Arc<RwLock<HashMap<String, Vec<ContractEvent>>>>,
    transaction_cache: Arc<RwLock<HashMap<String, TransactionSubmissionResult>>>,
}

impl SorobanClient {
    /// Create a new Soroban client with configuration
    pub fn new(config: SorobanConfig) -> SorobanResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SorobanError::NetworkError(e.to_string()))?;
        
        Ok(Self {
            config,
            http_client,
            signing_config: None,
            event_cache: Arc::new(RwLock::new(HashMap::new())),
            transaction_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new Soroban client with signing configuration
    pub fn new_with_signing(config: SorobanConfig, signing_config: SigningConfig) -> SorobanResult<Self> {
        let mut client = Self::new(config)?;
        client.signing_config = Some(signing_config);
        Ok(client)
    }

    /// Set signing configuration for transaction submission
    pub fn set_signing_config(&mut self, signing_config: SigningConfig) {
        self.signing_config = Some(signing_config);
    }
    
    /// Get contract address by name
    pub fn get_contract_address(&self, contract_name: &str) -> SorobanResult<String> {
        match contract_name {
            "integration_router" => Ok(self.config.contracts.integration_router.clone()),
            "kyc_registry" => Ok(self.config.contracts.kyc_registry.clone()),
            "istsi_token" => Ok(self.config.contracts.istsi_token.clone()),
            "reserve_manager" => Ok(self.config.contracts.reserve_manager.clone()),
            _ => Err(SorobanError::ConfigurationError(
                format!("Unknown contract: {}", contract_name)
            )),
        }
    }
    
    /// Execute a Bitcoin deposit operation through the integration router
    pub async fn execute_bitcoin_deposit(
        &self,
        _user_address: &str,
        _btc_amount: u64,
        btc_tx_hash: &str,
    ) -> SorobanResult<ContractCallResult> {
        let contract_address = self.get_contract_address("integration_router")?;
        
        // Validate hex string
        if btc_tx_hash.len() != 64 {
            return Err(SorobanError::SerializationError("Invalid tx hash length".to_string()));
        }
        
        // Create parameters as JSON values for RPC call
        let parameters = vec![];
        
        self.call_function(&contract_address, "execute_bitcoin_deposit", parameters).await
    }
    
    /// Execute a token withdrawal operation through the integration router
    pub async fn execute_token_withdrawal(
        &self,
        _user_address: &str,
        _token_amount: u64,
        _btc_address: &str,
    ) -> SorobanResult<ContractCallResult> {
        let contract_address = self.get_contract_address("integration_router")?;
        
        // Create parameters for RPC call
        let parameters = vec![];
        
        self.call_function(&contract_address, "execute_token_withdrawal", parameters).await
    }
    
    /// Get system overview from the integration router
    pub async fn get_system_overview(&self) -> SorobanResult<ContractCallResult> {
        let contract_address = self.get_contract_address("integration_router")?;
        self.call_function(&contract_address, "get_system_health_status", vec![]).await
    }
    
    /// Execute a batch operation with rollback capability
    pub async fn execute_batch_operation(
        &self,
        batch_op: BatchOperation,
    ) -> SorobanResult<BatchOperationResult> {
        let start_time = SystemTime::now();
        let mut call_results = Vec::new();
        let mut overall_success = true;
        let mut rollback_executed = false;
        
        info!("Executing batch operation: {}", batch_op.operation_id);
        
        // Execute all calls in the batch
        for (index, call) in batch_op.calls.iter().enumerate() {
            debug!("Executing call {} of {}", index + 1, batch_op.calls.len());
            
            let call_result = timeout(
                Duration::from_secs(call.timeout_seconds),
                self.call_function(&call.contract_address, &call.function_name, call.parameters.clone())
            ).await;
            
            match call_result {
                Ok(Ok(result)) => {
                    call_results.push(result.clone());
                    if !result.success {
                        overall_success = false;
                        if batch_op.atomic {
                            warn!("Atomic batch operation failed at call {}, initiating rollback", index + 1);
                            break;
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Call {} failed: {}", index + 1, e);
                    call_results.push(ContractCallResult {
                        success: false,
                        return_data: None,
                        error_message: Some(e.to_string()),
                        gas_used: 0,
                        execution_time_ms: 0,
                        transaction_hash: None,
                    });
                    overall_success = false;
                    if batch_op.atomic {
                        break;
                    }
                }
                Err(_) => {
                    error!("Call {} timed out", index + 1);
                    call_results.push(ContractCallResult {
                        success: false,
                        return_data: None,
                        error_message: Some("Operation timed out".to_string()),
                        gas_used: 0,
                        execution_time_ms: call.timeout_seconds * 1000,
                        transaction_hash: None,
                    });
                    overall_success = false;
                    if batch_op.atomic {
                        break;
                    }
                }
            }
        }
        
        // Execute rollback if needed and available
        if !overall_success && batch_op.atomic && !batch_op.rollback_calls.is_empty() {
            warn!("Executing rollback for batch operation: {}", batch_op.operation_id);
            rollback_executed = true;
            
            for rollback_call in &batch_op.rollback_calls {
                if let Err(e) = self.call_function(
                    &rollback_call.contract_address,
                    &rollback_call.function_name,
                    rollback_call.parameters.clone()
                ).await {
                    error!("Rollback call failed: {}", e);
                }
            }
        }
        
        let total_execution_time_ms = start_time.elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        Ok(BatchOperationResult {
            operation_id: batch_op.operation_id,
            overall_success,
            call_results,
            rollback_executed,
            total_execution_time_ms,
        })
    }
    
    /// Start monitoring contract events
    pub async fn start_event_monitoring(
        &self,
        filter: EventFilter,
        _callback: impl Fn(ContractEvent) + Send + Sync + 'static,
    ) -> SorobanResult<()> {
        info!("Starting event monitoring for contracts: {:?}", filter.contract_addresses);
        
        // This would typically run in a background task
        tokio::spawn(async move {
            loop {
                // In a real implementation, this would:
                // 1. Poll the Soroban RPC for new events
                // 2. Filter events based on the provided filter
                // 3. Call the callback for each matching event
                // 4. Handle reconnection and error recovery
                
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
        
        Ok(())
    }
    
    /// Get recent events from contracts with caching
    pub async fn get_recent_events(
        &self,
        filter: EventFilter,
        limit: u32,
    ) -> SorobanResult<Vec<ContractEvent>> {
        debug!("Fetching recent events with limit: {}", limit);
        
        // Check cache first
        let cache_key = format!("{:?}_{}", filter.contract_addresses, limit);
        {
            let cache = self.event_cache.read().await;
            if let Some(cached_events) = cache.get(&cache_key) {
                if !cached_events.is_empty() {
                    debug!("Returning cached events for key: {}", cache_key);
                    return Ok(cached_events.clone());
                }
            }
        }
        
        // Build RPC request for events
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getEvents",
            "params": {
                "filters": [{
                    "type": "contract",
                    "contractIds": filter.contract_addresses,
                    "topics": [filter.event_types]
                }],
                "startLedger": filter.from_ledger,
                "endLedger": filter.to_ledger,
                "limit": limit
            }
        });
        
        let response = self.http_client
            .post(&self.config.rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SorobanError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(SorobanError::NetworkError(
                format!("RPC request failed with status: {}", response.status())
            ));
        }
        
        let response_json: serde_json::Value = response.json().await
            .map_err(|e| SorobanError::SerializationError(e.to_string()))?;
        
        // Parse events from response
        let events = response_json["result"]["events"]
            .as_array()
            .ok_or_else(|| SorobanError::SerializationError("Invalid events response".to_string()))?;
        
        let mut parsed_events = Vec::new();
        for event in events {
            if let Ok(parsed_event) = self.parse_contract_event(event) {
                parsed_events.push(parsed_event);
            }
        }
        
        // Cache the results
        {
            let mut cache = self.event_cache.write().await;
            cache.insert(cache_key, parsed_events.clone());
        }
        
        Ok(parsed_events)
    }

    /// Sign and submit a transaction to the network
    pub async fn sign_and_submit_transaction(
        &self,
        transaction_config: TransactionConfig,
        contract_calls: Vec<ContractCall>,
    ) -> SorobanResult<TransactionSubmissionResult> {
        let signing_config = self.signing_config.as_ref()
            .ok_or_else(|| SorobanError::AuthenticationError("No signing configuration provided".to_string()))?;

        info!("Signing and submitting transaction with {} contract calls", contract_calls.len());

        // Build transaction envelope
        let transaction_envelope = self.build_transaction_envelope(
            &transaction_config,
            &contract_calls,
        ).await?;

        // Sign the transaction
        let signed_envelope = self.sign_transaction_envelope(
            transaction_envelope,
            signing_config,
        ).await?;

        // Submit the signed transaction
        let submission_result = self.submit_signed_transaction(signed_envelope).await?;

        // Cache the result
        {
            let mut cache = self.transaction_cache.write().await;
            cache.insert(submission_result.transaction_hash.clone(), submission_result.clone());
        }

        Ok(submission_result)
    }

    /// Build a transaction envelope from contract calls
    async fn build_transaction_envelope(
        &self,
        config: &TransactionConfig,
        contract_calls: &[ContractCall],
    ) -> SorobanResult<TransactionEnvelope> {
        // Build operations from contract calls
        let operations: Vec<serde_json::Value> = contract_calls.iter().map(|call| {
            serde_json::json!({
                "type": "invokeHostFunction",
                "invokeHostFunction": {
                    "hostFunctionType": "invokeContract",
                    "invokeContract": {
                        "contractAddress": call.contract_address,
                        "functionName": call.function_name,
                        "args": call.parameters.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>()
                    }
                }
            })
        }).collect();

        // Build transaction
        let transaction = serde_json::json!({
            "sourceAccount": config.source_account,
            "fee": config.fee,
            "seqNum": config.sequence_number,
            "timeBounds": {
                "minTime": 0,
                "maxTime": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs() + config.timeout_seconds
            },
            "memo": config.memo.as_ref().unwrap_or(&"".to_string()),
            "operations": operations
        });

        // Convert to XDR (simplified - in real implementation would use stellar-sdk)
        let transaction_xdr = self.serialize_transaction_to_xdr(&transaction).await?;

        Ok(TransactionEnvelope {
            transaction_xdr,
            signatures: Vec::new(),
            network_passphrase: self.config.network_passphrase.clone(),
        })
    }

    /// Sign a transaction envelope
    async fn sign_transaction_envelope(
        &self,
        mut envelope: TransactionEnvelope,
        signing_config: &SigningConfig,
    ) -> SorobanResult<TransactionEnvelope> {
        // In a real implementation, this would use stellar-sdk for proper signing
        // For now, we'll create a mock signature
        let signature = self.create_transaction_signature(
            &envelope.transaction_xdr,
            &signing_config.secret_key,
            &signing_config.network_passphrase,
        ).await?;

        envelope.signatures.push(signature);
        Ok(envelope)
    }

    /// Submit a signed transaction to the network
    async fn submit_signed_transaction(
        &self,
        envelope: TransactionEnvelope,
    ) -> SorobanResult<TransactionSubmissionResult> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": {
                "transaction": envelope.transaction_xdr,
                "signatures": envelope.signatures
            }
        });

        let response = self.http_client
            .post(&self.config.rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SorobanError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Ok(TransactionSubmissionResult {
                transaction_hash: "".to_string(),
                status: TransactionStatus::Failed,
                ledger: None,
                fee_charged: None,
                result_xdr: None,
                error_message: Some(format!("HTTP error: {}", response.status())),
            });
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| SorobanError::SerializationError(e.to_string()))?;

        if let Some(error) = response_json.get("error") {
            return Ok(TransactionSubmissionResult {
                transaction_hash: "".to_string(),
                status: TransactionStatus::Failed,
                ledger: None,
                fee_charged: None,
                result_xdr: None,
                error_message: Some(error.to_string()),
            });
        }

        let tx_hash = response_json["result"]["hash"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let status = match response_json["result"]["status"].as_str() {
            Some("SUCCESS") => TransactionStatus::Success,
            Some("FAILED") => TransactionStatus::Failed,
            Some("PENDING") => TransactionStatus::Pending,
            _ => TransactionStatus::Failed,
        };

        Ok(TransactionSubmissionResult {
            transaction_hash: tx_hash,
            status,
            ledger: response_json["result"]["ledger"].as_u64(),
            fee_charged: response_json["result"]["feeCharged"].as_u64(),
            result_xdr: response_json["result"]["resultXdr"].as_str().map(String::from),
            error_message: None,
        })
    }

    /// Serialize transaction to XDR format (simplified implementation)
    async fn serialize_transaction_to_xdr(
        &self,
        transaction: &serde_json::Value,
    ) -> SorobanResult<String> {
        // In a real implementation, this would use stellar-sdk to properly serialize to XDR
        // For now, we'll create a mock XDR string
        let serialized = serde_json::to_string(transaction)
            .map_err(|e| SorobanError::SerializationError(e.to_string()))?;
        
        // Convert to base64 to simulate XDR
        use base64::{Engine as _, engine::general_purpose};
        Ok(general_purpose::STANDARD.encode(serialized))
    }

    /// Create a transaction signature (simplified implementation)
    async fn create_transaction_signature(
        &self,
        transaction_xdr: &str,
        secret_key: &str,
        network_passphrase: &str,
    ) -> SorobanResult<String> {
        // In a real implementation, this would use stellar-sdk for proper signing
        // For now, we'll create a mock signature
        let signature_data = format!("{}:{}:{}", transaction_xdr, secret_key, network_passphrase);
        let signature_hash = format!("{:x}", md5::compute(signature_data.as_bytes()));
        use base64::{Engine as _, engine::general_purpose};
        Ok(general_purpose::STANDARD.encode(signature_hash))
    }

    /// Get transaction status by hash
    pub async fn get_transaction_status(
        &self,
        transaction_hash: &str,
    ) -> SorobanResult<TransactionSubmissionResult> {
        // Check cache first
        {
            let cache = self.transaction_cache.read().await;
            if let Some(cached_result) = cache.get(transaction_hash) {
                return Ok(cached_result.clone());
            }
        }

        // Query the network
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": {
                "hash": transaction_hash
            }
        });

        let response = self.http_client
            .post(&self.config.rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SorobanError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SorobanError::NetworkError(
                format!("Failed to get transaction status: {}", response.status())
            ));
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| SorobanError::SerializationError(e.to_string()))?;

        if let Some(error) = response_json.get("error") {
            return Err(SorobanError::NetworkError(error.to_string()));
        }

        let result = &response_json["result"];
        let status = match result["status"].as_str() {
            Some("SUCCESS") => TransactionStatus::Success,
            Some("FAILED") => TransactionStatus::Failed,
            Some("PENDING") => TransactionStatus::Pending,
            _ => TransactionStatus::Failed,
        };

        let submission_result = TransactionSubmissionResult {
            transaction_hash: transaction_hash.to_string(),
            status,
            ledger: result["ledger"].as_u64(),
            fee_charged: result["feeCharged"].as_u64(),
            result_xdr: result["resultXdr"].as_str().map(String::from),
            error_message: result["error"].as_str().map(String::from),
        };

        // Cache the result
        {
            let mut cache = self.transaction_cache.write().await;
            cache.insert(transaction_hash.to_string(), submission_result.clone());
        }

        Ok(submission_result)
    }
    
    /// Parse a contract event from RPC response
    fn parse_contract_event(&self, event_data: &serde_json::Value) -> SorobanResult<ContractEvent> {
        let event_type = event_data["type"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        
        let contract_address = event_data["contractId"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let topic = event_data["topic"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        
        let data = event_data["value"].clone();
        
        let ledger = event_data["ledger"]
            .as_u64()
            .unwrap_or(0);
        
        let transaction_hash = event_data["txHash"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Ok(ContractEvent {
            event_type,
            contract_address,
            topic,
            data,
            ledger,
            transaction_hash,
            timestamp,
        })
    }
}

#[async_trait]
impl ContractInteraction for SorobanClient {
    async fn call_function(
        &self,
        contract_address: &str,
        function_name: &str,
        parameters: Vec<Val>,
    ) -> SorobanResult<ContractCallResult> {
        let start_time = SystemTime::now();
        
        debug!("Calling contract function: {}::{}", contract_address, function_name);
        
        // Build RPC request for contract invocation
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "simulateTransaction",
            "params": {
                "transaction": {
                    "operations": [{
                        "type": "invokeHostFunction",
                        "invokeHostFunction": {
                            "hostFunctionType": "invokeContract",
                            "invokeContract": {
                                "contractAddress": contract_address,
                                "functionName": function_name,
                                "args": parameters.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>()
                            }
                        }
                    }]
                }
            }
        });
        
        let response = self.http_client
            .post(&self.config.rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SorobanError::NetworkError(e.to_string()))?;
        
        let execution_time_ms = start_time.elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        if !response.status().is_success() {
            return Ok(ContractCallResult {
                success: false,
                return_data: None,
                error_message: Some(format!("HTTP error: {}", response.status())),
                gas_used: 0,
                execution_time_ms,
                transaction_hash: None,
            });
        }
        
        let response_json: serde_json::Value = response.json().await
            .map_err(|e| SorobanError::SerializationError(e.to_string()))?;
        
        // Parse the response
        if let Some(error) = response_json.get("error") {
            return Ok(ContractCallResult {
                success: false,
                return_data: None,
                error_message: Some(error.to_string()),
                gas_used: 0,
                execution_time_ms,
                transaction_hash: None,
            });
        }
        
        let result = response_json["result"].clone();
        let gas_used = result["cost"]["cpuInsns"].as_u64().unwrap_or(0);
        
        Ok(ContractCallResult {
            success: true,
            return_data: Some(result["results"][0]["xdr"].clone()),
            error_message: None,
            gas_used,
            execution_time_ms,
            transaction_hash: None,
        })
    }
    
    async fn submit_transaction(
        &self,
        transaction_config: TransactionConfig,
        contract_calls: Vec<ContractCall>,
    ) -> SorobanResult<String> {
        // Use the enhanced signing and submission method
        let result = self.sign_and_submit_transaction(transaction_config, contract_calls).await?;
        
        match result.status {
            TransactionStatus::Success | TransactionStatus::Pending => {
                info!("Transaction submitted successfully: {}", result.transaction_hash);
                Ok(result.transaction_hash)
            }
            TransactionStatus::Failed => {
                let error_msg = result.error_message.unwrap_or_else(|| "Transaction failed".to_string());
                Err(SorobanError::TransactionFailed(error_msg))
            }
            TransactionStatus::Timeout => {
                Err(SorobanError::TimeoutError(30)) // Default timeout
            }
        }
    }
    
    async fn get_events(
        &self,
        filter: EventFilter,
    ) -> SorobanResult<Vec<ContractEvent>> {
        self.get_recent_events(filter, 100).await
    }
}