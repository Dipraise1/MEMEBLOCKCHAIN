pub mod app;
pub mod cmd;
pub mod config;
pub mod error;
pub mod modules;
pub mod storage;
pub mod types;

pub use app::MemeChainApp;
pub use error::MemeChainError;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Main blockchain application
pub struct MemeChain {
    app: Arc<RwLock<MemeChainApp>>,
    config: config::Config,
}

impl MemeChain {
    /// Create a new MemeChain instance
    pub async fn new(config: config::Config) -> Result<Self, MemeChainError> {
        info!("Initializing MemeChain with config: {:?}", config);
        
        let app = Arc::new(RwLock::new(MemeChainApp::new(config.clone()).await?));
        
        Ok(Self { app, config })
    }

    /// Start the blockchain node
    pub async fn start(&self) -> Result<(), MemeChainError> {
        info!("Starting MemeChain node...");
        
        // Initialize storage
        self.app.read().await.initialize_storage().await?;
        
        // Start consensus engine
        self.start_consensus().await?;
        
        // Start API server
        self.start_api_server().await?;
        
        info!("MemeChain node started successfully");
        Ok(())
    }

    /// Start the consensus engine
    async fn start_consensus(&self) -> Result<(), MemeChainError> {
        info!("Starting consensus engine...");
        
        // TODO: Implement Tendermint consensus
        // This would typically involve:
        // 1. Starting Tendermint Core
        // 2. Connecting to the ABCI application
        // 3. Starting block production
        
        Ok(())
    }

    /// Start the API server
    async fn start_api_server(&self) -> Result<(), MemeChainError> {
        info!("Starting API server on port {}", self.config.api_port);
        
        let app = self.app.clone();
        let port = self.config.api_port;
        
        tokio::spawn(async move {
            if let Err(e) = crate::app::start_api_server(app, port).await {
                warn!("API server error: {}", e);
            }
        });
        
        Ok(())
    }

    /// Get the application instance
    pub fn app(&self) -> Arc<RwLock<MemeChainApp>> {
        self.app.clone()
    }

    /// Get the configuration
    pub fn config(&self) -> &config::Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memechain_creation() {
        let config = config::Config::default();
        let chain = MemeChain::new(config).await;
        assert!(chain.is_ok());
    }
} 