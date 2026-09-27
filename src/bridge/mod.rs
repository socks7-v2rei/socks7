//! Local SOCKS5 Client / Forwarder
//! Runs a local SOCKS5 proxy and forwards traffic to a remote authenticated SOCKS5 server.
//! This makes the remote proxy easy to use on Desktop (set system proxy to 127.0.0.1:1080).

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, debug, error, warn};

pub struct BridgeConfig {
    pub listen_addr: SocketAddr,
    pub upstream: SocketAddr,
    pub username: String,
    pub password: String,
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
        info!("============================================================");
        info!("  Socks7 / V2rei Desktop Client is running");
        info!("============================================================");
        info!("  Local SOCKS5   : {}", self.config.listen_addr);
        info!("  Remote Server  : {}", self.config.upstream);
        info!("  Username       : {}", self.config.username);
        info!("============================================================");
        info!("  Set your system / browser proxy to:");
        info!("  SOCKS5  →  {} ", self.config.listen_addr);
        info!("============================================================");

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let cfg = Arc::clone(&self.config);
                    tokio::spawn(async move {
                        if let Err(e) = handle_local_client(stream, peer, cfg).await {
                            debug!("Client {} error: {}", peer, e);
                        }
                    });
                }
                Err(e) => error!("Accept error: {}", e),
            }
        }
    }
}

async fn handle_local_client(
    mut stream: TcpStream,
    peer: SocketAddr,
    config: Arc<BridgeConfig>,
) -> anyhow::Result<()> {
    stream.set_nodelay(true)?;

    let mut buf = [0u8; 512];
    let n = stream.read(&mut buf).await?;
    if n < 2 || buf[0] != 0x05 {
        return Ok(());
    }

    let nmethods = buf[1] as usize;
    if n < 2 + nmethods {
        return Ok(());
    }

    stream.write_all(&[0x05, 0x00]).await?;

    let n = stream.read(&mut buf).await?;
    if n < 7 || buf[0] != 0x05 {
        return Ok(());
    }

    let cmd = buf[1];
    if cmd != 0x01 {
        stream.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
        return Ok(());
    }

    let atyp = buf[3];
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
            let domain = String::from_utf8_lossy(&buf[5..5 + len]).to_string();
            let port = u16::from_be_bytes([buf[5 + len], buf[6 + len]]);
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
            stream.write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
            return Ok(());
        }
    };

    debug!("Local request from {} → {}:{}", peer, target_host, target_port);

    let mut upstream = match TcpStream::connect(config.upstream).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Cannot connect to upstream {}: {}", config.upstream, e);
            stream.write_all(&[0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
            return Ok(());
        }
    };
    upstream.set_nodelay(true)?;

    upstream.write_all(&[0x05, 0x01, 0x02]).await?;

    let n = upstream.read(&mut buf).await?;
    if n < 2 || buf[0] != 0x05 {
        stream.write_all(&[0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
        return Ok(());
    }

    if buf[1] == 0x02 {
        let user = config.username.as_bytes();
        let pass = config.password.as_bytes();
        let mut auth = Vec::with_capacity(3 + user.len() + pass.len());
        auth.push(0x01);
        auth.push(user.len() as u8);
        auth.extend_from_slice(user);
        auth.push(pass.len() as u8);
        auth.extend_from_slice(pass);
        upstream.write_all(&auth).await?;

        let n = upstream.read(&mut buf).await?;
        if n < 2 || buf[1] != 0x00 {
            debug!("Upstream auth failed");
            stream.write_all(&[0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
            return Ok(());
        }
    } else if buf[1] != 0x00 {
        stream.write_all(&[0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
        return Ok(());
    }

    let mut req = Vec::new();
    req.push(0x05);
    req.push(0x01);
    req.push(0x00);

    if let Ok(ip) = target_host.parse::<std::net::Ipv4Addr>() {
        req.push(0x01);
        req.extend_from_slice(&ip.octets());
    } else if let Ok(ip) = target_host.parse::<std::net::Ipv6Addr>() {
        req.push(0x04);
        req.extend_from_slice(&ip.octets());
    } else {
        req.push(0x03);
        req.push(target_host.len() as u8);
        req.extend_from_slice(target_host.as_bytes());
    }
    req.extend_from_slice(&target_port.to_be_bytes());

    upstream.write_all(&req).await?;

    let n = upstream.read(&mut buf).await?;
    if n < 2 || buf[1] != 0x00 {
        debug!("Upstream CONNECT failed");
        stream.write_all(&[0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
        return Ok(());
    }

    stream.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;

    let _ = tokio::io::copy_bidirectional(&mut stream, &mut upstream).await;
    Ok(())
}
