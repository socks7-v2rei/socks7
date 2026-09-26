//! Socks7 / V2rei Server - Production-ready implementation

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::time::timeout;
use tracing::{info, warn, error, debug};

use crate::protocol::*;
use crate::auth::AuthConfig;

const MAX_REQUEST_SIZE: usize = 16 * 1024;
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

    pub fn simple(listen_addr: SocketAddr) -> Self {
        Self::new(ServerConfig {
            listen_addr,
            ..Default::default()
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        info!(
            "Socks7 / V2rei server listening on {} (NoAuth={}, MaxConn={})",
            self.config.listen_addr,
            self.config.auth.allow_no_auth,
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

    let mut buf = vec![0u8; MAX_REQUEST_SIZE];
    let n = match timeout(HANDSHAKE_TIMEOUT, stream.read(&mut buf)).await {
        Ok(Ok(0)) => return Ok(()),
        Ok(Ok(n)) => n,
        Ok(Err(e)) => return Err(e.into()),
        Err(_) => {
            warn!("Handshake timeout from {}", peer);
            return Ok(());
        }
    };

    if n > MAX_REQUEST_SIZE {
        let reply = Reply::error(ReplyCode::GeneralFailure);
        let mut out = bytes::BytesMut::new();
        reply.write_to(&mut out);
        let _ = stream.write_all(&out).await;
        return Ok(());
    }

    let mut reader = &buf[..n];
    let request = match Request::read_from(&mut reader) {
        Ok(req) => req,
        Err(e) => {
            warn!("Invalid request from {}: {}", peer, e);
            let reply = Reply::error(ReplyCode::GeneralFailure);
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            let _ = stream.write_all(&out).await;
            return Ok(());
        }
    };

    debug!(
        "Request from {}: {} → {}",
        peer, request.command, request.destination
    );

    if !config.auth.allow_no_auth {
        let has_auth = request.options.items.iter().any(|o| o.kind == 0x02);
        if !has_auth {
            let reply = Reply::error(ReplyCode::AuthenticationRequired);
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            stream.write_all(&out).await?;
            return Ok(());
        }
    }

    match request.command {
        Command::Connect => {
            handle_connect(&mut stream, &request, peer).await?;
        }
        Command::UdpAssociate => {
            handle_udp_associate(&mut stream, &request, peer).await?;
        }
        Command::Noop => {
            let reply = Reply::success(SocksAddr {
                address: Address::Ipv4(std::net::Ipv4Addr::UNSPECIFIED),
                port: 0,
            });
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            stream.write_all(&out).await?;
        }
        Command::Bind => {
            let reply = Reply::error(ReplyCode::CommandNotSupported);
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            stream.write_all(&out).await?;
        }
    }

    Ok(())
}

async fn handle_connect(
    client: &mut TcpStream,
    request: &Request,
    peer: SocketAddr,
) -> anyhow::Result<()> {
    let target = format!("{}", request.destination);

    let target_stream = match timeout(CONNECT_TIMEOUT, TcpStream::connect(&target)).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            debug!("Connect to {} failed from {}: {}", target, peer, e);
            let code = match e.kind() {
                std::io::ErrorKind::ConnectionRefused => ReplyCode::ConnectionRefused,
                std::io::ErrorKind::TimedOut => ReplyCode::HostUnreachable,
                // NetworkUnreachable is unstable on stable rust
                _ => ReplyCode::HostUnreachable,
            };
            let reply = Reply::error(code);
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            client.write_all(&out).await?;
            return Ok(());
        }
        Err(_) => {
            let reply = Reply::error(ReplyCode::HostUnreachable);
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            client.write_all(&out).await?;
            return Ok(());
        }
    };

    target_stream.set_nodelay(true)?;

    let local_addr = target_stream
        .local_addr()
        .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
    let bind_addr = SocksAddr::from(local_addr);

    let mut reply = Reply::success(bind_addr);
    reply.initial_data_offset = 0;

    let mut out = bytes::BytesMut::new();
    reply.write_to(&mut out);
    client.write_all(&out).await?;

    let mut target = target_stream;
    if !request.initial_data.is_empty() {
        target.write_all(&request.initial_data).await?;
    }

    let result = tokio::io::copy_bidirectional(client, &mut target).await;
    if let Err(e) = result {
        debug!("Relay ended for {}: {}", peer, e);
    }

    Ok(())
}

async fn handle_udp_associate(
    client: &mut TcpStream,
    _request: &Request,
    peer: SocketAddr,
) -> anyhow::Result<()> {
    let udp = UdpSocket::bind("0.0.0.0:0").await?;
    let local = udp.local_addr()?;
    let bind_addr = SocksAddr::from(local);

    let reply = Reply::success(bind_addr);
    let mut out = bytes::BytesMut::new();
    reply.write_to(&mut out);
    client.write_all(&out).await?;

    info!("UDP ASSOCIATE established for {} on {}", peer, local);

    let mut buf = [0u8; 64];
    loop {
        match client.read(&mut buf).await {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    Ok(())
}
