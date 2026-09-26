//! Socks7 / V2rei CLI
//!
//! Dual-branded ultra-lightweight proxy protocol (Version 0x07)

use std::net::SocketAddr;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use socks7::server::{Server, ServerConfig};
use socks7::auth::AuthConfig;

#[derive(Parser)]
#[command(name = "socks7")]
#[command(about = "Socks7 / V2rei: Ultra-lightweight next-gen proxy protocol", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run Socks7 / V2rei server
    Server {
        /// Listen address
        #[arg(short, long, default_value = "0.0.0.0:1080")]
        listen: SocketAddr,

        /// Allow no authentication (default: true)
        #[arg(long, default_value_t = true)]
        no_auth: bool,

        /// Username for authentication (optional)
        #[arg(long)]
        username: Option<String>,

        /// Password for authentication (optional)
        #[arg(long)]
        password: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Server {
            listen,
            no_auth,
            username,
            password,
        } => {
            let auth = AuthConfig::new().with_no_auth(no_auth);

            if let (Some(u), Some(p)) = (username, password) {
                auth.add_user(u, p).await;
                tracing::info!("Authentication enabled with provided credentials");
            }

            let config = ServerConfig {
                listen_addr: listen,
                auth,
                max_connections: 10_000,
            };

            let server = Server::new(config);
            server.run().await?;
        }
    }

    Ok(())
}
