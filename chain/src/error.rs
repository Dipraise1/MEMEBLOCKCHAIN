use thiserror::Error;

/// Main error type for MemeChain
#[derive(Error, Debug)]
pub enum MemeChainError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Module error: {0}")]
    Module(#[from] ModuleError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("NFT not found: {0}")]
    NftNotFound(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Blockchain error: {0}")]
    Blockchain(String),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load config file: {0}")]
    LoadFailed(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid genesis configuration: {0}")]
    InvalidGenesis(String),
}

/// Storage-related errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Write failed: {0}")]
    WriteFailed(String),

    #[error("Read failed: {0}")]
    ReadFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Corrupted data: {0}")]
    CorruptedData(String),
}

/// Module-related errors
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("NFT module error: {0}")]
    Nft(#[from] NftError),

    #[error("Meme module error: {0}")]
    Meme(#[from] MemeError),

    #[error("Common module error: {0}")]
    Common(#[from] CommonError),
}

/// NFT module errors
#[derive(Error, Debug)]
pub enum NftError {
    #[error("Collection already exists: {0}")]
    CollectionExists(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("NFT already exists: {0}")]
    NftExists(String),

    #[error("NFT not found: {0}")]
    NftNotFound(String),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    #[error("Transfer failed: {0}")]
    TransferFailed(String),

    #[error("Unauthorized operation: {0}")]
    Unauthorized(String),

    #[error("Invalid collection ID: {0}")]
    InvalidCollectionId(String),

    #[error("Invalid NFT ID: {0}")]
    InvalidNftId(String),
}

/// Meme token module errors
#[derive(Error, Debug)]
pub enum MemeError {
    #[error("Token already exists: {0}")]
    TokenExists(String),

    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("Invalid token symbol: {0}")]
    InvalidSymbol(String),

    #[error("Invalid token name: {0}")]
    InvalidName(String),

    #[error("Invalid supply: {0}")]
    InvalidSupply(String),

    #[error("Transfer failed: {0}")]
    TransferFailed(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Max wallet limit exceeded: {0}")]
    MaxWalletLimitExceeded(String),

    #[error("Tax calculation failed: {0}")]
    TaxCalculationFailed(String),

    #[error("Liquidity not locked: {0}")]
    LiquidityNotLocked(String),

    #[error("Lock period not expired: {0}")]
    LockPeriodNotExpired(String),

    #[error("Invalid anti-rug settings: {0}")]
    InvalidAntiRugSettings(String),
}

/// Common module errors
#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("Hash calculation failed: {0}")]
    HashCalculationFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
}

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

// Type alias for Result
pub type Result<T> = std::result::Result<T, MemeChainError>;

impl From<toml::de::Error> for MemeChainError {
    fn from(err: toml::de::Error) -> Self {
        MemeChainError::Config(ConfigError::LoadFailed(err.to_string()))
    }
}

impl From<toml::ser::Error> for MemeChainError {
    fn from(err: toml::ser::Error) -> Self {
        MemeChainError::Config(ConfigError::LoadFailed(err.to_string()))
    }
}

impl From<rocksdb::Error> for MemeChainError {
    fn from(err: rocksdb::Error) -> Self {
        MemeChainError::Database(err.to_string())
    }
}

impl From<sled::Error> for MemeChainError {
    fn from(err: sled::Error) -> Self {
        MemeChainError::Database(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let memechain_error: MemeChainError = io_error.into();
        assert!(matches!(memechain_error, MemeChainError::Io(_)));
    }

    #[test]
    fn test_nft_error() {
        let nft_error = NftError::CollectionNotFound("test".to_string());
        let module_error = ModuleError::Nft(nft_error);
        let memechain_error = MemeChainError::Module(module_error);
        assert!(matches!(memechain_error, MemeChainError::Module(_)));
    }
} 