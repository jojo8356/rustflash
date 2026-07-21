# Test Design — QA Execution (RustFlash)

- **Projet**: rustflash
- **Date**: 2026-07-21
- **Mode**: mode QA / exécution

## Objectif

Fournir un plan de test actionnable aligné avec `implementation-readiness` et les niveaux de risque.

## Scope immédiat

- `src/platform/{linux,windows,macos}.rs` (détection / filtrage / unmount)
- `src/device/*` (normalisation + règles de sécurité)
- `src/core/*` (flash/clone/backup/restore/partition/verify)
- `src/cli/*` et `src/tui/*` (cohérence d’interface)
- `tests/*.rs` (intégration Rust)

## Ordre d’exécution recommandé

1. **Préconditions**
   - `cargo check`
   - lint minimal (`cargo fmt --check`)
   - validation de config (`config` + répertoire home temporaire)

2. **Noyau de régression**
   - tests existants `tests/*.rs`
   - scénarios de robustesse d’erreur (fichier manquant, device inconnu)

3. **Couches avancées**
   - ATDD ciblé sur les flux critiques (si possible en mode simulé)
   - tests d’acceptance pour scénarios destructeurs et fallback
   - audit de cohérence NFR

4. **Méta-couverture / traceabilité**
   - revue de qualité et matrice FR → tests
   - gate de couverture par priorité

## Critères d’entrée

- PRD/architecture valides
- Code compiles
- Les workflows de test peuvent écrire dans `_bmad-output/test-artifacts`

## Critères de sortie

- `test-review.md` produit
- `nfr-assessment.md` produit
- `traceability-matrix.md` + `gate-decision.json` cohérents
- régression CLI/TUI validée sur commandes de base (`help`, `list`, `version`, flags clés)
