use loco_rs::prelude::*;
use loco_rs::worker::{AppContext, Queue, Worker};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use async_trait::async_trait;

use crate::services::{
    integration_service::{IntegrationService, OperationStatus},
    soroban_client::{ContractEvent, EventFilter, SorobanError},
    config_service::ConfigService,
};

/// Event monitoring worker for processing Soroban contract events
pub struct EventMonitorWorker {
    integration_service: IntegrationService,
}

/// Event processing job data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventProcessingJob {
    pub event_type: String,
    pub contract_address: String,
    pub event_data: serde_json::Value,
    pub transaction_hash: String,
    pub ledger: u64,
    pub timestamp: u64,
}

/// Event monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMonitorConfig {
    pub polling_interval_seconds: u64,
    pub batch_size: u32,
    pub retry_attempts: u32,
    pub enabled_events: Vec<String>,
}

impl Default for EventMonitorConfig {
    fn default() -> Self {
        Self {
            polling_interval_seconds: 10,
            batch_size: 50,
            retry_attempts: 3,
            enabled_events: vec![
                "BitcoinDepositCompleted".to_string(),
                "TokenWithdrawalCompleted".to_string(),
                "CrossTokenExchangeCompleted".to_string(),
                "ComplianceViolation".to_string(),
                "SystemAlert".to_string(),
                "ReserveRatioAlert".to_string(),
                "KYCStatusChanged".to_string(),
            ],
        }
    }
}

impl EventMonitorWorker {
    /// Create a new event monitor worker
    pub fn new() -> Result<Self> {
        let soroban_config = ConfigService::load_soroban_config()?;
        let integration_service = IntegrationService::new(soroban_config)
            .map_err(|e| Error::string(format!("Failed to create integration service: {}", e)))?;
        
        Ok(Self {
            integration_service,
        })
    }
    
    /// Start monitoring contract events
    pub async fn start_monitoring(&self, config: EventMonitorConfig) -> Result<()> {
        info!("Starting contract event monitoring with config: {:?}", config);
        
        let filter = EventFilter {
            contract_addresses: vec![
                self.integration_service.soroban_client.get_contract_address("integration_router")
                    .map_err(|e| Error::string(e.to_string()))?,
                self.integration_service.soroban_client.get_contract_address("kyc_registry")
                    .map_err(|e| Error::string(e.to_string()))?,
                self.integration_service.soroban_client.get_contract_address("istsi_token")
                    .map_err(|e| Error::string(e.to_string()))?,
                self.integration_service.soroban_client.get_contract_address("reserve_manager")
                    .map_err(|e| Error::string(e.to_string()))?,
            ],
            event_types: config.enabled_events.clone(),
            from_ledger: None,
            to_ledger: None,
        };
        
        // Start the monitoring loop
        tokio::spawn(async move {
            let mut last_processed_ledger = 0u64;
            
            loop {
                match self.poll_events(&filter, &config, last_processed_ledger).await {
                    Ok(new_last_ledger) => {
                        if new_last_ledger > last_processed_ledger {
                            last_processed_ledger = new_last_ledger;
                            debug!("Updated last processed ledger to: {}", last_processed_ledger);
                        }
                    }
                    Err(e) => {
                        error!("Error polling events: {}", e);
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(config.polling_interval_seconds)).await;
            }
        });
        
        Ok(())
    }
    
    /// Poll for new events from contracts
    async fn poll_events(
        &self,
        filter: &EventFilter,
        config: &EventMonitorConfig,
        from_ledger: u64,
    ) -> Result<u64> {
        let mut filter_with_range = filter.clone();
        filter_with_range.from_ledger = Some(from_ledger + 1);
        
        let events = self.integration_service.get_recent_events(None, None, config.batch_size).await
            .map_err(|e| Error::string(e.to_string()))?;
        
        let mut max_ledger = from_ledger;
        
        for event in events {
            if event.ledger > max_ledger {
                max_ledger = event.ledger;
            }
            
            // Process each event
            if let Err(e) = self.process_event(event).await {
                error!("Failed to process event: {}", e);
            }
        }
        
        Ok(max_ledger)
    }
    
    /// Process a single contract event
    async fn process_event(&self, event: ContractEvent) -> Result<()> {
        info!("Processing event: {} from contract {}", event.event_type, event.contract_address);
        
        match event.event_type.as_str() {
            "BitcoinDepositCompleted" => {
                self.handle_bitcoin_deposit_completed(event).await?;
            }
            "TokenWithdrawalCompleted" => {
                self.handle_token_withdrawal_completed(event).await?;
            }
            "CrossTokenExchangeCompleted" => {
                self.handle_cross_token_exchange_completed(event).await?;
            }
            "ComplianceViolation" => {
                self.handle_compliance_violation(event).await?;
            }
            "SystemAlert" => {
                self.handle_system_alert(event).await?;
            }
            "ReserveRatioAlert" => {
                self.handle_reserve_ratio_alert(event).await?;
            }
            "KYCStatusChanged" => {
                self.handle_kyc_status_changed(event).await?;
            }
            _ => {
                debug!("Unhandled event type: {}", event.event_type);
            }
        }
        
        Ok(())
    }
    
    /// Handle Bitcoin deposit completion event
    async fn handle_bitcoin_deposit_completed(&self, event: ContractEvent) -> Result<()> {
        debug!("Handling Bitcoin deposit completion: {:?}", event.data);
        
        // Extract event data
        let user_address = event.data.get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let btc_amount = event.data.get("btc_amount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let istsi_amount = event.data.get("istsi_amount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        info!("Bitcoin deposit completed: {} BTC -> {} iSTSi for user {}", 
              btc_amount, istsi_amount, user_address);
        
        // TODO: Update database records, send notifications, etc.
        
        Ok(())
    }
    
    /// Handle token withdrawal completion event
    async fn handle_token_withdrawal_completed(&self, event: ContractEvent) -> Result<()> {
        debug!("Handling token withdrawal completion: {:?}", event.data);
        
        let user_address = event.data.get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let token_amount = event.data.get("token_amount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let btc_tx_hash = event.data.get("btc_tx_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        info!("Token withdrawal completed: {} tokens -> BTC tx {} for user {}", 
              token_amount, btc_tx_hash, user_address);
        
        // TODO: Update database records, send notifications, etc.
        
        Ok(())
    }
    
    /// Handle cross-token exchange completion event
    async fn handle_cross_token_exchange_completed(&self, event: ContractEvent) -> Result<()> {
        debug!("Handling cross-token exchange completion: {:?}", event.data);
        
        let user_address = event.data.get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let from_amount = event.data.get("from_amount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let to_amount = event.data.get("to_amount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        info!("Cross-token exchange completed: {} -> {} for user {}", 
              from_amount, to_amount, user_address);
        
        // TODO: Update database records, send notifications, etc.
        
        Ok(())
    }
    
    /// Handle compliance violation event
    async fn handle_compliance_violation(&self, event: ContractEvent) -> Result<()> {
        warn!("Handling compliance violation: {:?}", event.data);
        
        let user_address = event.data.get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let violation_type = event.data.get("violation_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let severity = event.data.get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");
        
        warn!("Compliance violation detected: {} violation for user {} (severity: {})", 
              violation_type, user_address, severity);
        
        // TODO: Alert compliance team, freeze account if necessary, etc.
        
        Ok(())
    }
    
    /// Handle system alert event
    async fn handle_system_alert(&self, event: ContractEvent) -> Result<()> {
        error!("Handling system alert: {:?}", event.data);
        
        let alert_type = event.data.get("alert_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let severity = event.data.get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");
        
        let message = event.data.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message");
        
        error!("System alert: {} (severity: {}) - {}", alert_type, severity, message);
        
        // TODO: Alert operations team, trigger automated responses, etc.
        
        Ok(())
    }
    
    /// Handle reserve ratio alert event
    async fn handle_reserve_ratio_alert(&self, event: ContractEvent) -> Result<()> {
        warn!("Handling reserve ratio alert: {:?}", event.data);
        
        let current_ratio = event.data.get("current_ratio")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let threshold = event.data.get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        warn!("Reserve ratio alert: current ratio {} below threshold {}", 
              current_ratio, threshold);
        
        // TODO: Alert treasury team, trigger reserve management actions, etc.
        
        Ok(())
    }
    
    /// Handle KYC status change event
    async fn handle_kyc_status_changed(&self, event: ContractEvent) -> Result<()> {
        info!("Handling KYC status change: {:?}", event.data);
        
        let user_address = event.data.get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let old_tier = event.data.get("old_tier")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let new_tier = event.data.get("new_tier")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        info!("KYC status changed for user {}: tier {} -> {}", 
              user_address, old_tier, new_tier);
        
        // TODO: Update user permissions, send notifications, etc.
        
        Ok(())
    }
}

#[async_trait]
impl Worker<EventProcessingJob> for EventMonitorWorker {
    async fn perform(&self, _ctx: &AppContext, job: &EventProcessingJob) -> Result<()> {
        info!("Processing event job: {} from {}", job.event_type, job.contract_address);
        
        let event = ContractEvent {
            event_type: job.event_type.clone(),
            contract_address: job.contract_address.clone(),
            topic: vec![], // Would be populated from job data
            data: job.event_data.clone(),
            ledger: job.ledger,
            transaction_hash: job.transaction_hash.clone(),
            timestamp: job.timestamp,
        };
        
        self.process_event(event).await?;
        
        Ok(())
    }
}

impl EventMonitorWorker {
    /// Build the worker for registration with the processor
    pub fn build(_ctx: &AppContext) -> Box<dyn Worker<EventProcessingJob> + Send + Sync> {
        Box::new(Self::new().expect("Failed to create EventMonitorWorker"))
    }
}