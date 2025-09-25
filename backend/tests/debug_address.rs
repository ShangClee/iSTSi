use bitcoin_custody_backend::services::config_service::ConfigService;
use bitcoin_custody_backend::services::soroban_client::{SorobanConfig, ContractAddresses};

#[test]
fn debug_address_validation() {
    let test_address = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1";
    println!("Test address: '{}'", test_address);
    println!("Length: {}", test_address.len());
    println!("Starts with C: {}", test_address.starts_with('C'));
    
    for (i, c) in test_address.chars().enumerate() {
        println!("Char {}: '{}' (ASCII: {}, is_alphanumeric: {}, is_uppercase: {})", 
                 i, c, c as u32, c.is_ascii_alphanumeric(), c.is_ascii_uppercase());
    }
    
    let config = SorobanConfig {
        network: "testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        contracts: ContractAddresses {
            integration_router: test_address.to_string(),
            kyc_registry: "".to_string(), // Empty to skip validation
            istsi_token: "".to_string(),
            reserve_manager: "".to_string(),
        },
    };
    
    let result = ConfigService::validate_contract_addresses(&config);
    println!("Validation result: {:?}", result);
    assert!(result.is_ok());
}