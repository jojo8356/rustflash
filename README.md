# RustFlash

RustFlash est un utilitaire Rust multiplateforme pour écrire des images disque, cloner des supports, gérer des sauvegardes et manipuler des partitions depuis une interface CLI ou TUI.

- **Projet** : rustflash
- **Version** : 0.1.0
- **Licence** : MIT *ou* Apache-2.0
- **Langage principal** : Rust 2024

## Table des matières

- [Aperçu](#aperçu)
- [Architecture](#architecture)
- [Préparer l’environnement](#préparer-lenvironnement)
- [Installation](#installation)
- [Utilisation](#utilisation)
- [Commandes principales](#commandes-principales)
- [Qualité & validation](#qualité--validation)
- [Dépannage](#dépannage)
- [Documentation associée](#documentation-associée)

## Aperçu

Le projet fournit une base commune pour les opérations bas niveau sur disque :

- Flash d’images (`.img`, `.iso`, `.raw`) avec vérification d’intégrité
- Décompression d’images (`.gz`, `.xz`, `.zst`, `.bz2`, `.zip`)
- Clonage disque à disque ou disque vers fichier (option compression)
- Sauvegarde/Restauration au format propriétaire `.rfb` (compression + checksums)
- Gestion de partitions (GPT/MBR) : création, suppression, formatage, effacement sécurisé
- Interfaces CLI scriptables et TUI ergonomique

## Architecture

Le dépôt est structuré autour de couches métier distinctes :

- `src/cli` : parsing des commandes et orchestration CLI
- `src/tui` : moteur d’état + vues terminales pour la navigation interactive
- `src/core` : logique métier (flash / clone / backup / partition / vérification)
- `src/io` : traitement flux, décompression, téléchargement, accès bloc
- `src/device` : détection, filtrage et montage des périphériques
- `src/platform` : adaptateurs spécifiques OS (`linux`, `macos`, `windows`)
- `src/config` : persistance de configuration applicative

```text
CLI/TUI  ──► couche applicative ──► moteur core ──► io/device/platform
```

## Préparer l’environnement

- Rust 2024 (1.85+ recommandé)
- Outils Git et un terminal adapté
- Droits root / admin pour les opérations disque

## Installation

### Depuis les sources

```bash
git clone https://github.com/TODO/rustflash.git
cd rustflash
cargo build --release
```

Binaire produit : `target/release/rustflash`.

## Utilisation

Sans argument, `rustflash` démarre la TUI.

```bash
rustflash
```

Pour la CLI, l’aide complète est disponible via :

```bash
rustflash --help
rustflash <commande> --help
```

### Commandes principales

```bash
# Flash d'une image
rustflash flash --image ubuntu.iso --target /dev/sdb --verify

# Clonage
rustflash clone --source /dev/sda --dest /dev/sdb

# Sauvegarde
rustflash backup --source /dev/sda --output backup.rfb --compression zstd

# Restauration
rustflash restore --input backup.rfb --target /dev/sdb

# Gestion de partitions
rustflash partition /dev/sdb show
rustflash partition /dev/sdb create gpt
rustflash partition /dev/sdb add -t ext4 -s 4G -l mydata

# Liste des périphériques
rustflash list
rustflash list --json
```

## Qualité & validation

```bash
cargo test
cargo fmt --check
cargo clippy
```

## Dépannage

- **Erreur d’accès périphérique** : exécuter avec privilèges suffisants.
- **Image compressée non reconnue** : vérifier l’extension et la validité de l’archive.
- **Mauvaise cible de disque** : vérifier soigneusement la table de partitions avant toute écriture.
- **Vérification SHA** : privilégier les flags de vérification avant validation final.

## Documentation associée

- `DOCS.md` : documentation technique et fonctionnement interne
- `NOTICE.md` : dépendances, attributions et notices de licences
- `docs/` : notes complémentaires de contexte projet

