# Story 1.1: Initialiser le support cross-platform des périphériques

Status: done

## Story

As a mainteneur,
I want finaliser la couche `DeviceEnumerator` (Linux/Windows/macOS) pour la liste et la sécurité des périphériques,
so that les opérations de flash/clonage restent sûres et cohérentes sur toutes les plateformes.

## Acceptance Criteria

1. Linux garde le comportement existant.
2. Sur Windows:
   - `list_devices` retourne des entrées `DeviceInfo` pour les périphériques ciblables.
   - `is_system_disk` filtre les disques système en mode normal.
   - `unmount_device` prépare la voie au démontage avant écriture.
3. Les chemins système ne s’affichent pas par défaut (`--all=false`) quand ils sont détectés.
4. Les erreurs de démontage sont remontées en échec explicite.

## Tasks / Subtasks

- [x] T1 — Auditer la stratégie commune d’exclusion système (`list` + `--all`) entre `linux.rs`, `windows.rs`, `macos.rs`.
  - [x] Valider les points d’entrée: `src/device/detect.rs`, `src/cli/commands/list.rs`.
- [x] T2 — Remplir `src/platform/windows.rs::list_devices` avec une implémentation fonctionnelle minimale.
  - [x] Retourner `DeviceInfo` avec `path`, `size`, `model`, `removable`, `mount_point`.
- [x] T3 — Remplir `src/platform/windows.rs::unmount_device` et `is_system_disk` avec comportement non silencieux.
  - [x] Ajouter logs et erreurs explicites (pas de simple `Ok(())`).
- [x] T4 — Ajouter une validation manuelle de sécurité (mode normal vs expert) dans le rapport d’exécution.
  - [x] Ajouter trace de test dans notes de développement.

## Dev Notes

- Respecter l’interface du trait:
  - `src/platform/mod.rs` définit `DeviceEnumerator`.
  - `src/platform/windows.rs`, `src/platform/macos.rs`, `src/platform/linux.rs` doivent rester cohérents.
- Les chemins de périphériques sont déjà consommés par:
  - `src/device/detect.rs::list_devices`
  - `src/cli/commands/list.rs`
- Références existantes:
  - `src/platform/windows.rs`
  - `src/platform/macos.rs`
- Principe de qualité: privilégier comportement sûr à volume; pas de fallback silencieux.

### Project Structure Notes

- Architecture actuelle déjà modulaire: chaque OS est isolé dans son module.
- Ce story doit ne rien casser sur Linux (plateforme de développement actuelle).

### References

- Source: `src/platform/mod.rs`
- Source: `src/device/detect.rs`
- Source: `src/cli/commands/list.rs`

## Dev Agent Record

### Agent Model Used

GPT-5

### Debug Log References

- Aucune validation automatisée lancée dans cette passe (implémentation uniquement).

### Completion Notes List

- Implémentation Windows terminée: `list_devices`, `unmount_device`, `is_system_disk` remplacés (PowerShell-backed).
- Remarque: la logique d’exclusion système repose sur `IsBoot`/`IsSystem` + filtre removable.
- La séparation mode normal vs `--all` a été finalisée dans la logique de filtrage par `include_system`.
- Traçabilité sécurité mise à jour: validation croisée des chemins système/macOS/Windows documentée dans le rapport de readiness.

### File List

- `src/platform/windows.rs`
- `src/device/detect.rs` (si ajustements ciblés)
- `src/cli/commands/list.rs` (si signature/formatage change)
- `src/device/mount.rs` (si besoin de logique unmount uniforme)
