---
name: rustflash-architecture
type: architecture-spine
purpose: build-substrate
altitude: epic
paradigm: layered + ports-and-adapters
scope: Core platform tools (CLI + TUI + core engines) for RustFlash
status: draft
created: 2026-07-21
updated: 2026-07-21
binds:
  - FR-1
  - FR-2
  - FR-3
  - FR-4
  - FR-5
  - FR-6
  - FR-7
  - FR-8
sources:
  - README.md
  - CDC_RustEtcher.md
  - _bmad-output/planning-artifacts/prd.md
companions: []
---

# Architecture Spine — RustFlash

## Design Paradigm

Architecture en couches avec adaptateurs de ports :

- **Surface** : `cli` et `tui` exposent un contrat d’usage unique.
- **Core** : règles métier (flash/clone/backup/partition/verify) sans dépendance OS directe.
- **IO** : gestion flux (`read/write/decompress/download`) réutilisable.
- **Platform** : abstraction des plateformes pour énumération / unmount / heuristique disque système.
- **Device** : modèles de périphériques + filtres de sécurité.
- **Config** : configuration locale (`toml`) chargée en entrée de tous les flux.

## Invariants & Rules

### AD-1 — Une seule voie d’écriture bloc par module métier
- **Binds:** FR-2, FR-4, FR-5
- **Prevents:** duplication d’implémentations inconsistantes de buffer/erreur.
- **Rule:** Toute écriture brute utilise `src/io` + `core` plutôt que des appels ad hoc dans `tui`.

### AD-2 — Interface utilisateur = orchestration seulement
- **Binds:** FR-7
- **Prevents:** logique métier dans TUI/CLI.
- **Rule:** `cli` et `tui` déclenchent le même core engine avec mêmes paramètres de `Config`.

### AD-3 — Défense destructive par défaut
- **Binds:** FR-1, NFR-1
- **Prevents:** corruption de média système.
- **Rule:** Les disques systèmes sont exclus par défaut (`DeviceEnumerator::list_devices(false)` + filtre `is_system_disk`).

### AD-4 — Vérification post-opération dès que possible
- **Binds:** FR-2, FR-4, FR-5
- **Prevents:** livraisons avec erreurs silencieuses.
- **Rule:** Vérification activée par défaut sur flash/clone, avec options de fallback explicites.

## Consistency Conventions

| Concern | Convention |
| --- | --- |
| Naming (entités/modules) | Modules courts en anglais métier (`flasher`, `cloner`, `backup`, `partition`, `detect`, `mount`, `decompress`). |
| Erreurs | Erreurs contextuelles (`anyhow`/`tracing`), messages orientés opérateur. |
| Formats | `.rfb` = `MAGIC` `RFLASH\01\00` + longueur header JSON + chunks + footer hash. |
| Logs | `tracing` partout pour début/fin d’opération et erreur critique; logs de debug optionnels via niveau configurable. |
| Configuration | Valeurs par défaut codées + override via config TOML, chargement non-bloquant au démarrage. |

## Stack

| Name | Version |
| --- | --- |
| Rust | 2024 |
| clap | 4 |
| tokio | 1 |
| ratatui | 0.29 |
| crossterm | 0.29 |
| reqwest | 0.12 |
| flate2 / xz2 / zstd / bzip2 / zip | 1 / 0.1 / 0.13 / 0.5 / 2 |
| sha2 / md-5 / blake3 | 0.10 / 0.10 / 1 |
| serde / toml / tracing / anyhow | 1 / 0.8 / 0.1 / 1 |

## Structural Seed

```text
src/
  cli/
  core/
  io/
  device/
  platform/
  tui/
  config/
  tests/
```

## Capability → Architecture Map

| Capability / Area | Lives in | Governed by |
| --- | --- | --- |
| Détection périphériques | `src/device` + `src/platform` | AD-1, AD-3 |
| Flash | `src/core/flasher`, `src/io/decompress` | AD-1, AD-4 |
| Clone | `src/core/cloner` | AD-1, AD-4 |
| Backup/Restore | `src/core/backup` + `src/io/decompress` | AD-1 |
| Vérification intégrité | `src/core/verify` | AD-4 |
| Partitionnement | `src/core/partition` + `src/tui/ui/partition` + `src/cli/commands/partition` | AD-2 |
| Orchestration utilisateur | `src/cli` + `src/tui` | AD-2 |
| Configuration | `src/config` | AD-2 |

## Deferred

- Validation exhaustive cross-platform (packaging, privilèges UAC/Authorization Services/démontage système) est reportée à l’étape d’intégration systémique après stabilisation des adaptateurs `platform`.
- Smart-clone sémantique et redimensionnement de partitions restent en phase de raffinement.
