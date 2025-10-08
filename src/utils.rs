//! Utility functions for DeepVault

use crate::{DeepVaultError, Result};
use std::path::PathBuf;

/// Check if running with administrator privileges
pub fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::processthreadsapi::GetCurrentProcess;
        use winapi::um::processthreadsapi::OpenProcessToken;
        use winapi::um::securitybaseapi::GetTokenInformation;
        use winapi::um::winnt::TokenElevation;
        use winapi::um::winnt::TOKEN_QUERY;
        use winapi::um::handleapi::CloseHandle;
        
        unsafe {
            let process = GetCurrentProcess();
            let mut token = std::ptr::null_mut();
            
            if OpenProcessToken(process, TOKEN_QUERY, &mut token) != 0 {
                let mut elevation: u32 = 0;
                let mut size = std::mem::size_of::<u32>() as u32;
                
                let result = GetTokenInformation(
                    token,
                    TokenElevation,
                    &mut elevation as *mut _ as *mut _,
                    size,
                    &mut size,
                );
                
                CloseHandle(token);
                result != 0 && elevation != 0
            } else {
                false
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("id")
            .arg("-u")
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).trim() == "0"
            })
            .unwrap_or(false)
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("id")
            .arg("-u")
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).trim() == "0"
            })
            .unwrap_or(false)
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

/// Format bytes to human readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Validate password strength
pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(DeepVaultError::InvalidInput("Password must be at least 8 characters long".to_string()));
    }
    
    if password.len() > 128 {
        return Err(DeepVaultError::InvalidInput("Password must be less than 128 characters long".to_string()));
    }
    
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    if !has_upper || !has_lower || !has_digit || !has_special {
        return Err(DeepVaultError::InvalidInput(
            "Password must contain at least one uppercase letter, lowercase letter, digit, and special character".to_string()
        ));
    }
    
    Ok(())
}

/// Generate a secure random string
pub fn generate_random_string(length: usize) -> Result<String> {
    use rand::Rng;
    
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    Ok(password)
}

/// Check if a path is a valid device path
pub fn is_valid_device_path(path: &PathBuf) -> bool {
    #[cfg(target_os = "linux")]
    {
        path.to_string_lossy().starts_with("/dev/")
    }
    
    #[cfg(target_os = "windows")]
    {
        path.to_string_lossy().starts_with("\\\\.\\PhysicalDrive")
    }
    
    #[cfg(target_os = "macos")]
    {
        path.to_string_lossy().starts_with("/dev/disk")
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        false
    }
}
