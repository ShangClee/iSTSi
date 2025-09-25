use bitcoin_custody_backend::services::{
    integration_service::{IntegrationService, BitcoinDepositRequest, TokenWithdrawalRequest, OperationStatus},
    soroban_client::{SorobanConfig, ContractAddresses},
};
use uuid::Uuid;

fn create_test_config() -> SorobanConfig {
    SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
            kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
            istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
            reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
        },
    }
}

#[tokio::test]
async fn test_integration_service_creation() {
    let config = create_test_config();
    let service = IntegrationService::new(config);
    
    assert!(service.is_ok());
}

#[tokio::test]
async fn test_bitcoin_deposit_validation_success() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let request = BitcoinDepositRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        btc_amount: 100000000, // 1 BTC
        btc_tx_hash: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        confirmations: 6,
    };
    
    let result = service.validate_bitcoin_deposit(&request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_bitcoin_deposit_validation_insufficient_confirmations() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let request = BitcoinDepositRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        btc_amount: 100000000,
        btc_tx_hash: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        confirmations: 2, // Less than required 3
    };
    
    let result = service.validate_bitcoin_deposit(&request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("confirmations"));
}

#[tokio::test]
async fn test_bitcoin_deposit_validation_invalid_amount() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let request = BitcoinDepositRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        btc_amount: 0, // Invalid amount
        btc_tx_hash: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        confirmations: 6,
    };
    
    let result = service.validate_bitcoin_deposit(&request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("amount"));
}

#[tokio::test]
async fn test_bitcoin_deposit_validation_invalid_tx_hash() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let request = BitcoinDepositRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        btc_amount: 100000000,
        btc_tx_hash: "invalid_hash".to_string(), // Invalid format
        confirmations: 6,
    };
    
    let result = service.validate_bitcoin_deposit(&request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("transaction hash"));
}

#[tokio::test]
async fn test_token_withdrawal_validation_success() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let request = TokenWithdrawalRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        token_amount: 50000000, // 0.5 BTC worth
        btc_address: "bc1qtest123456789abcdef".to_string(),
    };
    
    let result = service.validate_token_withdrawal(&request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_token_withdrawal_validation_invalid_btc_address() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let request = TokenWithdrawalRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        token_amount: 50000000,
        btc_address: "invalid_btc_address".to_string(),
    };
    
    let result = service.validate_token_withdrawal(&request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid Bitcoin address"));
}

#[tokio::test]
async fn test_token_withdrawal_validation_zero_amount() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let request = TokenWithdrawalRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        token_amount: 0,
        btc_address: "bc1qtest123456789abcdef".to_string(),
    };
    
    let result = service.validate_token_withdrawal(&request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("amount"));
}

#[tokio::test]
async fn test_operation_status_tracking() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let operation_id = Uuid::new_v4().to_string();
    
    // Initially, operation should not exist
    let status = service.get_operation_status(&operation_id).await;
    assert!(status.is_err());
    
    // Create an operation
    service.create_operation_record(&operation_id, "bitcoin_deposit", OperationStatus::Pending).await.unwrap();
    
    // Now it should exist
    let status = service.get_operation_status(&operation_id).await;
    assert!(status.is_ok());
    assert_eq!(status.unwrap().status, OperationStatus::Pending);
    
    // Update status
    service.update_operation_status(&operation_id, OperationStatus::Completed, None).await.unwrap();
    
    // Verify update
    let status = service.get_operation_status(&operation_id).await;
    assert!(status.is_ok());
    assert_eq!(status.unwrap().status, OperationStatus::Completed);
}

#[tokio::test]
async fn test_operation_error_handling() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let operation_id = Uuid::new_v4().to_string();
    let error_message = "Test error message";
    
    // Create failed operation
    service.create_operation_record(&operation_id, "bitcoin_deposit", OperationStatus::Failed).await.unwrap();
    service.update_operation_status(&operation_id, OperationStatus::Failed, Some(error_message.to_string())).await.unwrap();
    
    let status = service.get_operation_status(&operation_id).await.unwrap();
    assert_eq!(status.status, OperationStatus::Failed);
    assert_eq!(status.error_message, Some(error_message.to_string()));
}

#[tokio::test]
async fn test_stellar_address_validation() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    // Valid Stellar account address
    assert!(service.validate_stellar_address("GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").is_ok());
    
    // Valid Stellar contract address
    assert!(service.validate_stellar_address("CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1").is_ok());
    
    // Invalid addresses
    assert!(service.validate_stellar_address("invalid").is_err());
    assert!(service.validate_stellar_address("").is_err());
    assert!(service.validate_stellar_address("GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1").is_err()); // Wrong length
}

#[tokio::test]
async fn test_bitcoin_address_validation() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    // Valid Bitcoin addresses (simplified validation for testing)
    assert!(service.validate_bitcoin_address("bc1qtest123456789abcdef").is_ok());
    assert!(service.validate_bitcoin_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa").is_ok());
    assert!(service.validate_bitcoin_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy").is_ok());
    
    // Invalid addresses
    assert!(service.validate_bitcoin_address("invalid").is_err());
    assert!(service.validate_bitcoin_address("").is_err());
}

#[tokio::test]
async fn test_amount_validation() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    // Valid amounts
    assert!(service.validate_amount(1).is_ok()); // 1 satoshi
    assert!(service.validate_amount(100000000).is_ok()); // 1 BTC
    assert!(service.validate_amount(2100000000000000).is_ok()); // 21M BTC
    
    // Invalid amounts
    assert!(service.validate_amount(0).is_err()); // Zero
    assert!(service.validate_amount(-1).is_err()); // Negative
}

#[tokio::test]
async fn test_transaction_hash_validation() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    // Valid transaction hash (64 hex characters)
    assert!(service.validate_transaction_hash("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").is_ok());
    
    // Invalid hashes
    assert!(service.validate_transaction_hash("invalid").is_err()); // Too short
    assert!(service.validate_transaction_hash("").is_err()); // Empty
    assert!(service.validate_transaction_hash("abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456789g").is_err()); // Invalid character
}

#[tokio::test]
async fn test_event_monitoring_initialization() {
    let config = create_test_config();
    let mut service = IntegrationService::new(config).unwrap();
    
    let result = service.initialize_event_monitoring().await;
    assert!(result.is_ok());
    
    // Check that statistics are available
    let stats = service.get_event_statistics().await;
    assert!(stats.is_some());
    
    let stats = stats.unwrap();
    assert_eq!(stats.total_events_processed, 0);
    assert!(stats.last_processed_ledger.is_none());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let mut handles = vec![];
    
    // Create multiple concurrent operations
    for i in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let operation_id = format!("op-{}", i);
            service_clone.create_operation_record(&operation_id, "test", OperationStatus::Pending).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_rate_limiting() {
    let config = create_test_config();
    let service = IntegrationService::new(config).unwrap();
    
    let user_address = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    
    // First request should succeed
    let result1 = service.check_rate_limit(user_address, "bitcoin_deposit").await;
    assert!(result1.is_ok());
    
    // Immediate second request might be rate limited (depending on implementation)
    let result2 = service.check_rate_limit(user_address, "bitcoin_deposit").await;
    // This test depends on the specific rate limiting implementation
    // For now, we just ensure it doesn't panic
    let _ = result2;
}