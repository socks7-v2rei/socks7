//! Option framework for Socks7 - v2rei

use bytes::{Buf, BufMut, BytesMut};
use super::error::ProtocolError;

/// Option Kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OptionKind {
    AuthMethodAdvertisement = 0x01,
    AuthenticationData = 0x02,
    SocketOptions = 0x03,
    Multiplexing = 0x04,
    IdempotenceToken = 0x05,
    Padding = 0x06,
    // 0xC0 - 0xFF : Vendor specific (v2rei)
}

impl OptionKind {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::AuthMethodAdvertisement),
            0x02 => Some(Self::AuthenticationData),
            0x03 => Some(Self::SocketOptions),
            0x04 => Some(Self::Multiplexing),
            0x05 => Some(Self::IdempotenceToken),
            0x06 => Some(Self::Padding),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// A single protocol option
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocksOption {
    pub kind: u8,
    pub data: Vec<u8>,
}

impl SocksOption {
    pub fn new(kind: u8, data: Vec<u8>) -> Self {
        Self { kind, data }
    }

    pub fn write_to(&self, buf: &mut BytesMut) {
        buf.put_u8(self.kind);
        buf.put_u16(self.data.len() as u16);
        buf.extend_from_slice(&self.data);
    }

    pub fn read_from(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        if buf.remaining() < 3 {
            return Err(ProtocolError::IncompleteMessage);
        }
        let kind = buf.get_u8();
        let len = buf.get_u16() as usize;
        if buf.remaining() < len {
            return Err(ProtocolError::IncompleteMessage);
        }
        let mut data = vec![0u8; len];
        buf.copy_to_slice(&mut data);
        Ok(Self { kind, data })
    }
}

/// Collection of options
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    pub items: Vec<SocksOption>,
}

impl Options {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, option: SocksOption) {
        self.items.push(option);
    }

    pub fn write_to(&self, buf: &mut BytesMut) {
        for opt in &self.items {
            opt.write_to(buf);
        }
    }

    pub fn read_from(buf: &mut impl Buf, total_len: usize) -> Result<Self, ProtocolError> {
        let mut items = Vec::new();
        let mut remaining = total_len;

        while remaining > 0 {
            if remaining < 3 {
                return Err(ProtocolError::InvalidOptions);
            }
            let start_remaining = buf.remaining();
            let opt = SocksOption::read_from(buf)?;
            let consumed = start_remaining - buf.remaining();
            if consumed > remaining {
                return Err(ProtocolError::InvalidOptions);
            }
            remaining -= consumed;
            items.push(opt);
        }

        Ok(Self { items })
    }

    pub fn encoded_len(&self) -> usize {
        self.items.iter().map(|o| 3 + o.data.len()).sum()
    }
}
