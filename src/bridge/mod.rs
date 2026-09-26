//! SOCKS5 → Socks7 Bridge
//! Makes Socks7 usable by any standard SOCKS5 client (browser, curl, Telegram, etc.)

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn, debug, error};

use crate::client::Client as Socks7Client;
use crate::protocol::{Address, SocksAddr};

pub struct BridgeConfig {
    pub listen_addr: SocketAddr,
    pub upstream: SocketAddr, // Socks7 server address
}

pub struct Bridge {
    config: Arc<BridgeConfig>,
}

impl Bridge {
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        info!(
            "SOCKS5 Bridge listening on {} → upstream Socks7 {}",
            self.config.listen_addr, self.config.upstream
        );

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let cfg = Arc::clone(&self.config);
                    tokio::spawn(async move {
                        if let Err(e) = handle_socks5_client(stream, peer, cfg).await {
                            debug!("Bridge client {} error: {}", peer, e);
                        }
                    });
                }
                Err(e) => error!("Accept error: {}", e),
            }
        }
    }
}

async fn handle_socks5_client(
    mut stream: TcpStream,
    peer: SocketAddr,
    config: Arc<BridgeConfig>,
) -> anyhow::Result<()> {
    stream.set_nodelay(true)?;

    // --- SOCKS5 Greeting ---
    let mut buf = [0u8; 258];
    let n = stream.read(&mut buf).await?;
    if n < 2 || buf[0] != 0x05 {
        return Ok(()); // not socks5
    }

    let nmethods = buf[1] as usize;
    if n < 2 + nmethods {
        return Ok(());
    }

    // We only support NoAuth for the local bridge (simple & practical)
    let mut has_noauth = false;
    for i in 0..nmethods {
        if buf[2 + i] == 0x00 {
            has_noauth = true;
            break;
        }
    }

    if has_noauth {
        stream.write_all(&[0x05, 0x00]).await?; // NoAuth
    } else {
        stream.write_all(&[0x05, 0xFF]).await?; // No acceptable method
        return Ok(());
    }

    // --- SOCKS5 Request ---
    let n = stream.read(&mut buf).await?;
    if n < 7 || buf[0] != 0x05 {
        return Ok(());
    }

    let cmd = buf[1];
    let atyp = buf[3];

    if cmd != 0x01 {
        // Only CONNECT supported in bridge for now
        let mut reply = vec![0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
        stream.write_all(&reply).await?;
        return Ok(());
    }

    // Parse target address
    let (target_host, target_port, _addr_len) = match atyp {
        0x01 => {
            if n < 10 {
                return Ok(());
            }
            let ip = format!("{}.{}.{}.{}", buf[4], buf[5], buf[6], buf[7]);
            let port = u16::from_be_bytes([buf[8], buf[9]]);
            (ip, port, 4)
        }
        0x03 => {
            let len = buf[4] as usize;
            if n < 5 + len + 2 {
                return Ok(());
            }
            let domain = String::from_utf8_lossy(&buf[5..5 + len]).to_string();
            let port = u16::from_be_bytes([buf[5 + len], buf[6 + len]]);
            (domain, port, len)
        }
        0x04 => {
            if n < 22 {
                return Ok(());
            }
            let mut ip_bytes = [0u8; 16];
            ip_bytes.copy_from_slice(&buf[4..20]);
            let ip = std::net::Ipv6Addr::from(ip_bytes).to_string();
            let port = u16::from_be_bytes([buf[20], buf[21]]);
            (ip, port, 16)
        }
        _ => {
            let mut reply = vec![0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
            stream.write_all(&reply).await?;
            return Ok(());
        }
    };

    debug!("SOCKS5 request from {} → {}:{}", peer, target_host, target_port);

    // Connect through real Socks7 upstream
    let socks7_client = Socks7Client::new(config.upstream);
    let target_addr = if let Ok(ip) = target_host.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(v4) => SocksAddr::new(Address::Ipv4(v4), target_port),
            std::net::IpAddr::V6(v6) => SocksAddr::new(Address::Ipv6(v6), target_port),
        }
    } else {
        SocksAddr::new(Address::Domain(target_host.clone()), target_port)
    };

    match socks7_client.connect(target_addr).await {
        Ok(mut upstream) => {
            // Success reply to SOCKS5 client
            let mut reply = vec![0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
            stream.write_all(&reply).await?;

            // Bidirectional copy
            let _ = tokio::io::copy_bidirectional(&mut stream, &mut upstream).await;
        }
        Err(e) => {
            debug!("Upstream Socks7 connect failed: {}", e);
            let mut reply = vec![0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
            stream.write_all(&reply).await?;
        }
    }

    Ok(())
}
