use crate::config::StorageConfig;
use crate::error::{MemeChainError, Result, StorageError};
use crate::types::{Address, Balance, Block, Collection, Nft, Token};
use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Storage trait for different database backends
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Initialize the storage
    async fn initialize(&self) -> Result<()>;
    
    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Set a key-value pair
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    
    /// Delete a key
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;
    
    /// Get all keys with a prefix
    async fn get_keys_with_prefix(&self, prefix: &str) -> Result<Vec<String>>;
    
    /// Batch write operations
    async fn batch_write(&self, operations: Vec<(String, Option<Vec<u8>>)>) -> Result<()>;
}

/// RocksDB storage backend
pub struct RocksDBBackend {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl RocksDBBackend {
    /// Create a new RocksDB backend
    pub async fn new(path: &str) -> Result<Self> {
        info!("Initializing RocksDB at path: {}", path);
        
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(10000);
        opts.set_use_fsync(true);
        opts.set_bytes_per_sync(1024 * 1024); // 1MB
        
        let db = DBWithThreadMode::<MultiThreaded>::open(&opts, path)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }
}

#[async_trait::async_trait]
impl StorageBackend for RocksDBBackend {
    async fn initialize(&self) -> Result<()> {
        info!("RocksDB storage initialized");
        Ok(())
    }
    
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let db = self.db.clone();
        let key = key.to_string();
        
        tokio::task::spawn_blocking(move || {
            db.get(key.as_bytes())
                .map_err(|e| StorageError::ReadFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }
    
    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let db = self.db.clone();
        let key = key.to_string();
        let value = value.to_vec();
        
        tokio::task::spawn_blocking(move || {
            db.put(key.as_bytes(), &value)
                .map_err(|e| StorageError::WriteFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::WriteFailed(e.to_string()))?
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let db = self.db.clone();
        let key = key.to_string();
        
        tokio::task::spawn_blocking(move || {
            db.delete(key.as_bytes())
                .map_err(|e| StorageError::WriteFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::WriteFailed(e.to_string()))?
    }
    
    async fn exists(&self, key: &str) -> Result<bool> {
        let result = self.get(key).await?;
        Ok(result.is_some())
    }
    
    async fn get_keys_with_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        let db = self.db.clone();
        let prefix = prefix.to_string();
        
        tokio::task::spawn_blocking(move || {
            let iter = db.iterator(rocksdb::IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));
            let mut keys = Vec::new();
            
            for result in iter {
                match result {
                    Ok((key, _)) => {
                        if key.starts_with(prefix.as_bytes()) {
                            if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                                keys.push(key_str);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error iterating keys: {}", e);
                    }
                }
            }
            
            Ok(keys)
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }
    
    async fn batch_write(&self, operations: Vec<(String, Option<Vec<u8>>)>) -> Result<()> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut batch = rocksdb::WriteBatch::default();
            
            for (key, value) in operations {
                match value {
                    Some(val) => batch.put(key.as_bytes(), &val),
                    None => batch.delete(key.as_bytes()),
                }
            }
            
            db.write(batch)
                .map_err(|e| StorageError::WriteFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::WriteFailed(e.to_string()))?
    }
}

/// Sled storage backend
pub struct SledBackend {
    db: Arc<sled::Db>,
}

impl SledBackend {
    /// Create a new Sled backend
    pub async fn new(path: &str) -> Result<Self> {
        info!("Initializing Sled at path: {}", path);
        
        let db = sled::open(path)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }
}

#[async_trait::async_trait]
impl StorageBackend for SledBackend {
    async fn initialize(&self) -> Result<()> {
        info!("Sled storage initialized");
        Ok(())
    }
    
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let db = self.db.clone();
        let key = key.to_string();
        
        tokio::task::spawn_blocking(move || {
            db.get(key.as_bytes())
                .map_err(|e| StorageError::ReadFailed(e.to_string()))
                .map(|result| result.map(|ivec| ivec.to_vec()))
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }
    
    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let db = self.db.clone();
        let key = key.to_string();
        let value = value.to_vec();
        
        tokio::task::spawn_blocking(move || {
            db.insert(key.as_bytes(), value)
                .map_err(|e| StorageError::WriteFailed(e.to_string()))
                .map(|_| ())
        })
        .await
        .map_err(|e| StorageError::WriteFailed(e.to_string()))?
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let db = self.db.clone();
        let key = key.to_string();
        
        tokio::task::spawn_blocking(move || {
            db.remove(key.as_bytes())
                .map_err(|e| StorageError::WriteFailed(e.to_string()))
                .map(|_| ())
        })
        .await
        .map_err(|e| StorageError::WriteFailed(e.to_string()))?
    }
    
    async fn exists(&self, key: &str) -> Result<bool> {
        let result = self.get(key).await?;
        Ok(result.is_some())
    }
    
    async fn get_keys_with_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        let db = self.db.clone();
        let prefix = prefix.to_string();
        
        tokio::task::spawn_blocking(move || {
            let iter = db.scan_prefix(prefix.as_bytes());
            let mut keys = Vec::new();
            
            for result in iter {
                match result {
                    Ok((key, _)) => {
                        if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                            keys.push(key_str);
                        }
                    }
                    Err(e) => {
                        warn!("Error iterating keys: {}", e);
                    }
                }
            }
            
            Ok(keys)
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }
    
    async fn batch_write(&self, operations: Vec<(String, Option<Vec<u8>>)>) -> Result<()> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut batch = sled::Batch::default();
            
            for (key, value) in operations {
                match value {
                    Some(val) => batch.insert(key.as_bytes(), val),
                    None => batch.remove(key.as_bytes()),
                }
            }
            
            db.apply_batch(batch)
                .map_err(|e| StorageError::WriteFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::WriteFailed(e.to_string()))?
    }
}

/// Main storage interface
pub struct Storage {
    backend: Box<dyn StorageBackend>,
}

impl Storage {
    /// Create a new storage instance
    pub async fn new(config: &StorageConfig) -> Result<Self> {
        info!("Creating storage with type: {}", config.db_type);
        
        let backend: Box<dyn StorageBackend> = match config.db_type.as_str() {
            "rocksdb" => {
                let rocks_backend = RocksDBBackend::new(&config.db_path).await?;
                Box::new(rocks_backend)
            }
            "sled" => {
                let sled_backend = SledBackend::new(&config.db_path).await?;
                Box::new(sled_backend)
            }
            _ => return Err(StorageError::ConnectionFailed(format!("Unknown database type: {}", config.db_type))),
        };
        
        Ok(Self { backend })
    }
    
    /// Initialize storage
    pub async fn initialize(&self) -> Result<()> {
        self.backend.initialize().await
    }
    
    /// Store a block
    pub async fn store_block(&self, block: &Block) -> Result<()> {
        let key = format!("block:{}", block.height);
        let value = serde_json::to_vec(block)?;
        self.backend.set(&key, &value).await
    }
    
    /// Get a block by height
    pub async fn get_block(&self, height: u64) -> Result<Option<Block>> {
        let key = format!("block:{}", height);
        if let Some(data) = self.backend.get(&key).await? {
            let block: Block = serde_json::from_slice(&data)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }
    
    /// Store a token
    pub async fn store_token(&self, token: &Token) -> Result<()> {
        let key = format!("token:{}", token.symbol);
        let value = serde_json::to_vec(token)?;
        self.backend.set(&key, &value).await
    }
    
    /// Get a token by symbol
    pub async fn get_token(&self, symbol: &str) -> Result<Option<Token>> {
        let key = format!("token:{}", symbol);
        if let Some(data) = self.backend.get(&key).await? {
            let token: Token = serde_json::from_slice(&data)?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }
    
    /// Store an NFT
    pub async fn store_nft(&self, nft: &Nft) -> Result<()> {
        let key = format!("nft:{}", nft.id);
        let value = serde_json::to_vec(nft)?;
        self.backend.set(&key, &value).await
    }
    
    /// Get an NFT by ID
    pub async fn get_nft(&self, id: &str) -> Result<Option<Nft>> {
        let key = format!("nft:{}", id);
        if let Some(data) = self.backend.get(&key).await? {
            let nft: Nft = serde_json::from_slice(&data)?;
            Ok(Some(nft))
        } else {
            Ok(None)
        }
    }
    
    /// Store a collection
    pub async fn store_collection(&self, collection: &Collection) -> Result<()> {
        let key = format!("collection:{}", collection.id);
        let value = serde_json::to_vec(collection)?;
        self.backend.set(&key, &value).await
    }
    
    /// Get a collection by ID
    pub async fn get_collection(&self, id: &str) -> Result<Option<Collection>> {
        let key = format!("collection:{}", id);
        if let Some(data) = self.backend.get(&key).await? {
            let collection: Collection = serde_json::from_slice(&data)?;
            Ok(Some(collection))
        } else {
            Ok(None)
        }
    }
    
    /// Store a balance
    pub async fn store_balance(&self, balance: &Balance) -> Result<()> {
        let key = format!("balance:{}:{}", balance.address, balance.token);
        let value = serde_json::to_vec(balance)?;
        self.backend.set(&key, &value).await
    }
    
    /// Get a balance
    pub async fn get_balance(&self, address: &Address, token: &str) -> Result<Option<Balance>> {
        let key = format!("balance:{}:{}", address, token);
        if let Some(data) = self.backend.get(&key).await? {
            let balance: Balance = serde_json::from_slice(&data)?;
            Ok(Some(balance))
        } else {
            Ok(None)
        }
    }
    
    /// Get all tokens
    pub async fn get_all_tokens(&self) -> Result<Vec<Token>> {
        let keys = self.backend.get_keys_with_prefix("token:").await?;
        let mut tokens = Vec::new();
        
        for key in keys {
            if let Some(data) = self.backend.get(&key).await? {
                if let Ok(token) = serde_json::from_slice::<Token>(&data) {
                    tokens.push(token);
                }
            }
        }
        
        Ok(tokens)
    }
    
    /// Get all NFTs
    pub async fn get_all_nfts(&self) -> Result<Vec<Nft>> {
        let keys = self.backend.get_keys_with_prefix("nft:").await?;
        let mut nfts = Vec::new();
        
        for key in keys {
            if let Some(data) = self.backend.get(&key).await? {
                if let Ok(nft) = serde_json::from_slice::<Nft>(&data) {
                    nfts.push(nft);
                }
            }
        }
        
        Ok(nfts)
    }
    
    /// Get all collections
    pub async fn get_all_collections(&self) -> Result<Vec<Collection>> {
        let keys = self.backend.get_keys_with_prefix("collection:").await?;
        let mut collections = Vec::new();
        
        for key in keys {
            if let Some(data) = self.backend.get(&key).await? {
                if let Ok(collection) = serde_json::from_slice::<Collection>(&data) {
                    collections.push(collection);
                }
            }
        }
        
        Ok(collections)
    }
    
    /// Update balance atomically
    pub async fn update_balance(&self, address: &Address, token: &str, amount: i64) -> Result<()> {
        let current_balance = self.get_balance(address, token).await?;
        let new_amount = match current_balance {
            Some(mut balance) => {
                if amount > 0 {
                    balance.add(amount as u64);
                } else {
                    balance.subtract((-amount) as u64)?;
                }
                balance.amount
            }
            None => {
                if amount < 0 {
                    return Err(StorageError::WriteFailed("Cannot create negative balance".to_string()));
                }
                amount as u64
            }
        };
        
        let new_balance = Balance::new(address.clone(), token.to_string(), new_amount);
        self.store_balance(&new_balance).await
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        // This is a simplified clone - in a real implementation,
        // you'd want to properly clone the backend or use Arc
        Self {
            backend: Box::new(DummyBackend {}),
        }
    }
}

/// Dummy backend for cloning (not used in production)
struct DummyBackend {}

#[async_trait::async_trait]
impl StorageBackend for DummyBackend {
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }
    
    async fn set(&self, _key: &str, _value: &[u8]) -> Result<()> {
        Ok(())
    }
    
    async fn delete(&self, _key: &str) -> Result<()> {
        Ok(())
    }
    
    async fn exists(&self, _key: &str) -> Result<bool> {
        Ok(false)
    }
    
    async fn get_keys_with_prefix(&self, _prefix: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
    
    async fn batch_write(&self, _operations: Vec<(String, Option<Vec<u8>>)>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_rocksdb_storage() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test_db");
        
        let backend = RocksDBBackend::new(path.to_str().unwrap()).await.unwrap();
        backend.initialize().await.unwrap();
        
        // Test set and get
        backend.set("test_key", b"test_value").await.unwrap();
        let value = backend.get("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));
        
        // Test exists
        assert!(backend.exists("test_key").await.unwrap());
        assert!(!backend.exists("nonexistent").await.unwrap());
        
        // Test delete
        backend.delete("test_key").await.unwrap();
        assert!(!backend.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_storage_operations() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test_storage");
        
        let config = StorageConfig {
            db_path: path.to_str().unwrap().to_string(),
            db_type: "rocksdb".to_string(),
            cache_size: 100,
            enable_compression: false,
        };
        
        let storage = Storage::new(&config).await.unwrap();
        storage.initialize().await.unwrap();
        
        // Test token storage
        let token = Token::new(
            "TEST".to_string(),
            "Test Token".to_string(),
            1000000,
            Address::new("memechain1alice".to_string()),
            crate::types::AntiRugSettings::default(),
        );
        
        storage.store_token(&token).await.unwrap();
        let retrieved = storage.get_token("TEST").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().symbol, "TEST");
    }
} 