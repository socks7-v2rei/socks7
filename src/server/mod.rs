//! Socks7 - v2rei Server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn, error, debug};

use crate::protocol::*;

pub struct Server {
    listen_addr: SocketAddr,
}

impl Server {
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self { listen_addr }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.listen_addr).await?;
        info!("Socks7 - v2rei server listening on {}", self.listen_addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    debug!("New connection from {}", peer);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, peer).await {
                            warn!("Connection from {} error: {}", peer, e);
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

async fn handle_connection(mut stream: TcpStream, peer: SocketAddr) -> anyhow::Result<()> {
    // Read request
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let mut reader = &buf[..n];
    let request = match Request::read_from(&mut reader) {
        Ok(req) => req,
        Err(e) => {
            warn!("Failed to parse request from {}: {}", peer, e);
            let reply = Reply::error(ReplyCode::GeneralFailure);
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            let _ = stream.write_all(&out).await;
            return Ok(());
        }
    };

    info!(
        "Request from {}: {} → {}",
        peer, request.command, request.destination
    );

    // Currently only CONNECT is fully implemented in this skeleton
    match request.command {
        Command::Connect => {
            handle_connect(&mut stream, &request).await?;
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
        _ => {
            let reply = Reply::error(ReplyCode::CommandNotSupported);
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            stream.write_all(&out).await?;
        }
    }

    Ok(())
}

async fn handle_connect(client: &mut TcpStream, request: &Request) -> anyhow::Result<()> {
    // Resolve and connect to target
    let target = format!("{}", request.destination);
    
    let target_stream = match TcpStream::connect(&target).await {
        Ok(s) => s,
        Err(e) => {
            debug!("Failed to connect to {}: {}", target, e);
            let code = match e.kind() {
                std::io::ErrorKind::ConnectionRefused => ReplyCode::ConnectionRefused,
                std::io::ErrorKind::TimedOut => ReplyCode::HostUnreachable,
                _ => ReplyCode::HostUnreachable,
            };
            let reply = Reply::error(code);
            let mut out = bytes::BytesMut::new();
            reply.write_to(&mut out);
            client.write_all(&out).await?;
            return Ok(());
        }
    };

    // Success reply
    let local_addr = target_stream.local_addr().unwrap_or_else(|_| {
        "0.0.0.0:0".parse().unwrap()
    });
    let bind_addr = SocksAddr::from(local_addr);

    let mut reply = Reply::success(bind_addr);
    // If client sent initial data, we can note how much we consumed (here 0 for simplicity)
    reply.initial_data_offset = 0;

    let mut out = bytes::BytesMut::new();
    reply.write_to(&mut out);
    client.write_all(&out).await?;

    // If there was initial data, forward it
    if !request.initial_data.is_empty() {
        let mut target = target_stream;
        target.write_all(&request.initial_data).await?;
        
        // Bidirectional copy
        tokio::io::copy_bidirectional(client, &mut target).await?;
    } else {
        let mut target = target_stream;
        tokio::io::copy_bidirectional(client, &mut target).await?;
    }

    Ok(())
}
