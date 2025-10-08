//! DeepVault Core Library
//! 
//! A secure USB encryption tool with cross-platform support for Linux (LUKS), 
//! Windows (VeraCrypt), and macOS (APFS/cryptsetup).

pub mod device;
pub mod crypto;
pub mod partition;
pub mod mount;
pub mod wipe;
pub mod config;
pub mod error;
pub mod utils;

pub use error::{DeepVaultError, Result};

/// Re-export commonly used types
pub use device::{UsbDevice, DeviceManager};
pub use crypto::{CryptoConfig, KeyDerivation};
pub use partition::{PartitionManager, PartitionType};
pub use mount::{MountManager, MountStatus};
pub use config::{AppConfig, SecurityConfig};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "DeepVault";

/// Initialize logging
pub fn init_logging() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    Ok(())
}
