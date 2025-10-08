//! Secure wipe functionality for DeepVault

use crate::{DeepVaultError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Wipe options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeOptions {
    /// Wipe only the header metadata
    pub header_only: bool,
    /// Number of passes for secure wipe
    pub passes: u32,
    /// Wipe pattern (random, zeros, ones)
    pub pattern: WipePattern,
}

/// Wipe patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WipePattern {
    Random,
    Zeros,
    Ones,
    DoD5220_22M, // 3-pass DoD standard
}

impl Default for WipeOptions {
    fn default() -> Self {
        Self {
            header_only: false,
            passes: 3,
            pattern: WipePattern::DoD5220_22M,
        }
    }
}

/// Secure wipe manager
pub struct WipeManager {
    device_path: PathBuf,
}

impl WipeManager {
    /// Create a new wipe manager
    pub fn new(device_path: PathBuf) -> Self {
        Self { device_path }
    }

    /// Perform secure wipe
    pub async fn wipe(&self, options: WipeOptions) -> Result<()> {
        log::warn!("Starting secure wipe with options: {:?}", options);
        
        if options.header_only {
            self.wipe_header().await?;
        } else {
            self.wipe_full_device(options).await?;
        }

        log::info!("Secure wipe completed successfully");
        Ok(())
    }

    /// Wipe only the header metadata
    async fn wipe_header(&self) -> Result<()> {
        log::info!("Wiping header metadata only");
        
        // Find and wipe header files
        let header_files = self.find_header_files().await?;
        
        for header_file in header_files {
            self.secure_delete_file(&header_file).await?;
        }

        Ok(())
    }

    /// Wipe the entire device
    async fn wipe_full_device(&self, options: WipeOptions) -> Result<()> {
        log::warn!("Wiping entire device - THIS IS IRREVERSIBLE!");
        
        // Platform-specific secure wipe
        #[cfg(target_os = "linux")]
        self.wipe_linux(options).await?;
        
        #[cfg(target_os = "windows")]
        self.wipe_windows(options).await?;
        
        #[cfg(target_os = "macos")]
        self.wipe_macos(options).await?;

        Ok(())
    }

    /// Find header metadata files
    async fn find_header_files(&self) -> Result<Vec<PathBuf>> {
        let mut header_files = Vec::new();
        
        // Look for common header file patterns
        let patterns = [".dv_meta", ".deepvault_meta", "header.bin"];
        
        for pattern in &patterns {
            let header_path = self.device_path.join(pattern);
            if header_path.exists() {
                header_files.push(header_path);
            }
        }

        Ok(header_files)
    }

    /// Securely delete a file
    async fn secure_delete_file(&self, file_path: &PathBuf) -> Result<()> {
        log::info!("Securely deleting file: {:?}", file_path);
        
        // Overwrite file with random data multiple times
        let file = std::fs::File::open(file_path)
            .map_err(|e| DeepVaultError::Io(e))?;
        
        let file_size = file.metadata()
            .map_err(|e| DeepVaultError::Io(e))?
            .len();
        
        // Perform multiple overwrite passes
        for pass in 0..3 {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .open(file_path)
                .map_err(|e| DeepVaultError::Io(e))?;
            
            match pass {
                0 => self.overwrite_with_pattern(&mut file, file_size, &[0xFF]).await?,
                1 => self.overwrite_with_pattern(&mut file, file_size, &[0x00]).await?,
                2 => self.overwrite_with_random(&mut file, file_size).await?,
                _ => {}
            }
        }
        
        // Finally delete the file
        std::fs::remove_file(file_path)
            .map_err(|e| DeepVaultError::Io(e))?;

        Ok(())
    }

    /// Overwrite file with a specific pattern
    async fn overwrite_with_pattern(
        &self,
        file: &mut std::fs::File,
        size: u64,
        pattern: &[u8],
    ) -> Result<()> {
        use std::io::Write;
        
        let mut buffer = Vec::new();
        let pattern_len = pattern.len();
        
        for i in 0..size {
            buffer.push(pattern[i as usize % pattern_len]);
        }
        
        file.write_all(&buffer)
            .map_err(|e| DeepVaultError::Io(e))?;
        file.sync_all()
            .map_err(|e| DeepVaultError::Io(e))?;

        Ok(())
    }

    /// Overwrite file with random data
    async fn overwrite_with_random(
        &self,
        file: &mut std::fs::File,
        size: u64,
    ) -> Result<()> {
        use std::io::Write;
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let mut buffer = vec![0u8; size as usize];
        
        for byte in &mut buffer {
            *byte = rng.gen();
        }
        
        file.write_all(&buffer)
            .map_err(|e| DeepVaultError::Io(e))?;
        file.sync_all()
            .map_err(|e| DeepVaultError::Io(e))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn wipe_linux(&self, options: WipeOptions) -> Result<()> {
        use std::process::Command;
        
        // Use dd to overwrite the entire device
        for pass in 0..options.passes {
            log::info!("Wipe pass {}/{}", pass + 1, options.passes);
            
            let output = match options.pattern {
                WipePattern::Random => {
                    Command::new("dd")
                        .arg("if=/dev/urandom")
                        .arg(format!("of={}", self.device_path.display()))
                        .arg("bs=1M")
                        .arg("status=progress")
                        .output()
                }
                WipePattern::Zeros => {
                    Command::new("dd")
                        .arg("if=/dev/zero")
                        .arg(format!("of={}", self.device_path.display()))
                        .arg("bs=1M")
                        .arg("status=progress")
                        .output()
                }
                _ => {
                    // For other patterns, use shred
                    Command::new("shred")
                        .arg("-vfz")
                        .arg("-n")
                        .arg("3")
                        .arg(&self.device_path)
                        .output()
                }
            }
            .map_err(|e| DeepVaultError::System(format!("Failed to wipe device: {}", e)))?;

            if !output.status.success() {
                return Err(DeepVaultError::System(format!(
                    "Wipe pass {} failed: {}",
                    pass + 1,
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn wipe_windows(&self, options: WipeOptions) -> Result<()> {
        use std::process::Command;
        
        // Use diskpart to clean the disk
        let script = format!(
            "select disk {}\nclean all\nexit",
            self.device_path.to_string_lossy()
        );
        
        let output = Command::new("diskpart")
            .arg("/s")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .output()
            .map_err(|e| DeepVaultError::System(format!("Failed to wipe device: {}", e)))?;

        if !output.status.success() {
            return Err(DeepVaultError::System(format!(
                "Failed to wipe device: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn wipe_macos(&self, options: WipeOptions) -> Result<()> {
        use std::process::Command;
        
        // Use diskutil to secure erase
        let output = Command::new("diskutil")
            .arg("secureErase")
            .arg("freespace")
            .arg("3") // 3-pass erase
            .arg(&self.device_path)
            .output()
            .map_err(|e| DeepVaultError::System(format!("Failed to wipe device: {}", e)))?;

        if !output.status.success() {
            return Err(DeepVaultError::System(format!(
                "Failed to wipe device: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }
}
