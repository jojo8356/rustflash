# ATDD Checklist — Story 1-1

- **Story Key**: 1-1-initialize-cross-platform-device-support
- **Mode**: Red-phase
- **Date**: 2026-07-21

## Hypothèse

Les tests ci-dessous doivent être rédigés en premier (avant stabilisation complète de toutes implémentations).

## Scénarios d’acceptance (priorité P0)

1. **Listing device safe by default**
   - Étant donné un disque système et un disque amovible, quand `rustflash list` est exécuté sans `--all`, alors le disque système est absent.

2. **Listing include-system**
   - Avec `--all`, le disque système devient visible avec une indication claire.

3. **Sécurité destructive**
   - Pour une opération destructive, une confirmation explicite bloque l’action si non confirmée.

4. **Fallback robustesse list**
   - En cas d’échec `platform`/commande système, l’erreur est propagée comme message explicite.

## Gaps détectés

- Les tests unitaires spécifiques `Windows`/`macOS` détaillés requièrent environnements natifs dédiés.
- Prévoir backstops de parsing dans CI simulée.
