# Changelog

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet adhère au [Versioning Sémantique](https://semver.org/lang/fr/).

## [0.1.0] - 2024-01-XX

### Ajouté
- Structure de base du projet Rust avec architecture modulaire
- Détection automatique des périphériques USB
- Support multi-plateforme (Linux, Windows, macOS)
- Interface CLI complète avec commandes list, configure, mount, unmount, wipe
- Interface graphique Vue.js avec 6 écrans selon spécifications
- Chiffrement Argon2id avec paramètres configurables
- Gestion des partitions (création, formatage, montage)
- Support LUKS (Linux) et VeraCrypt (Windows)
- Effacement sécurisé selon standard DoD 5220.22-M
- Support des volumes cachés pour déni plausible
- Configuration JSON avec validation
- Logs détaillés et exportables
- Scripts de build pour Windows et Linux
- Documentation complète avec README

### Technique
- Core library Rust avec modules séparés
- Wrappers natifs pour cryptsetup/veracrypt/diskpart
- Architecture Tauri pour l'interface graphique
- Interface Vue.js avec router et composants modulaires
- Gestion d'erreurs robuste avec types personnalisés
- Tests de compilation et validation des dépendances
