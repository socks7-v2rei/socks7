//! Socks7 - v2rei Client implementation

use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{info, debug};

use crate::protocol::*;

pub struct Client {
    proxy_addr: SocketAddr,
}

impl Client {
    pub fn new(proxy_addr: SocketAddr) -> Self {
        Self { proxy_addr }
    }

    /// Connect to target through the Socks7 - v2rei proxy
    pub async fn connect(&self, target: SocksAddr) -> anyhow::Result<TcpStream> {
        let mut stream = TcpStream::connect(self.proxy_addr).await?;
        
        let request = Request::new(Command::Connect, target);
        
        let mut buf = bytes::BytesMut::new();
        request.write_to(&mut buf);
        stream.write_all(&buf).await?;

        // Read reply
        let mut reply_buf = vec![0u8; 512];
        let n = stream.read(&mut reply_buf).await?;
        let mut reader = &reply_buf[..n];
        
        let reply = Reply::read_from(&mut reader)?;
        
        if !reply.code.is_success() {
            anyhow::bail!("Proxy returned error: {}", reply.code);
        }

        info!("Connected via Socks7 - v2rei, bind addr: {}", reply.bind_addr);
        Ok(stream)
    }
}
