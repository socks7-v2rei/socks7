//! Protocol errors for Socks7 - v2rei

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Incomplete message")]
    IncompleteMessage,

    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),

    #[error("Unsupported command: {0}")]
    UnsupportedCommand(u8),

    #[error("Unsupported address type: {0}")]
    UnsupportedAddressType(u8),

    #[error("Invalid domain length: {0}")]
    InvalidDomainLength(usize),

    #[error("Invalid domain encoding (not UTF-8)")]
    InvalidDomainEncoding,

    #[error("Invalid options")]
    InvalidOptions,

    #[error("Initial data too large: {0} bytes")]
    InitialDataTooLarge(usize),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
