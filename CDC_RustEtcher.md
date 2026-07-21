# Cahier des Charges — RustFlash

## Clone de Balena Etcher en Rust — Suite complète de gestion de médias bootables

---

## 1. Présentation du projet

### 1.1 Contexte

Balena Etcher est un outil open-source populaire permettant de flasher des images OS sur des clés USB et cartes SD. Cependant, il repose sur Electron, ce qui entraîne une consommation mémoire excessive (~300-500 Mo) et des performances sous-optimales pour une opération fondamentalement système.

**RustFlash** vise à recréer et dépasser Etcher en Rust, avec une interface TUI légère et une suite complète d'outils de gestion de médias bootables.

### 1.2 Objectifs

| Objectif | Description |
|----------|-------------|
| Performance | Empreinte mémoire < 30 Mo, vitesse d'écriture proche du débit max du périphérique |
| Légèreté | Binaire unique, aucune dépendance runtime |
| Cross-platform | Linux, Windows, macOS |
| Sécurité | Protection contre l'écriture accidentelle sur disques système |
| Extensibilité | Architecture modulaire permettant l'ajout de fonctionnalités |

### 1.3 Nom du projet

**RustFlash** (nom de travail, à confirmer)

---

## 2. Spécifications fonctionnelles

### 2.1 Module 1 — Flash d'images (Core)

#### 2.1.1 Formats d'images supportés

| Format | Extension | Priorité |
|--------|-----------|----------|
| Raw image | `.img`, `.raw`, `.bin` | P0 |
| ISO 9660 | `.iso` | P0 |
| Gzip compressé | `.img.gz`, `.gz` | P0 |
| XZ compressé | `.img.xz`, `.xz` | P0 |
| Zstandard | `.img.zst`, `.zst` | P0 |
| Bzip2 | `.img.bz2`, `.bz2` | P1 |
| ZIP archive | `.zip` | P1 |
| 7z archive | `.7z` | P2 |
| DMG (macOS) | `.dmg` | P2 |
| WIC (Windows) | `.wic` | P2 |

#### 2.1.2 Sources d'images

- **Fichier local** : sélection via navigateur de fichiers TUI
- **URL distante** : téléchargement HTTP/HTTPS avec barre de progression
- **Téléchargement résumable** : reprise après interruption (HTTP Range)
- **Vérification de checksum** : SHA256, SHA512, MD5 automatique si fichier `.sha256` / `.sha512` trouvé à côté de l'image ou à l'URL

#### 2.1.3 Sélection du périphérique cible

- Listage automatique des périphériques amovibles
- Affichage : nom, taille, fabricant, point de montage
- **Filtrage de sécurité** : masquage des disques système par défaut
- **Mode expert** : affichage de tous les disques (avec avertissement)
- Détection des partitions montées et démontage automatique (avec confirmation)
- Rafraîchissement en temps réel (hot-plug USB)

#### 2.1.4 Processus de flash

```
[Sélection image] → [Sélection cible] → [Confirmation] → [Écriture] → [Validation] → [Résultat]
```

- Écriture par blocs configurables (défaut : 4 Mo)
- Barre de progression avec : pourcentage, vitesse (Mo/s), temps restant estimé
- Validation post-écriture (relecture + comparaison hash bloc par bloc)
- Support du **multi-flash** : écriture simultanée sur plusieurs périphériques
- Annulation propre à tout moment (avec nettoyage)

### 2.2 Module 2 — Clonage de disques

#### 2.2.1 Modes de clonage

| Mode | Description |
|------|-------------|
| Disque → Disque | Copie bit-à-bit d'un périphérique vers un autre |
| Disque → Image | Sauvegarde d'un périphérique vers un fichier image |
| Image → Disque | Restauration (équivalent au flash) |

#### 2.2.2 Options de clonage

- **Clonage intelligent** : ne copier que les secteurs utilisés (ext4, NTFS, FAT32, APFS, HFS+)
- **Clonage brut** : copie bit-à-bit complète (défaut)
- Compression à la volée lors de l'export (gzip, xz, zstd)
- Redimensionnement automatique des partitions si disque cible plus grand/petit
- Vérification d'intégrité post-clonage

### 2.3 Module 3 — Backup et restauration

#### 2.3.1 Création de backups

- Sauvegarde complète d'un périphérique vers une image compressée
- Sauvegarde incrémentielle (via snapshot des blocs modifiés)
- Métadonnées embarquées : date, source, taille originale, table de partitions
- Format de backup propriétaire `.rfb` (RustFlash Backup) avec header de métadonnées + données compressées
- Export possible vers formats standards (`.img.zst`, `.img.gz`)

#### 2.3.2 Restauration

- Restauration depuis `.rfb` ou tout format d'image supporté
- Vérification d'intégrité avant restauration
- Mode simulation (dry-run) : vérifier la compatibilité sans écrire

### 2.4 Module 4 — Partitionnement

#### 2.4.1 Opérations supportées

| Opération | Description |
|-----------|-------------|
| Créer partition | Avec type (ext4, FAT32, NTFS, exFAT, APFS) |
| Supprimer partition | Avec confirmation |
| Redimensionner | Agrandir/réduire une partition |
| Formater | Reformater une partition existante |
| Changer label | Modifier le nom de volume |
| Changer flags | boot, esp, lvm, raid |
| Créer table | GPT ou MBR |
| Effacement sécurisé | Zéro-fill, random fill, DoD 5220.22-M |

#### 2.4.2 Visualisation

```
┌─ /dev/sdb (32 Go - SanDisk Ultra) ──────────────────────────┐
│ [████ boot ████|████████ rootfs ████████|░░ libre ░░]        │
│  FAT32 256Mo     ext4 28Go               3.7Go              │
└──────────────────────────────────────────────────────────────┘
```

- Représentation graphique ASCII des partitions
- Couleurs distinctes par type de filesystem
- Informations détaillées au survol/sélection

### 2.5 Module 5 — Interface TUI

#### 2.5.1 Navigation principale

```
┌─ RustFlash v1.0.0 ──────────────────────────────────────────┐
│                                                              │
│   ┌─────────────────────────────────────────────┐            │
│   │  [F] Flash image      │  [C] Cloner disque  │            │
│   │  [B] Backup            │  [R] Restaurer     │            │
│   │  [P] Partitionner      │  [S] Paramètres    │            │
│   │  [Q] Quitter                                │            │
│   └─────────────────────────────────────────────┘            │
│                                                              │
│  Périphériques détectés :                                    │
│  ● /dev/sdb - SanDisk Ultra 32Go (USB 3.0)                  │
│  ● /dev/sdc - Samsung EVO 64Go (USB 3.1)                    │
│                                                              │
│  ─────────────────────────────────────────────────           │
│  Raccourcis: Tab=naviguer  Enter=valider  Esc=retour         │
└──────────────────────────────────────────────────────────────┘
```

#### 2.5.2 Composants UI

| Composant | Usage |
|-----------|-------|
| Barre de progression | Flash, clonage, backup — avec vitesse et ETA |
| Navigateur de fichiers | Sélection d'image avec filtrage par extension |
| Liste sélectionnable | Choix de périphérique, partitions |
| Dialogues de confirmation | Actions destructives avec saisie de confirmation |
| Logs en temps réel | Panneau de logs scrollable |
| Tableau de partitions | Visualisation graphique (cf. 2.4.2) |

#### 2.5.3 Thèmes

- Thème sombre (défaut)
- Thème clair
- Thème haut contraste (accessibilité)
- Configuration via fichier TOML

#### 2.5.4 Mode CLI non-interactif

En plus de la TUI, supporter un mode CLI scriptable :

```bash
# Flash simple
rustflash flash --image ubuntu.iso --target /dev/sdb --verify

# Clonage
rustflash clone --source /dev/sda --dest /dev/sdb --smart

# Backup
rustflash backup --source /dev/sdb --output backup.rfb --compress zstd

# Partitionnement
rustflash partition /dev/sdb --create gpt
rustflash partition /dev/sdb --add fat32 256M --label BOOT --flag boot,esp
rustflash partition /dev/sdb --add ext4 remaining --label rootfs

# Lister les périphériques
rustflash list --json
```

---

## 3. Spécifications techniques

### 3.1 Architecture

```
┌─────────────────────────────────────────────────────┐
│                    CLI / TUI Layer                   │
│              (clap + ratatui + crossterm)            │
├─────────────────────────────────────────────────────┤
│                   Core Engine                        │
│  ┌───────────┐ ┌──────────┐ ┌─────────────────┐    │
│  │  Flasher  │ │  Cloner  │ │ Partition Mgr   │    │
│  └─────┬─────┘ └────┬─────┘ └───────┬─────────┘    │
│        │             │               │               │
│  ┌─────▼─────────────▼───────────────▼─────────┐    │
│  │           Block I/O Layer                    │    │
│  │    (lecture/écriture async par blocs)         │    │
│  └─────────────────┬───────────────────────────┘    │
│                    │                                 │
│  ┌─────────────────▼───────────────────────────┐    │
│  │        Platform Abstraction Layer            │    │
│  │   Linux: /dev/sdX, udev, udisks2            │    │
│  │   Windows: \\.\PhysicalDriveN, WMI          │    │
│  │   macOS: /dev/diskN, diskutil               │    │
│  └─────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────┤
│              Decompression Layer                     │
│         (flate2, xz2, zstd, bzip2, zip)            │
├─────────────────────────────────────────────────────┤
│                Integrity Layer                       │
│            (sha2, md5, blake3, crc32)               │
└─────────────────────────────────────────────────────┘
```

### 3.2 Stack technique

| Catégorie | Crate | Justification |
|-----------|-------|---------------|
| **TUI** | `ratatui` + `crossterm` | Framework TUI mature, cross-platform |
| **CLI** | `clap` (derive) | Parsing d'arguments standard en Rust |
| **Async** | `tokio` | I/O asynchrone pour flash parallèle et téléchargement |
| **HTTP** | `reqwest` | Client HTTP pour téléchargement d'images |
| **Compression** | `flate2`, `xz2`, `zstd`, `bzip2`, `zip` | Décompression multi-format |
| **Hash** | `sha2`, `md-5`, `blake3` | Vérification d'intégrité |
| **Sérialisation** | `serde` + `toml` | Configuration et métadonnées |
| **Logs** | `tracing` + `tracing-subscriber` | Logging structuré |
| **Erreurs** | `thiserror` + `anyhow` | Gestion d'erreurs idiomatique |
| **Filesystem** | `gpt`, `mbr` (custom) | Lecture/écriture de tables de partitions |
| **Plateforme** | `sysinfo`, `udev` (Linux), `windows` (Win) | Détection de périphériques |
| **Tests** | `assert_cmd`, `predicates`, `tempfile` | Tests d'intégration CLI |

### 3.3 Structure du projet

```
rustflash/
├── Cargo.toml
├── Cargo.lock
├── LICENSE                    # MIT ou Apache-2.0
├── README.md
├── src/
│   ├── main.rs                # Point d'entrée, dispatch CLI/TUI
│   ├── cli/
│   │   ├── mod.rs             # Définition clap
│   │   └── commands/          # Handlers par commande
│   │       ├── flash.rs
│   │       ├── clone.rs
│   │       ├── backup.rs
│   │       ├── partition.rs
│   │       └── list.rs
│   ├── tui/
│   │   ├── mod.rs             # App TUI principale
│   │   ├── app.rs             # État de l'application
│   │   ├── event.rs           # Gestion des événements
│   │   ├── ui/
│   │   │   ├── mod.rs
│   │   │   ├── home.rs        # Écran d'accueil
│   │   │   ├── flash.rs       # Vue flash
│   │   │   ├── clone.rs       # Vue clonage
│   │   │   ├── backup.rs      # Vue backup
│   │   │   ├── partition.rs   # Vue partitionnement
│   │   │   ├── file_browser.rs
│   │   │   ├── progress.rs    # Barre de progression
│   │   │   └── dialog.rs      # Dialogues de confirmation
│   │   └── theme.rs           # Thèmes de couleurs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── flasher.rs         # Logique de flash
│   │   ├── cloner.rs          # Logique de clonage
│   │   ├── backup.rs          # Logique de backup
│   │   ├── partition.rs       # Gestion des partitions
│   │   └── verify.rs          # Vérification d'intégrité
│   ├── io/
│   │   ├── mod.rs
│   │   ├── block.rs           # I/O par blocs async
│   │   ├── decompress.rs      # Pipeline de décompression
│   │   └── download.rs        # Téléchargement HTTP
│   ├── platform/
│   │   ├── mod.rs             # Trait DeviceEnumerator
│   │   ├── linux.rs           # Implémentation Linux
│   │   ├── windows.rs         # Implémentation Windows
│   │   └── macos.rs           # Implémentation macOS
│   ├── device/
│   │   ├── mod.rs
│   │   ├── detect.rs          # Détection de périphériques
│   │   ├── filter.rs          # Filtrage (système vs amovible)
│   │   └── mount.rs           # Montage/démontage
│   └── config/
│       ├── mod.rs
│       └── settings.rs        # Fichier de configuration TOML
├── tests/
│   ├── integration/
│   │   ├── flash_test.rs
│   │   ├── clone_test.rs
│   │   └── cli_test.rs
│   └── fixtures/
│       └── test.img           # Image de test minimale
└── docs/
    └── architecture.md
```

### 3.4 Gestion cross-platform

#### Linux
- Accès disque : `/dev/sdX`, `/dev/mmcblkN`
- Énumération : `udev` / `sysfs` (`/sys/block/`)
- Démontage : `umount` syscall ou `udisks2` D-Bus
- Élévation : `sudo`, `pkexec`, ou capabilities (`CAP_SYS_RAWIO`)

#### Windows
- Accès disque : `\\.\PhysicalDriveN`
- Énumération : `SetupDi*` API ou WMI
- Démontage : `FSCTL_DISMOUNT_VOLUME`
- Élévation : UAC prompt, manifeste avec `requireAdministrator`
- Lock volume : `FSCTL_LOCK_VOLUME` avant écriture

#### macOS
- Accès disque : `/dev/diskN`, `/dev/rdiskN` (raw pour la performance)
- Énumération : `diskutil list -plist` ou IOKit
- Démontage : `diskutil unmountDisk`
- Élévation : `osascript -e 'do shell script "..." with administrator privileges'`

### 3.5 Format de backup RustFlash (.rfb)

```
┌────────────────────────────────────────┐
│ Magic: "RFLASH\x01\x00" (8 bytes)     │
├────────────────────────────────────────┤
│ Header (JSON, taille variable)         │
│ {                                      │
│   "version": 1,                        │
│   "created": "2026-02-28T...",         │
│   "source_size": 32000000000,          │
│   "block_size": 4194304,              │
│   "compression": "zstd",              │
│   "hash_algorithm": "blake3",          │
│   "partition_table": "gpt",            │
│   "partitions": [...],                 │
│   "source_device": "SanDisk Ultra",    │
│   "checksum": "abcdef..."             │
│ }                                      │
├────────────────────────────────────────┤
│ Header size (u32 LE)                   │
├────────────────────────────────────────┤
│ Block bitmap (secteurs utilisés)       │
├────────────────────────────────────────┤
│ Data blocks (compressés)               │
│ [block0][block1][block2]...            │
├────────────────────────────────────────┤
│ Block index (offset de chaque bloc)    │
├────────────────────────────────────────┤
│ Footer checksum (BLAKE3 du fichier)    │
└────────────────────────────────────────┘
```

---

## 4. Sécurité

### 4.1 Protection contre l'écriture accidentelle

| Mesure | Description |
|--------|-------------|
| Filtrage système | Masquage des disques contenant `/`, `/home`, `C:\`, partition EFI active |
| Confirmation explicite | Saisie du nom du périphérique (`sdb`) pour confirmer les opérations destructives |
| Verrouillage | Lock exclusif sur le périphérique pendant l'opération |
| Démontage préalable | Démontage automatique de toutes les partitions avant écriture |
| Dry-run | Mode simulation disponible pour toute opération |

### 4.2 Élévation de privilèges

- Demander l'élévation uniquement au moment nécessaire (pas au lancement)
- Linux : vérifier les capabilities, fallback sur `sudo`/`pkexec`
- Windows : manifeste UAC embarqué ou re-lancement élevé
- macOS : `Authorization Services` ou prompt `osascript`
- Journalisation de toutes les opérations élevées

### 4.3 Intégrité des données

- Hash bloc par bloc pendant l'écriture (pas seulement à la fin)
- Validation post-écriture par relecture complète
- Vérification de checksum des images téléchargées
- Détection des secteurs défectueux pendant le clonage

---

## 5. Performance

### 5.1 Objectifs

| Métrique | Objectif |
|----------|----------|
| Empreinte RAM | < 30 Mo en flash simple, < 100 Mo en multi-flash |
| Taille binaire | < 15 Mo (release, stripped) |
| Débit écriture | > 90% du débit théorique du périphérique |
| Temps de démarrage | < 200 ms |
| Latence UI | < 16 ms (60 fps TUI) |

### 5.2 Stratégies d'optimisation

- **I/O direct** : `O_DIRECT` (Linux), `FILE_FLAG_NO_BUFFERING` (Windows) pour bypasser le cache OS
- **Pipeline** : décompression → hash → écriture en pipeline async (pas de copie mémoire intermédiaire)
- **Double buffering** : pendant qu'un bloc s'écrit, le suivant se décompresse
- **Allocation fixe** : buffers pré-alloués, pas d'allocation dynamique dans la boucle chaude
- **Multi-flash** : écriture parallèle via `tokio::spawn` par périphérique

---

## 6. Configuration

### 6.1 Fichier de configuration

Emplacement : `~/.config/rustflash/config.toml` (Linux/macOS), `%APPDATA%\rustflash\config.toml` (Windows)

```toml
[general]
theme = "dark"                # dark, light, high-contrast
language = "fr"               # fr, en, de, es, pt, ja, zh
confirm_destructive = true    # demander confirmation
show_system_drives = false    # mode expert

[flash]
block_size = 4194304          # 4 Mo
verify_after_write = true
auto_unmount = true
decompress_threads = 0        # 0 = auto (nombre de CPU)

[network]
download_timeout = 300        # secondes
resume_downloads = true
proxy = ""                    # http://proxy:port

[backup]
default_compression = "zstd"
compression_level = 3         # 1-22 pour zstd
default_output_dir = "~/backups"

[logging]
level = "info"                # trace, debug, info, warn, error
file = ""                     # chemin vers fichier de log
```

---

## 7. Internationalisation (i18n)

- Support multilingue via fichiers de traduction embarqués (format Fluent ou JSON)
- Langues initiales : français, anglais
- Architecture permettant l'ajout communautaire de langues
- Détection automatique de la locale système

---

## 8. Tests

### 8.1 Stratégie de tests

| Type | Couverture | Outils |
|------|-----------|--------|
| Unitaires | Logique core, parsing, compression | `cargo test` |
| Intégration | CLI complète, workflow bout en bout | `assert_cmd`, fichiers images de test |
| Platform | Code spécifique OS | CI multi-OS (GitHub Actions) |
| Performance | Benchmarks de débit I/O | `criterion` |
| Fuzz | Parsing de formats d'image, header .rfb | `cargo-fuzz` |

### 8.2 Environnement de test

- Utilisation de fichiers loopback (`/dev/loop*` Linux) pour simuler des périphériques
- Images de test minimales (1 Mo) avec différents filesystems
- CI/CD : GitHub Actions avec matrix Linux/Windows/macOS
- Tests de non-régression sur les filtres de sécurité

---

## 9. Planning prévisionnel

### Phase 1 — Fondations (4-6 semaines)

- [ ] Setup projet Cargo, CI/CD, linting
- [ ] Platform Abstraction Layer (détection périphériques)
- [ ] Block I/O Layer (lecture/écriture par blocs)
- [ ] Flash basique : image raw → périphérique
- [ ] CLI minimale (`flash`, `list`)
- [ ] Tests sur loopback devices

### Phase 2 — Flash complet (3-4 semaines)

- [ ] Décompression multi-format (gz, xz, zst, bz2, zip)
- [ ] Vérification post-écriture
- [ ] Téléchargement HTTP avec reprise
- [ ] Vérification de checksum
- [ ] Multi-flash parallèle
- [ ] TUI : écran de flash avec progression

### Phase 3 — Clonage et backup (3-4 semaines)

- [ ] Clonage brut disque → disque
- [ ] Clonage intelligent (lecture FS)
- [ ] Format .rfb : création et restauration
- [ ] Compression à la volée
- [ ] TUI : écrans clonage et backup

### Phase 4 — Partitionnement (3-4 semaines)

- [ ] Lecture de tables GPT et MBR
- [ ] Opérations CRUD sur partitions
- [ ] Formatage (ext4, FAT32, NTFS, exFAT)
- [ ] Visualisation ASCII des partitions
- [ ] TUI : écran de partitionnement

### Phase 5 — Polish (2-3 semaines)

- [ ] Thèmes et accessibilité
- [ ] Internationalisation (fr + en)
- [ ] Documentation utilisateur
- [ ] Packaging (deb, rpm, AUR, brew, scoop, MSI)
- [ ] Benchmarks et optimisations finales

### Total estimé : 15-21 semaines

---

## 10. Critères de validation (Definition of Done)

### 10.1 Fonctionnels

- [ ] Flash d'une image ISO/IMG sur clé USB avec vérification → succès
- [ ] Flash depuis URL avec checksum → succès
- [ ] Multi-flash sur 2+ périphériques simultanément → succès
- [ ] Clonage disque → disque avec vérification → données identiques
- [ ] Backup .rfb + restauration → données identiques à l'original
- [ ] Création de table GPT + partitions + formatage → périphérique bootable
- [ ] Mode CLI non-interactif → toutes les commandes fonctionnelles
- [ ] TUI → navigation fluide, toutes les vues accessibles

### 10.2 Non-fonctionnels

- [ ] Cross-platform : build et tests passent sur Linux, Windows, macOS
- [ ] RAM < 30 Mo en opération standard
- [ ] Binaire < 15 Mo (stripped)
- [ ] Aucune opération destructive possible sans confirmation explicite
- [ ] Disques système jamais proposés par défaut
- [ ] Taux de couverture de tests > 70%

### 10.3 Sécurité

- [ ] Audit des chemins d'élévation de privilèges
- [ ] Fuzzing du parser de format .rfb sans crash
- [ ] Aucune injection de commande possible via les entrées utilisateur
- [ ] Lock exclusif empêche les opérations concurrentes sur un même périphérique

---

## 11. Risques et mitigations

| Risque | Impact | Mitigation |
|--------|--------|------------|
| Accès raw disk difficile sur Windows | Élevé | Wrapper win32 API, tests CI Windows dès Phase 1 |
| Clonage intelligent nécessite parsing FS | Élevé | Commencer par clonage brut, parsing FS incrémental |
| macOS restrictions SIP/diskutil | Moyen | Utiliser `rdisk` (raw), tester sur CI macOS |
| Format .rfb non standard | Faible | Export vers formats standards toujours disponible |
| Performance I/O variable selon OS | Moyen | Benchmarks automatisés, tuning par plateforme |

---

## 12. Licences et distribution

- **Licence** : MIT ou Apache-2.0 (double licence Rust standard)
- **Distribution** :
  - Binaires pré-compilés (GitHub Releases)
  - `cargo install rustflash`
  - Packages Linux : `.deb`, `.rpm`, AUR
  - macOS : Homebrew tap
  - Windows : Scoop bucket, MSI installer
