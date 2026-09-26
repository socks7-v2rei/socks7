//! Socks7 - v2rei Protocol Core Definitions
//! 
//! Ultra-lightweight, stable and powerful proxy protocol.
//! Protocol Version: 0x07

pub mod address;
pub mod command;
pub mod error;
pub mod message;
pub mod option;
pub mod reply;

pub use address::*;
pub use command::*;
pub use error::*;
pub use message::*;
pub use option::*;
pub use reply::*;

/// Official protocol version byte
pub const PROTOCOL_VERSION: u8 = 0x07;

/// Maximum allowed domain length
pub const MAX_DOMAIN_LENGTH: usize = 255;

/// Maximum Initial Data size (to prevent abuse)
pub const MAX_INITIAL_DATA: usize = 65535;
