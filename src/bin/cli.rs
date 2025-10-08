//! DeepVault CLI application

use deepvault_core::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "deepvault-cli")]
#[command(about = "DeepVault - Secure USB encryption tool")]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available USB devices
    List,
    /// Configure a USB device
    Configure {
        /// Device path
        device: PathBuf,
        /// Configuration type
        #[arg(short, long)]
        config_type: Option<String>,
    },
    /// Mount an encrypted volume
    Mount {
        /// Device path
        device: PathBuf,
        /// Volume name
        #[arg(short, long)]
        volume: String,
    },
    /// Unmount a volume
    Unmount {
        /// Volume name
        volume: String,
    },
    /// Wipe a device
    Wipe {
        /// Device path
        device: PathBuf,
        /// Wipe only header
        #[arg(long)]
        header_only: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::List => {
            list_devices().await?;
        }
        Commands::Configure { device, config_type } => {
            configure_device(device, config_type).await?;
        }
        Commands::Mount { device, volume } => {
            mount_volume(device, volume).await?;
        }
        Commands::Unmount { volume } => {
            unmount_volume(volume).await?;
        }
        Commands::Wipe { device, header_only } => {
            wipe_device(device, header_only).await?;
        }
    }
    
    Ok(())
}

async fn list_devices() -> Result<()> {
    let mut device_manager = DeviceManager::new();
    device_manager.refresh();
    
    let devices = device_manager.get_usb_devices()?;
    
    println!("DeepVault - Détecteur de périphériques USB");
    println!("==========================================");
    println!();
    
    if devices.is_empty() {
        println!("Aucun périphérique USB détecté.");
        return Ok(());
    }
    
    println!("{:<20} {:<20} {:<15} {:<10} {:<10}", "Nom", "Device", "Taille", "Partitions", "Monté");
    println!("{:-<75}", "");
    
    for device in devices {
        println!(
            "{:<20} {:<20} {:<15} {:<10} {:<10}",
            device.name,
            device.device_path.display(),
            crate::utils::format_bytes(device.size),
            device.partitions.len(),
            if device.is_mounted { "Oui" } else { "Non" }
        );
    }
    
    Ok(())
}

async fn configure_device(device: PathBuf, config_type: Option<String>) -> Result<()> {
    println!("Configuration du périphérique: {}", device.display());
    
    if !crate::utils::is_admin() {
        return Err(DeepVaultError::Permission("Administrator privileges required".to_string()));
    }
    
    // TODO: Implement device configuration
    println!("Configuration non implémentée dans cette version CLI.");
    
    Ok(())
}

async fn mount_volume(device: PathBuf, volume: String) -> Result<()> {
    println!("Montage du volume: {} sur {}", volume, device.display());
    
    // TODO: Implement volume mounting
    println!("Montage non implémenté dans cette version CLI.");
    
    Ok(())
}

async fn unmount_volume(volume: String) -> Result<()> {
    println!("Démontage du volume: {}", volume);
    
    // TODO: Implement volume unmounting
    println!("Démontage non implémenté dans cette version CLI.");
    
    Ok(())
}

async fn wipe_device(device: PathBuf, header_only: bool) -> Result<()> {
    println!("Effacement du périphérique: {}", device.display());
    
    if !crate::utils::is_admin() {
        return Err(DeepVaultError::Permission("Administrator privileges required".to_string()));
    }
    
    if header_only {
        println!("Mode: Effacement du header uniquement");
    } else {
        println!("Mode: Effacement complet (IRRÉVERSIBLE)");
    }
    
    // TODO: Implement device wiping
    println!("Effacement non implémenté dans cette version CLI.");
    
    Ok(())
}
