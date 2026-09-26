//! Core message structures for Socks7 - v2rei

use bytes::{Buf, BufMut, BytesMut};
use super::address::SocksAddr;
use super::command::Command;
use super::error::ProtocolError;
use super::option::Options;
use super::reply::ReplyCode;
use super::{PROTOCOL_VERSION, MAX_INITIAL_DATA};

/// Client Request
#[derive(Debug, Clone)]
pub struct Request {
    pub command: Command,
    pub destination: SocksAddr,
    pub options: Options,
    pub initial_data: Vec<u8>,
}

impl Request {
    pub fn new(command: Command, destination: SocksAddr) -> Self {
        Self {
            command,
            destination,
            options: Options::new(),
            initial_data: Vec::new(),
        }
    }

    pub fn with_initial_data(mut self, data: Vec<u8>) -> Self {
        self.initial_data = data;
        self
    }

    pub fn write_to(&self, buf: &mut BytesMut) {
        buf.put_u8(PROTOCOL_VERSION);
        buf.put_u8(self.command.as_u8());
        self.destination.write_to(buf);

        let options_len = self.options.encoded_len();
        buf.put_u16(options_len as u16);
        self.options.write_to(buf);

        let initial_len = self.initial_data.len().min(MAX_INITIAL_DATA);
        buf.put_u16(initial_len as u16);
        buf.extend_from_slice(&self.initial_data[..initial_len]);
    }

    pub fn read_from(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        if buf.remaining() < 1 {
            return Err(ProtocolError::IncompleteMessage);
        }

        let version = buf.get_u8();
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        if buf.remaining() < 1 {
            return Err(ProtocolError::IncompleteMessage);
        }
        let cmd_byte = buf.get_u8();
        let command = Command::from_u8(cmd_byte)
            .ok_or(ProtocolError::UnsupportedCommand(cmd_byte))?;

        let destination = SocksAddr::read_from(buf)?;

        if buf.remaining() < 2 {
            return Err(ProtocolError::IncompleteMessage);
        }
        let options_len = buf.get_u16() as usize;
        let options = Options::read_from(buf, options_len)?;

        if buf.remaining() < 2 {
            return Err(ProtocolError::IncompleteMessage);
        }
        let initial_len = buf.get_u16() as usize;
        if initial_len > MAX_INITIAL_DATA {
            return Err(ProtocolError::InitialDataTooLarge(initial_len));
        }
        if buf.remaining() < initial_len {
            return Err(ProtocolError::IncompleteMessage);
        }
        let mut initial_data = vec![0u8; initial_len];
        buf.copy_to_slice(&mut initial_data);

        Ok(Self {
            command,
            destination,
            options,
            initial_data,
        })
    }
}

/// Proxy Reply (both Authentication Reply and Operation Reply share this structure)
#[derive(Debug, Clone)]
pub struct Reply {
    pub code: ReplyCode,
    pub bind_addr: SocksAddr,
    pub options: Options,
    /// How many bytes of the client's Initial Data were consumed by the proxy
    pub initial_data_offset: u16,
}

impl Reply {
    pub fn success(bind_addr: SocksAddr) -> Self {
        Self {
            code: ReplyCode::Succeeded,
            bind_addr,
            options: Options::new(),
            initial_data_offset: 0,
        }
    }

    pub fn error(code: ReplyCode) -> Self {
        Self {
            code,
            bind_addr: SocksAddr {
                address: super::address::Address::Ipv4(std::net::Ipv4Addr::UNSPECIFIED),
                port: 0,
            },
            options: Options::new(),
            initial_data_offset: 0,
        }
    }

    pub fn write_to(&self, buf: &mut BytesMut) {
        buf.put_u8(PROTOCOL_VERSION);
        buf.put_u8(self.code.as_u8());
        self.bind_addr.write_to(buf);

        let options_len = self.options.encoded_len();
        buf.put_u16(options_len as u16);
        self.options.write_to(buf);

        buf.put_u16(self.initial_data_offset);
    }

    pub fn read_from(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        if buf.remaining() < 1 {
            return Err(ProtocolError::IncompleteMessage);
        }

        let version = buf.get_u8();
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        if buf.remaining() < 1 {
            return Err(ProtocolError::IncompleteMessage);
        }
        let code_byte = buf.get_u8();
        let code = ReplyCode::from_u8(code_byte)
            .unwrap_or(ReplyCode::GeneralFailure);

        let bind_addr = SocksAddr::read_from(buf)?;

        if buf.remaining() < 2 {
            return Err(ProtocolError::IncompleteMessage);
        }
        let options_len = buf.get_u16() as usize;
        let options = Options::read_from(buf, options_len)?;

        if buf.remaining() < 2 {
            return Err(ProtocolError::IncompleteMessage);
        }
        let initial_data_offset = buf.get_u16();

        Ok(Self {
            code,
            bind_addr,
            options,
            initial_data_offset,
        })
    }
}
