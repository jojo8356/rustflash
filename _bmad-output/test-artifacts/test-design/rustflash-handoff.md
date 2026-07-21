# Handoff TEA → BMad (RustFlash)

## Référence

- `prd.md`, `architecture.md`, `epics.md`
- Stories réalisées: `1-1`, `1-2`
- Exécutions techniques récentes: implémentations Linux/Windows/macOS listées en `implementation-artifacts`

## Décision de niveau

- **Niveau de test**: `system-level` (phase 3)
- **Risque principal**: plateforme et garde-fou système
- **Priorité**: P0 sur chemins destructeurs, P1 sur partitionnement

## Artefacts attendus pour suites suivantes

- `traceability-matrix.md`
- `gate-decision.json`
- `automation-summary.md`
- `atdd-checklist-1-1.md`
- `atdd-checklist-1-2.md`

## Instructions de continuité BMad

- Utiliser ce design comme source unique pour les prochains tickets d’automatisation.
- Mettre à jour les stories d’implémentation avec preuve d’exécution des cas critiques et du comportement de sécurité.
