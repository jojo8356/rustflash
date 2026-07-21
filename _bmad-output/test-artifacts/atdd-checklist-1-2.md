# ATDD Checklist — Story 1-2

- **Story Key**: 1-2-implement-cross-platform-device-enumeration
- **Mode**: Red-phase
- **Date**: 2026-07-21

## Scénarios d’acceptance (priorité P0)

1. **list_devices macOS fonctionnel**
   - Avec un périphérique USB présent, `list_devices` retourne path + size + modèle exploitable.

2. **Filtre système macOS**
   - En mode standard, disque système non retourné.

3. **Filtre systemes harmonisé**
   - Les mêmes règles applicatives que Linux/Windows (cohérence de sortie).

4. **Robustesse d’erreur**
   - En cas d’absence d’outil système, la fonction remonte erreur explicite + log contextuel.

## Gaps détectés

- Validation fine de parsing sur macOS réel : à confirmer en environnement macOS (commande `diskutil`).
