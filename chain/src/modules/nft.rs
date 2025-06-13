use crate::error::{MemeChainError, Result, NftError};
use crate::storage::Storage;
use crate::types::{Address, Collection, Nft, Transaction, TransactionResult};
use serde_json::Value;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// NFT module for managing collections and NFTs
pub struct NftModule {
    storage: Storage,
}

impl NftModule {
    /// Create a new NFT module
    pub async fn new(storage: Storage) -> Result<Self> {
        info!("Initializing NFT module");
        Ok(Self { storage })
    }

    /// Process NFT-related transactions
    pub async fn process_transaction(&self, tx: Transaction) -> Result<TransactionResult> {
        debug!("Processing NFT transaction: {} - {}", tx.module, tx.action);

        match tx.action.as_str() {
            "create_collection" => self.create_collection(tx).await,
            "mint" => self.mint_nft(tx).await,
            "transfer" => self.transfer_nft(tx).await,
            "burn" => self.burn_nft(tx).await,
            "update_metadata" => self.update_metadata(tx).await,
            _ => Err(NftError::InvalidNftId(format!("Unknown action: {}", tx.action))),
        }
    }

    /// Create a new collection
    async fn create_collection(&self, tx: Transaction) -> Result<TransactionResult> {
        let name = tx.data["name"]
            .as_str()
            .ok_or_else(|| NftError::InvalidMetadata("Missing collection name".to_string()))?;
        
        let description = tx.data["description"]
            .as_str()
            .unwrap_or("");

        let collection_id = Uuid::new_v4().to_string();
        let creator = tx.from;

        // Check if collection already exists
        if self.storage.get_collection(&collection_id).await?.is_some() {
            return Err(NftError::CollectionExists(collection_id));
        }

        let collection = Collection::new(
            collection_id.clone(),
            name.to_string(),
            creator.clone(),
            description.to_string(),
        );

        // Store collection
        self.storage.store_collection(&collection).await?;

        info!("Created collection: {} by {}", name, creator);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "collection_id": collection_id,
            "name": name,
            "creator": creator.to_string(),
        }))))
    }

    /// Mint a new NFT
    async fn mint_nft(&self, tx: Transaction) -> Result<TransactionResult> {
        let collection_id = tx.data["collection"]
            .as_str()
            .ok_or_else(|| NftError::InvalidCollectionId("Missing collection ID".to_string()))?;
        
        let name = tx.data["name"]
            .as_str()
            .ok_or_else(|| NftError::InvalidMetadata("Missing NFT name".to_string()))?;
        
        let metadata = tx.data["metadata"].clone();
        let owner = tx.from;

        // Verify collection exists
        let collection = self.storage.get_collection(collection_id).await?
            .ok_or_else(|| NftError::CollectionNotFound(collection_id.to_string()))?;

        // Generate unique NFT ID
        let nft_id = Uuid::new_v4().to_string();

        // Check if NFT already exists
        if self.storage.get_nft(&nft_id).await?.is_some() {
            return Err(NftError::NftExists(nft_id));
        }

        let nft = Nft::new(
            nft_id.clone(),
            collection_id.to_string(),
            name.to_string(),
            owner.clone(),
            metadata,
        );

        // Store NFT
        self.storage.store_nft(&nft).await?;

        info!("Minted NFT: {} in collection: {} for owner: {}", name, collection_id, owner);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "nft_id": nft_id,
            "collection_id": collection_id,
            "name": name,
            "owner": owner.to_string(),
        }))))
    }

    /// Transfer an NFT
    async fn transfer_nft(&self, tx: Transaction) -> Result<TransactionResult> {
        let nft_id = tx.data["nft_id"]
            .as_str()
            .ok_or_else(|| NftError::InvalidNftId("Missing NFT ID".to_string()))?;
        
        let to_address = tx.to
            .ok_or_else(|| NftError::TransferFailed("Missing recipient address".to_string()))?;
        
        let from_address = tx.from;

        // Get NFT
        let mut nft = self.storage.get_nft(nft_id).await?
            .ok_or_else(|| NftError::NftNotFound(nft_id.to_string()))?;

        // Verify ownership
        if nft.owner != from_address {
            return Err(NftError::Unauthorized(format!(
                "NFT {} is not owned by {}", nft_id, from_address
            )));
        }

        // Update owner
        nft.owner = to_address.clone();
        nft.updated_at = chrono::Utc::now().timestamp();

        // Store updated NFT
        self.storage.store_nft(&nft).await?;

        info!("Transferred NFT: {} from {} to {}", nft_id, from_address, to_address);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "nft_id": nft_id,
            "from": from_address.to_string(),
            "to": to_address.to_string(),
        }))))
    }

    /// Burn an NFT
    async fn burn_nft(&self, tx: Transaction) -> Result<TransactionResult> {
        let nft_id = tx.data["nft_id"]
            .as_str()
            .ok_or_else(|| NftError::InvalidNftId("Missing NFT ID".to_string()))?;
        
        let owner = tx.from;

        // Get NFT
        let nft = self.storage.get_nft(nft_id).await?
            .ok_or_else(|| NftError::NftNotFound(nft_id.to_string()))?;

        // Verify ownership
        if nft.owner != owner {
            return Err(NftError::Unauthorized(format!(
                "NFT {} is not owned by {}", nft_id, owner
            )));
        }

        // Delete NFT
        let key = format!("nft:{}", nft_id);
        self.storage.backend.delete(&key).await?;

        info!("Burned NFT: {} by owner: {}", nft_id, owner);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "nft_id": nft_id,
            "burned_by": owner.to_string(),
        }))))
    }

    /// Update NFT metadata
    async fn update_metadata(&self, tx: Transaction) -> Result<TransactionResult> {
        let nft_id = tx.data["nft_id"]
            .as_str()
            .ok_or_else(|| NftError::InvalidNftId("Missing NFT ID".to_string()))?;
        
        let new_metadata = tx.data["metadata"].clone();
        let owner = tx.from;

        // Get NFT
        let mut nft = self.storage.get_nft(nft_id).await?
            .ok_or_else(|| NftError::NftNotFound(nft_id.to_string()))?;

        // Verify ownership
        if nft.owner != owner {
            return Err(NftError::Unauthorized(format!(
                "NFT {} is not owned by {}", nft_id, owner
            )));
        }

        // Update metadata
        nft.metadata = new_metadata;
        nft.updated_at = chrono::Utc::now().timestamp();

        // Store updated NFT
        self.storage.store_nft(&nft).await?;

        info!("Updated metadata for NFT: {} by owner: {}", nft_id, owner);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "nft_id": nft_id,
            "updated_by": owner.to_string(),
        }))))
    }

    /// Get NFT by ID
    pub async fn get_nft(&self, nft_id: &str) -> Result<Option<Nft>> {
        self.storage.get_nft(nft_id).await
    }

    /// Get collection by ID
    pub async fn get_collection(&self, collection_id: &str) -> Result<Option<Collection>> {
        self.storage.get_collection(collection_id).await
    }

    /// Get all NFTs
    pub async fn list_nfts(&self) -> Result<Vec<Value>> {
        let nfts = self.storage.get_all_nfts().await?;
        let mut result = Vec::new();
        
        for nft in nfts {
            result.push(serde_json::json!({
                "id": nft.id,
                "collection_id": nft.collection_id,
                "name": nft.name,
                "owner": nft.owner.to_string(),
                "metadata": nft.metadata,
                "created_at": nft.created_at,
                "updated_at": nft.updated_at,
            }));
        }
        
        Ok(result)
    }

    /// Get all collections
    pub async fn list_collections(&self) -> Result<Vec<Value>> {
        let collections = self.storage.get_all_collections().await?;
        let mut result = Vec::new();
        
        for collection in collections {
            result.push(serde_json::json!({
                "id": collection.id,
                "name": collection.name,
                "creator": collection.creator.to_string(),
                "description": collection.description,
                "created_at": collection.created_at,
                "updated_at": collection.updated_at,
            }));
        }
        
        Ok(result)
    }

    /// Get NFTs by collection
    pub async fn get_nfts_by_collection(&self, collection_id: &str) -> Result<Vec<Value>> {
        let nfts = self.storage.get_all_nfts().await?;
        let mut result = Vec::new();
        
        for nft in nfts {
            if nft.collection_id == collection_id {
                result.push(serde_json::json!({
                    "id": nft.id,
                    "name": nft.name,
                    "owner": nft.owner.to_string(),
                    "metadata": nft.metadata,
                    "created_at": nft.created_at,
                    "updated_at": nft.updated_at,
                }));
            }
        }
        
        Ok(result)
    }

    /// Get NFTs by owner
    pub async fn get_nfts_by_owner(&self, owner: &Address) -> Result<Vec<Value>> {
        let nfts = self.storage.get_all_nfts().await?;
        let mut result = Vec::new();
        
        for nft in nfts {
            if nft.owner == *owner {
                result.push(serde_json::json!({
                    "id": nft.id,
                    "collection_id": nft.collection_id,
                    "name": nft.name,
                    "metadata": nft.metadata,
                    "created_at": nft.created_at,
                    "updated_at": nft.updated_at,
                }));
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageConfig;
    use tempfile::tempdir;

    async fn create_test_storage() -> Storage {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test_nft_db");
        
        let config = StorageConfig {
            db_path: path.to_str().unwrap().to_string(),
            db_type: "rocksdb".to_string(),
            cache_size: 100,
            enable_compression: false,
        };
        
        Storage::new(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_collection() {
        let storage = create_test_storage().await;
        let module = NftModule::new(storage).await.unwrap();
        
        let tx = Transaction::new(
            "nft".to_string(),
            "create_collection".to_string(),
            Address::new("memechain1alice".to_string()),
            None,
            serde_json::json!({
                "name": "Test Collection",
                "description": "A test collection"
            }),
        );
        
        let result = module.process_transaction(tx).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_mint_nft() {
        let storage = create_test_storage().await;
        let module = NftModule::new(storage).await.unwrap();
        
        // First create a collection
        let collection_tx = Transaction::new(
            "nft".to_string(),
            "create_collection".to_string(),
            Address::new("memechain1alice".to_string()),
            None,
            serde_json::json!({
                "name": "Test Collection",
                "description": "A test collection"
            }),
        );
        
        let collection_result = module.process_transaction(collection_tx).await.unwrap();
        let collection_id = collection_result.data.unwrap()["collection_id"].as_str().unwrap();
        
        // Then mint an NFT
        let mint_tx = Transaction::new(
            "nft".to_string(),
            "mint".to_string(),
            Address::new("memechain1alice".to_string()),
            None,
            serde_json::json!({
                "collection": collection_id,
                "name": "Test NFT",
                "metadata": {"rarity": "legendary"}
            }),
        );
        
        let result = module.process_transaction(mint_tx).await.unwrap();
        assert!(result.success);
    }
} 