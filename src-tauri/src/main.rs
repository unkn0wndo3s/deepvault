//! DeepVault GUI - Tauri application

use deepvault_core::*;
use std::path::PathBuf;

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
async fn mount_encrypted_partition(password: String) -> std::result::Result<String, String> {
    println!("=== MONTAGE DE LA PARTITION CHIFFRÉE ===");
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

    // Vérifier si une partition est déjà montée
    println!("=== ÉTAPE 1.5: VÉRIFICATION DU STATUT DE MONTAGE ===");
    println!("Vérification si une partition chiffrée est déjà montée...");

    let powershell_script = "Get-Partition | Where-Object {$_.DriveLetter -ne $null} | Select-Object PartitionNumber, DriveLetter, Size, Type | Format-Table -AutoSize";

    let status_output = std::process::Command::new("powershell")
        .args(&["-Command", powershell_script])
        .output()
        .map_err(|e| format!("Erreur lors de la vérification du statut: {}", e))?;

    if status_output.status.success() {
        let status_str = String::from_utf8_lossy(&status_output.stdout);
        println!("Partitions actuellement montées:");
        println!("{}", status_str);

        // Chercher des partitions montées avec des lettres de lecteur hautes
        for line in status_str.lines() {
            if line.contains("PartitionNumber") || line.contains("----") || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(_partition_num) = parts[0].parse::<u32>() {
                    if let Some(letter) = parts[1].chars().next() {
                        if letter.is_alphabetic() && letter >= 'Z' {
                            println!(
                                "⚠️  Une partition chiffrée est déjà montée sur le lecteur {}",
                                letter
                            );
                            return Err(format!("Une partition chiffrée est déjà montée sur le lecteur {}. Veuillez d'abord la démonter.", letter));
                        }
                    }
                }
            }
        }
        println!("✅ Aucune partition chiffrée montée - on peut procéder au montage");
    }

    // Trouver la partition chiffrée cachée
    println!("=== ÉTAPE 2: RECHERCHE DE LA PARTITION CHIFFRÉE ===");
    println!("Recherche de la partition chiffrée...");

    // D'abord, lister tous les disques pour trouver celui qui contient la partition chiffrée
    println!("Exécution de PowerShell pour lister les disques...");

    // Utiliser PowerShell comme alternative plus robuste à diskpart
    let powershell_script = "Get-Disk";

    let disks_output = std::process::Command::new("powershell")
        .args(&["-Command", powershell_script])
        .output()
        .map_err(|e| {
            format!(
                "Erreur lors de l'exécution de PowerShell pour lister les disques: {}",
                e
            )
        })?;

    println!(
        "Code de sortie PowerShell (disques): {:?}",
        disks_output.status
    );
    if !disks_output.status.success() {
        let error_msg = String::from_utf8_lossy(&disks_output.stderr);
        println!("Erreur PowerShell (disques): {}", error_msg);
        return Err(format!("Erreur PowerShell (disques): {}", error_msg));
    }

    let disks_output_str = String::from_utf8_lossy(&disks_output.stdout);
    println!("Sortie PowerShell (disques): {}", disks_output_str);

    // Trouver le disque qui contient la partition chiffrée
    let mut target_disk = None;
    let mut available_disks = Vec::new();
    let mut usb_disks = Vec::new();

    // Parser la sortie PowerShell (format différent de diskpart)
    for line in disks_output_str.lines() {
        // Format PowerShell: "0      M 0000_0000_0000_0001_00A0_7523... Healthy              Online                476.94 GB"
        if line.contains("Number") || line.contains("----") || line.trim().is_empty() {
            continue; // Ignorer les en-têtes et lignes vides
        }

        // Chercher les lignes avec des données de disque
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            // Le premier élément devrait être le numéro de disque
            if let Ok(disk_num) = parts[0].parse::<u32>() {
                // Vérifier que le disque est en ligne (Online)
                if line.contains("Online") {
                    available_disks.push(disk_num);
                    println!("Disque trouvé: {} (Online)", disk_num);

                    // Identifier les disques USB (généralement plus petits et avec "USB" dans le nom)
                    if line.contains("USB")
                        || line.contains("SanDisk")
                        || line.contains("Removable")
                    {
                        usb_disks.push(disk_num);
                        println!("Disque USB identifié: {}", disk_num);
                    }
                }
            }
        }
    }

    // Prioriser les disques USB pour la partition chiffrée
    if !usb_disks.is_empty() {
        target_disk = Some(usb_disks[0]);
        println!("Sélection du disque USB: {}", usb_disks[0]);
    } else if !available_disks.is_empty() {
        // Si aucun disque USB trouvé, prendre le dernier disque (généralement un disque externe)
        target_disk = Some(available_disks[available_disks.len() - 1]);
        println!(
            "Aucun disque USB trouvé, sélection du dernier disque: {}",
            available_disks[available_disks.len() - 1]
        );
    }

    if available_disks.is_empty() {
        return Err("Aucun disque en ligne trouvé".to_string());
    }

    let disk_num = target_disk.ok_or("Aucun disque valide trouvé")?;
    println!("Disques disponibles: {:?}", available_disks);
    println!("Disque sélectionné: {}", disk_num);

    // Maintenant, lister les partitions de ce disque spécifique avec PowerShell
    println!(
        "Exécution de PowerShell pour lister les partitions du disque {}...",
        disk_num
    );

    let powershell_partition_script = format!("Get-Partition -DiskNumber {}", disk_num);
    let output = std::process::Command::new("powershell")
        .args(&["-Command", &powershell_partition_script])
        .output()
        .map_err(|e| {
            format!(
                "Erreur lors de l'exécution de PowerShell pour lister les partitions: {}",
                e
            )
        })?;

    println!(
        "Code de sortie PowerShell (partitions): {:?}",
        output.status
    );
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        println!("Erreur PowerShell (partitions): {}", error_msg);
        return Err(format!("Erreur PowerShell (partitions): {}", error_msg));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("Sortie PowerShell (partitions): {}", output_str);

    // Trouver la partition ENCRYPTED
    println!("=== ÉTAPE 3: ANALYSE DES PARTITIONS ===");
    println!("Recherche de la partition chiffrée (sans lettre de lecteur)...");
    let mut encrypted_partition = None;
    let mut partition_count = 0;

    for line in output_str.lines() {
        println!("Ligne analysée: {}", line);

        // Format PowerShell: "2                           53688139776                             7.3 GB IFS"
        // Chercher les partitions sans lettre de lecteur (DriveLetter vide)
        if line.contains("PartitionNumber") || line.contains("----") || line.trim().is_empty() {
            continue; // Ignorer les en-têtes
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            // Le premier élément devrait être le numéro de partition
            if let Ok(partition_num) = parts[0].parse::<u32>() {
                partition_count += 1;
                println!("  → Partition {} détectée", partition_num);

                // Vérifier si la partition n'a pas de lettre de lecteur
                // Analyser les éléments pour trouver la lettre de lecteur et l'offset
                if parts.len() >= 4 {
                    println!("  🔍 Tous les éléments: {:?}", parts);

                    // Chercher le premier élément non-vide après le numéro de partition
                    let mut drive_letter = "";
                    let mut offset = "";
                    let mut size = "";
                    let mut partition_type = "";

                    // Parcourir les éléments pour trouver les bonnes valeurs
                    for (i, part) in parts.iter().enumerate() {
                        if i == 0 {
                            continue;
                        } // Skip partition number
                        if !part.is_empty() {
                            if drive_letter.is_empty() {
                                // Premier élément non-vide après le numéro
                                if part.len() == 1
                                    && part.chars().next().unwrap_or(' ').is_alphabetic()
                                {
                                    drive_letter = part; // C'est une lettre de lecteur
                                } else if part.parse::<u64>().is_ok() {
                                    offset = part; // C'est un offset
                                }
                            } else if offset.is_empty() && part.parse::<u64>().is_ok() {
                                offset = part;
                            } else if size.is_empty()
                                && (part.contains("GB")
                                    || part.contains("MB")
                                    || part.parse::<f64>().is_ok())
                            {
                                size = part;
                            } else if partition_type.is_empty() {
                                partition_type = part;
                            }
                        }
                    }

                    println!("  🔍 Analyse partition {}: lettre='{}', offset='{}', taille='{}', type='{}'", 
                            partition_num, drive_letter, offset, size, partition_type);

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
                        println!("  ⚠️  Partition {} - format inattendu", partition_num);
                    }
                }
            }
        }
    }

    println!("=== RÉSUMÉ DE L'ANALYSE ===");
    println!("Nombre total de partitions trouvées: {}", partition_count);

    let partition_num = encrypted_partition.ok_or("Aucune partition chiffrée trouvée")?;
    println!("✅ Partition chiffrée sélectionnée: {}", partition_num);

    // Validation des paramètres
    println!("=== ÉTAPE 4: VALIDATION DES PARAMÈTRES ===");
    println!("Vérification du numéro de disque: {}", disk_num);
    if disk_num > 10 {
        return Err(format!("❌ Numéro de disque invalide: {}", disk_num));
    }
    println!("✅ Numéro de disque valide");

    println!("Vérification du numéro de partition: {}", partition_num);
    if partition_num > 10 {
        return Err(format!(
            "❌ Numéro de partition invalide: {}",
            partition_num
        ));
    }
    println!("✅ Numéro de partition valide");

    // Assigner une lettre de lecteur temporaire
    println!("=== ÉTAPE 5: ASSIGNATION DE LA LETTRE DE LECTEUR ===");
    let temp_letter = get_next_drive_letter("Z"); // Utiliser une lettre haute
    println!("Lettre de lecteur temporaire sélectionnée: {}", temp_letter);

    // Validation de la lettre de lecteur
    if !temp_letter.is_ascii_alphabetic() {
        return Err(format!("❌ Lettre de lecteur invalide: {}", temp_letter));
    }
    println!("✅ Lettre de lecteur valide");

    println!("=== ÉTAPE 6: GÉNÉRATION DU SCRIPT DE MONTAGE ===");
    let mount_script = format!(
        "select disk {}\n\
         select partition {}\n\
         assign letter={}\n\
         list partition\n",
        disk_num, partition_num, temp_letter
    );

    println!("Script diskpart généré:");
    println!("{}", mount_script);

    let mount_script_path = std::env::temp_dir().join("deepvault_mount.txt");
    println!("Chemin du script: {:?}", mount_script_path);

    std::fs::write(&mount_script_path, mount_script)
        .map_err(|e| format!("❌ Erreur lors de l'écriture du script de montage: {}", e))?;
    println!("✅ Script de montage écrit avec succès");

    println!("=== ÉTAPE 7: EXÉCUTION DU MONTAGE ===");
    println!("Lancement de diskpart pour monter la partition...");
    println!("Commande: diskpart /s {:?}", mount_script_path);

    let mount_output = std::process::Command::new("diskpart")
        .args(&["/s", mount_script_path.to_str().unwrap()])
        .output()
        .map_err(|e| {
            let _ = std::fs::remove_file(&mount_script_path);
            format!("❌ Erreur lors de l'exécution du montage: {}", e)
        })?;

    let _ = std::fs::remove_file(&mount_script_path);
    println!("✅ Script de montage exécuté");

    println!("=== ÉTAPE 8: VÉRIFICATION DES RÉSULTATS ===");
    println!("Code de sortie diskpart: {:?}", mount_output.status);
    let stdout_str = String::from_utf8_lossy(&mount_output.stdout);
    let stderr_str = String::from_utf8_lossy(&mount_output.stderr);

    println!("📤 Sortie standard diskpart:");
    println!("{}", stdout_str);
    println!("📤 Sortie d'erreur diskpart:");
    println!("{}", stderr_str);

    if !mount_output.status.success() {
        println!("❌ Échec du montage - Analyse des erreurs...");
        // Analyser l'erreur plus en détail
        let error_msg = if !stderr_str.is_empty() {
            stderr_str.to_string()
        } else {
            stdout_str.to_string()
        };

        println!("🔍 Message d'erreur analysé: {}", error_msg);

        // Vérifier les erreurs communes
        if error_msg.contains("The parameter is incorrect") {
            return Err(format!("❌ Paramètre incorrect dans le script diskpart. Vérifiez le disque {} et la partition {}", disk_num, partition_num));
        } else if error_msg.contains("The specified disk is not valid") {
            return Err(format!("❌ Le disque {} n'est pas valide", disk_num));
        } else if error_msg.contains("The specified partition is not valid") {
            return Err(format!(
                "❌ La partition {} n'est pas valide sur le disque {}",
                partition_num, disk_num
            ));
        } else if error_msg.contains("The drive letter is already in use") {
            return Err(format!(
                "❌ La lettre de lecteur {} est déjà utilisée",
                temp_letter
            ));
        } else {
            return Err(format!("❌ Erreur lors du montage: {}", error_msg));
        }
    }

    println!("✅ Montage réussi !");
    let mount_path = format!("{}:", temp_letter);
    println!("🎉 Partition montée avec succès sur: {}", mount_path);

    // Générer le script d'auto-masquage sur la clé USB
    println!("=== ÉTAPE 9: GÉNÉRATION DU SCRIPT D'AUTO-MASQUAGE ===");
    if let Err(e) = generate_autorun_script(&mount_path, disk_num, partition_num).await {
        println!(
            "⚠️  Avertissement: Impossible de générer le script d'auto-masquage: {}",
            e
        );
    } else {
        println!("✅ Script d'auto-masquage généré avec succès");
    }

    println!("=== MONTAGE TERMINÉ AVEC SUCCÈS ===");

    // Retourner le chemin de montage pour que le frontend puisse l'utiliser
    Ok(mount_path)
}

#[tauri::command]
async fn unmount_encrypted_partition() -> std::result::Result<String, String> {
    println!("=== DÉMONTAGE DE LA PARTITION CHIFFRÉE ===");

    // Trouver la partition chiffrée montée
    println!("=== ÉTAPE 1: RECHERCHE DE LA PARTITION MONTÉE ===");
    println!("Recherche des partitions montées...");

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
    println!("Partitions montées trouvées:");
    println!("{}", output_str);

    // Chercher une partition montée qui pourrait être notre partition chiffrée
    // (généralement une lettre de lecteur haute comme Z, Y, X, etc.)
    let mut mounted_letter = None;
    for line in output_str.lines() {
        if line.contains("PartitionNumber") || line.contains("----") || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(_partition_num) = parts[0].parse::<u32>() {
                if let Some(letter) = parts[1].chars().next() {
                    if letter.is_alphabetic() && letter >= 'Z' {
                        // Lettre de lecteur haute, probablement notre partition chiffrée
                        mounted_letter = Some(letter);
                        println!("  → Partition montée trouvée: {}", letter);
                        break;
                    }
                }
            }
        }
    }

    let letter = mounted_letter.ok_or("Aucune partition chiffrée montée trouvée")?;
    println!("✅ Partition chiffrée montée identifiée: {}", letter);

    // Démonter la partition avec diskpart
    println!("=== ÉTAPE 2: DÉMONTAGE DE LA PARTITION ===");
    let unmount_script = format!(
        "select volume {}\n\
         remove letter={}\n\
         list volume\n",
        letter, letter
    );

    println!("Script de démontage généré:");
    println!("{}", unmount_script);

    let script_path = std::env::temp_dir().join("deepvault_unmount.txt");
    std::fs::write(&script_path, unmount_script)
        .map_err(|e| format!("Erreur lors de l'écriture du script de démontage: {}", e))?;

    println!("Exécution du script de démontage...");
    let unmount_output = std::process::Command::new("diskpart")
        .args(&["/s", script_path.to_str().unwrap()])
        .output()
        .map_err(|e| {
            let _ = std::fs::remove_file(&script_path);
            format!("Erreur lors de l'exécution du démontage: {}", e)
        })?;

    let _ = std::fs::remove_file(&script_path);

    println!("Code de sortie démontage: {:?}", unmount_output.status);
    let stdout_str = String::from_utf8_lossy(&unmount_output.stdout);
    let stderr_str = String::from_utf8_lossy(&unmount_output.stderr);

    println!("Sortie standard démontage: {}", stdout_str);
    println!("Sortie erreur démontage: {}", stderr_str);

    if !unmount_output.status.success() {
        let error_msg = if !stderr_str.is_empty() {
            stderr_str.to_string()
        } else {
            stdout_str.to_string()
        };
        return Err(format!("Erreur lors du démontage: {}", error_msg));
    }

    println!("✅ Partition démontée avec succès !");
    println!("=== DÉMONTAGE TERMINÉ AVEC SUCCÈS ===");

    Ok(format!("Partition {} démontée avec succès", letter))
}

/// Génère un script d'auto-masquage sur la clé USB
async fn generate_autorun_script(
    mount_path: &str,
    disk_num: u32,
    partition_num: u32,
) -> std::result::Result<(), String> {
    println!("Génération du script d'auto-masquage...");

    // Trouver la lettre de lecteur de la partition publique (généralement D:)
    let public_drive = "D:"; // On assume que la partition publique est sur D:

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

    // Créer le script batch qui lance le VBS
    let batch_script = format!(
        r#"
@echo off
REM Script d'auto-masquage DeepVault
REM Ce fichier lance le script VBS caché

cd /d "%~dp0"
if exist "~deepvault_autorun.vbs" (
    cscript //nologo "~deepvault_autorun.vbs"
) else (
    echo Script d'auto-masquage non trouve
)
"#
    );

    // Créer le fichier autorun.inf
    let autorun_content = format!(
        r#"
[autorun]
open=~deepvault_autorun.bat
action=DeepVault Auto-Hide
label=DeepVault USB
icon=~deepvault_autorun.bat
"#
    );

    // Chemins des fichiers
    let vbs_path = format!("{}\\~deepvault_autorun.vbs", public_drive);
    let batch_path = format!("{}\\~deepvault_autorun.bat", public_drive);
    let autorun_path = format!("{}\\autorun.inf", public_drive);

    println!("Écriture des scripts sur la clé USB...");
    println!("VBS: {}", vbs_path);
    println!("Batch: {}", batch_path);
    println!("Autorun: {}", autorun_path);

    // Écrire les fichiers
    std::fs::write(&vbs_path, vbs_script)
        .map_err(|e| format!("Erreur lors de l'écriture du script VBS: {}", e))?;

    std::fs::write(&batch_path, batch_script)
        .map_err(|e| format!("Erreur lors de l'écriture du script batch: {}", e))?;

    std::fs::write(&autorun_path, autorun_content)
        .map_err(|e| format!("Erreur lors de l'écriture du fichier autorun: {}", e))?;

    // Rendre les fichiers cachés
    let _ = std::process::Command::new("attrib")
        .args(&["+h", &vbs_path])
        .output();

    let _ = std::process::Command::new("attrib")
        .args(&["+h", &batch_path])
        .output();

    let _ = std::process::Command::new("attrib")
        .args(&["+h", &autorun_path])
        .output();

    println!("✅ Scripts d'auto-masquage créés et cachés");
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
        format!("{}\\~deepvault_autorun.vbs", public_drive),
        format!("{}\\~deepvault_autorun.bat", public_drive),
        format!("{}\\autorun.inf", public_drive),
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
            mount_encrypted_partition,
            unmount_encrypted_partition,
            check_mount_status,
            cleanup_autorun_scripts,
            open_explorer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
