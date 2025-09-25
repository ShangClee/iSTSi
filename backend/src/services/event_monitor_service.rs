use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::soroban_client::{
    SorobanClient, ContractEvent, EventFilter, SorobanResult
};

/// Event monitoring service for real-time contract event processing
pub struct EventMonitorService {
    soroban_client: Arc<SorobanClient>,
    subscriptions: Arc<RwLock<HashMap<String, EventSubscription>>>,
    event_handlers: Arc<RwLock<HashMap<String, Box<dyn EventHandler + Send + Sync>>>>,
    monitoring_active: Arc<RwLock<bool>>,
    last_processed_ledger: Arc<RwLock<u64>>,
    event_statistics: Arc<RwLock<EventStatistics>>,
}

/// Event subscription configuration
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub id: String,
    pub filter: EventFilter,
    pub active: bool,
    pub created_at: u64,
    pub last_processed: u64,
    pub retry_count: u32,
    pub max_retries: u32,
}

/// Event handler trait for processing contract events
pub trait EventHandler {
    fn handle_event(&self, event: &ContractEvent) -> Result<()>;
    fn get_handler_type(&self) -> String;
}

/// Event processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStatistics {
    pub total_events_processed: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_contract: HashMap<String, u64>,
    pub processing_errors: u64,
    pub last_processed_ledger: u64,
    pub monitoring_uptime_seconds: u64,
    pub average_processing_time_ms: f64,
    pub last_updated: u64,
}

impl Default for EventStatistics {
    fn default() -> Self {
        Self {
            total_events_processed: 0,
            events_by_type: HashMap::new(),
            events_by_contract: HashMap::new(),
            processing_errors: 0,
            last_processed_ledger: 0,
            monitoring_uptime_seconds: 0,
            average_processing_time_ms: 0.0,
            last_updated: 0,
        }
    }
}

/// Bitcoin deposit event handler
pub struct BitcoinDepositHandler;

impl EventHandler for BitcoinDepositHandler {
    fn handle_event(&self, event: &ContractEvent) -> Result<()> {
        info!("Processing Bitcoin deposit event: {:?}", event);
        
        // Extract deposit information from event
        // In a real implementation, this would:
        // 1. Update database records
        // 2. Send notifications
        // 3. Trigger downstream processes
        
        debug!("Bitcoin deposit processed successfully");
        Ok(())
    }

    fn get_handler_type(&self) -> String {
        "bitcoin_deposit".to_string()
    }
}

/// Token withdrawal event handler
pub struct TokenWithdrawalHandler;

impl EventHandler for TokenWithdrawalHandler {
    fn handle_event(&self, event: &ContractEvent) -> Result<()> {
        info!("Processing token withdrawal event: {:?}", event);
        
        // Extract withdrawal information from event
        // In a real implementation, this would:
        // 1. Initiate Bitcoin withdrawal process
        // 2. Update withdrawal status
        // 3. Send user notifications
        
        debug!("Token withdrawal processed successfully");
        Ok(())
    }

    fn get_handler_type(&self) -> String {
        "token_withdrawal".to_string()
    }
}

/// Compliance violation event handler
pub struct ComplianceViolationHandler;

impl EventHandler for ComplianceViolationHandler {
    fn handle_event(&self, event: &ContractEvent) -> Result<()> {
        warn!("Processing compliance violation event: {:?}", event);
        
        // Handle compliance violation
        // In a real implementation, this would:
        // 1. Log security incident
        // 2. Freeze affected accounts
        // 3. Send alerts to compliance team
        
        error!("Compliance violation detected and handled");
        Ok(())
    }

    fn get_handler_type(&self) -> String {
        "compliance_violation".to_string()
    }
}

/// System alert event handler
pub struct SystemAlertHandler;

impl EventHandler for SystemAlertHandler {
    fn handle_event(&self, event: &ContractEvent) -> Result<()> {
        error!("Processing system alert event: {:?}", event);
        
        // Handle system alert
        // In a real implementation, this would:
        // 1. Send alerts to operations team
        // 2. Trigger automated responses
        // 3. Update system status
        
        warn!("System alert processed");
        Ok(())
    }

    fn get_handler_type(&self) -> String {
        "system_alert".to_string()
    }
}

impl EventMonitorService {
    /// Create a new event monitoring service
    pub fn new(soroban_client: Arc<SorobanClient>) -> Self {
        Self {
            soroban_client,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            monitoring_active: Arc::new(RwLock::new(false)),
            last_processed_ledger: Arc::new(RwLock::new(0)),
            event_statistics: Arc::new(RwLock::new(EventStatistics::default())),
        }
    }

    /// Initialize the event monitoring service with default handlers
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing event monitoring service");

        // Register default event handlers
        self.register_handler("bitcoin_deposit", Box::new(BitcoinDepositHandler)).await?;
        self.register_handler("token_withdrawal", Box::new(TokenWithdrawalHandler)).await?;
        self.register_handler("compliance_violation", Box::new(ComplianceViolationHandler)).await?;
        self.register_handler("system_alert", Box::new(SystemAlertHandler)).await?;

        // Create default subscriptions for all contract events
        self.create_default_subscriptions().await?;

        info!("Event monitoring service initialized successfully");
        Ok(())
    }

    /// Start monitoring contract events
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting contract event monitoring");

        {
            let mut active = self.monitoring_active.write().await;
            *active = true;
        }

        // Start the monitoring loop
        let service = Arc::new(self.clone());
        tokio::spawn(async move {
            service.monitoring_loop().await;
        });

        Ok(())
    }

    /// Stop monitoring contract events
    pub async fn stop_monitoring(&self) -> Result<()> {
        info!("Stopping contract event monitoring");

        {
            let mut active = self.monitoring_active.write().await;
            *active = false;
        }

        Ok(())
    }

    /// Register an event handler
    pub async fn register_handler(
        &self,
        handler_type: &str,
        handler: Box<dyn EventHandler + Send + Sync>,
    ) -> Result<()> {
        let mut handlers = self.event_handlers.write().await;
        handlers.insert(handler_type.to_string(), handler);
        
        debug!("Registered event handler: {}", handler_type);
        Ok(())
    }

    /// Subscribe to specific contract events
    pub async fn subscribe_to_events(
        &self,
        filter: EventFilter,
        max_retries: Option<u32>,
    ) -> Result<String> {
        let subscription_id = Uuid::new_v4().to_string();
        
        let subscription = EventSubscription {
            id: subscription_id.clone(),
            filter,
            active: true,
            created_at: chrono::Utc::now().timestamp() as u64,
            last_processed: 0,
            retry_count: 0,
            max_retries: max_retries.unwrap_or(3),
        };

        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }

        info!("Created event subscription: {}", subscription_id);
        Ok(subscription_id)
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(subscription_id);
        }

        info!("Removed event subscription: {}", subscription_id);
        Ok(())
    }

    /// Get event processing statistics
    pub async fn get_statistics(&self) -> EventStatistics {
        let stats = self.event_statistics.read().await;
        stats.clone()
    }

    /// Main monitoring loop
    async fn monitoring_loop(&self) {
        let mut interval = interval(Duration::from_secs(5)); // Poll every 5 seconds
        
        while *self.monitoring_active.read().await {
            interval.tick().await;
            
            if let Err(e) = self.process_new_events().await {
                error!("Error processing events: {}", e);
                
                // Update error statistics
                {
                    let mut stats = self.event_statistics.write().await;
                    stats.processing_errors += 1;
                    stats.last_updated = chrono::Utc::now().timestamp() as u64;
                }
            }
        }
        
        info!("Event monitoring loop stopped");
    }

    /// Process new events from all subscriptions
    async fn process_new_events(&self) -> SorobanResult<()> {
        let subscriptions = {
            let subs = self.subscriptions.read().await;
            subs.clone()
        };

        for (subscription_id, subscription) in subscriptions {
            if !subscription.active {
                continue;
            }

            match self.process_subscription_events(&subscription).await {
                Ok(processed_count) => {
                    if processed_count > 0 {
                        debug!("Processed {} events for subscription {}", processed_count, subscription_id);
                    }
                }
                Err(e) => {
                    warn!("Error processing subscription {}: {}", subscription_id, e);
                    
                    // Update retry count
                    {
                        let mut subs = self.subscriptions.write().await;
                        if let Some(sub) = subs.get_mut(&subscription_id) {
                            sub.retry_count += 1;
                            if sub.retry_count >= sub.max_retries {
                                warn!("Disabling subscription {} after {} retries", subscription_id, sub.retry_count);
                                sub.active = false;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Process events for a specific subscription
    async fn process_subscription_events(&self, subscription: &EventSubscription) -> SorobanResult<u32> {
        let events = self.soroban_client.get_recent_events(subscription.filter.clone(), 100).await?;
        
        let mut processed_count = 0;
        let start_time = std::time::Instant::now();

        for event in events {
            // Skip events we've already processed
            if event.ledger <= subscription.last_processed {
                continue;
            }

            // Process the event
            if let Err(e) = self.handle_contract_event(&event).await {
                error!("Error handling event: {}", e);
                continue;
            }

            processed_count += 1;

            // Update statistics
            {
                let mut stats = self.event_statistics.write().await;
                stats.total_events_processed += 1;
                *stats.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
                *stats.events_by_contract.entry(event.contract_address.to_string()).or_insert(0) += 1;
                
                if event.ledger > stats.last_processed_ledger {
                    stats.last_processed_ledger = event.ledger;
                }
                
                stats.last_updated = chrono::Utc::now().timestamp() as u64;
            }

            // Update subscription last processed
            {
                let mut subs = self.subscriptions.write().await;
                if let Some(sub) = subs.get_mut(&subscription.id) {
                    sub.last_processed = event.ledger;
                }
            }
        }

        // Update average processing time
        if processed_count > 0 {
            let processing_time_ms = start_time.elapsed().as_millis() as f64;
            let mut stats = self.event_statistics.write().await;
            stats.average_processing_time_ms = 
                (stats.average_processing_time_ms + processing_time_ms) / 2.0;
        }

        Ok(processed_count)
    }

    /// Handle a specific contract event
    async fn handle_contract_event(&self, event: &ContractEvent) -> Result<()> {
        debug!("Handling contract event: {} from {}", event.event_type, event.contract_address);

        // Determine handler type based on event type
        let handler_type = match event.event_type.as_str() {
            "btc_dep" | "bitcoin_deposit" => "bitcoin_deposit",
            "tok_with" | "token_withdrawal" => "token_withdrawal",
            "comp_viol" | "compliance_violation" => "compliance_violation",
            "sys_alert" | "system_alert" => "system_alert",
            _ => {
                debug!("No specific handler for event type: {}", event.event_type);
                return Ok(());
            }
        };

        // Get and execute the handler
        {
            let handlers = self.event_handlers.read().await;
            if let Some(handler) = handlers.get(handler_type) {
                handler.handle_event(event)?;
            } else {
                warn!("No handler registered for type: {}", handler_type);
            }
        }

        Ok(())
    }

    /// Create default event subscriptions
    async fn create_default_subscriptions(&self) -> Result<()> {
        // Subscribe to Bitcoin deposit events
        let deposit_filter = EventFilter {
            contract_addresses: vec![],
            event_types: vec!["btc_dep".to_string(), "bitcoin_deposit".to_string()],
            from_ledger: None,
            to_ledger: None,
        };
        self.subscribe_to_events(deposit_filter, Some(5)).await?;

        // Subscribe to token withdrawal events
        let withdrawal_filter = EventFilter {
            contract_addresses: vec![],
            event_types: vec!["tok_with".to_string(), "token_withdrawal".to_string()],
            from_ledger: None,
            to_ledger: None,
        };
        self.subscribe_to_events(withdrawal_filter, Some(5)).await?;

        // Subscribe to compliance events
        let compliance_filter = EventFilter {
            contract_addresses: vec![],
            event_types: vec!["comp_viol".to_string(), "compliance_violation".to_string()],
            from_ledger: None,
            to_ledger: None,
        };
        self.subscribe_to_events(compliance_filter, Some(10)).await?;

        // Subscribe to system alerts
        let alert_filter = EventFilter {
            contract_addresses: vec![],
            event_types: vec!["sys_alert".to_string(), "system_alert".to_string()],
            from_ledger: None,
            to_ledger: None,
        };
        self.subscribe_to_events(alert_filter, Some(10)).await?;

        info!("Created default event subscriptions");
        Ok(())
    }
}

impl Clone for EventMonitorService {
    fn clone(&self) -> Self {
        Self {
            soroban_client: Arc::clone(&self.soroban_client),
            subscriptions: Arc::clone(&self.subscriptions),
            event_handlers: Arc::clone(&self.event_handlers),
            monitoring_active: Arc::clone(&self.monitoring_active),
            last_processed_ledger: Arc::clone(&self.last_processed_ledger),
            event_statistics: Arc::clone(&self.event_statistics),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::soroban_client::{SorobanConfig, ContractAddresses};

    #[tokio::test]
    async fn test_event_monitor_initialization() {
        let config = SorobanConfig {
            network: "testnet".to_string(),
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            contracts: ContractAddresses {
                integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string(),
                istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC".to_string(),
                reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".to_string(),
            },
        };

        let soroban_client = Arc::new(SorobanClient::new(config).expect("Failed to create client"));
        let monitor = EventMonitorService::new(soroban_client);

        assert!(monitor.initialize().await.is_ok());
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.total_events_processed, 0);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let config = SorobanConfig {
            network: "testnet".to_string(),
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            contracts: ContractAddresses {
                integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string(),
                istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC".to_string(),
                reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".to_string(),
            },
        };

        let soroban_client = Arc::new(SorobanClient::new(config).expect("Failed to create client"));
        let monitor = EventMonitorService::new(soroban_client);

        let filter = EventFilter {
            contract_addresses: vec![],
            event_types: vec!["test_event".to_string()],
            from_ledger: None,
            to_ledger: None,
        };

        let subscription_id = monitor.subscribe_to_events(filter, Some(3)).await.expect("Failed to subscribe");
        assert!(!subscription_id.is_empty());

        assert!(monitor.unsubscribe(&subscription_id).await.is_ok());
    }
}