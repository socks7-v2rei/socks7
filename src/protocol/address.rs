//! Address handling for Socks7 - v2rei

use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use bytes::{Buf, BufMut, BytesMut};

use super::error::ProtocolError;
use super::MAX_DOMAIN_LENGTH;

/// Address Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AddressType {
    Ipv4 = 0x01,
    Domain = 0x03,
    Ipv6 = 0x04,
}

impl AddressType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Ipv4),
            0x03 => Some(Self::Domain),
            0x04 => Some(Self::Ipv6),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Unified address representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Ipv4(Ipv4Addr),
    Domain(String),
    Ipv6(Ipv6Addr),
}

impl Address {
    pub fn address_type(&self) -> AddressType {
        match self {
            Self::Ipv4(_) => AddressType::Ipv4,
            Self::Domain(_) => AddressType::Domain,
            Self::Ipv6(_) => AddressType::Ipv6,
        }
    }

    /// Serialize address into buffer (without port)
    pub fn write_to(&self, buf: &mut BytesMut) {
        match self {
            Self::Ipv4(ip) => {
                buf.put_u8(AddressType::Ipv4.as_u8());
                buf.extend_from_slice(&ip.octets());
            }
            Self::Domain(domain) => {
                buf.put_u8(AddressType::Domain.as_u8());
                let bytes = domain.as_bytes();
                buf.put_u8(bytes.len() as u8);
                buf.extend_from_slice(bytes);
            }
            Self::Ipv6(ip) => {
                buf.put_u8(AddressType::Ipv6.as_u8());
                buf.extend_from_slice(&ip.octets());
            }
        }
    }

    /// Deserialize address from buffer
    pub fn read_from(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        if buf.remaining() < 1 {
            return Err(ProtocolError::IncompleteMessage);
        }

        let atyp = buf.get_u8();
        let addr_type = AddressType::from_u8(atyp)
            .ok_or(ProtocolError::UnsupportedAddressType(atyp))?;

        match addr_type {
            AddressType::Ipv4 => {
                if buf.remaining() < 4 {
                    return Err(ProtocolError::IncompleteMessage);
                }
                let mut octets = [0u8; 4];
                buf.copy_to_slice(&mut octets);
                Ok(Self::Ipv4(Ipv4Addr::from(octets)))
            }
            AddressType::Domain => {
                if buf.remaining() < 1 {
                    return Err(ProtocolError::IncompleteMessage);
                }
                let len = buf.get_u8() as usize;
                if len == 0 || len > MAX_DOMAIN_LENGTH {
                    return Err(ProtocolError::InvalidDomainLength(len));
                }
                if buf.remaining() < len {
                    return Err(ProtocolError::IncompleteMessage);
                }
                let mut domain_bytes = vec![0u8; len];
                buf.copy_to_slice(&mut domain_bytes);
                let domain = String::from_utf8(domain_bytes)
                    .map_err(|_| ProtocolError::InvalidDomainEncoding)?;
                Ok(Self::Domain(domain))
            }
            AddressType::Ipv6 => {
                if buf.remaining() < 16 {
                    return Err(ProtocolError::IncompleteMessage);
                }
                let mut octets = [0u8; 16];
                buf.copy_to_slice(&mut octets);
                Ok(Self::Ipv6(Ipv6Addr::from(octets)))
            }
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4(ip) => write!(f, "{}", ip),
            Self::Domain(d) => write!(f, "{}", d),
            Self::Ipv6(ip) => write!(f, "{}", ip),
        }
    }
}

/// Full socket address (Address + Port)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocksAddr {
    pub address: Address,
    pub port: u16,
}

impl SocksAddr {
    pub fn new(address: Address, port: u16) -> Self {
        Self { address, port }
    }

    pub fn write_to(&self, buf: &mut BytesMut) {
        self.address.write_to(buf);
        buf.put_u16(self.port);
    }

    pub fn read_from(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let address = Address::read_from(buf)?;
        if buf.remaining() < 2 {
            return Err(ProtocolError::IncompleteMessage);
        }
        let port = buf.get_u16();
        Ok(Self { address, port })
    }
}

impl fmt::Display for SocksAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.address, self.port)
    }
}

impl From<SocketAddr> for SocksAddr {
    fn from(addr: SocketAddr) -> Self {
        match addr {
            SocketAddr::V4(v4) => Self {
                address: Address::Ipv4(*v4.ip()),
                port: v4.port(),
            },
            SocketAddr::V6(v6) => Self {
                address: Address::Ipv6(*v6.ip()),
                port: v6.port(),
            },
        }
    }
}
