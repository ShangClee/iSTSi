use bitcoin_custody_backend::services::{
    soroban_client::{SorobanClient, SorobanConfig, ContractAddresses, SigningConfig},
    integration_service::{IntegrationService, BitcoinDepositRequest, TokenWithdrawalRequest},
    config_service::ConfigService,
};

/// Test Soroban client configuration loading
#[tokio::test]
async fn test_soroban_config_loading() {
    // Set test environment variables (all addresses must be exactly 56 characters)
    std::env::set_var("SOROBAN_NETWORK", "testnet");
    std::env::set_var("CONTRACT_INTEGRATION_ROUTER", "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1");
    std::env::set_var("CONTRACT_KYC_REGISTRY", "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2");
    std::env::set_var("CONTRACT_ISTSI_TOKEN", "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3");
    std::env::set_var("CONTRACT_RESERVE_MANAGER", "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4");
    
    let config = ConfigService::load_soroban_config().expect("Failed to load config");
    
    assert_eq!(config.network, "testnet");
    assert_eq!(config.rpc_url, "https://soroban-testnet.stellar.org");
    assert!(!config.contracts.integration_router.is_empty());
    
    // Validate contract addresses
    assert!(ConfigService::validate_contract_addresses(&config).is_ok());
}

/// Test Soroban client creation
#[tokio::test]
async fn test_soroban_client_creation() {
    let config = SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
            kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
            istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
            reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
        },
    };
    
    let client = SorobanClient::new(config).expect("Failed to create Soroban client");
    
    // Test contract address retrieval
    let router_address = client.get_contract_address("integration_router")
        .expect("Failed to get integration router address");
    assert_eq!(router_address, "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1");
    
    // Test invalid contract name
    assert!(client.get_contract_address("invalid_contract").is_err());
}

/// Test integration service creation
#[tokio::test]
async fn test_integration_service_creation() {
    let config = SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
            kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
            istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
            reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
        },
    };
    
    let _service = IntegrationService::new(config).expect("Failed to create integration service");
    
    // Test that service was created successfully
    // In a real test, we would mock the Soroban RPC calls
}

/// Test integration service with signing configuration
#[tokio::test]
async fn test_integration_service_with_signing() {
    let config = SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
            kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
            istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
            reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
        },
    };

    let signing_config = SigningConfig {
        secret_key: "SAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
    };
    
    let _service = IntegrationService::new_with_signing(config, signing_config)
        .expect("Failed to create integration service with signing");
}

/// Test event monitoring initialization
#[tokio::test]
async fn test_event_monitoring_initialization() {
    let config = SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
            kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
            istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
            reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
        },
    };
    
    let mut service = IntegrationService::new(config).expect("Failed to create integration service");
    
    // Initialize event monitoring
    let result = service.initialize_event_monitoring().await;
    assert!(result.is_ok(), "Event monitoring initialization should succeed");
    
    // Check that statistics are available
    let stats = service.get_event_statistics().await;
    assert!(stats.is_some(), "Event statistics should be available after initialization");
    
    let stats = stats.unwrap();
    assert_eq!(stats.total_events_processed, 0, "Initial event count should be zero");
}

/// Test Bitcoin deposit request validation
#[tokio::test]
async fn test_bitcoin_deposit_validation() {
    let config = SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
            kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
            istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
            reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
        },
    };
    
    let service = IntegrationService::new(config).expect("Failed to create integration service");
    
    // Test insufficient confirmations
    let request = BitcoinDepositRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        btc_amount: 100000000, // 1 BTC in satoshis
        btc_tx_hash: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        confirmations: 2, // Less than required 3
    };
    
    let result = service.execute_bitcoin_deposit(request).await
        .expect("Failed to execute deposit");
    
    // Should fail due to insufficient confirmations
    assert!(matches!(result.status, bitcoin_custody_backend::services::integration_service::OperationStatus::Failed));
    assert!(result.error_message.is_some());
    assert!(result.error_message.unwrap().contains("confirmations"));
}

/// Test token withdrawal request validation
#[tokio::test]
async fn test_token_withdrawal_validation() {
    let config = SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
            kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
            istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
            reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
        },
    };
    
    let service = IntegrationService::new(config).expect("Failed to create integration service");
    
    // Test invalid Bitcoin address
    let request = TokenWithdrawalRequest {
        user_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        token_amount: 100000000, // 1 token
        btc_address: "invalid_btc_address".to_string(),
    };
    
    let result = service.execute_token_withdrawal(request).await
        .expect("Failed to execute withdrawal");
    
    // Should fail due to invalid Bitcoin address
    assert!(matches!(result.status, bitcoin_custody_backend::services::integration_service::OperationStatus::Failed));
    assert!(result.error_message.is_some());
    assert!(result.error_message.unwrap().contains("Invalid Bitcoin address"));
}

/// Test network configuration
#[test]
fn test_network_configuration() {
    let mainnet_config = ConfigService::get_network_config("mainnet");
    assert_eq!(mainnet_config.get("rpc_url").unwrap(), "https://soroban-mainnet.stellar.org");
    assert_eq!(mainnet_config.get("network_passphrase").unwrap(), "Public Global Stellar Network ; September 2015");
    
    let testnet_config = ConfigService::get_network_config("testnet");
    assert_eq!(testnet_config.get("rpc_url").unwrap(), "https://soroban-testnet.stellar.org");
    assert_eq!(testnet_config.get("network_passphrase").unwrap(), "Test SDF Network ; September 2015");
    
    let futurenet_config = ConfigService::get_network_config("futurenet");
    assert_eq!(futurenet_config.get("rpc_url").unwrap(), "https://rpc-futurenet.stellar.org");
    assert_eq!(futurenet_config.get("network_passphrase").unwrap(), "Test SDF Future Network ; October 2022");
}

/// Test Stellar address validation
#[test]
fn test_stellar_address_validation() {
    // Valid contract addresses (start with 'C' and are 56 characters)
    assert!(ConfigService::validate_contract_addresses(&SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
            kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
            istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
            reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
        },
    }).is_ok());
    
    // Invalid addresses (account addresses start with 'G')
    assert!(ConfigService::validate_contract_addresses(&SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(), // Invalid: account, not contract
            kyc_registry: "".to_string(), // Empty to skip validation
            istsi_token: "".to_string(),
            reserve_manager: "".to_string(),
        },
    }).is_err());
}