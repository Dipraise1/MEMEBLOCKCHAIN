use crate::config::Config;
use crate::error::{MemeChainError, Result};
use crate::modules::{nft::NftModule, meme::MemeModule, common::CommonModule};
use crate::storage::Storage;
use crate::types::{Address, Block, Transaction, TransactionResult};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Main blockchain application
pub struct MemeChainApp {
    /// Application configuration
    config: Config,
    /// Storage layer
    storage: Storage,
    /// NFT module
    nft_module: NftModule,
    /// Meme token module
    meme_module: MemeModule,
    /// Common utilities module
    common_module: CommonModule,
    /// Current block height
    block_height: u64,
    /// Transaction pool
    tx_pool: Arc<RwLock<Vec<Transaction>>>,
    /// Rate limiting
    rate_limiter: Arc<RwLock<HashMap<String, u64>>>,
}

impl MemeChainApp {
    /// Create a new MemeChain application
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing MemeChain application...");

        // Initialize storage
        let storage = Storage::new(&config.storage).await?;

        // Initialize modules
        let nft_module = NftModule::new(storage.clone()).await?;
        let meme_module = MemeModule::new(storage.clone()).await?;
        let common_module = CommonModule::new(storage.clone()).await?;

        // Initialize transaction pool
        let tx_pool = Arc::new(RwLock::new(Vec::new()));
        let rate_limiter = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            config,
            storage,
            nft_module,
            meme_module,
            common_module,
            block_height: 0,
            tx_pool,
            rate_limiter,
        })
    }

    /// Initialize storage
    pub async fn initialize_storage(&self) -> Result<()> {
        info!("Initializing storage...");
        self.storage.initialize().await?;
        Ok(())
    }

    /// Process a transaction
    pub async fn process_transaction(&mut self, tx: Transaction) -> Result<TransactionResult> {
        debug!("Processing transaction: {:?}", tx);

        // Validate transaction
        self.validate_transaction(&tx).await?;

        // Apply rate limiting
        self.check_rate_limit(&tx.from).await?;

        // Route transaction to appropriate module
        let result = match tx.module {
            "nft" => self.nft_module.process_transaction(tx).await?,
            "meme" => self.meme_module.process_transaction(tx).await?,
            "common" => self.common_module.process_transaction(tx).await?,
            _ => return Err(MemeChainError::Validation(format!("Unknown module: {}", tx.module))),
        };

        // Update rate limiter
        self.update_rate_limiter(&tx.from).await?;

        Ok(result)
    }

    /// Validate a transaction
    async fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // Check if transaction is not expired
        if tx.timestamp + self.config.chain.block_time * 10 < chrono::Utc::now().timestamp() {
            return Err(MemeChainError::Validation("Transaction expired".to_string()));
        }

        // Validate signature
        self.common_module.validate_signature(tx).await?;

        // Validate address format
        self.common_module.validate_address(&tx.from).await?;

        Ok(())
    }

    /// Check rate limiting
    async fn check_rate_limit(&self, address: &Address) -> Result<()> {
        let mut rate_limiter = self.rate_limiter.write().await;
        let current_time = chrono::Utc::now().timestamp() as u64;
        let window = 60; // 1 minute window

        if let Some(last_time) = rate_limiter.get(&address.to_string()) {
            if current_time - last_time < window {
                return Err(MemeChainError::RateLimitExceeded);
            }
        }

        rate_limiter.insert(address.to_string(), current_time);
        Ok(())
    }

    /// Update rate limiter
    async fn update_rate_limiter(&self, address: &Address) -> Result<()> {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.insert(address.to_string(), chrono::Utc::now().timestamp() as u64);
        Ok(())
    }

    /// Create a new block
    pub async fn create_block(&mut self) -> Result<Block> {
        info!("Creating new block at height {}", self.block_height + 1);

        // Get transactions from pool
        let mut tx_pool = self.tx_pool.write().await;
        let transactions = tx_pool.drain(..).collect::<Vec<_>>();

        // Process transactions
        let mut results = Vec::new();
        for tx in transactions {
            match self.process_transaction(tx.clone()).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Transaction failed: {}", e);
                    results.push(TransactionResult {
                        success: false,
                        error: Some(e.to_string()),
                        data: None,
                    });
                }
            }
        }

        // Create block
        let block = Block {
            height: self.block_height + 1,
            timestamp: chrono::Utc::now().timestamp(),
            transactions,
            results,
            hash: "".to_string(), // Will be calculated
            previous_hash: "".to_string(), // Will be set
        };

        // Update block height
        self.block_height += 1;

        // Store block
        self.storage.store_block(&block).await?;

        info!("Block {} created with {} transactions", block.height, block.transactions.len());
        Ok(block)
    }

    /// Get current block height
    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    /// Get transaction pool size
    pub async fn tx_pool_size(&self) -> usize {
        self.tx_pool.read().await.len()
    }

    /// Get NFT module
    pub fn nft_module(&self) -> &NftModule {
        &self.nft_module
    }

    /// Get meme module
    pub fn meme_module(&self) -> &MemeModule {
        &self.meme_module
    }

    /// Get common module
    pub fn common_module(&self) -> &CommonModule {
        &self.common_module
    }

    /// Get storage
    pub fn storage(&self) -> &Storage {
        &self.storage
    }
}

/// API request types
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub symbol: String,
    pub supply: u64,
    pub creator: String,
    pub anti_rug: Option<AntiRugSettings>,
}

#[derive(Debug, Deserialize)]
pub struct MintNftRequest {
    pub collection: String,
    pub name: String,
    pub owner: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub to: String,
    pub amount: u64,
    pub token: String,
    pub from: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AntiRugSettings {
    pub max_wallet_percentage: u8,
    pub buy_tax_percentage: u8,
    pub sell_tax_percentage: u8,
    pub liquidity_locked_percentage: u8,
    pub lock_duration_blocks: u64,
}

/// Start the API server
pub async fn start_api_server(app: Arc<RwLock<MemeChainApp>>, port: u16) -> Result<()> {
    info!("Starting API server on port {}", port);

    let app_state = AppState { app };

    let router = Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        .route("/tokens/create", post(create_token))
        .route("/nft/mint", post(mint_nft))
        .route("/transfer", post(transfer))
        .route("/tokens", get(list_tokens))
        .route("/nfts", get(list_nfts))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

/// Application state for API
#[derive(Clone)]
struct AppState {
    app: Arc<RwLock<MemeChainApp>>,
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("OK".to_string()),
        error: None,
    })
}

/// Get blockchain status
async fn get_status(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let app = state.app.read().await;
    let status = serde_json::json!({
        "block_height": app.block_height(),
        "tx_pool_size": app.tx_pool_size().await,
        "chain_id": app.config().chain.chain_id,
    });

    Json(ApiResponse {
        success: true,
        data: Some(status),
        error: None,
    })
}

/// Create a new token
async fn create_token(
    State(state): State<AppState>,
    Json(request): Json<CreateTokenRequest>,
) -> Json<ApiResponse<String>> {
    let mut app = state.app.write().await;
    
    // Create transaction
    let tx = Transaction {
        module: "meme".to_string(),
        action: "create_token".to_string(),
        from: request.creator.clone(),
        to: None,
        data: serde_json::json!({
            "name": request.name,
            "symbol": request.symbol,
            "supply": request.supply,
            "anti_rug": request.anti_rug,
        }),
        timestamp: chrono::Utc::now().timestamp(),
        signature: "".to_string(), // Will be validated
    };

    match app.process_transaction(tx).await {
        Ok(result) => Json(ApiResponse {
            success: result.success,
            data: Some(format!("Token created: {}", request.symbol)),
            error: result.error,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Mint an NFT
async fn mint_nft(
    State(state): State<AppState>,
    Json(request): Json<MintNftRequest>,
) -> Json<ApiResponse<String>> {
    let mut app = state.app.write().await;
    
    // Create transaction
    let tx = Transaction {
        module: "nft".to_string(),
        action: "mint".to_string(),
        from: request.owner.clone(),
        to: None,
        data: serde_json::json!({
            "collection": request.collection,
            "name": request.name,
            "metadata": request.metadata,
        }),
        timestamp: chrono::Utc::now().timestamp(),
        signature: "".to_string(), // Will be validated
    };

    match app.process_transaction(tx).await {
        Ok(result) => Json(ApiResponse {
            success: result.success,
            data: Some(format!("NFT minted: {}", request.name)),
            error: result.error,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Transfer tokens
async fn transfer(
    State(state): State<AppState>,
    Json(request): Json<TransferRequest>,
) -> Json<ApiResponse<String>> {
    let mut app = state.app.write().await;
    
    // Create transaction
    let tx = Transaction {
        module: "meme".to_string(),
        action: "transfer".to_string(),
        from: request.from.clone(),
        to: Some(request.to.clone()),
        data: serde_json::json!({
            "amount": request.amount,
            "token": request.token,
        }),
        timestamp: chrono::Utc::now().timestamp(),
        signature: "".to_string(), // Will be validated
    };

    match app.process_transaction(tx).await {
        Ok(result) => Json(ApiResponse {
            success: result.success,
            data: Some("Transfer completed".to_string()),
            error: result.error,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// List all tokens
async fn list_tokens(State(state): State<AppState>) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let app = state.app.read().await;
    
    match app.meme_module().list_tokens().await {
        Ok(tokens) => Json(ApiResponse {
            success: true,
            data: Some(tokens),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// List all NFTs
async fn list_nfts(State(state): State<AppState>) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let app = state.app.read().await;
    
    match app.nft_module().list_nfts().await {
        Ok(nfts) => Json(ApiResponse {
            success: true,
            data: Some(nfts),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_creation() {
        let config = Config::default();
        let app = MemeChainApp::new(config).await;
        assert!(app.is_ok());
    }

    #[tokio::test]
    async fn test_block_creation() {
        let config = Config::default();
        let mut app = MemeChainApp::new(config).await.unwrap();
        let block = app.create_block().await;
        assert!(block.is_ok());
    }
} 