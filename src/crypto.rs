//! Cryptographic operations for DeepVault

use crate::{DeepVaultError, Result};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use base64::Engine;

/// Cryptographic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub algorithm: String,
    pub kdf_params: KeyDerivation,
    pub salt: Vec<u8>,
    pub header_file: String,
}

/// Key derivation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivation {
    pub memory: u32,      // Memory in KB
    pub iterations: u32,  // Iterations
    pub parallelism: u32, // Parallelism
}

impl Default for KeyDerivation {
    fn default() -> Self {
        Self {
            memory: 65536,    // 64MB
            iterations: 3,
            parallelism: 1,
        }
    }
}

/// Cryptographic operations manager
pub struct CryptoManager {
    config: CryptoConfig,
}

impl CryptoManager {
    /// Create a new crypto manager with default config
    pub fn new() -> Result<Self> {
        let salt = Self::generate_salt()?;
        Ok(Self {
            config: CryptoConfig {
                algorithm: "Argon2id".to_string(),
                kdf_params: KeyDerivation::default(),
                salt,
                header_file: ".dv_meta".to_string(),
            },
        })
    }

    /// Create a new crypto manager with custom config
    pub fn with_config(config: CryptoConfig) -> Self {
        Self { config }
    }

    /// Generate a random salt
    fn generate_salt() -> Result<Vec<u8>> {
        let mut salt_bytes = vec![0u8; 16]; // 128 bits
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.fill(&mut salt_bytes[..]);
        Ok(salt_bytes)
    }

    /// Derive key from password using Argon2id
    pub fn derive_key(&self, password: &str) -> Result<Vec<u8>> {
        // Create salt string from raw bytes
        let salt_string = SaltString::from_b64(&base64::engine::general_purpose::STANDARD.encode(&self.config.salt))
            .map_err(|e| DeepVaultError::Crypto(format!("Invalid salt: {}", e)))?;

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                self.config.kdf_params.memory,
                self.config.kdf_params.iterations,
                self.config.kdf_params.parallelism,
                Some(32), // Output length
            )
            .map_err(|e| DeepVaultError::Crypto(format!("Invalid Argon2 params: {}", e)))?,
        );

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| DeepVaultError::Crypto(format!("Key derivation failed: {}", e)))?;

        Ok(password_hash.hash.unwrap().as_bytes().to_vec())
    }

    /// Verify password against derived key
    pub fn verify_password(&self, password: &str, derived_key: &[u8]) -> Result<bool> {
        let salt_string = SaltString::from_b64(&base64::engine::general_purpose::STANDARD.encode(&self.config.salt))
            .map_err(|e| DeepVaultError::Crypto(format!("Invalid salt: {}", e)))?;

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                self.config.kdf_params.memory,
                self.config.kdf_params.iterations,
                self.config.kdf_params.parallelism,
                Some(32),
            )
            .map_err(|e| DeepVaultError::Crypto(format!("Invalid Argon2 params: {}", e)))?,
        );

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| DeepVaultError::Crypto(format!("Key derivation failed: {}", e)))?;

        Ok(password_hash.hash.unwrap().as_bytes() == derived_key)
    }

    /// Generate header metadata
    pub fn generate_header_metadata(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut metadata = HashMap::new();
        metadata.insert("algorithm".to_string(), serde_json::Value::String(self.config.algorithm.clone()));
        metadata.insert("kdf_params".to_string(), serde_json::to_value(&self.config.kdf_params)?);
        metadata.insert("salt".to_string(), serde_json::Value::String(hex::encode(&self.config.salt)));
        metadata.insert("version".to_string(), serde_json::Value::String(crate::VERSION.to_string()));
        metadata.insert("created_at".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));

        Ok(metadata)
    }

    /// Get current configuration
    pub fn config(&self) -> &CryptoConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: CryptoConfig) {
        self.config = config;
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default crypto manager")
    }
}