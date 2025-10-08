//! Tests d'intégration pour DeepVault

use deepvault_core::*;
use std::path::PathBuf;

#[tokio::test]
async fn test_device_detection() {
    let mut device_manager = DeviceManager::new();
    device_manager.refresh();

    let devices = device_manager.get_usb_devices().unwrap();
    // Note: Ce test peut échouer s'il n'y a pas de périphériques USB connectés
    println!("Détecté {} périphériques USB", devices.len());
}

#[tokio::test]
async fn test_crypto_manager_creation() {
    let crypto_manager = crate::crypto::CryptoManager::new().unwrap();
    let config = crypto_manager.config();

    assert_eq!(config.algorithm, "Argon2id");
    assert_eq!(config.kdf_params.memory, 65536);
    assert_eq!(config.kdf_params.iterations, 3);
    assert_eq!(config.kdf_params.parallelism, 1);
}

#[tokio::test]
async fn test_key_derivation() {
    let crypto_manager = crate::crypto::CryptoManager::new().unwrap();
    let password = "TestPassword123!";

    let key = crypto_manager.derive_key(password).unwrap();
    assert_eq!(key.len(), 32); // 256 bits

    // Test de vérification
    let is_valid = crypto_manager.verify_password(password, &key).unwrap();
    assert!(is_valid);

    // Test avec mauvais mot de passe
    let is_invalid = crypto_manager
        .verify_password("WrongPassword", &key)
        .unwrap();
    assert!(!is_invalid);
}

#[tokio::test]
async fn test_password_validation() {
    // Test mot de passe valide
    let valid_password = "ValidPass123!";
    assert!(crate::utils::validate_password(valid_password).is_ok());

    // Test mot de passe trop court
    let short_password = "Short1!";
    assert!(crate::utils::validate_password(short_password).is_err());

    // Test mot de passe sans majuscule
    let no_upper = "validpass123!";
    assert!(crate::utils::validate_password(no_upper).is_err());

    // Test mot de passe sans chiffre
    let no_digit = "ValidPassword!";
    assert!(crate::utils::validate_password(no_digit).is_err());

    // Test mot de passe sans caractère spécial
    let no_special = "ValidPassword123";
    assert!(crate::utils::validate_password(no_special).is_err());
}

#[tokio::test]
async fn test_config_loading() {
    let config = AppConfig::load(&PathBuf::from("config.example.json")).unwrap();

    assert_eq!(config.security.default_algorithm, "Argon2id");
    assert_eq!(config.ui.theme, "dark");
    assert_eq!(config.ui.language, "fr");
}

#[tokio::test]
async fn test_wipe_options() {
    let options = crate::wipe::WipeOptions::default();

    assert_eq!(options.passes, 3);
    assert_eq!(options.pattern, crate::wipe::WipePattern::DoD5220_22M);
    assert!(!options.header_only);
}

#[tokio::test]
async fn test_utils() {
    // Test formatage des bytes
    assert_eq!(crate::utils::format_bytes(0), "0 B");
    assert_eq!(crate::utils::format_bytes(1024), "1.0 KB");
    assert_eq!(crate::utils::format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(crate::utils::format_bytes(1024 * 1024 * 1024), "1.0 GB");

    // Test génération de chaîne aléatoire
    let random_string = crate::utils::generate_random_string(16).unwrap();
    assert_eq!(random_string.len(), 16);

    // Test validation de chemin de périphérique
    let valid_path = PathBuf::from("/dev/sdb");
    let invalid_path = PathBuf::from("/invalid/path");

    #[cfg(target_os = "linux")]
    {
        assert!(crate::utils::is_valid_device_path(&valid_path));
        assert!(!crate::utils::is_valid_device_path(&invalid_path));
    }
}
