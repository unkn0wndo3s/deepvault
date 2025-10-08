//! Error types for DeepVault

use thiserror::Error;

/// Main error type for DeepVault operations
#[derive(Error, Debug)]
pub enum DeepVaultError {
    #[error("Device error: {0}")]
    Device(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Partition error: {0}")]
    Partition(String),

    #[error("Mount error: {0}")]
    Mount(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("System error: {0}")]
    System(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, DeepVaultError>;
