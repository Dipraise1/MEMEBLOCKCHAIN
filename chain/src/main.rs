use clap::{Parser, Subcommand};
use memechain::{MemeChain, MemeChainError};
use tracing::{error, info, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "memechain")]
#[command(about = "High-performance Layer 1 blockchain for NFTs and meme tokens")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the blockchain node
    Start {
        /// Configuration file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
    },
    /// Initialize a new blockchain
    Init {
        /// Chain ID
        #[arg(short, long, default_value = "memechain-dev")]
        chain_id: String,
        /// Validator moniker
        #[arg(short, long)]
        moniker: String,
    },
    /// Create a new meme token
    CreateToken {
        /// Token name
        #[arg(short, long)]
        name: String,
        /// Token symbol
        #[arg(short, long)]
        symbol: String,
        /// Total supply
        #[arg(short, long)]
        supply: u64,
        /// Creator address
        #[arg(short, long)]
        creator: String,
    },
    /// Mint an NFT
    MintNft {
        /// Collection name
        #[arg(short, long)]
        collection: String,
        /// NFT name
        #[arg(short, long)]
        name: String,
        /// Owner address
        #[arg(short, long)]
        owner: String,
    },
    /// Transfer tokens
    Transfer {
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount
        #[arg(short, long)]
        amount: u64,
        /// Token symbol
        #[arg(short, long)]
        token: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), MemeChainError> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config } => {
            info!("Starting MemeChain node with config: {}", config);
            
            let config = memechain::config::Config::from_file(&config)?;
            let chain = MemeChain::new(config).await?;
            
            chain.start().await?;
            
            // Keep the main thread alive
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl+c");
            
            info!("Shutting down MemeChain node...");
        }
        
        Commands::Init { chain_id, moniker } => {
            info!("Initializing new blockchain: {} with moniker: {}", chain_id, moniker);
            
            // Create genesis configuration
            let genesis = memechain::config::GenesisConfig::new(chain_id, moniker);
            genesis.save("genesis.json")?;
            
            // Create default config
            let config = memechain::config::Config::default();
            config.save("config.toml")?;
            
            info!("Blockchain initialized successfully!");
            info!("Genesis file: genesis.json");
            info!("Config file: config.toml");
        }
        
        Commands::CreateToken { name, symbol, supply, creator } => {
            info!("Creating token: {} ({}) with supply: {}", name, symbol, supply);
            
            // TODO: Implement token creation logic
            // This would typically involve:
            // 1. Validating the request
            // 2. Creating the token in the meme module
            // 3. Broadcasting the transaction
            
            println!("Token creation request submitted: {} ({})", name, symbol);
        }
        
        Commands::MintNft { collection, name, owner } => {
            info!("Minting NFT: {} in collection: {} for owner: {}", name, collection, owner);
            
            // TODO: Implement NFT minting logic
            // This would typically involve:
            // 1. Validating the request
            // 2. Minting the NFT in the NFT module
            // 3. Broadcasting the transaction
            
            println!("NFT minting request submitted: {} in {}", name, collection);
        }
        
        Commands::Transfer { to, amount, token } => {
            info!("Transferring {} {} to {}", amount, token, to);
            
            // TODO: Implement transfer logic
            // This would typically involve:
            // 1. Validating the request
            // 2. Executing the transfer
            // 3. Broadcasting the transaction
            
            println!("Transfer request submitted: {} {} to {}", amount, token, to);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let args = vec!["memechain", "start", "--config", "test.toml"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
    }
} 