//! Socks7 / V2rei CLI
//! Dual-branded ultra-lightweight proxy protocol

use std::net::SocketAddr;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use socks7::server::{Server, ServerConfig};
use socks7::auth::AuthConfig;
use socks7::bridge::{Bridge, BridgeConfig};

#[derive(Parser)]
#[command(name = "socks7")]
#[command(about = "Socks7 / V2rei - Proxy Server & Desktop Client", long_about = None)]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the proxy server
    Server {
        #[arg(short, long, default_value = "0.0.0.0:7777")]
        listen: SocketAddr,

        #[arg(long, default_value_t = true)]
        no_auth: bool,

        #[arg(long, env = "SOCKS7_USERNAME")]
        username: Option<String>,

        #[arg(long, env = "SOCKS7_PASSWORD")]
        password: Option<String>,
    },

    /// Run Desktop Client (local SOCKS5 → remote server)
    Client {
        #[arg(short, long, default_value = "127.0.0.1:1080")]
        listen: SocketAddr,

        #[arg(short, long)]
        upstream: SocketAddr,

        #[arg(long)]
        username: String,

        #[arg(long)]
        password: String,
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
            mut no_auth,
            username,
            password,
        } => {
            let auth = AuthConfig::new();

            if let (Some(u), Some(p)) = (&username, &password) {
                auth.add_user(u.clone(), p.clone()).await;
                no_auth = false;
                tracing::info!("Authentication enabled → user: {}", u);
            } else {
                tracing::info!("Running with NoAuth (open proxy)");
            }

            let auth = auth.with_no_auth(no_auth);

            let config = ServerConfig {
                listen_addr: listen,
                auth,
                max_connections: 10_000,
            };

            let server = Server::new(config);
            server.run().await?;
        }

        Commands::Client {
            listen,
            upstream,
            username,
            password,
        } => {
            let config = BridgeConfig {
                listen_addr: listen,
                upstream,
                username,
                password,
            };
            let client = Bridge::new(config);
            client.run().await?;
        }
    }

    Ok(())
}
