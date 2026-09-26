//! Socks7 / V2rei
//!
//! Next-generation ultra-lightweight, high-performance proxy protocol.
//!
//! Protocol Version: 0x07
//! Dual branding: Socks7 and V2rei
//! Website: https://v2rei.surf

pub mod protocol;
pub mod server;
pub mod client;
pub mod auth;
pub mod bridge;

pub use protocol::*;
pub use auth::*;
