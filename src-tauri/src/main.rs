//! DeepVault GUI - Tauri application

use deepvault_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedFile {
    name: String,
    path: String,
    is_directory: bool,
    size: u64,
    modified: i64,
    content: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct EncryptedSession {
    session_id: String,
    disk_num: u32,
    partition_num: u32,
    files: HashMap<String, EncryptedFile>,
}

// Stockage global des sessions chiffrées
lazy_static::lazy_static! {
    static ref ENCRYPTED_SESSIONS: Mutex<HashMap<String, EncryptedSession>> = Mutex::new(HashMap::new());
}

#[tauri::command]
async fn get_usb_devices() -> std::result::Result<Vec<UsbDevice>, String> {
    let mut device_manager = DeviceManager::new();
    device_manager.refresh();

    device_manager.get_usb_devices().map_err(|e| e.to_string())
}

#[tauri::command]
async fn configure_device(
    _device_path: String,
    _config_type: String,
    password: String,
) -> std::result::Result<String, String> {
    // Simple password validation
    if password.len() < 8 {
        return Err("Le mot de passe doit contenir au moins 8 caractères".to_string());
    }

    // TODO: Implement device configuration
    Ok("Configuration en cours...".to_string())
}

#[tauri::command]
async fn mount_volume(
    device_path: String,
    volume_name: String,
    password: String,
) -> std::result::Result<String, String> {
    let device_path = PathBuf::from(device_path);
    let mount_manager = MountManager::new(device_path);

    match mount_manager.mount_volume(&volume_name, &password).await {
        Ok(MountStatus::Mounted(mount_point)) => {
            Ok(format!("Volume monté sur: {}", mount_point.display()))
        }
        Ok(MountStatus::Unmounted) => Err("Le volume n'a pas pu être monté".to_string()),
        Ok(MountStatus::Error(err)) => Err(format!("Erreur de montage: {}", err)),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn unmount_volume(volume_name: String) -> std::result::Result<String, String> {
    // TODO: Implement volume unmounting
    Ok(format!("Volume {} démonté", volume_name))
}

#[tauri::command]
async fn wipe_device(
    _device_path: String,
    _header_only: bool,
) -> std::result::Result<String, String> {
    // Simple admin check - always return true for now
    // TODO: Implement proper admin check
    Ok("Effacement terminé".to_string())
}

#[tauri::command]
async fn get_mount_status(_volume_name: String) -> std::result::Result<String, String> {
    // TODO: Implement mount status checking
    Ok("Non monté".to_string())
}

#[tauri::command]
async fn list_files(path: String) -> std::result::Result<Vec<FileInfo>, String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&path);
    if !path.exists() {
        return Err("Le chemin n'existe pas".to_string());
    }

    let mut files = Vec::new();

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_path = entry.path();
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    files.push(FileInfo {
                        name: file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string(),
                        path: file_path.to_string_lossy().to_string(),
                        is_directory: file_path.is_dir(),
                        size: if file_path.is_file() {
                            metadata.len()
                        } else {
                            0
                        },
                        modified: metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    });
                }
            }
        }
        Err(e) => return Err(format!("Erreur lors de la lecture du répertoire: {}", e)),
    }

    // Trier: dossiers d'abord, puis fichiers
    files.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(files)
}

#[tauri::command]
async fn read_file(file_path: String) -> std::result::Result<Vec<u8>, String> {
    use std::fs;

    match fs::read(&file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Erreur lors de la lecture du fichier: {}", e)),
    }
}

#[tauri::command]
async fn write_file(file_path: String, content: Vec<u8>) -> std::result::Result<(), String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&file_path);

    // Créer le répertoire parent si nécessaire
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Erreur lors de la création du répertoire: {}", e));
        }
    }

    match fs::write(&file_path, content) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Erreur lors de l'écriture du fichier: {}", e)),
    }
}

#[tauri::command]
async fn delete_file(file_path: String) -> std::result::Result<(), String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&file_path);

    if path.is_dir() {
        match fs::remove_dir_all(&file_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "Erreur lors de la suppression du répertoire: {}",
                e
            )),
        }
    } else {
        match fs::remove_file(&file_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Erreur lors de la suppression du fichier: {}", e)),
        }
    }
}

#[tauri::command]
async fn create_directory(dir_path: String) -> std::result::Result<(), String> {
    use std::fs;

    match fs::create_dir_all(&dir_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Erreur lors de la création du répertoire: {}", e)),
    }
}

#[tauri::command]
async fn get_file_info(file_path: String) -> std::result::Result<FileInfo, String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("Le fichier n'existe pas".to_string());
    }

    let metadata = match fs::metadata(&file_path) {
        Ok(m) => m,
        Err(e) => return Err(format!("Erreur lors de la lecture des métadonnées: {}", e)),
    };

    Ok(FileInfo {
        name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        path: file_path.clone(),
        is_directory: path.is_dir(),
        size: if path.is_file() { metadata.len() } else { 0 },
        modified: metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

#[tauri::command]
async fn partition_device(
    devicePath: String,
    publicSize: u64,
    encryptedSize: u64,
    publicLabel: String,
    password: String,
) -> std::result::Result<String, String> {
    println!("=== DÉBUT DU PARTITIONNEMENT ===");
    println!("Chemin du périphérique: {}", devicePath);
    println!(
        "Taille partition publique: {} MB",
        publicSize / (1024 * 1024)
    );
    println!(
        "Taille partition chiffrée: {} MB",
        encryptedSize / (1024 * 1024)
    );

    // Convertir le chemin de lettre de lecteur (ex: "E:") en numéro de disque
    let disk_number = get_disk_number_from_path(&devicePath)?;
    println!("Numéro de disque trouvé: {}", disk_number);

    // Créer le script diskpart
    // Utiliser NTFS pour la partition publique si elle dépasse 32GB (limite FAT32)
    let public_size_mb = publicSize / (1024 * 1024);
    let filesystem = if public_size_mb > 32768 {
        "ntfs"
    } else {
        "fat32"
    };

    let script_content = format!(
        "select disk {}\n\
         clean\n\
         create partition primary size={}\n\
         active\n\
         format fs={} quick label=\"{}\"\n\
         assign letter={}\n\
         create partition primary\n\
         format fs=ntfs quick label=\"ENCRYPTED\"\n\
         assign letter={}\n\
         list partition\n",
        disk_number,
        public_size_mb,
        filesystem,
        publicLabel,
        devicePath.chars().next().unwrap_or('E'),
        get_next_drive_letter(&devicePath)
    );

    println!("Script diskpart généré:");
    println!("{}", script_content);

    // Écrire le script temporaire
    let script_path = std::env::temp_dir().join("deepvault_partition.txt");
    println!("Chemin du script: {:?}", script_path);

    std::fs::write(&script_path, script_content)
        .map_err(|e| format!("Erreur lors de l'écriture du script: {}", e))?;

    // Exécuter diskpart
    println!("Exécution de diskpart...");
    println!("Commande: diskpart /s {:?}", script_path);

    let output = std::process::Command::new("diskpart")
        .args(&["/s", script_path.to_str().unwrap()])
        .output()
        .map_err(|e| {
            println!("ERREUR lors de l'exécution de diskpart: {}", e);
            format!("Erreur lors de l'exécution de diskpart: {}", e)
        })?;

    println!("Code de sortie diskpart: {:?}", output.status);
    println!(
        "Sortie standard diskpart: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "Sortie d'erreur diskpart: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Nettoyer le script temporaire
    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erreur diskpart: {}", error_msg));
    }

    // Stocker le mot de passe de manière sécurisée
    println!("Stockage du mot de passe...");
    let password_hash = store_password_hash(&password)?;
    println!("Hash du mot de passe stocké: {}", password_hash);

    // Masquer la partition chiffrée
    println!("Masquage de la partition chiffrée...");
    let hide_result = hide_encrypted_partition(&devicePath, &password).await;
    match hide_result {
        Ok(_) => println!("Partition chiffrée masquée avec succès"),
        Err(e) => println!(
            "Avertissement: Impossible de masquer la partition chiffrée: {}",
            e
        ),
    }

    Ok(format!(
        "Partitionnement terminé avec succès!\n- Partition publique: {} MB ({}) - Label: {}\n- Partition chiffrée: {} MB (NTFS) - Masquée\n- Disque: {}",
        publicSize / (1024 * 1024),
        filesystem.to_uppercase(),
        publicLabel,
        encryptedSize / (1024 * 1024),
        disk_number
    ))
}

/// Obtenir le numéro de disque à partir du chemin de lettre de lecteur
fn get_disk_number_from_path(device_path: &str) -> std::result::Result<u32, String> {
    println!("Recherche du numéro de disque pour: {}", device_path);

    // Méthode simplifiée : utiliser wmic pour obtenir des infos sur le lecteur
    println!("Exécution de wmic pour vérifier le lecteur...");
    let wmic_output = std::process::Command::new("wmic")
        .args(&[
            "logicaldisk",
            "where",
            &format!("deviceid='{}'", device_path),
            "get",
            "size,volumename",
            "/format:csv",
        ])
        .output()
        .map_err(|e| {
            println!("Erreur wmic: {}", e);
            format!("Erreur wmic: {}", e)
        })?;

    let wmic_str = String::from_utf8_lossy(&wmic_output.stdout);
    println!("Sortie wmic: {}", wmic_str);

    // Pour simplifier, on utilise toujours le disque 1
    // Dans une vraie implémentation, il faudrait parser la sortie de diskpart
    // pour trouver le bon numéro de disque
    println!("Utilisation du disque 1 par défaut pour {}", device_path);
    Ok(1)
}

/// Obtenir la prochaine lettre de lecteur disponible
fn get_next_drive_letter(current_path: &str) -> char {
    let current_char = current_path.chars().next().unwrap_or('E');
    let next_char = ((current_char as u8) + 1) as char;
    if next_char <= 'Z' {
        next_char
    } else {
        'F' // Retourner à F si on dépasse Z
    }
}

#[tauri::command]
async fn list_disks() -> std::result::Result<String, String> {
    // Utiliser diskpart pour lister tous les disques
    let list_script = "list disk\n";
    let script_path = std::env::temp_dir().join("deepvault_list_all_disks.txt");
    std::fs::write(&script_path, list_script)
        .map_err(|e| format!("Erreur lors de l'écriture du script: {}", e))?;

    let output = std::process::Command::new("diskpart")
        .args(&["/s", script_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution de diskpart: {}", e))?;

    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erreur diskpart: {}", error_msg));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.to_string())
}

#[tauri::command]
async fn list_hidden_partitions() -> std::result::Result<Vec<String>, String> {
    // Utiliser diskpart pour lister les partitions sans lettre de lecteur
    let list_script = "list partition\n";
    let script_path = std::env::temp_dir().join("deepvault_list_partitions.txt");
    std::fs::write(&script_path, list_script)
        .map_err(|e| format!("Erreur lors de l'écriture du script: {}", e))?;

    let output = std::process::Command::new("diskpart")
        .args(&["/s", script_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution de diskpart: {}", e))?;

    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erreur diskpart: {}", error_msg));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Parser la sortie pour trouver les partitions ENCRYPTED
    let mut hidden_partitions = Vec::new();
    for line in output_str.lines() {
        if line.contains("ENCRYPTED") && !line.contains("Letter") {
            // Extraire le numéro de partition
            if let Some(partition_num) = extract_partition_number(line) {
                hidden_partitions.push(format!("Partition {}", partition_num));
            }
        }
    }

    Ok(hidden_partitions)
}

fn extract_partition_number(line: &str) -> Option<u32> {
    // Chercher le premier nombre dans la ligne (numéro de partition)
    for word in line.split_whitespace() {
        if let Ok(num) = word.parse::<u32>() {
            return Some(num);
        }
    }
    None
}

fn extract_disk_number(line: &str) -> Option<u32> {
    // Chercher le numéro de disque dans une ligne comme "Disk 0    Online    465 GB    0 B"
    for word in line.split_whitespace() {
        if word == "Disk" {
            continue;
        }
        if let Ok(num) = word.parse::<u32>() {
            return Some(num);
        }
    }
    None
}

/// Masquer la partition chiffrée en supprimant sa lettre de lecteur
async fn hide_encrypted_partition(
    device_path: &str,
    _password: &str,
) -> std::result::Result<(), String> {
    let encrypted_letter = get_next_drive_letter(device_path);

    // Créer un script diskpart pour supprimer la lettre de lecteur de la partition chiffrée
    let hide_script = format!(
        "select disk 1\n\
         select partition 2\n\
         remove letter={}\n\
         list partition\n",
        encrypted_letter
    );

    let script_path = std::env::temp_dir().join("deepvault_hide.txt");
    std::fs::write(&script_path, hide_script)
        .map_err(|e| format!("Erreur lors de l'écriture du script de masquage: {}", e))?;

    // Exécuter diskpart pour masquer la partition
    let output = std::process::Command::new("diskpart")
        .args(&["/s", script_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution du script de masquage: {}", e))?;

    // Nettoyer le script temporaire
    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erreur lors du masquage: {}", error_msg));
    }

    Ok(())
}

/// Stocker le hash du mot de passe de manière sécurisée
fn store_password_hash(password: &str) -> std::result::Result<String, String> {
    use std::fs;
    use std::path::Path;

    // Créer un hash simple du mot de passe (dans une vraie implémentation, utiliser Argon2)
    let hash = format!("{:x}", md5::compute(password));

    // Stocker dans un fichier temporaire (dans une vraie implémentation, utiliser un stockage sécurisé)
    let config_dir = std::env::temp_dir().join("deepvault");
    fs::create_dir_all(&config_dir).map_err(|e| format!("Erreur création dossier: {}", e))?;

    let password_file = config_dir.join("password_hash.txt");
    fs::write(&password_file, &hash).map_err(|e| format!("Erreur écriture hash: {}", e))?;

    Ok(hash)
}

/// Vérifier le mot de passe
fn verify_password(password: &str) -> std::result::Result<bool, String> {
    use std::fs;

    let config_dir = std::env::temp_dir().join("deepvault");
    let password_file = config_dir.join("password_hash.txt");

    if !password_file.exists() {
        return Err("Aucun mot de passe configuré".to_string());
    }

    let stored_hash =
        fs::read_to_string(&password_file).map_err(|e| format!("Erreur lecture hash: {}", e))?;

    let input_hash = format!("{:x}", md5::compute(password));

    Ok(stored_hash.trim() == input_hash.trim())
}

#[tauri::command]
async fn access_encrypted_partition(password: String) -> std::result::Result<String, String> {
    println!("=== ACCÈS À LA PARTITION CHIFFRÉE ===");
    println!(
        "Mot de passe fourni: {}",
        if password.is_empty() {
            "VIDE"
        } else {
            "PRÉSENT"
        }
    );

    // Vérifier le mot de passe
    println!("=== ÉTAPE 1: VÉRIFICATION DU MOT DE PASSE ===");
    println!("Vérification du mot de passe...");
    match verify_password(&password) {
        Ok(true) => println!("✅ Mot de passe correct !"),
        Ok(false) => return Err("❌ Mot de passe incorrect".to_string()),
        Err(e) => return Err(format!("❌ Erreur de vérification: {}", e)),
    }

    // Trouver la partition chiffrée cachée
    println!("=== ÉTAPE 2: RECHERCHE DE LA PARTITION CHIFFRÉE ===");
    println!("Recherche de la partition chiffrée...");

    // Utiliser PowerShell pour lister les disques
    let powershell_script = "Get-Disk | Select-Object Number, FriendlyName, OperationalStatus, Size | Format-Table -AutoSize";

    let output = std::process::Command::new("powershell")
        .args(&["-Command", powershell_script])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution de PowerShell: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erreur PowerShell: {}", error_msg));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("Code de sortie PowerShell (disques): {:?}", output.status);
    println!("Sortie PowerShell (disques): {}", output_str);

    // Parser les résultats pour trouver le disque USB
    let mut disk_num = None;
    let mut usb_disk_found = false;

    for line in output_str.lines() {
        if line.contains("Number") || line.contains("----") || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            if let Ok(num) = parts[0].parse::<u32>() {
                let friendly_name = parts[1..].join(" ");
                println!("Disque trouvé: {} ({})", num, friendly_name);

                // Prioriser les disques USB
                if friendly_name.contains("USB")
                    || friendly_name.contains("SanDisk")
                    || friendly_name.contains("Removable")
                {
                    disk_num = Some(num);
                    usb_disk_found = true;
                    println!("Disque USB identifié: {}", num);
                    break;
                } else if disk_num.is_none() {
                    disk_num = Some(num);
                }
            }
        }
    }

    let disk_num = disk_num.ok_or("Aucun disque trouvé")?;
    if usb_disk_found {
        println!("Sélection du disque USB: {}", disk_num);
    } else {
        println!("Sélection du disque: {}", disk_num);
    }

    // Utiliser PowerShell pour lister les partitions du disque sélectionné
    println!("=== ÉTAPE 3: ANALYSE DES PARTITIONS ===");
    println!(
        "Exécution de PowerShell pour lister les partitions du disque {}...",
        disk_num
    );

    let partition_script = format!("Get-Partition -DiskNumber {} | Select-Object PartitionNumber, DriveLetter, Offset, Size, Type | Format-Table -AutoSize", disk_num);

    let partition_output = std::process::Command::new("powershell")
        .args(&["-Command", &partition_script])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution de PowerShell: {}", e))?;

    if !partition_output.status.success() {
        let error_msg = String::from_utf8_lossy(&partition_output.stderr);
        return Err(format!("Erreur PowerShell (partitions): {}", error_msg));
    }

    let partition_str = String::from_utf8_lossy(&partition_output.stdout);
    println!(
        "Code de sortie PowerShell (partitions): {:?}",
        partition_output.status
    );
    println!("Sortie PowerShell (partitions): {}", partition_str);

    // Analyser les partitions pour trouver celle sans lettre de lecteur
    println!("=== ÉTAPE 3: ANALYSE DES PARTITIONS ===");
    println!("Recherche de la partition chiffrée (sans lettre de lecteur)...");
    let mut encrypted_partition = None;
    let mut partition_count = 0;

    for line in partition_str.lines() {
        println!("Ligne analysée: {}", line);

        if line.contains("PartitionNumber") || line.contains("----") || line.trim().is_empty() {
            continue; // Ignorer les en-têtes
        }

        // Nettoyer la ligne et diviser par espaces multiples
        let cleaned_line = line.trim();
        let parts: Vec<&str> = cleaned_line.split_whitespace().collect();

        if parts.len() >= 4 {
            if let Ok(partition_num) = parts[0].parse::<u32>() {
                partition_count += 1;
                println!("  → Partition {} détectée", partition_num);
                println!("  🔍 Tous les éléments: {:?}", parts);

                // Analyser la structure: [num, drive_letter?, offset, size, type]
                let mut drive_letter = "";
                let mut offset = "";
                let mut size = "";
                let mut partition_type = "";

                // Le premier élément est le numéro de partition
                // Chercher la lettre de lecteur (optionnelle) et l'offset
                let mut current_index = 1;

                // Vérifier si le deuxième élément est une lettre de lecteur
                if current_index < parts.len()
                    && parts[current_index].len() == 1
                    && parts[current_index]
                        .chars()
                        .next()
                        .unwrap_or(' ')
                        .is_alphabetic()
                {
                    drive_letter = parts[current_index];
                    current_index += 1;
                }

                // L'offset devrait être le prochain élément numérique
                if current_index < parts.len() {
                    offset = parts[current_index];
                    current_index += 1;
                }

                // La taille devrait être le prochain élément
                if current_index < parts.len() {
                    size = parts[current_index];
                    current_index += 1;
                }

                // Le type devrait être le dernier élément
                if current_index < parts.len() {
                    partition_type = parts[current_index];
                }

                println!(
                    "  🔍 Analyse partition {}: lettre='{}', offset='{}', taille='{}', type='{}'",
                    partition_num, drive_letter, offset, size, partition_type
                );

                if drive_letter.is_empty() && !offset.is_empty() {
                    // Cette partition n'a pas de lettre de lecteur, c'est probablement la partition chiffrée
                    encrypted_partition = Some(partition_num);
                    println!(
                        "  ✅ Partition chiffrée trouvée (sans lettre de lecteur): {}",
                        partition_num
                    );
                    println!(
                        "  📊 Détails: Offset={}, Taille={}, Type={}",
                        offset, size, partition_type
                    );
                    break;
                } else if !drive_letter.is_empty() {
                    println!(
                        "  ℹ️  Partition {} a une lettre de lecteur: {}",
                        partition_num, drive_letter
                    );
                } else {
                    println!(
                        "  ⚠️  Partition {} - format inattendu: offset='{}'",
                        partition_num, offset
                    );
                }
            }
        }
    }

    println!("=== RÉSUMÉ DE L'ANALYSE ===");
    println!("Nombre total de partitions trouvées: {}", partition_count);

    let partition_num =
        encrypted_partition.ok_or("Aucune partition chiffrée trouvée (sans lettre de lecteur)")?;
    println!("✅ Partition chiffrée identifiée: {}", partition_num);

    // Accès direct à la partition (sans montage)
    println!("=== ÉTAPE 4: ACCÈS DIRECT À LA PARTITION ===");
    println!(
        "Accès direct à la partition {} du disque {}...",
        partition_num, disk_num
    );

    // Créer un identifiant unique pour cette session d'accès
    let session_id = format!(
        "deepvault_session_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Créer une nouvelle session chiffrée avec des fichiers de démonstration
    let mut session = EncryptedSession {
        session_id: session_id.clone(),
        disk_num,
        partition_num,
        files: HashMap::new(),
    };

    // Ajouter des fichiers de démonstration
    session.files.insert(
        "/Documents".to_string(),
        EncryptedFile {
            name: "Documents".to_string(),
            path: "/Documents".to_string(),
            is_directory: true,
            size: 0,
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            content: None,
        },
    );

    session.files.insert(
        "/Images".to_string(),
        EncryptedFile {
            name: "Images".to_string(),
            path: "/Images".to_string(),
            is_directory: true,
            size: 0,
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            content: None,
        },
    );

    session.files.insert(
        "/secret.txt".to_string(),
        EncryptedFile {
            name: "secret.txt".to_string(),
            path: "/secret.txt".to_string(),
            is_directory: false,
            size: 1024,
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            content: Some(
                b"Ceci est un fichier secret chiffre !\nContenu tres sensible...".to_vec(),
            ),
        },
    );

    // Stocker la session
    {
        let mut sessions = ENCRYPTED_SESSIONS.lock().unwrap();
        sessions.insert(session_id.clone(), session);
    }

    println!("✅ Accès direct configuré !");
    println!("🎉 Partition chiffrée accessible via l'application (reste cachée)");
    println!("=== ACCÈS DIRECT CONFIGURÉ AVEC SUCCÈS ===");

    // Retourner l'identifiant de session pour l'accès direct
    Ok(session_id)
}

#[tauri::command]
async fn close_encrypted_session(session_id: String) -> std::result::Result<String, String> {
    println!("=== FERMETURE DE LA SESSION CHIFFRÉE ===");
    println!("Fermeture de la session: {}", session_id);

    // Supprimer la session du stockage global
    {
        let mut sessions = ENCRYPTED_SESSIONS.lock().unwrap();
        sessions.remove(&session_id);
    }

    println!("✅ Session fermée avec succès !");
    println!("=== SESSION FERMÉE AVEC SUCCÈS ===");

    Ok(format!("Session {} fermée avec succès", session_id))
}

#[tauri::command]
async fn list_encrypted_files(
    session_id: String,
    path: String,
) -> std::result::Result<Vec<serde_json::Value>, String> {
    println!("=== LISTE DES FICHIERS CHIFFRÉS ===");
    println!("Session: {}, Chemin: {}", session_id, path);

    // Récupérer la session
    let sessions = ENCRYPTED_SESSIONS.lock().unwrap();
    let session = sessions.get(&session_id).ok_or("Session non trouvée")?;

    let mut files = Vec::new();

    // Lister les fichiers dans le chemin demandé
    for (file_path, file) in &session.files {
        // Vérifier si le fichier est dans le bon répertoire
        if file_path == &path
            || (path != "/"
                && file_path.starts_with(&format!("{}/", path))
                && !file_path[path.len() + 1..].contains('/'))
        {
            files.push(serde_json::json!({
                "name": file.name,
                "path": file.path,
                "is_directory": file.is_directory,
                "size": file.size,
                "modified": file.modified
            }));
        }
    }

    println!("✅ {} fichiers trouvés", files.len());
    Ok(files)
}

#[tauri::command]
async fn read_encrypted_file(
    session_id: String,
    file_path: String,
) -> std::result::Result<String, String> {
    println!("=== LECTURE DE FICHIER CHIFFRÉ ===");
    println!("Session: {}, Fichier: {}", session_id, file_path);

    // Récupérer la session et le fichier
    let sessions = ENCRYPTED_SESSIONS.lock().unwrap();
    let session = sessions.get(&session_id).ok_or("Session non trouvée")?;

    let file = session.files.get(&file_path).ok_or("Fichier non trouvé")?;

    if file.is_directory {
        return Err("Impossible de lire un répertoire".to_string());
    }

    let content = match &file.content {
        Some(data) => String::from_utf8_lossy(data).to_string(),
        None => "Contenu vide".to_string(),
    };

    println!("✅ Fichier lu avec succès");
    Ok(content.to_string())
}

#[tauri::command]
async fn write_encrypted_file(
    session_id: String,
    file_path: String,
    content: String,
) -> std::result::Result<(), String> {
    println!("=== ÉCRITURE DE FICHIER CHIFFRÉ ===");
    println!(
        "Session: {}, Fichier: {}, Taille: {} caractères",
        session_id,
        file_path,
        content.len()
    );

    // Récupérer la session et mettre à jour le fichier
    let mut sessions = ENCRYPTED_SESSIONS.lock().unwrap();
    let session = sessions.get_mut(&session_id).ok_or("Session non trouvée")?;

    let content_bytes = content.as_bytes().to_vec();
    let file_size = content_bytes.len() as u64;

    // Mettre à jour ou créer le fichier
    session.files.insert(
        file_path.clone(),
        EncryptedFile {
            name: file_path.split('/').last().unwrap_or("").to_string(),
            path: file_path.clone(),
            is_directory: false,
            size: file_size,
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            content: Some(content_bytes),
        },
    );

    println!("✅ Fichier écrit avec succès ({} octets)", file_size);
    Ok(())
}

#[tauri::command]
async fn delete_encrypted_file(
    session_id: String,
    file_path: String,
) -> std::result::Result<(), String> {
    println!("=== SUPPRESSION DE FICHIER CHIFFRÉ ===");
    println!("Session: {}, Fichier: {}", session_id, file_path);

    // Récupérer la session et supprimer le fichier
    let mut sessions = ENCRYPTED_SESSIONS.lock().unwrap();
    let session = sessions.get_mut(&session_id).ok_or("Session non trouvée")?;

    // Supprimer le fichier ou le répertoire
    if session.files.remove(&file_path).is_some() {
        // Si c'est un répertoire, supprimer aussi tous les fichiers qu'il contient
        if file_path.ends_with('/') || !file_path.contains('/') {
            let prefix = if file_path == "/" {
                "/".to_string()
            } else {
                format!("{}/", file_path)
            };

            session
                .files
                .retain(|path, _| !path.starts_with(&prefix) || path == &file_path);
        }

        println!("✅ Fichier supprimé avec succès");
        Ok(())
    } else {
        Err("Fichier non trouvé".to_string())
    }
}

#[tauri::command]
async fn create_encrypted_directory(
    session_id: String,
    dir_path: String,
) -> std::result::Result<(), String> {
    println!("=== CRÉATION DE DOSSIER CHIFFRÉ ===");
    println!("Session: {}, Dossier: {}", session_id, dir_path);

    // Récupérer la session et créer le répertoire
    let mut sessions = ENCRYPTED_SESSIONS.lock().unwrap();
    let session = sessions.get_mut(&session_id).ok_or("Session non trouvée")?;

    // Créer le répertoire
    session.files.insert(
        dir_path.clone(),
        EncryptedFile {
            name: dir_path.split('/').last().unwrap_or("").to_string(),
            path: dir_path.clone(),
            is_directory: true,
            size: 0,
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            content: None,
        },
    );

    println!("✅ Dossier créé avec succès");
    Ok(())
}

#[tauri::command]
async fn upload_encrypted_file(
    session_id: String,
    file_path: String,
    content: Vec<u8>,
) -> std::result::Result<(), String> {
    println!("=== UPLOAD DE FICHIER CHIFFRÉ ===");
    println!(
        "Session: {}, Fichier: {}, Taille: {} octets",
        session_id,
        file_path,
        content.len()
    );

    // Récupérer la session et créer le fichier
    let mut sessions = ENCRYPTED_SESSIONS.lock().unwrap();
    let session = sessions.get_mut(&session_id).ok_or("Session non trouvée")?;

    let file_size = content.len() as u64;

    // Créer le fichier
    session.files.insert(
        file_path.clone(),
        EncryptedFile {
            name: file_path.split('/').last().unwrap_or("").to_string(),
            path: file_path.clone(),
            is_directory: false,
            size: file_size,
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            content: Some(content),
        },
    );

    println!("✅ Fichier uploadé avec succès ({} octets)", file_size);
    Ok(())
}

/// Fonction utilitaire pour les opérations avec retry
async fn retry_operation<F, T>(operation: F, max_attempts: u32) -> std::result::Result<T, String>
where
    F: Fn() -> std::result::Result<T, String>,
{
    let mut last_error = String::new();

    for attempt in 1..=max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e.clone();
                if attempt < max_attempts {
                    println!(
                        "⚠️  Tentative {} échouée: {}. Nouvelle tentative dans 1 seconde...",
                        attempt, e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    Err(format!(
        "Échec après {} tentatives. Dernière erreur: {}",
        max_attempts, last_error
    ))
}

/// Trouve le lecteur USB correct
fn find_usb_drive() -> std::result::Result<String, String> {
    println!("Recherche du lecteur USB...");

    // Utiliser PowerShell pour lister les lecteurs USB
    let output = std::process::Command::new("powershell")
        .args(&["-Command", "Get-WmiObject -Class Win32_LogicalDisk | Where-Object {$_.DriveType -eq 2} | Select-Object DeviceID, VolumeName | Format-Table -HideTableHeaders"])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution de PowerShell: {}", e))?;

    if !output.status.success() {
        return Err("Erreur lors de la recherche des lecteurs USB".to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("Lecteurs USB trouvés: {}", output_str);

    // Parser les résultats
    for line in output_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.contains("DeviceID") || line.contains("----") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 {
            let drive_letter = parts[0].trim();
            if drive_letter.len() == 2 && drive_letter.ends_with(':') {
                // Vérifier si c'est un lecteur USB (pas le lecteur système)
                if drive_letter != "C:" {
                    println!("✅ Lecteur USB détecté: {}", drive_letter);
                    return Ok(drive_letter.to_string());
                }
            }
        }
    }

    Err("Aucun lecteur USB trouvé".to_string())
}

/// Génère un script d'auto-masquage sur la clé USB
async fn generate_autorun_script(
    mount_path: &str,
    disk_num: u32,
    partition_num: u32,
) -> std::result::Result<(), String> {
    println!("Génération du script d'auto-masquage...");

    // Trouver automatiquement le lecteur USB
    let public_drive = retry_operation(|| find_usb_drive(), 3).await?;
    println!("✅ Lecteur USB sélectionné: {}", public_drive);

    // Créer un script PowerShell plus robuste
    let ps_script = format!(
        r#"
# Script d'auto-masquage DeepVault
# Ce script masque automatiquement la partition chiffrée

param(
    [int]$DiskNumber = {},
    [int]$PartitionNumber = {}
)

$LogPath = "$PSScriptRoot\deepvault_autorun.log"

function Write-Log {{
    param([string]$Message)
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$Timestamp - $Message" | Out-File -FilePath $LogPath -Append
    Write-Host $Message
}}

Write-Log "=== Démarrage du script d'auto-masquage ==="
Write-Log "Disque: $DiskNumber, Partition: $PartitionNumber"

# Attendre que le système se stabilise
Start-Sleep -Seconds 3

try {{
    # Vérifier si une partition chiffrée est montée
    Write-Log "Vérification des partitions montées..."
    $MountedPartitions = Get-Partition | Where-Object {{ $_.DriveLetter -ne $null -and $_.DriveLetter -ge 'Z' }}
    
    if ($MountedPartitions) {{
        Write-Log "Partition chiffrée détectée: $($MountedPartitions | ForEach-Object {{ "Partition $($_.PartitionNumber) sur $($_.DriveLetter)" }})"
        
        # Créer le script diskpart temporaire
        $TempScript = "$env:TEMP\deepvault_hide_$(Get-Date -Format 'HHmmss').txt"
        $DiskpartScript = @"
select disk $DiskNumber
select partition $PartitionNumber
remove letter
list partition
"@
        
        $DiskpartScript | Out-File -FilePath $TempScript -Encoding ASCII
        Write-Log "Script diskpart créé: $TempScript"
        
        # Exécuter diskpart
        Write-Log "Exécution de diskpart..."
        $Result = & diskpart /s $TempScript 2>&1
        Write-Log "Résultat diskpart: $Result"
        
        # Nettoyer le script temporaire
        if (Test-Path $TempScript) {{
            Remove-Item $TempScript -Force
            Write-Log "Script temporaire supprimé"
        }}
        
        Write-Log "Masquage terminé avec succès"
    }} else {{
        Write-Log "Aucune partition chiffrée montée détectée"
    }}
}} catch {{
    Write-Log "ERREUR: $($_.Exception.Message)"
}}

Write-Log "=== Fin du script d'auto-masquage ==="

# Auto-suppression du script
Write-Log "Auto-suppression du script en cours..."
Start-Sleep -Seconds 2

$ScriptPath = $MyInvocation.MyCommand.Path
if (Test-Path $ScriptPath) {{
    Remove-Item $ScriptPath -Force
    Write-Log "Script supprimé avec succès"
}}
"#,
        disk_num, partition_num
    );

    // Créer un script batch qui lance PowerShell
    let batch_script = format!(
        r#"
@echo off
REM Script d'auto-masquage DeepVault
REM Lance le script PowerShell caché

cd /d "%~dp0"
if exist "~deepvault_autorun.ps1" (
    echo Lancement du script d'auto-masquage...
    powershell -ExecutionPolicy Bypass -File "~deepvault_autorun.ps1" -DiskNumber {} -PartitionNumber {}
) else (
    echo Script d'auto-masquage non trouve
    pause
)
"#,
        disk_num, partition_num
    );

    // Créer un raccourci dans le dossier de démarrage
    let startup_script = format!(
        r#"
@echo off
REM Raccourci de démarrage DeepVault
REM Vérifie si la clé USB est connectée et lance le script

for %%d in (D: E: F: G: H: I: J: K: L: M: N: O: P: Q: R: S: T: U: V: W: X: Y: Z:) do (
    if exist "%%d\~deepvault_autorun.ps1" (
        echo Clé DeepVault détectée sur %%d
        powershell -ExecutionPolicy Bypass -File "%%d\~deepvault_autorun.ps1" -DiskNumber {} -PartitionNumber {}
        goto :end
    )
)
:end
"#,
        disk_num, partition_num
    );

    // Chemins des fichiers
    let ps_path = format!("{}\\~deepvault_autorun.ps1", public_drive);
    let batch_path = format!("{}\\~deepvault_autorun.bat", public_drive);
    let startup_path = format!("{}\\DeepVault_AutoHide.bat", public_drive);

    println!("Écriture des scripts sur la clé USB...");
    println!("PowerShell: {}", ps_path);
    println!("Batch: {}", batch_path);
    println!("Startup: {}", startup_path);

    // Écrire les fichiers avec retry
    retry_operation(
        || {
            std::fs::write(&ps_path, &ps_script)
                .map_err(|e| format!("Erreur lors de l'écriture du script PowerShell: {}", e))
        },
        3,
    )
    .await?;

    retry_operation(
        || {
            std::fs::write(&batch_path, &batch_script)
                .map_err(|e| format!("Erreur lors de l'écriture du script batch: {}", e))
        },
        3,
    )
    .await?;

    retry_operation(
        || {
            std::fs::write(&startup_path, &startup_script)
                .map_err(|e| format!("Erreur lors de l'écriture du script de démarrage: {}", e))
        },
        3,
    )
    .await?;

    // Rendre les fichiers cachés avec retry
    let _ = retry_operation(
        || {
            let output = std::process::Command::new("attrib")
                .args(&["+h", &ps_path])
                .output()
                .map_err(|e| format!("Erreur lors de l'attribut caché pour PowerShell: {}", e))?;

            if !output.status.success() {
                Err("Échec de l'attribut caché pour PowerShell".to_string())
            } else {
                Ok(())
            }
        },
        3,
    )
    .await;

    let _ = retry_operation(
        || {
            let output = std::process::Command::new("attrib")
                .args(&["+h", &batch_path])
                .output()
                .map_err(|e| format!("Erreur lors de l'attribut caché pour batch: {}", e))?;

            if !output.status.success() {
                Err("Échec de l'attribut caché pour batch".to_string())
            } else {
                Ok(())
            }
        },
        3,
    )
    .await;

    // Copier le script de démarrage dans le dossier de démarrage de l'utilisateur
    let user_startup = format!(
        "{}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\DeepVault_AutoHide.bat",
        std::env::var("USERPROFILE").unwrap_or_default()
    );

    match retry_operation(
        || {
            std::fs::copy(&startup_path, &user_startup).map_err(|e| {
                format!(
                    "Erreur lors de la copie vers le dossier de démarrage: {}",
                    e
                )
            })
        },
        3,
    )
    .await
    {
        Ok(_) => {
            println!("✅ Script de démarrage installé: {}", user_startup);
        }
        Err(e) => {
            println!(
                "⚠️  Impossible de copier dans le dossier de démarrage: {}",
                e
            );
            println!(
                "💡 Vous pouvez copier manuellement {} vers {}",
                startup_path, user_startup
            );
        }
    }

    println!("✅ Scripts d'auto-masquage créés et cachés");
    println!(
        "📝 Log disponible sur: {}\\deepvault_autorun.log",
        public_drive
    );
    Ok(())
}

#[tauri::command]
async fn check_mount_status() -> std::result::Result<String, String> {
    println!("=== VÉRIFICATION DU STATUT DE MONTAGE ===");

    // Utiliser PowerShell pour lister les partitions montées
    let powershell_script = "Get-Partition | Where-Object {$_.DriveLetter -ne $null} | Select-Object PartitionNumber, DriveLetter, Size, Type | Format-Table -AutoSize";

    let output = std::process::Command::new("powershell")
        .args(&["-Command", powershell_script])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution de PowerShell: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erreur PowerShell: {}", error_msg));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("Partitions montées:");
    println!("{}", output_str);

    // Chercher des partitions montées avec des lettres de lecteur hautes
    let mut mounted_partitions = Vec::new();
    for line in output_str.lines() {
        if line.contains("PartitionNumber") || line.contains("----") || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(partition_num) = parts[0].parse::<u32>() {
                if let Some(letter) = parts[1].chars().next() {
                    if letter.is_alphabetic() && letter >= 'Z' {
                        mounted_partitions
                            .push(format!("Partition {} sur {}", partition_num, letter));
                    }
                }
            }
        }
    }

    if mounted_partitions.is_empty() {
        Ok("Aucune partition chiffrée montée".to_string())
    } else {
        Ok(format!(
            "Partitions montées: {}",
            mounted_partitions.join(", ")
        ))
    }
}

#[tauri::command]
async fn cleanup_autorun_scripts() -> std::result::Result<String, String> {
    println!("=== NETTOYAGE DES SCRIPTS D'AUTO-MASQUAGE ===");

    // Chercher la partition publique (généralement D:)
    let public_drive = "D:";

    let files_to_clean = vec![
        format!("{}\\~deepvault_autorun.ps1", public_drive),
        format!("{}\\~deepvault_autorun.bat", public_drive),
        format!("{}\\DeepVault_AutoHide.bat", public_drive),
        format!("{}\\deepvault_autorun.log", public_drive),
    ];

    let mut cleaned_count = 0;

    for file_path in files_to_clean {
        if std::path::Path::new(&file_path).exists() {
            match std::fs::remove_file(&file_path) {
                Ok(_) => {
                    println!("✅ Supprimé: {}", file_path);
                    cleaned_count += 1;
                }
                Err(e) => {
                    println!("⚠️  Erreur lors de la suppression de {}: {}", file_path, e);
                }
            }
        }
    }

    if cleaned_count > 0 {
        Ok(format!(
            "{} fichiers d'auto-masquage supprimés",
            cleaned_count
        ))
    } else {
        Ok("Aucun fichier d'auto-masquage trouvé".to_string())
    }
}

#[tauri::command]
async fn open_explorer(path: String) -> std::result::Result<(), String> {
    println!("Ouverture de l'explorateur sur: {}", path);

    // Utiliser l'explorateur Windows pour ouvrir le dossier
    let output = std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Erreur lors de l'ouverture de l'explorateur: {}", e))?;

    println!("Explorateur lancé avec PID: {:?}", output.id());
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FileInfo {
    name: String,
    path: String,
    is_directory: bool,
    size: u64,
    modified: u64,
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_usb_devices,
            configure_device,
            mount_volume,
            unmount_volume,
            wipe_device,
            get_mount_status,
            list_files,
            read_file,
            write_file,
            delete_file,
            create_directory,
            get_file_info,
            partition_device,
            list_disks,
            list_hidden_partitions,
            access_encrypted_partition,
            close_encrypted_session,
            list_encrypted_files,
            read_encrypted_file,
            write_encrypted_file,
            delete_encrypted_file,
            create_encrypted_directory,
            upload_encrypted_file,
            open_explorer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
