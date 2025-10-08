//! USB device detection and management

use crate::{DeepVaultError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use sysinfo::{DiskExt, System, SystemExt};

/// USB device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    pub name: String,
    pub device_path: PathBuf,
    pub size: u64,
    pub partitions: Vec<PartitionInfo>,
    pub is_mounted: bool,
    pub is_removable: bool,
}

/// Partition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub filesystem: Option<String>,
    pub is_mounted: bool,
    pub mount_point: Option<PathBuf>,
    pub partition_type: String, // "public" or "encrypted"
    pub device_path: String,    // Physical device path (e.g., /dev/sdb1)
}

/// Device manager for USB detection and management
pub struct DeviceManager {
    system: System,
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_disks();
        Self { system }
    }

    /// Refresh device information
    pub fn refresh(&mut self) {
        self.system.refresh_disks();
    }

    /// Get all USB devices
    pub fn get_usb_devices(&self) -> Result<Vec<UsbDevice>> {
        #[cfg(target_os = "windows")]
        {
            self.get_usb_devices_windows()
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.get_usb_devices_unix()
        }
    }

    /// Get USB devices on Windows using wmic
    #[cfg(target_os = "windows")]
    fn get_usb_devices_windows(&self) -> Result<Vec<UsbDevice>> {
        let mut devices = Vec::new();

        // Use wmic to get USB disk drives
        let output = Command::new("wmic")
            .args(&[
                "logicaldisk",
                "where",
                "drivetype=2",
                "get",
                "deviceid,size,volumename,freespace",
                "/format:csv",
            ])
            .output()
            .map_err(|e| DeepVaultError::Device(format!("Failed to run wmic: {}", e)))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let device_id = parts[1].trim();
                let size_str = parts[2].trim();
                let volume_name = parts[3].trim();
                let free_space_str = parts[4].trim();

                if let Ok(size) = u64::from_str(size_str) {
                    if size > 0 {
                        let device_path = PathBuf::from(device_id);
                        let device_name = if volume_name.is_empty() {
                            format!("USB Drive {}", device_id)
                        } else {
                            volume_name.to_string()
                        };

                        let device = UsbDevice {
                            name: device_name,
                            device_path: device_path.clone(),
                            size,
                            partitions: self.get_partitions_for_device_windows(device_id)?,
                            is_mounted: true, // If we can see it via wmic, it's mounted
                            is_removable: true,
                        };
                        devices.push(device);
                    }
                }
            }
        }

        Ok(devices)
    }

    /// Get USB devices on Unix-like systems
    #[cfg(not(target_os = "windows"))]
    fn get_usb_devices_unix(&self) -> Result<Vec<UsbDevice>> {
        let mut devices = Vec::new();

        for disk in self.system.disks() {
            if disk.is_removable() {
                let device = UsbDevice {
                    name: disk.name().to_string_lossy().to_string(),
                    device_path: disk.mount_point().to_path_buf(),
                    size: disk.total_space(),
                    partitions: self.get_partitions_for_device_unix(disk)?,
                    is_mounted: self.is_device_mounted(disk),
                    is_removable: disk.is_removable(),
                };
                devices.push(device);
            }
        }

        Ok(devices)
    }

    /// Get partitions for a specific device on Windows
    #[cfg(target_os = "windows")]
    fn get_partitions_for_device_windows(&self, device_id: &str) -> Result<Vec<PartitionInfo>> {
        let mut partitions = Vec::new();

        // For now, treat the entire USB drive as one partition
        // In a real implementation, you'd parse the partition table
        let partition = PartitionInfo {
            name: format!("{}_partition", device_id),
            path: PathBuf::from(device_id),
            size: 0, // Will be filled by the caller
            filesystem: Some("FAT32".to_string()),
            is_mounted: true,
            mount_point: Some(PathBuf::from(device_id)),
            partition_type: "public".to_string(),
            device_path: device_id.to_string(),
        };
        partitions.push(partition);

        Ok(partitions)
    }

    /// Get partitions for a specific device on Unix
    #[cfg(not(target_os = "windows"))]
    fn get_partitions_for_device_unix(&self, disk: &sysinfo::Disk) -> Result<Vec<PartitionInfo>> {
        let mut partitions = Vec::new();

        let partition = PartitionInfo {
            name: disk.name().to_string_lossy().to_string(),
            path: disk.mount_point().to_path_buf(),
            size: disk.total_space(),
            filesystem: Some("Unknown".to_string()),
            is_mounted: self.is_device_mounted(disk),
            mount_point: if self.is_device_mounted(disk) {
                Some(disk.mount_point().to_path_buf())
            } else {
                None
            },
            partition_type: "public".to_string(),
            device_path: disk.name().to_string_lossy().to_string(),
        };
        partitions.push(partition);

        Ok(partitions)
    }

    /// Check if device is mounted
    fn is_device_mounted(&self, disk: &sysinfo::Disk) -> bool {
        !disk.mount_point().as_os_str().is_empty()
    }

    /// Get device by path
    pub fn get_device_by_path(&self, path: &PathBuf) -> Result<Option<UsbDevice>> {
        for disk in self.system.disks() {
            if disk.mount_point() == path {
                #[cfg(target_os = "windows")]
                {
                    return Ok(Some(UsbDevice {
                        name: disk.name().to_string_lossy().to_string(),
                        device_path: disk.mount_point().to_path_buf(),
                        size: disk.total_space(),
                        partitions: self.get_partitions_for_device_windows(
                            disk.name().to_string_lossy().as_ref(),
                        )?,
                        is_mounted: self.is_device_mounted(disk),
                        is_removable: disk.is_removable(),
                    }));
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Ok(Some(UsbDevice {
                        name: disk.name().to_string_lossy().to_string(),
                        device_path: disk.mount_point().to_path_buf(),
                        size: disk.total_space(),
                        partitions: self.get_partitions_for_device_unix(disk)?,
                        is_mounted: self.is_device_mounted(disk),
                        is_removable: disk.is_removable(),
                    }));
                }
            }
        }
        Ok(None)
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
