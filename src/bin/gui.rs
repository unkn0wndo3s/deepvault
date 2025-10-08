//! DeepVault GUI application (Tauri)

use deepvault_core::*;
use std::path::PathBuf;

// Note: This is a placeholder for the Tauri GUI
// The actual Tauri implementation would be in src-tauri/

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;
    
    println!("DeepVault GUI - Interface graphique");
    println!("Cette version est un placeholder pour l'interface Tauri");
    println!("Pour lancer l'interface graphique complète, utilisez: npm run tauri:dev");
    
    Ok(())
}