# Test Design — Architecture Level (RustFlash)

- **Projet**: rustflash
- **Date**: 2026-07-21
- **Mode**: Mode système (phase 3)
- **Auteur**: rustflash
- **Scope**: cross-platform CLI/TUI et moteur de flash/clone/backup/partition

## Objectif du design

Établir une stratégie de test orientée risques qui couvre:

- stabilité de la couche plateforme (`platform` / `device`)
- sécurité destructive (`DeviceEnumerator`, filtres système)
- robustesse des chemins critiques I/O (flash, clone, backup, restore, partition)
- cohérence CLI ↔ TUI
- conformité de qualité non-fonctionnelle (NFR) aux exigences PRD/architecture.

## Architecture de test recommandée

### Couches testées

1. **Unité (Rust)**
   - logique de conversion/parsing (dimensions, formats)
   - heuristiques de filtrage systèmes
   - orchestration de la validation et du montage/démontage

2. **Intégration (Rust)**
   - exécutions CLI (`assert_cmd`) existantes
   - flux flash/clone/restore en mode simulé ou intégration légère
   - erreurs d’initialisation `AppConfig` et gestion de fallback

3. **Systèmes (contractuels)**
   - invariants des interfaces de plateforme
   - comportements croisés Linux/Windows/macOS pour `list`, `unmount`, `system disk`

4. **Smoke E2E / opérateur**
   - commandes critiques sur environnement local contrôlé
   - validations de non-régression TUI/CLI

## Risques principaux et couverture ciblée

- **Risque R1 – Mauvaise classification système/removable**
  - Impact: corruption accidentelle ou blocage de la sélection.
  - Couverture cible: tests de filtre et cas limites dans `src/platform/*` et `src/device`.

- **Risque R2 – Divergence CLI/TUI**
  - Impact: incohérence d’UX et comportements incohérents.
  - Couverture cible: validation end-to-end de paramètres de liste/flash/restore.

- **Risque R3 – Intégrité I/O**
  - Impact: pertes de données.
  - Couverture cible: tests sur utilitaires core/io avec chemins de base + erreurs d’E/S.

- **Risque R4 – Détection multi-plateforme partielle**
  - Impact: fonctionnalités non disponibles sur Windows/macOS.
  - Couverture cible: contrats d’implémentation et tests de parsing de sortie.

## Matrice FR → stratégie

| Regroupement FR | Risque | Test type | Priorité |
| --- | --- | --- | --- |
| FR-1 Détection périphériques | Élevé | Intégration + contrat plateforme | P0 |
| FR-2 Sécurité destructive | Élevé | Intégration + scénarios négatifs | P0 |
| FR-3/FR-5 listage & confirmation | Moyen | Intégration CLI + unit | P1 |
| FR-4 Unmount automatique | Moyen | Unit + intégration ciblée | P1 |
| FR-6 Partitionnement | Moyen | Intégration CLI | P1 |
| FR-7 CLI/TUI parity | Moyen | Smoke + snapshots de sortie | P2 |
| FR-8 Configuration | Moyen | Unit + vérification fallback | P1 |

## Sorties attendues

- `test-design-architecture.md` (ce document)
- `test-design-qa.md` (plan d’exécution)
- `test-design/rustflash-handoff.md` (liaison BMad)
