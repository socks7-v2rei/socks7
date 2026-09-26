//! Reply codes for Socks7 - v2rei

use std::fmt;

/// Socks7 Reply codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReplyCode {
    Succeeded = 0x00,
    GeneralFailure = 0x01,
    NotAllowed = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TtlExpired = 0x06,
    CommandNotSupported = 0x07,
    AddressTypeNotSupported = 0x08,
    AuthenticationRequired = 0x09,
    AuthenticationFailed = 0x0A,
    InvalidOptions = 0x0B,
    InitialDataTooLarge = 0x0C,
}

impl ReplyCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Succeeded),
            0x01 => Some(Self::GeneralFailure),
            0x02 => Some(Self::NotAllowed),
            0x03 => Some(Self::NetworkUnreachable),
            0x04 => Some(Self::HostUnreachable),
            0x05 => Some(Self::ConnectionRefused),
            0x06 => Some(Self::TtlExpired),
            0x07 => Some(Self::CommandNotSupported),
            0x08 => Some(Self::AddressTypeNotSupported),
            0x09 => Some(Self::AuthenticationRequired),
            0x0A => Some(Self::AuthenticationFailed),
            0x0B => Some(Self::InvalidOptions),
            0x0C => Some(Self::InitialDataTooLarge),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn is_success(self) -> bool {
        matches!(self, Self::Succeeded)
    }
}

impl fmt::Display for ReplyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Succeeded => write!(f, "Succeeded"),
            Self::GeneralFailure => write!(f, "General Failure"),
            Self::NotAllowed => write!(f, "Not Allowed"),
            Self::NetworkUnreachable => write!(f, "Network Unreachable"),
            Self::HostUnreachable => write!(f, "Host Unreachable"),
            Self::ConnectionRefused => write!(f, "Connection Refused"),
            Self::TtlExpired => write!(f, "TTL Expired"),
            Self::CommandNotSupported => write!(f, "Command Not Supported"),
            Self::AddressTypeNotSupported => write!(f, "Address Type Not Supported"),
            Self::AuthenticationRequired => write!(f, "Authentication Required"),
            Self::AuthenticationFailed => write!(f, "Authentication Failed"),
            Self::InvalidOptions => write!(f, "Invalid Options"),
            Self::InitialDataTooLarge => write!(f, "Initial Data Too Large"),
        }
    }
}
