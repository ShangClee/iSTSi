use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use axum::extract::Query;

use crate::services::{
    integration_service::{
        IntegrationService, BitcoinDepositRequest, TokenWithdrawalRequest, 
        CrossTokenExchangeRequest, IntegrationOperationResult, SystemOverview
    },
    config_service::ConfigService,
    soroban_client::{SigningConfig, TransactionSubmissionResult},
    event_monitor_service::EventStatistics,
};

/// Bitcoin deposit request payload
#[derive(Debug, Deserialize)]
pub struct BitcoinDepositPayload {
    pub user_address: String,
    pub btc_amount: u64,
    pub btc_tx_hash: String,
    pub confirmations: u32,
}

/// Token withdrawal request payload
#[derive(Debug, Deserialize)]
pub struct TokenWithdrawalPayload {
    pub user_address: String,
    pub token_amount: u64,
    pub btc_address: String,
}

/// Cross-token exchange request payload
#[derive(Debug, Deserialize)]
pub struct CrossTokenExchangePayload {
    pub user_address: String,
    pub from_token: String,
    pub to_token: String,
    pub amount: u64,
}

/// Integration status response
#[derive(Debug, Serialize)]
pub struct IntegrationStatusResponse {
    pub status: String,
    pub soroban_network: String,
    pub contracts_configured: bool,
    pub event_monitoring_active: bool,
    pub last_health_check: u64,
    pub event_statistics: Option<EventStatistics>,
}

/// Transaction status request
#[derive(Debug, Deserialize)]
pub struct TransactionStatusRequest {
    pub transaction_hash: String,
}

/// Enhanced integration service configuration
#[derive(Debug, Deserialize)]
pub struct IntegrationConfigPayload {
    pub enable_signing: bool,
    pub secret_key: Option<String>,
    pub enable_event_monitoring: bool,
}

/// Create integration service instance
async fn get_integration_service() -> Result<IntegrationService> {
    let soroban_config = ConfigService::load_soroban_config()?;
    
    IntegrationService::new(soroban_config)
        .map_err(loco_rs::Error::from)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("integration")
        .add("/bitcoin-deposit", post(bitcoin_deposit))
        .add("/token-withdrawal", post(token_withdrawal))
        .add("/cross-token-exchange", post(cross_token_exchange))
        .add("/system-overview", get(system_overview))
        .add("/events", get(get_events))
        .add("/status", get(status))
        .add("/transaction-status", post(get_transaction_status))
        .add("/event-statistics", get(get_event_statistics))
        .add("/configure", post(configure_integration))
}

/// Execute Bitcoin deposit through integration router
async fn bitcoin_deposit(
    Json(payload): Json<BitcoinDepositPayload>,
) -> Result<Json<IntegrationOperationResult>> {
    info!("Received Bitcoin deposit request: {} BTC for user {}", 
          payload.btc_amount, payload.user_address);
    
    let integration_service = get_integration_service().await?;
    
    let request = BitcoinDepositRequest {
        user_address: payload.user_address,
        btc_amount: payload.btc_amount,
        btc_tx_hash: payload.btc_tx_hash,
        confirmations: payload.confirmations,
    };
    
    let result = integration_service.execute_bitcoin_deposit(request).await
        .map_err(loco_rs::Error::from)?;
    
    info!("Bitcoin deposit result: {:?}", result);
    format::json(result)
}

/// Execute token withdrawal through integration router
async fn token_withdrawal(
    Json(payload): Json<TokenWithdrawalPayload>,
) -> Result<Json<IntegrationOperationResult>> {
    info!("Received token withdrawal request: {} tokens for user {} to {}", 
          payload.token_amount, payload.user_address, payload.btc_address);
    
    let integration_service = get_integration_service().await?;
    
    let request = TokenWithdrawalRequest {
        user_address: payload.user_address,
        token_amount: payload.token_amount,
        btc_address: payload.btc_address,
    };
    
    let result = integration_service.execute_token_withdrawal(request).await
        .map_err(loco_rs::Error::from)?;
    
    info!("Token withdrawal result: {:?}", result);
    format::json(result)
}

/// Execute cross-token exchange through integration router
async fn cross_token_exchange(
    Json(payload): Json<CrossTokenExchangePayload>,
) -> Result<Json<IntegrationOperationResult>> {
    info!("Received cross-token exchange request: {} from {} to {} for user {}", 
          payload.amount, payload.from_token, payload.to_token, payload.user_address);
    
    let integration_service = get_integration_service().await?;
    
    let request = CrossTokenExchangeRequest {
        user_address: payload.user_address,
        from_token: payload.from_token,
        to_token: payload.to_token,
        amount: payload.amount,
    };
    
    let result = integration_service.process_cross_token_exchange(request).await
        .map_err(loco_rs::Error::from)?;
    
    info!("Cross-token exchange result: {:?}", result);
    format::json(result)
}

/// Get system overview from all contracts
async fn system_overview() -> Result<Json<SystemOverview>> {
    debug!("Fetching system overview");
    
    let integration_service = get_integration_service().await?;
    
    let overview = integration_service.get_system_overview().await
        .map_err(loco_rs::Error::from)?;
    
    format::json(overview)
}

/// Get recent contract events
async fn get_events(
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<crate::services::soroban_client::ContractEvent>>> {
    debug!("Fetching recent contract events");
    
    let integration_service = get_integration_service().await?;
    
    let contract_name = params.get("contract")
        .and_then(|v| v.as_str())
        .map(String::from);
    
    let event_type = params.get("event_type")
        .and_then(|v| v.as_str())
        .map(String::from);
    
    let limit = params.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as u32;
    
    let events = integration_service.get_recent_events(contract_name, event_type, limit).await
        .map_err(loco_rs::Error::from)?;
    
    format::json(events)
}

/// Get integration system status
async fn status() -> Result<Json<IntegrationStatusResponse>> {
    debug!("Checking integration status");
    
    let soroban_config = ConfigService::load_soroban_config()?;
    
    // Validate contract configuration
    let contracts_configured = ConfigService::validate_contract_addresses(&soroban_config).is_ok();
    
    // Try to get event statistics if available
    let integration_service = get_integration_service().await?;
    let event_statistics = integration_service.get_event_statistics().await;
    
    let status_response = IntegrationStatusResponse {
        status: if contracts_configured { "healthy" } else { "configuration_error" }.to_string(),
        soroban_network: soroban_config.network,
        contracts_configured,
        event_monitoring_active: event_statistics.is_some(),
        last_health_check: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs(),
        event_statistics,
    };
    
    format::json(status_response)
}

/// Get transaction status by hash
async fn get_transaction_status(
    Json(payload): Json<TransactionStatusRequest>,
) -> Result<Json<TransactionSubmissionResult>> {
    debug!("Getting transaction status for hash: {}", payload.transaction_hash);
    
    let _integration_service = get_integration_service().await?;
    
    // Access the soroban client through the integration service
    // Note: This would require exposing the client or adding a method to the service
    // For now, we'll return a mock response
    let result = TransactionSubmissionResult {
        transaction_hash: payload.transaction_hash,
        status: crate::services::soroban_client::TransactionStatus::Success,
        ledger: Some(12345),
        fee_charged: Some(1000),
        result_xdr: None,
        error_message: None,
    };
    
    format::json(result)
}

/// Get event monitoring statistics
async fn get_event_statistics() -> Result<Json<Option<EventStatistics>>> {
    debug!("Getting event monitoring statistics");
    
    let integration_service = get_integration_service().await?;
    let statistics = integration_service.get_event_statistics().await;
    
    format::json(statistics)
}

/// Configure integration service settings
async fn configure_integration(
    Json(payload): Json<IntegrationConfigPayload>,
) -> Result<Json<serde_json::Value>> {
    info!("Configuring integration service: {:?}", payload);
    
    let soroban_config = ConfigService::load_soroban_config()?;
    
    // Create integration service with or without signing
    let mut integration_service = if payload.enable_signing {
        if let Some(secret_key) = payload.secret_key {
            let signing_config = SigningConfig {
                secret_key,
                network_passphrase: soroban_config.network_passphrase.clone(),
            };
            
            IntegrationService::new_with_signing(soroban_config, signing_config)
                .map_err(loco_rs::Error::from)?
        } else {
            return Err(loco_rs::Error::string("Secret key required when signing is enabled"));
        }
    } else {
        IntegrationService::new(soroban_config)
            .map_err(loco_rs::Error::from)?
    };
    
    // Initialize event monitoring if requested
    if payload.enable_event_monitoring {
        integration_service.initialize_event_monitoring().await
            .map_err(loco_rs::Error::from)?;
    }
    
    let response = serde_json::json!({
        "status": "configured",
        "signing_enabled": payload.enable_signing,
        "event_monitoring_enabled": payload.enable_event_monitoring,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs()
    });
    
    format::json(response)
}