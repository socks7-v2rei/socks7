//! Socks7 / V2rei CLI
//! Dual-branded ultra-lightweight proxy protocol (Version 0x07)

use std::net::SocketAddr;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use socks7::server::{Server, ServerConfig};
use socks7::auth::AuthConfig;
use socks7::bridge::{Bridge, BridgeConfig};

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

        /// Allow no authentication
        #[arg(long, default_value_t = true)]
        no_auth: bool,

        /// Username
        #[arg(long, env = "SOCKS7_USERNAME")]
        username: Option<String>,

        /// Password
        #[arg(long, env = "SOCKS7_PASSWORD")]
        password: Option<String>,
    },

    /// Run local SOCKS5 Bridge (makes Socks7 usable by normal apps)
    Bridge {
        /// Local SOCKS5 listen address
        #[arg(short, long, default_value = "127.0.0.1:1080")]
        listen: SocketAddr,

        /// Upstream Socks7 server address
        #[arg(short, long)]
        upstream: SocketAddr,
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

        Commands::Bridge { listen, upstream } => {
            let config = BridgeConfig {
                listen_addr: listen,
                upstream,
            };
            let bridge = Bridge::new(config);
            bridge.run().await?;
        }
    }

    Ok(())
}
