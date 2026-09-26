//! Socks7 - v2rei
//! 
//! Next-generation ultra-lightweight, stable and powerful proxy protocol.
//! Brand: v2rei (v2rei.surf)

use std::net::SocketAddr;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use socks7::server::Server;

#[derive(Parser)]
#[command(name = "socks7")]
#[command(about = "Socks7 - v2rei: Ultra-lightweight next-gen proxy protocol", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run Socks7 - v2rei server
    Server {
        /// Listen address (default: 0.0.0.0:1080)
        #[arg(short, long, default_value = "0.0.0.0:1080")]
        listen: SocketAddr,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { listen } => {
            let server = Server::new(listen);
            server.run().await?;
        }
    }

    Ok(())
}
