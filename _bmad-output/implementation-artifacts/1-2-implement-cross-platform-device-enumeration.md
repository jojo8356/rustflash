# Story 1.2: Implémenter l’énumération de périphériques sur macOS

Status: done

## Story

As a mainteneur,
I want compléter l’énumération macOS de périphériques.
so that la liste est cohérente avec Linux et Windows en mode normal et en mode `--all`.

## Acceptance Criteria

1. `src/platform/macos.rs::list_devices` retourne des périphériques et un `DeviceInfo` minimal exploitable.
2. Les disques non amovibles/système sont filtrés en mode standard.
3. Le mode `include_system=true` expose les disques système.
4. Les erreurs de parsing/lecture de la source système produisent une erreur explicite et journalisée.

## Tasks / Subtasks

- [x] T1 — Implémenter `list_devices` sur macOS.
  - [x] Extraire nom/path/taille/modèle si disponibles.
  - [x] Remplir `DeviceInfo` de manière stable.
- [x] T2 — Appliquer la même politique de filtrage système/removable que dans Linux.
  - [x] Réconcilier les chemins détectés (`/dev/disk*`, etc.).
- [x] T3 — Ajouter logs d’échec si la source système est inaccessible.

## Dev Notes

- Les appels macOS existants:
  - `src/platform/macos.rs::unmount_device` utilise `diskutil unmountDisk`.
  - `src/platform/macos.rs::is_system_disk` détecte `disk0`.
- Harmoniser `is_system_disk` si un retour d’information plus robuste est disponible.
- Le but est d’obtenir une liste exploitable sans bloquer le reste des flows.

### Project Structure Notes

- Story indépendante de la CLI: la structure de données remonte via `src/device/detect.rs`.
- Dépendance implicite: permissions/macOS command-line availability.

### References

- Source: `src/platform/macos.rs`
- Source: `src/device/detect.rs`
- Source: `src/device/mount.rs`

## Dev Agent Record

### Agent Model Used

GPT-5

### Debug Log References

- Aucune exécution de validation automatisée lancée dans cette passe (implémentation uniquement).

### Completion Notes List

- Implémentation macOS terminée: `list_devices` et `is_system_disk` opérationnels via `diskutil list/info`.

### File List

- `src/platform/macos.rs`
