//! Command definitions for Socks7 - v2rei

use std::fmt;

/// Socks7 Command codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    /// No operation / keep-alive / test
    Noop = 0x00,
    /// Establish TCP connection to target
    Connect = 0x01,
    /// Bind a port for incoming connection (e.g. FTP active mode)
    Bind = 0x02,
    /// Associate UDP relay
    UdpAssociate = 0x03,
}

impl Command {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Noop),
            0x01 => Some(Self::Connect),
            0x02 => Some(Self::Bind),
            0x03 => Some(Self::UdpAssociate),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noop => write!(f, "NOOP"),
            Self::Connect => write!(f, "CONNECT"),
            Self::Bind => write!(f, "BIND"),
            Self::UdpAssociate => write!(f, "UDP_ASSOCIATE"),
        }
    }
}
