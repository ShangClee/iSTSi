use soroban_sdk::contracterror;

/// Common error types used across Bitcoin custody contracts

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IntegrationError {
    // Authentication & Authorization
    Unauthorized = 1,
    InsufficientPermissions = 2,
    
    // Contract Communication
    ContractNotFound = 10,
    ContractCallFailed = 11,
    InvalidContractResponse = 12,
    
    // Compliance & KYC
    ComplianceCheckFailed = 20,
    InsufficientKYCTier = 21,
    AddressBlacklisted = 22,
    
    // Reserve Management
    InsufficientReserves = 30,
    ReserveRatioTooLow = 31,
    BitcoinTransactionFailed = 32,
    
    // Operation Processing
    OperationTimeout = 40,
    InvalidOperationState = 41,
    DuplicateOperation = 42,
    
    // System State
    SystemPaused = 50,
    EmergencyMode = 51,
    MaintenanceMode = 52,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    InvalidAddress = 100,
    InvalidAmount = 101,
    InvalidTimestamp = 102,
    InvalidSignature = 103,
    InvalidParameters = 104,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StorageError {
    KeyNotFound = 200,
    SerializationFailed = 201,
    DeserializationFailed = 202,
    StorageFull = 203,
}