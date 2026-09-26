//! Socks7 / V2rei Client implementation

use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::info;

use crate::protocol::*;

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Client {
    proxy_addr: SocketAddr,
}

impl Client {
    pub fn new(proxy_addr: SocketAddr) -> Self {
        Self { proxy_addr }
    }

    /// Connect to target through the Socks7 / V2rei proxy
    pub async fn connect(&self, target: SocksAddr) -> anyhow::Result<TcpStream> {
        let mut stream = timeout(HANDSHAKE_TIMEOUT, TcpStream::connect(self.proxy_addr)).await??;
        stream.set_nodelay(true)?;

        let request = Request::new(Command::Connect, target);

        let mut buf = bytes::BytesMut::with_capacity(512);
        request.write_to(&mut buf);
        stream.write_all(&buf).await?;

        // Read reply
        let mut reply_buf = vec![0u8; 512];
        let n = timeout(HANDSHAKE_TIMEOUT, stream.read(&mut reply_buf)).await??;
        if n == 0 {
            anyhow::bail!("Proxy closed connection during handshake");
        }

        let mut reader = &reply_buf[..n];
        let reply = Reply::read_from(&mut reader)?;

        if !reply.code.is_success() {
            anyhow::bail!("Proxy returned error: {}", reply.code);
        }

        info!("Connected via Socks7 / V2rei → bind {}", reply.bind_addr);
        Ok(stream)
    }

    /// Convenience method: connect using host:port string
    pub async fn connect_host_port(&self, host: &str, port: u16) -> anyhow::Result<TcpStream> {
        let addr = if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            match ip {
                std::net::IpAddr::V4(v4) => SocksAddr::new(Address::Ipv4(v4), port),
                std::net::IpAddr::V6(v6) => SocksAddr::new(Address::Ipv6(v6), port),
            }
        } else {
            SocksAddr::new(Address::Domain(host.to_string()), port)
        };
        self.connect(addr).await
    }
}
