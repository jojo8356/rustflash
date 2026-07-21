# Traceability Matrix — RustFlash

- **Date**: 2026-07-21
- **Scope**: FR/AC → tests

## Matrice FR → tests

| Requirement | Type | Test(s) | Statut |
| --- | --- | --- | --- |
| FR-1 Détection périphériques | Integration | `tests/cli_test.rs`, `tests/clone_test.rs` | Couvert |
| FR-2 Réservation système/désactivation | Integration | `tests/cli_test.rs`, tests de listing | Partiel |
| FR-3 / FR-4 Dispositif système | Integration | `src/platform/*` + atdd docs | Partiel |
| FR-5 Montage/démontage | Integration | `src/platform/*` | Partiel |
| FR-6 Partitionnement | Integration | `tests/partition_test.rs` | Couvert |
| FR-7 CLI/TUI parity | Manual/Automated | `tests/cli_test.rs` + TUI tests futurs | Partiel |
| FR-8 Config | Integration | `tests/cli_test.rs` + `src/config` | Partiel |

## Résultats de couverture

- Fichiers testés: 5
- Cas de test détectés: 38
- Couverture cible opérationnelle (P0/P1): partiellement démontrée

## Décision de gate (préliminaire)

- **Gating**: **CONCERNS**
- Motif: certaines exigences critiques sont couvertes de manière partielle hors-plateforme hardware.

