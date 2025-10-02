use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn};

mod blockchain;
mod streaming;
mod integration;
mod mobile;
mod web_simple;

use crate::integration::SutantraNode;

/// Sutantra: Integrated Layer 1 Streaming Blockchain
#[derive(Parser)]
#[command(name = "sutantra-node")]
#[command(about = "A Layer 1 blockchain with native WebRTC streaming")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a full node (validator + streaming relay)
    Start {
        /// Node port for P2P networking
        #[arg(long, default_value = "30333")]
        port: u16,
        
        /// Enable validator mode
        #[arg(long)]
        validator: bool,
        
        /// Enable streaming relay
        #[arg(long, default_value = "true")]
        streaming: bool,
        
        /// Enable web UI server
        #[arg(long)]
        web_ui: bool,
        
        /// Web UI port
        #[arg(long, default_value = "8080")]
        web_port: u16,
    },
    
    /// Start a light node (mobile-optimized)
    StartLight {
        /// Disable validator functionality
        #[arg(long, default_value = "8080")]
        port: u16,
        
        /// Bootstrap peers
        #[arg(long)]
        bootnodes: Vec<String>,
        
        /// Enable web UI server
        #[arg(long)]
        web_ui: bool,
        
        /// Web UI port
        #[arg(long, default_value = "8080")]
        web_port: u16,
    },
    
    /// Stream management commands
    Stream {
        #[command(subcommand)]
        action: StreamCommands,
    },
    
    /// Show node status
    Status,
}

#[derive(Subcommand)]
enum StreamCommands {
    /// Create a new stream
    Create {
        /// Stream title
        title: String,
        
        /// Stream description
        #[arg(long)]
        description: Option<String>,
        
        /// Price per minute in STREAM tokens
        #[arg(long, default_value = "10")]
        price: u64,
    },
    
    /// List available streams
    List,
    
    /// Connect to a stream as viewer
    Watch {
        /// Stream ID to watch
        stream_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { port, validator, streaming, web_ui, web_port } => {
            info!("🚀 Starting Sutantra full node on port {}", port);
            info!("📡 Validator mode: {}", validator);
            info!("🎥 Streaming relay: {}", streaming);
            info!("🌐 Web UI: {} (port: {})", web_ui, web_port);
            
            let node = SutantraNode::new(port, validator, streaming).await?;
            
            if web_ui {
                // Start simple web server in background
                let event_sender = node.get_event_sender();
                let streaming_sender = node.get_streaming_sender();
                let web_server = crate::web_simple::SimpleWebServer::new(web_port, event_sender, streaming_sender);
                
                tokio::spawn(async move {
                    if let Err(e) = web_server.start().await {
                        warn!("Web server error: {}", e);
                    }
                });
            }
            
            node.run().await?;
        }
        
        Commands::StartLight { port, bootnodes, web_ui, web_port } => {
            info!("📱 Starting Sutantra light node on port {}", port);
            info!("🔗 Bootstrap nodes: {:?}", bootnodes);
            info!("🌐 Web UI: {} (port: {})", web_ui, web_port);
            
            let node = SutantraNode::new_light(port, bootnodes).await?;
            
            if web_ui {
                // Start simple web server in background
                let event_sender = node.get_event_sender();
                let streaming_sender = node.get_streaming_sender();
                let web_server = crate::web_simple::SimpleWebServer::new(web_port, event_sender, streaming_sender);
                
                tokio::spawn(async move {
                    if let Err(e) = web_server.start().await {
                        warn!("Web server error: {}", e);
                    }
                });
            }
            
            node.run().await?;
        }
        
        Commands::Stream { action } => {
            handle_stream_command(action).await?;
        }
        
        Commands::Status => {
            show_node_status().await?;
        }
    }

    Ok(())
}

async fn handle_stream_command(action: StreamCommands) -> Result<()> {
    match action {
        StreamCommands::Create { title, description: _, price } => {
            info!("🎬 Creating stream: {}", title);
            info!("💰 Price: {} STREAM/minute", price);
            
            // TODO: Implement stream creation
            warn!("Stream creation not yet implemented");
        }
        
        StreamCommands::List => {
            info!("📺 Available streams:");
            // TODO: List streams from blockchain state
            warn!("Stream listing not yet implemented");
        }
        
        StreamCommands::Watch { stream_id } => {
            info!("👀 Connecting to stream: {}", stream_id);
            // TODO: Connect to stream via WebRTC
            warn!("Stream watching not yet implemented");
        }
    }
    
    Ok(())
}

async fn show_node_status() -> Result<()> {
    info!("📊 Sutantra Node Status");
    info!("🔗 Blockchain: Syncing...");
    info!("🎥 Streaming: Active");
    info!("💰 Balance: 1000 STREAM");
    info!("👥 Connected peers: 5");
    info!("📺 Active streams: 2");
    
    // TODO: Get real status from node
    warn!("Status display not yet implemented");
    
    Ok(())
}