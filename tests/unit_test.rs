//! Tests unitaires pour DeepVault

use deepvault_core::*;
use std::path::PathBuf;

#[test]
fn test_key_derivation_params() {
    let params = KeyDerivation::default();
    assert_eq!(params.memory, 65536);
    assert_eq!(params.iterations, 3);
    assert_eq!(params.parallelism, 1);
}

#[test]
fn test_crypto_config_creation() {
    let config = CryptoConfig {
        algorithm: "Argon2id".to_string(),
        kdf_params: KeyDerivation::default(),
        salt: vec![1, 2, 3, 4],
        header_file: ".dv_meta".to_string(),
    };

    assert_eq!(config.algorithm, "Argon2id");
    assert_eq!(config.salt, vec![1, 2, 3, 4]);
    assert_eq!(config.header_file, ".dv_meta");
}

#[test]
fn test_mount_status() {
    let mounted = MountStatus::Mounted(PathBuf::from("/mnt/test"));
    let unmounted = MountStatus::Unmounted;
    let error = MountStatus::Error("Test error".to_string());

    assert!(matches!(mounted, MountStatus::Mounted(_)));
    assert!(matches!(unmounted, MountStatus::Unmounted));
    assert!(matches!(error, MountStatus::Error(_)));
}

#[test]
fn test_partition_type() {
    let public = PartitionType::Public;
    let encrypted = PartitionType::Encrypted;
    let hidden = PartitionType::Hidden;

    assert_eq!(public, PartitionType::Public);
    assert_eq!(encrypted, PartitionType::Encrypted);
    assert_eq!(hidden, PartitionType::Hidden);
}

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();

    assert_eq!(config.security.default_algorithm, "Argon2id");
    assert_eq!(config.ui.theme, "dark");
    assert_eq!(config.ui.language, "fr");
    assert!(config.security.enable_hidden_volumes);
    assert!(config.ui.confirm_destructive_actions);
}

#[test]
fn test_app_config_validation() {
    let mut config = AppConfig::default();

    // Test validation réussie
    assert!(config.validate().is_ok());

    // Test validation échec - timeout à 0
    config.security.auto_unmount_timeout = 0;
    assert!(config.validate().is_err());

    // Reset
    config.security.auto_unmount_timeout = 300;

    // Test validation échec - passes à 0
    config.security.secure_wipe_passes = 0;
    assert!(config.validate().is_err());

    // Reset
    config.security.secure_wipe_passes = 3;

    // Test validation échec - algorithme invalide
    config.security.default_algorithm = "InvalidAlgo".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn test_error_types() {
    let device_error = DeepVaultError::Device("Test error".to_string());
    let crypto_error = DeepVaultError::Crypto("Test error".to_string());
    let io_error = DeepVaultError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Test"));

    assert!(matches!(device_error, DeepVaultError::Device(_)));
    assert!(matches!(crypto_error, DeepVaultError::Crypto(_)));
    assert!(matches!(io_error, DeepVaultError::Io(_)));
}

#[test]
fn test_serialization() {
    let config = AppConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(
        config.security.default_algorithm,
        deserialized.security.default_algorithm
    );
    assert_eq!(config.ui.theme, deserialized.ui.theme);
    assert_eq!(config.ui.language, deserialized.ui.language);
}
