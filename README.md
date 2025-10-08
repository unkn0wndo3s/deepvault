# DeepVault

**Chiffrement sécurisé de périphériques USB avec support multi-plateforme**

DeepVault est un outil de chiffrement avancé pour périphériques USB, offrant une sécurité maximale avec une interface utilisateur intuitive. L'application supporte Linux (LUKS), Windows (VeraCrypt) et macOS (APFS/cryptsetup).

## 🚀 Fonctionnalités

- **Détection automatique** des périphériques USB
- **Chiffrement robuste** avec Argon2id et LUKS/VeraCrypt
- **Interface graphique moderne** (Tauri + Vue.js)
- **Support multi-plateforme** (Linux, Windows, macOS)
- **Volumes cachés** pour déni plausible
- **Effacement sécurisé** selon les standards DoD
- **Mode portable** avec application sur partition publique

## 🏗️ Architecture

- **Core Library** (Rust) : Logique de chiffrement et gestion des périphériques
- **CLI** : Interface en ligne de commande
- **GUI** : Interface graphique avec Tauri + Vue.js
- **Wrappers natifs** : Appels système pour cryptsetup/veracrypt/diskpart

## 📋 Prérequis

### Linux
```bash
sudo apt-get install cryptsetup
```

### Windows
- VeraCrypt CLI installé et dans le PATH
- Privilèges administrateur

### macOS
```bash
brew install cryptsetup veracrypt
```

## 🛠️ Installation

### Développement

1. **Cloner le repository**
```bash
git clone https://github.com/deepvault/usb-deepvault.git
cd usb-deepvault
```

2. **Installer les dépendances**
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (pour l'interface graphique)
npm install
```

3. **Compiler et lancer**
```bash
# CLI uniquement
cargo run --bin deepvault-cli

# Interface graphique
npm run tauri:dev
```

### Production

```bash
# Compiler l'application
npm run tauri:build

# L'exécutable sera dans src-tauri/target/release/
```

## 🎯 Utilisation

### Interface Graphique

1. **Détection** : L'application détecte automatiquement les périphériques USB
2. **Configuration** : Choisissez le type de chiffrement (conteneur, partition, volume caché)
3. **Sécurité** : Configurez le mot de passe et les paramètres de dérivation de clé
4. **Création** : Suivez la progression de la création du volume chiffré
5. **Utilisation** : Montez/démontez vos volumes selon vos besoins

### Interface CLI

```bash
# Lister les périphériques USB
deepvault-cli list

# Configurer un périphérique
deepvault-cli configure /dev/sdb --config-type container

# Monter un volume
deepvault-cli mount /dev/sdb --volume encrypted

# Démonter un volume
deepvault-cli unmount encrypted

# Effacer un périphérique
deepvault-cli wipe /dev/sdb --header-only
```

## 🔒 Sécurité

- **Algorithme** : Argon2id (recommandé par l'OWASP)
- **Paramètres par défaut** : 64MB mémoire, 3 itérations, parallélisme 1
- **Effacement** : Standard DoD 5220.22-M (3 passes)
- **Headers** : Stockage sécurisé des métadonnées
- **Volumes cachés** : Support VeraCrypt pour déni plausible

## 📁 Structure du Projet

```
deepvault/
├── src/                    # Core library (Rust)
│   ├── lib.rs
│   ├── device.rs          # Détection USB
│   ├── crypto.rs          # Chiffrement
│   ├── partition.rs       # Gestion partitions
│   ├── mount.rs           # Montage/démontage
│   ├── wipe.rs            # Effacement sécurisé
│   ├── config.rs          # Configuration
│   └── bin/               # Binaires CLI/GUI
├── src-tauri/             # Application Tauri
├── src/                   # Interface Vue.js
│   ├── components/        # Composants UI
│   └── main.js
└── dist/                  # Build frontend
```

## 🤝 Contribution

1. Fork le projet
2. Créer une branche feature (`git checkout -b feature/AmazingFeature`)
3. Commit les changements (`git commit -m 'Add some AmazingFeature'`)
4. Push vers la branche (`git push origin feature/AmazingFeature`)
5. Ouvrir une Pull Request

## 📄 Licence

Ce projet est sous licence MIT. Voir le fichier `LICENSE` pour plus de détails.

## ⚠️ Avertissement

Cet outil manipule des partitions et des données sensibles. Utilisez-le avec précaution et testez toujours sur des données non-critiques en premier.

## 🔗 Liens

- [Documentation](https://github.com/deepvault/usb-deepvault/wiki)
- [Issues](https://github.com/deepvault/usb-deepvault/issues)
- [Releases](https://github.com/deepvault/usb-deepvault/releases)