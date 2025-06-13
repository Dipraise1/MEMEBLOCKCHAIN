use crate::error::{MemeChainError, Result, MemeError};
use crate::storage::Storage;
use crate::types::{Address, AntiRugSettings, Balance, Token, Transaction, TransactionResult};
use serde_json::Value;
use tracing::{debug, info, warn};

/// Meme token module for managing tokens with anti-rug features
pub struct MemeModule {
    storage: Storage,
    current_block_height: u64,
}

impl MemeModule {
    /// Create a new meme token module
    pub async fn new(storage: Storage) -> Result<Self> {
        info!("Initializing Meme token module");
        Ok(Self {
            storage,
            current_block_height: 0,
        })
    }

    /// Process meme token-related transactions
    pub async fn process_transaction(&self, tx: Transaction) -> Result<TransactionResult> {
        debug!("Processing meme transaction: {} - {}", tx.module, tx.action);

        match tx.action.as_str() {
            "create_token" => self.create_token(tx).await,
            "transfer" => self.transfer_token(tx).await,
            "buy" => self.buy_token(tx).await,
            "sell" => self.sell_token(tx).await,
            "lock_liquidity" => self.lock_liquidity(tx).await,
            _ => Err(MemeError::InvalidSymbol(format!("Unknown action: {}", tx.action))),
        }
    }

    /// Create a new token
    async fn create_token(&self, tx: Transaction) -> Result<TransactionResult> {
        let name = tx.data["name"]
            .as_str()
            .ok_or_else(|| MemeError::InvalidName("Missing token name".to_string()))?;
        
        let symbol = tx.data["symbol"]
            .as_str()
            .ok_or_else(|| MemeError::InvalidSymbol("Missing token symbol".to_string()))?;
        
        let supply = tx.data["supply"]
            .as_u64()
            .ok_or_else(|| MemeError::InvalidSupply("Missing or invalid supply".to_string()))?;
        
        let creator = tx.from;

        // Check if token already exists
        if self.storage.get_token(symbol).await?.is_some() {
            return Err(MemeError::TokenExists(symbol.to_string()));
        }

        // Parse anti-rug settings
        let anti_rug = if let Some(anti_rug_data) = tx.data.get("anti_rug") {
            serde_json::from_value(anti_rug_data.clone())?
        } else {
            AntiRugSettings::default()
        };

        let token = Token::new(
            symbol.to_string(),
            name.to_string(),
            supply,
            creator.clone(),
            anti_rug,
        );

        // Store token
        self.storage.store_token(&token).await?;

        // Create initial balance for creator
        let initial_balance = Balance::new(creator.clone(), symbol.to_string(), supply);
        self.storage.store_balance(&initial_balance).await?;

        info!("Created token: {} ({}) with supply: {} by {}", name, symbol, supply, creator);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "symbol": symbol,
            "name": name,
            "supply": supply,
            "creator": creator.to_string(),
        }))))
    }

    /// Transfer tokens
    async fn transfer_token(&self, tx: Transaction) -> Result<TransactionResult> {
        let token_symbol = tx.data["token"]
            .as_str()
            .ok_or_else(|| MemeError::InvalidSymbol("Missing token symbol".to_string()))?;
        
        let amount = tx.data["amount"]
            .as_u64()
            .ok_or_else(|| MemeError::InvalidAmount("Missing or invalid amount".to_string()))?;
        
        let from_address = tx.from;
        let to_address = tx.to
            .ok_or_else(|| MemeError::TransferFailed("Missing recipient address".to_string()))?;

        // Get sender balance
        let mut from_balance = self.storage.get_balance(&from_address, token_symbol).await?
            .ok_or_else(|| MemeError::InsufficientBalance(format!("No balance for {}", from_address)))?;

        // Check sufficient balance
        if from_balance.amount < amount {
            return Err(MemeError::InsufficientBalance(format!(
                "Insufficient balance: {} < {}", from_balance.amount, amount
            )));
        }

        // Update balances
        from_balance.subtract(amount)?;
        self.storage.store_balance(&from_balance).await?;

        // Get or create recipient balance
        let mut to_balance = self.storage.get_balance(&to_address, token_symbol).await
            .unwrap_or_else(|_| Balance::new(to_address.clone(), token_symbol.to_string(), 0));
        
        to_balance.add(amount);
        self.storage.store_balance(&to_balance).await?;

        info!("Transferred {} {} from {} to {}", amount, token_symbol, from_address, to_address);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "token": token_symbol,
            "amount": amount,
            "from": from_address.to_string(),
            "to": to_address.to_string(),
        }))))
    }

    /// Buy tokens (simulated DEX interaction)
    async fn buy_token(&self, tx: Transaction) -> Result<TransactionResult> {
        let token_symbol = tx.data["token"]
            .as_str()
            .ok_or_else(|| MemeError::InvalidSymbol("Missing token symbol".to_string()))?;
        
        let amount = tx.data["amount"]
            .as_u64()
            .ok_or_else(|| MemeError::InvalidAmount("Missing or invalid amount".to_string()))?;
        
        let buyer = tx.from;

        // Get token
        let token = self.storage.get_token(token_symbol).await?
            .ok_or_else(|| MemeError::TokenNotFound(token_symbol.to_string()))?;

        // Calculate buy tax
        let buy_tax = token.anti_rug.calculate_buy_tax(amount);
        let tokens_received = amount - buy_tax;

        // Get or create buyer balance
        let mut buyer_balance = self.storage.get_balance(&buyer, token_symbol).await
            .unwrap_or_else(|_| Balance::new(buyer.clone(), token_symbol.to_string(), 0));
        
        buyer_balance.add(tokens_received);
        self.storage.store_balance(&buyer_balance).await?;

        info!("Buy: {} received {} {} (tax: {})", buyer, tokens_received, token_symbol, buy_tax);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "token": token_symbol,
            "amount": tokens_received,
            "tax": buy_tax,
            "buyer": buyer.to_string(),
        }))))
    }

    /// Sell tokens (simulated DEX interaction)
    async fn sell_token(&self, tx: Transaction) -> Result<TransactionResult> {
        let token_symbol = tx.data["token"]
            .as_str()
            .ok_or_else(|| MemeError::InvalidSymbol("Missing token symbol".to_string()))?;
        
        let amount = tx.data["amount"]
            .as_u64()
            .ok_or_else(|| MemeError::InvalidAmount("Missing or invalid amount".to_string()))?;
        
        let seller = tx.from;

        // Get token
        let token = self.storage.get_token(token_symbol).await?
            .ok_or_else(|| MemeError::TokenNotFound(token_symbol.to_string()))?;

        // Check if liquidity is locked
        if token.anti_rug.is_liquidity_locked(self.current_block_height) {
            return Err(MemeError::LiquidityNotLocked("Liquidity is currently locked".to_string()));
        }

        // Get seller balance
        let mut seller_balance = self.storage.get_balance(&seller, token_symbol).await?
            .ok_or_else(|| MemeError::InsufficientBalance(format!("No balance for {}", seller)))?;

        // Check sufficient balance
        if seller_balance.amount < amount {
            return Err(MemeError::InsufficientBalance(format!(
                "Insufficient balance: {} < {}", seller_balance.amount, amount
            )));
        }

        // Calculate sell tax
        let sell_tax = token.anti_rug.calculate_sell_tax(amount);
        let tokens_sold = amount - sell_tax;

        // Update seller balance
        seller_balance.subtract(amount)?;
        self.storage.store_balance(&seller_balance).await?;

        info!("Sell: {} sold {} {} (tax: {})", seller, tokens_sold, token_symbol, sell_tax);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "token": token_symbol,
            "amount": tokens_sold,
            "tax": sell_tax,
            "seller": seller.to_string(),
        }))))
    }

    /// Lock liquidity
    async fn lock_liquidity(&self, tx: Transaction) -> Result<TransactionResult> {
        let token_symbol = tx.data["token"]
            .as_str()
            .ok_or_else(|| MemeError::InvalidSymbol("Missing token symbol".to_string()))?;
        
        let lock_duration = tx.data["duration_blocks"]
            .as_u64()
            .ok_or_else(|| MemeError::InvalidAmount("Missing lock duration".to_string()))?;
        
        let locker = tx.from;

        // Get token
        let mut token = self.storage.get_token(token_symbol).await?
            .ok_or_else(|| MemeError::TokenNotFound(token_symbol.to_string()))?;

        // Verify locker is the creator
        if token.creator != locker {
            return Err(MemeError::Unauthorized("Only token creator can lock liquidity".to_string()));
        }

        // Set lock parameters
        token.anti_rug.lock_start_block = Some(self.current_block_height);
        token.anti_rug.lock_duration_blocks = lock_duration;
        token.updated_at = chrono::Utc::now().timestamp();

        // Store updated token
        self.storage.store_token(&token).await?;

        info!("Liquidity locked for token: {} by {} for {} blocks", token_symbol, locker, lock_duration);

        Ok(TransactionResult::success(Some(serde_json::json!({
            "token": token_symbol,
            "lock_start_block": self.current_block_height,
            "lock_duration_blocks": lock_duration,
            "locked_by": locker.to_string(),
        }))))
    }

    /// Update current block height
    pub fn update_block_height(&mut self, height: u64) {
        self.current_block_height = height;
    }

    /// Get token by symbol
    pub async fn get_token(&self, symbol: &str) -> Result<Option<Token>> {
        self.storage.get_token(symbol).await
    }

    /// Get balance
    pub async fn get_balance(&self, address: &Address, token: &str) -> Result<Option<Balance>> {
        self.storage.get_balance(address, token).await
    }

    /// List all tokens
    pub async fn list_tokens(&self) -> Result<Vec<Value>> {
        let tokens = self.storage.get_all_tokens().await?;
        let mut result = Vec::new();
        
        for token in tokens {
            result.push(serde_json::json!({
                "symbol": token.symbol,
                "name": token.name,
                "total_supply": token.total_supply,
                "creator": token.creator.to_string(),
                "anti_rug": token.anti_rug,
                "created_at": token.created_at,
                "updated_at": token.updated_at,
            }));
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
        let path = temp_dir.path().join("test_meme_db");
        
        let config = StorageConfig {
            db_path: path.to_str().unwrap().to_string(),
            db_type: "rocksdb".to_string(),
            cache_size: 100,
            enable_compression: false,
        };
        
        Storage::new(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_token() {
        let storage = create_test_storage().await;
        let module = MemeModule::new(storage).await.unwrap();
        
        let tx = Transaction::new(
            "meme".to_string(),
            "create_token".to_string(),
            Address::new("memechain1alice".to_string()),
            None,
            serde_json::json!({
                "name": "Test Token",
                "symbol": "TEST",
                "supply": 1000000
            }),
        );
        
        let result = module.process_transaction(tx).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_transfer_token() {
        let storage = create_test_storage().await;
        let module = MemeModule::new(storage).await.unwrap();
        
        // First create a token
        let create_tx = Transaction::new(
            "meme".to_string(),
            "create_token".to_string(),
            Address::new("memechain1alice".to_string()),
            None,
            serde_json::json!({
                "name": "Test Token",
                "symbol": "TEST",
                "supply": 1000000
            }),
        );
        
        module.process_transaction(create_tx).await.unwrap();
        
        // Then transfer some tokens
        let transfer_tx = Transaction::new(
            "meme".to_string(),
            "transfer".to_string(),
            Address::new("memechain1alice".to_string()),
            Some(Address::new("memechain1bob".to_string())),
            serde_json::json!({
                "token": "TEST",
                "amount": 1000
            }),
        );
        
        let result = module.process_transaction(transfer_tx).await.unwrap();
        assert!(result.success);
    }
} 