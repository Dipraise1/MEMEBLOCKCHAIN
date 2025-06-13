use crate::error::{MemeChainError, Result, CommonError};
use crate::storage::Storage;
use crate::types::{Address, Transaction, TransactionResult};
use ed25519_dalek::{PublicKey, SecretKey, Signature, Verifier};
use sha2::{Digest, Sha256};
use tracing::{debug, info};

/// Common utilities module for shared functionality
pub struct CommonModule {
    storage: Storage,
}

impl CommonModule {
    /// Create a new common module
    pub async fn new(storage: Storage) -> Result<Self> {
        info!("Initializing Common module");
        Ok(Self { storage })
    }

    /// Process common module transactions
    pub async fn process_transaction(&self, tx: Transaction) -> Result<TransactionResult> {
        debug!("Processing common transaction: {} - {}", tx.module, tx.action);

        match tx.action.as_str() {
            "validate_address" => self.validate_address_tx(tx).await,
            "generate_keypair" => self.generate_keypair(tx).await,
            "hash_data" => self.hash_data(tx).await,
            _ => Err(CommonError::InvalidAddress(format!("Unknown action: {}", tx.action))),
        }
    }

    /// Validate address format
    pub async fn validate_address(&self, address: &Address) -> Result<()> {
        if !address.is_valid() {
            return Err(CommonError::InvalidAddress(format!(
                "Invalid address format: {}", address
            )));
        }
        Ok(())
    }

    /// Validate transaction signature
    pub async fn validate_signature(&self, tx: &Transaction) -> Result<()> {
        // TODO: Implement proper signature validation
        // For now, just check if signature is not empty
        if tx.signature.is_empty() {
            return Err(CommonError::InvalidSignature("Empty signature".to_string()));
        }
        Ok(())
    }

    /// Generate a new keypair
    async fn generate_keypair(&self, tx: Transaction) -> Result<TransactionResult> {
        let mut rng = rand::thread_rng();
        let secret_key = SecretKey::generate(&mut rng);
        let public_key = PublicKey::from(&secret_key);

        let keypair_data = serde_json::json!({
            "public_key": hex::encode(public_key.to_bytes()),
            "private_key": hex::encode(secret_key.to_bytes()),
        });

        info!("Generated new keypair for {}", tx.from);

        Ok(TransactionResult::success(Some(keypair_data)))
    }

    /// Hash data
    async fn hash_data(&self, tx: Transaction) -> Result<TransactionResult> {
        let data = tx.data["data"]
            .as_str()
            .ok_or_else(|| CommonError::HashCalculationFailed("Missing data to hash".to_string()))?;

        let hash = self.calculate_hash(data.as_bytes());

        let hash_data = serde_json::json!({
            "data": data,
            "hash": hash,
        });

        Ok(TransactionResult::success(Some(hash_data)))
    }

    /// Validate address transaction
    async fn validate_address_tx(&self, tx: Transaction) -> Result<TransactionResult> {
        let address_str = tx.data["address"]
            .as_str()
            .ok_or_else(|| CommonError::InvalidAddress("Missing address".to_string()))?;

        let address = Address::new(address_str.to_string());
        let is_valid = address.is_valid();

        let result_data = serde_json::json!({
            "address": address_str,
            "is_valid": is_valid,
        });

        Ok(TransactionResult::success(Some(result_data)))
    }

    /// Calculate SHA256 hash
    pub fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Verify signature
    pub fn verify_signature(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
        let pub_key = PublicKey::from_bytes(public_key)
            .map_err(|e| CommonError::InvalidPublicKey(e.to_string()))?;
        
        let sig = Signature::from_bytes(signature)
            .map_err(|e| CommonError::InvalidSignature(e.to_string()))?;

        match pub_key.verify(message, &sig) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Sign data
    pub fn sign_data(&self, message: &[u8], private_key: &[u8]) -> Result<String> {
        let secret_key = SecretKey::from_bytes(private_key)
            .map_err(|e| CommonError::InvalidPrivateKey(e.to_string()))?;
        
        let signature = secret_key.sign(message);
        Ok(hex::encode(signature.to_bytes()))
    }

    /// Generate address from public key
    pub fn generate_address(&self, public_key: &[u8]) -> Result<Address> {
        let hash = self.calculate_hash(public_key);
        let address = format!("memechain1{}", &hash[..32]);
        Ok(Address::new(address))
    }

    /// Encrypt data (placeholder)
    pub fn encrypt_data(&self, _data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement proper encryption
        Err(CommonError::EncryptionFailed("Encryption not implemented".to_string()))
    }

    /// Decrypt data (placeholder)
    pub fn decrypt_data(&self, _data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement proper decryption
        Err(CommonError::DecryptionFailed("Decryption not implemented".to_string()))
    }

    /// Validate amount
    pub fn validate_amount(&self, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(CommonError::InvalidAmount("Amount cannot be zero".to_string()));
        }
        Ok(())
    }

    /// Format amount with decimals
    pub fn format_amount(&self, amount: u64, decimals: u8) -> String {
        let divisor = 10_u64.pow(decimals as u32);
        let whole = amount / divisor;
        let fraction = amount % divisor;
        
        if fraction == 0 {
            whole.to_string()
        } else {
            format!("{}.{:0width$}", whole, fraction, width = decimals as usize)
        }
    }

    /// Parse amount from string
    pub fn parse_amount(&self, amount_str: &str, decimals: u8) -> Result<u64> {
        let parts: Vec<&str> = amount_str.split('.').collect();
        
        match parts.len() {
            1 => {
                let whole = parts[0].parse::<u64>()
                    .map_err(|_| CommonError::InvalidAmount("Invalid whole number".to_string()))?;
                Ok(whole * 10_u64.pow(decimals as u32))
            }
            2 => {
                let whole = parts[0].parse::<u64>()
                    .map_err(|_| CommonError::InvalidAmount("Invalid whole number".to_string()))?;
                let fraction_str = parts[1];
                
                if fraction_str.len() > decimals as usize {
                    return Err(CommonError::InvalidAmount("Too many decimal places".to_string()));
                }
                
                let fraction = format!("{:0<width$}", fraction_str, width = decimals as usize)
                    .parse::<u64>()
                    .map_err(|_| CommonError::InvalidAmount("Invalid fraction".to_string()))?;
                
                Ok(whole * 10_u64.pow(decimals as u32) + fraction)
            }
            _ => Err(CommonError::InvalidAmount("Invalid amount format".to_string())),
        }
    }

    /// Get current timestamp
    pub fn get_timestamp(&self) -> i64 {
        chrono::Utc::now().timestamp()
    }

    /// Validate timestamp
    pub fn validate_timestamp(&self, timestamp: i64, max_age: i64) -> Result<()> {
        let current_time = self.get_timestamp();
        if current_time - timestamp > max_age {
            return Err(CommonError::InvalidAmount("Timestamp too old".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageConfig;
    use tempfile::tempdir;

    async fn create_test_storage() -> Storage {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test_common_db");
        
        let config = StorageConfig {
            db_path: path.to_str().unwrap().to_string(),
            cache_size: 100,
            enable_compression: false,
        };
        
        Storage::new(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_validate_address() {
        let storage = create_test_storage().await;
        let module = CommonModule::new(storage).await.unwrap();
        
        let valid_address = Address::new("memechain1abcdefghijklmnopqrstuvwxyz123456".to_string());
        assert!(module.validate_address(&valid_address).await.is_ok());
        
        let invalid_address = Address::new("invalid".to_string());
        assert!(module.validate_address(&invalid_address).await.is_err());
    }

    #[test]
    fn test_calculate_hash() {
        let storage = create_test_storage().await;
        let module = CommonModule::new(storage).await.unwrap();
        
        let data = b"Hello, World!";
        let hash = module.calculate_hash(data);
        
        // SHA256 of "Hello, World!" should be consistent
        assert_eq!(hash.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_format_amount() {
        let storage = create_test_storage().await;
        let module = CommonModule::new(storage).await.unwrap();
        
        assert_eq!(module.format_amount(1234567, 6), "1.234567");
        assert_eq!(module.format_amount(1000000, 6), "1");
        assert_eq!(module.format_amount(123456, 6), "0.123456");
    }

    #[test]
    fn test_parse_amount() {
        let storage = create_test_storage().await;
        let module = CommonModule::new(storage).await.unwrap();
        
        assert_eq!(module.parse_amount("1.234567", 6).unwrap(), 1234567);
        assert_eq!(module.parse_amount("1", 6).unwrap(), 1000000);
        assert_eq!(module.parse_amount("0.123456", 6).unwrap(), 123456);
        
        assert!(module.parse_amount("1.2345678", 6).is_err()); // Too many decimals
        assert!(module.parse_amount("invalid", 6).is_err()); // Invalid format
    }

    #[tokio::test]
    async fn test_generate_keypair() {
        let storage = create_test_storage().await;
        let module = CommonModule::new(storage).await.unwrap();
        
        let tx = Transaction::new(
            "common".to_string(),
            "generate_keypair".to_string(),
            Address::new("memechain1alice".to_string()),
            None,
            serde_json::json!({}),
        );
        
        let result = module.process_transaction(tx).await.unwrap();
        assert!(result.success);
        
        let data = result.data.unwrap();
        assert!(data["public_key"].as_str().is_some());
        assert!(data["private_key"].as_str().is_some());
    }
} 