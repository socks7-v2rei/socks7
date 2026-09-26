//! Socks7 / V2rei Server
//! Now with full standard SOCKS5 support (usable from mobile)

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tracing::{info, warn, error, debug};

use crate::auth::AuthConfig;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct ServerConfig {
    pub listen_addr: SocketAddr,
    pub auth: AuthConfig,
    pub max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:1080".parse().unwrap(),
            auth: AuthConfig::default(),
            max_connections: 10_000,
        }
    }
}

pub struct Server {
    config: Arc<ServerConfig>,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        info!(
            "Socks5 / Socks7 server listening on {} (Auth={}, MaxConn={})",
            self.config.listen_addr,
            !self.config.auth.allow_no_auth,
            self.config.max_connections
        );

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let cfg = Arc::clone(&self.config);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, peer, cfg).await {
                            debug!("Connection from {} ended: {}", peer, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    peer: SocketAddr,
    config: Arc<ServerConfig>,
) -> anyhow::Result<()> {
    stream.set_nodelay(true)?;

    let mut buf = [0u8; 257];
    let n = match timeout(HANDSHAKE_TIMEOUT, stream.read(&mut buf)).await {
        Ok(Ok(0)) => return Ok(()),
        Ok(Ok(n)) => n,
        Ok(Err(e)) => return Err(e.into()),
        Err(_) => {
            warn!("Handshake timeout from {}", peer);
            return Ok(());
        }
    };

    if n < 2 {
        return Ok(());
    }

    if buf[0] != 0x05 {
        debug!("Non-SOCKS5 connection from {}, closing", peer);
        return Ok(());
    }

    let nmethods = buf[1] as usize;
    if n < 2 + nmethods {
        return Ok(());
    }

    let mut support_noauth = false;
    let mut support_userpass = false;
    for i in 0..nmethods {
        match buf[2 + i] {
            0x00 => support_noauth = true,
            0x02 => support_userpass = true,
            _ => {}
        }
    }

    let use_auth = !config.auth.allow_no_auth;

    if use_auth {
        if support_userpass {
            stream.write_all(&[0x05, 0x02]).await?;
            
            let n = stream.read(&mut buf).await?;
            if n < 3 || buf[0] != 0x01 {
                stream.write_all(&[0x01, 0x01]).await?;
                return Ok(());
            }
            let ulen = buf[1] as usize;
            if n < 2 + ulen + 1 {
                stream.write_all(&[0x01, 0x01]).await?;
                return Ok(());
            }
            let username = String::from_utf8_lossy(&buf[2..2+ulen]).to_string();
            let plen = buf[2+ulen] as usize;
            if n < 3 + ulen + plen {
                stream.write_all(&[0x01, 0x01]).await?;
                return Ok(());
            }
            let password = String::from_utf8_lossy(&buf[3+ulen..3+ulen+plen]).to_string();

            if !config.auth.users.verify(&username, &password).await {
                stream.write_all(&[0x01, 0x01]).await?;
                debug!("Auth failed for user {} from {}", username, peer);
                return Ok(());
            }
            stream.write_all(&[0x01, 0x00]).await?;
            debug!("Auth success for user {} from {}", username, peer);
        } else {
            stream.write_all(&[0x05, 0xFF]).await?;
            return Ok(());
        }
    } else {
        if support_noauth {
            stream.write_all(&[0x05, 0x00]).await?;
        } else {
            stream.write_all(&[0x05, 0xFF]).await?;
            return Ok(());
        }
    }

    let n = match timeout(HANDSHAKE_TIMEOUT, stream.read(&mut buf)).await {
        Ok(Ok(n)) => n,
        _ => return Ok(()),
    };

    if n < 7 || buf[0] != 0x05 {
        return Ok(());
    }

    let cmd = buf[1];
    let atyp = buf[3];

    if cmd != 0x01 {
        let reply = [0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
        stream.write_all(&reply).await?;
        return Ok(());
    }

    let (target_host, target_port) = match atyp {
        0x01 => {
            if n < 10 { return Ok(()); }
            let ip = format!("{}.{}.{}.{}", buf[4], buf[5], buf[6], buf[7]);
            let port = u16::from_be_bytes([buf[8], buf[9]]);
            (ip, port)
        }
        0x03 => {
            let len = buf[4] as usize;
            if n < 5 + len + 2 { return Ok(()); }
            let domain = String::from_utf8_lossy(&buf[5..5+len]).to_string();
            let port = u16::from_be_bytes([buf[5+len], buf[6+len]]);
            (domain, port)
        }
        0x04 => {
            if n < 22 { return Ok(()); }
            let mut ip_bytes = [0u8; 16];
            ip_bytes.copy_from_slice(&buf[4..20]);
            let ip = std::net::Ipv6Addr::from(ip_bytes).to_string();
            let port = u16::from_be_bytes([buf[20], buf[21]]);
            (ip, port)
        }
        _ => {
            let reply = [0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
            stream.write_all(&reply).await?;
            return Ok(());
        }
    };

    debug!("CONNECT from {} → {}:{}", peer, target_host, target_port);

    let target = format!("{}:{}", target_host, target_port);

    let target_stream = match timeout(CONNECT_TIMEOUT, TcpStream::connect(&target)).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            debug!("Connect to {} failed: {}", target, e);
            let code = match e.kind() {
                std::io::ErrorKind::ConnectionRefused => 0x05,
                std::io::ErrorKind::TimedOut => 0x04,
                _ => 0x01,
            };
            let reply = [0x05, code, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
            stream.write_all(&reply).await?;
            return Ok(());
        }
        Err(_) => {
            let reply = [0x05, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
            stream.write_all(&reply).await?;
            return Ok(());
        }
    };

    let reply = [0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
    stream.write_all(&reply).await?;

    let mut target = target_stream;
    let _ = tokio::io::copy_bidirectional(&mut stream, &mut target).await;

    Ok(())
}
