//! Authentication for Socks7 / V2rei

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Supported authentication methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AuthMethod {
    NoAuth = 0x00,
    UsernamePassword = 0x02,
}

impl AuthMethod {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Self::NoAuth),
            0x02 => Some(Self::UsernamePassword),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Simple in-memory user store
#[derive(Debug, Clone, Default)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<String, String>>>,
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_user(&self, username: impl Into<String>, password: impl Into<String>) {
        self.users
            .write()
            .await
            .insert(username.into(), password.into());
    }

    pub async fn verify(&self, username: &str, password: &str) -> bool {
        self.users
            .read()
            .await
            .get(username)
            .map(|p| p == password)
            .unwrap_or(false)
    }

    pub async fn is_empty(&self) -> bool {
        self.users.read().await.is_empty()
    }
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub allow_no_auth: bool,
    pub users: UserStore,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            allow_no_auth: true,
            users: UserStore::new(),
        }
    }
}

impl AuthConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_no_auth(mut self, allow: bool) -> Self {
        self.allow_no_auth = allow;
        self
    }

    pub async fn add_user(&self, username: impl Into<String>, password: impl Into<String>) {
        self.users.add_user(username, password).await;
    }
}
