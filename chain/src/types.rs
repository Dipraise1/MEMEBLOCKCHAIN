use serde::{Deserialize, Serialize};
use std::fmt;

/// Blockchain address type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(String);

impl Address {
    /// Create a new address
    pub fn new(addr: String) -> Self {
        Self(addr)
    }

    /// Get the address string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate address format
    pub fn is_valid(&self) -> bool {
        // Basic validation - should start with memechain1 and be 39 characters
        self.0.starts_with("memechain1") && self.0.len() == 39
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Address {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Address {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Transaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Module that handles this transaction
    pub module: String,
    /// Action to perform
    pub action: String,
    /// Sender address
    pub from: Address,
    /// Recipient address (optional)
    pub to: Option<Address>,
    /// Transaction data
    pub data: serde_json::Value,
    /// Transaction timestamp
    pub timestamp: i64,
    /// Transaction signature
    pub signature: String,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        module: String,
        action: String,
        from: Address,
        to: Option<Address>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            module,
            action,
            from,
            to,
            data,
            timestamp: chrono::Utc::now().timestamp(),
            signature: String::new(),
        }
    }

    /// Sign the transaction
    pub fn sign(&mut self, private_key: &str) -> crate::error::Result<()> {
        // TODO: Implement proper signature generation
        self.signature = format!("signed_{}", private_key);
        Ok(())
    }

    /// Get transaction hash
    pub fn hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", self).as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    /// Whether the transaction was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Result data
    pub data: Option<serde_json::Value>,
}

impl TransactionResult {
    /// Create a successful result
    pub fn success(data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            error: None,
            data,
        }
    }

    /// Create a failed result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            error: Some(error),
            data: None,
        }
    }
}

/// Block type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block height
    pub height: u64,
    /// Block timestamp
    pub timestamp: i64,
    /// Transactions in this block
    pub transactions: Vec<Transaction>,
    /// Transaction results
    pub results: Vec<TransactionResult>,
    /// Block hash
    pub hash: String,
    /// Previous block hash
    pub previous_hash: String,
}

impl Block {
    /// Create a new block
    pub fn new(
        height: u64,
        transactions: Vec<Transaction>,
        results: Vec<TransactionResult>,
        previous_hash: String,
    ) -> Self {
        Self {
            height,
            timestamp: chrono::Utc::now().timestamp(),
            transactions,
            results,
            hash: String::new(),
            previous_hash,
        }
    }

    /// Calculate block hash
    pub fn calculate_hash(&mut self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}", self.height, self.timestamp, self.previous_hash).as_bytes());
        self.hash = format!("{:x}", hasher.finalize());
        self.hash.clone()
    }
}

/// NFT Collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Collection ID
    pub id: String,
    /// Collection name
    pub name: String,
    /// Creator address
    pub creator: Address,
    /// Description
    pub description: String,
    /// Created timestamp
    pub created_at: i64,
    /// Updated timestamp
    pub updated_at: i64,
}

impl Collection {
    /// Create a new collection
    pub fn new(id: String, name: String, creator: Address, description: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            name,
            creator,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

/// NFT Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nft {
    /// NFT ID
    pub id: String,
    /// Collection ID
    pub collection_id: String,
    /// NFT name
    pub name: String,
    /// Owner address
    pub owner: Address,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Created timestamp
    pub created_at: i64,
    /// Updated timestamp
    pub updated_at: i64,
}

impl Nft {
    /// Create a new NFT
    pub fn new(
        id: String,
        collection_id: String,
        name: String,
        owner: Address,
        metadata: serde_json::Value,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            collection_id,
            name,
            owner,
            metadata,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Meme Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Total supply
    pub total_supply: u64,
    /// Creator address
    pub creator: Address,
    /// Anti-rug settings
    pub anti_rug: AntiRugSettings,
    /// Created timestamp
    pub created_at: i64,
    /// Updated timestamp
    pub updated_at: i64,
}

impl Token {
    /// Create a new token
    pub fn new(
        symbol: String,
        name: String,
        total_supply: u64,
        creator: Address,
        anti_rug: AntiRugSettings,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            symbol,
            name,
            total_supply,
            creator,
            anti_rug,
            created_at: now,
            updated_at: now,
        }
    }
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
    /// Lock start block
    pub lock_start_block: Option<u64>,
}

impl AntiRugSettings {
    /// Create default anti-rug settings
    pub fn default() -> Self {
        Self {
            max_wallet_percentage: 5,
            buy_tax_percentage: 2,
            sell_tax_percentage: 3,
            liquidity_locked_percentage: 80,
            lock_duration_blocks: 1000,
            lock_start_block: None,
        }
    }

    /// Check if liquidity is locked
    pub fn is_liquidity_locked(&self, current_block: u64) -> bool {
        if let Some(start_block) = self.lock_start_block {
            current_block < start_block + self.lock_duration_blocks
        } else {
            false
        }
    }

    /// Calculate buy tax
    pub fn calculate_buy_tax(&self, amount: u64) -> u64 {
        (amount * self.buy_tax_percentage as u64) / 100
    }

    /// Calculate sell tax
    pub fn calculate_sell_tax(&self, amount: u64) -> u64 {
        (amount * self.sell_tax_percentage as u64) / 100
    }

    /// Check if transfer exceeds max wallet limit
    pub fn exceeds_max_wallet(&self, current_balance: u64, transfer_amount: u64, total_supply: u64) -> bool {
        let max_wallet_amount = (total_supply * self.max_wallet_percentage as u64) / 100;
        current_balance + transfer_amount > max_wallet_amount
    }
}

/// Account balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Account address
    pub address: Address,
    /// Token symbol
    pub token: String,
    /// Balance amount
    pub amount: u64,
    /// Updated timestamp
    pub updated_at: i64,
}

impl Balance {
    /// Create a new balance
    pub fn new(address: Address, token: String, amount: u64) -> Self {
        Self {
            address,
            token,
            amount,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Add to balance
    pub fn add(&mut self, amount: u64) {
        self.amount += amount;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Subtract from balance
    pub fn subtract(&mut self, amount: u64) -> crate::error::Result<()> {
        if self.amount < amount {
            return Err(crate::error::MemeChainError::InsufficientBalance(
                format!("Insufficient balance: {} < {}", self.amount, amount)
            ));
        }
        self.amount -= amount;
        self.updated_at = chrono::Utc::now().timestamp();
        Ok(())
    }
}

/// Network peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// Peer ID
    pub id: String,
    /// Peer address
    pub address: String,
    /// Peer port
    pub port: u16,
    /// Is persistent peer
    pub persistent: bool,
    /// Last seen timestamp
    pub last_seen: i64,
}

impl Peer {
    /// Create a new peer
    pub fn new(id: String, address: String, port: u16, persistent: bool) -> Self {
        Self {
            id,
            address,
            port,
            persistent,
            last_seen: chrono::Utc::now().timestamp(),
        }
    }

    /// Update last seen
    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now().timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_validation() {
        let valid_addr = Address::new("memechain1abcdefghijklmnopqrstuvwxyz123456".to_string());
        assert!(valid_addr.is_valid());

        let invalid_addr = Address::new("invalid".to_string());
        assert!(!invalid_addr.is_valid());
    }

    #[test]
    fn test_transaction_creation() {
        let from = Address::new("memechain1alice".to_string());
        let to = Address::new("memechain1bob".to_string());
        let data = serde_json::json!({"amount": 100});

        let tx = Transaction::new(
            "meme".to_string(),
            "transfer".to_string(),
            from,
            Some(to),
            data,
        );

        assert_eq!(tx.module, "meme");
        assert_eq!(tx.action, "transfer");
    }

    #[test]
    fn test_anti_rug_settings() {
        let settings = AntiRugSettings::default();
        assert_eq!(settings.max_wallet_percentage, 5);
        assert_eq!(settings.calculate_buy_tax(1000), 20);
        assert_eq!(settings.calculate_sell_tax(1000), 30);
    }

    #[test]
    fn test_balance_operations() {
        let mut balance = Balance::new(
            Address::new("memechain1alice".to_string()),
            "MEME".to_string(),
            1000,
        );

        balance.add(500);
        assert_eq!(balance.amount, 1500);

        balance.subtract(300).unwrap();
        assert_eq!(balance.amount, 1200);

        let result = balance.subtract(2000);
        assert!(result.is_err());
    }
} 