use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Main blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Chain configuration
    pub chain: ChainConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// API configuration
    pub api: ApiConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Consensus configuration
    pub consensus: ConsensusConfig,
}

/// Chain-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain ID
    pub chain_id: String,
    /// Block time in seconds
    pub block_time: u64,
    /// Maximum block size in bytes
    pub max_block_size: u64,
    /// Gas limit per block
    pub gas_limit: u64,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P port
    pub p2p_port: u16,
    /// RPC port
    pub rpc_port: u16,
    /// Seeds for peer discovery
    pub seeds: Vec<String>,
    /// Persistent peers
    pub persistent_peers: Vec<String>,
    /// Maximum number of peers
    pub max_peers: u32,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API server port
    pub api_port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Rate limiting
    pub rate_limit: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path
    pub db_path: String,
    /// Database type (rocksdb, sled)
    pub db_type: String,
    /// Cache size in MB
    pub cache_size: u64,
    /// Enable compression
    pub enable_compression: bool,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Validator moniker
    pub moniker: String,
    /// Validator private key path
    pub validator_key_path: String,
    /// Consensus timeout
    pub timeout_commit: u64,
    /// Block size limit
    pub max_block_size_txs: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            chain: ChainConfig::default(),
            network: NetworkConfig::default(),
            api: ApiConfig::default(),
            storage: StorageConfig::default(),
            consensus: ConsensusConfig::default(),
        }
    }
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: "memechain-dev".to_string(),
            block_time: 6,
            max_block_size: 1024 * 1024, // 1MB
            gas_limit: 10_000_000,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            p2p_port: 26656,
            rpc_port: 26657,
            seeds: vec![],
            persistent_peers: vec![],
            max_peers: 50,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_port: 8080,
            enable_cors: true,
            allowed_origins: vec!["*".to_string()],
            rate_limit: 1000,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: "./data".to_string(),
            db_type: "rocksdb".to_string(),
            cache_size: 512, // 512MB
            enable_compression: true,
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            moniker: "validator".to_string(),
            validator_key_path: "./config/priv_validator_key.json".to_string(),
            timeout_commit: 5000, // 5 seconds
            max_block_size_txs: 10000,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> crate::error::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> crate::error::Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// Genesis configuration for blockchain initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Genesis time
    pub genesis_time: String,
    /// Chain ID
    pub chain_id: String,
    /// Initial validators
    pub validators: Vec<Validator>,
    /// Initial accounts
    pub accounts: Vec<Account>,
    /// App state
    pub app_state: AppState,
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator address
    pub address: String,
    /// Validator public key
    pub pub_key: String,
    /// Validator power
    pub power: u64,
    /// Validator name
    pub name: String,
}

/// Account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account address
    pub address: String,
    /// Account balance
    pub balance: u64,
    /// Account name
    pub name: String,
}

/// Application state in genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// NFT module state
    pub nft: NftState,
    /// Meme token module state
    pub meme: MemeState,
}

/// NFT module genesis state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftState {
    /// Collections
    pub collections: Vec<Collection>,
}

/// Meme token module genesis state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeState {
    /// Tokens
    pub tokens: Vec<Token>,
}

/// NFT Collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Collection ID
    pub id: String,
    /// Collection name
    pub name: String,
    /// Creator address
    pub creator: String,
    /// Description
    pub description: String,
}

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Total supply
    pub total_supply: u64,
    /// Creator address
    pub creator: String,
    /// Anti-rug settings
    pub anti_rug: AntiRugSettings,
}

/// Anti-rug protection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiRugSettings {
    /// Maximum wallet percentage
    pub max_wallet_percentage: u8,
    /// Buy tax percentage
    pub buy_tax_percentage: u8,
    /// Sell tax percentage
    pub sell_tax_percentage: u8,
    /// Liquidity locked percentage
    pub liquidity_locked_percentage: u8,
    /// Lock duration in blocks
    pub lock_duration_blocks: u64,
}

impl GenesisConfig {
    /// Create a new genesis configuration
    pub fn new(chain_id: String, moniker: String) -> Self {
        let genesis_time = chrono::Utc::now().to_rfc3339();
        
        Self {
            genesis_time,
            chain_id: chain_id.clone(),
            validators: vec![
                Validator {
                    address: "memechain1validator".to_string(),
                    pub_key: "memechainvalconspub1...".to_string(),
                    power: 100,
                    name: moniker,
                }
            ],
            accounts: vec![
                Account {
                    address: "memechain1alice".to_string(),
                    balance: 1_000_000_000, // 1 billion tokens
                    name: "alice".to_string(),
                },
                Account {
                    address: "memechain1bob".to_string(),
                    balance: 1_000_000_000,
                    name: "bob".to_string(),
                }
            ],
            app_state: AppState {
                nft: NftState {
                    collections: vec![],
                },
                meme: MemeState {
                    tokens: vec![],
                },
            },
        }
    }

    /// Save genesis configuration to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> crate::error::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

impl Default for AntiRugSettings {
    fn default() -> Self {
        Self {
            max_wallet_percentage: 5, // 5% max per wallet
            buy_tax_percentage: 2,     // 2% buy tax
            sell_tax_percentage: 3,    // 3% sell tax
            liquidity_locked_percentage: 80, // 80% locked
            lock_duration_blocks: 1000, // ~100 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed_config: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.chain.chain_id, parsed_config.chain.chain_id);
    }

    #[test]
    fn test_genesis_creation() {
        let genesis = GenesisConfig::new("test-chain".to_string(), "test-validator".to_string());
        assert_eq!(genesis.chain_id, "test-chain");
        assert_eq!(genesis.validators.len(), 1);
        assert_eq!(genesis.accounts.len(), 2);
    }
} 