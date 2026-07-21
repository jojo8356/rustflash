# PRD: RustFlash

---
title: RustFlash
status: draft
created: 2026-07-21
updated: 2026-07-21
project_name: rustflash
---

## 0. Document purpose

Ce PRD fixe la base de réalisation pour une suite d’outillage disque en Rust qui couvre le cœur des usages Balena Etcher (flash, clone, backup, restauration, partitionnement) avec une interface CLI + TUI.

Le périmètre retenu ici est pragmatique : livrer une version autonome opérationnelle, orientée stabilité et sécurité, avec une stratégie cross-platform cohérente.

## 1. Vision

RustFlash vise à fournir un remplaçant léger et robuste pour le flash d’images disque, la création de backup et la gestion de partitions sur postes de travail Linux/Windows/macOS.
Les priorités sont :

1. Fiabilité opérationnelle (pas de corruption silencieuse, pas de suppression accidentelle).
2. Performance acceptable sur flux de type I/O brut.
3. Expérience opératoire cohérente via CLI non-interactive + TUI.
4. Extensibilité multi-plateforme sans casser l’écosystème Rust natif.

## 2. Cible utilisateur

### 2.1 Utilisateurs principaux

- Utilisateurs techniques sur Linux, Windows, macOS.
- Utilisateurs de scripts CI ou automation.
- Développeurs qui maintiennent des outils de maintenance bas niveau.

### 2.2 Périmètre hors-scope v1

- Interface graphique desktop complète (Electron, Qt, etc.).
- Gestion de cloud / remote fleets.
- Gestion avancée de permissions OS pour UI graphique.

## 3. Contexte et exigences métier

- Supporter un flux de flash standard en local, sans dépendances lourdes.
- Préserver la sécurité du poste (bloquage implicite des disques systèmes).
- Être exécutable en mode batch pour scripting.
- Réduire la dette technique liée aux implémentations incomplètes plateforme (surtout Windows/macOS).

## 4. Exigences fonctionnelles

### FR-1 — Détection et sélection de périphériques

Le système doit détecter et lister les périphériques de stockage avec filtrage par défaut des disques systèmes, puis proposer une option explicite pour inclure les disques système.

### FR-2 — Flash multi-format robuste

L’utilisateur doit pouvoir flasher une image à partir :

- d’un fichier local (`.img`, `.iso`, `.bin`, `.raw`, `.gz`, `.xz`, `.zst`, `.bz2`, `.zip`)
- d’une URL HTTP/HTTPS avec reprise si disponible.

Le processus doit proposer confirmation explicite en mode destructif et fournir avancement + vérification optionnelle.

### FR-3 — Flash multi-cibles

Le système doit autoriser un flash simultané sur plusieurs cibles avec suivi de l’état par cible.

### FR-4 — Clone de disque

Le système doit prendre en charge :

- clone brut disque → disque/fichier,
- clone image → disque,
- compression d’export (gzip, xz, zstd),
- vérification d’intégrité après clone.

### FR-5 — Backup / Restore avec format propriétaire `.rfb`

Le système doit créer des backups `.rfb` compressés et restaurer depuis `.rfb` avec lecture d’en-tête, décompression et reconstruction.

### FR-6 — Gestion de partitions

L’utilisateur doit pouvoir lire/créer tables GPT/MBR, ajouter/supprimer/formater des partitions et effectuer un effacement sécurisé (zéro, random, DoD).

### FR-7 — Orchestration CLI + TUI

Toutes les fonctionnalités majeures doivent être accessibles par commandes CLI et via TUI :

- `flash`, `clone`, `backup`, `restore`, `partition`, `list`.

### FR-8 — Configuration persistante

Le système doit charger/enregistrer une configuration minimale (bloc, vérification, réseau, compression).

## 5. Exigences non-fonctionnelles

### NFR-1 — Sécurité opérationnelle

- Masquage des disques systèmes par défaut.
- Confirmation explicite sur opérations destructrices.
- Démontage automatique des partitions quand possible.

### NFR-2 — Intégrité

- Vérification de checksum d’images (`sha256`, `sha512`, `md5`, `blake3`) pour les sources de fichiers.
- Vérification post-flash quand demandée.

### NFR-3 — Performance

- Flux de lecture/écriture en mode bloc.
- Décompression transparente pour le pipeline flash/restore.

### NFR-4 — Expérience

- Messages d’état compréhensibles.
- États explicites dans le TUI (select → confirm → running → done/failed).

### NFR-5 — Maintenabilité

- Architecture modulaire par module (`core`, `io`, `device`, `platform`, `tui`, `cli`).
- Tests existants pour les chemins principaux de flash/clone.

## 6. MVP scope

### 6.1 Inclus

- Flash/clone/backup/restore/partitionnement fonctionnels.
- Vérification de checksum d’images.
- Multi-cibles pour flash.
- TUI + CLI couvrant les actions courantes.

### 6.2 Exclusions explicitement hors-scope (v1)

- Gestion complète de l’unicité du disque système sur tous OS (certaines implémentations encore incomplètes).
- Support complet du smart-clone sémantique basé filesystem.
- Intégration CI multi-OS validée pour toutes plateformes.

## 7. Hypothèses

1. Les opérations destructrices sur périphériques de production restent lancées par opérateurs informés.
2. Les environnements cible fournissent les droits nécessaires pour accès bloc brut.
3. Le coût de complexité lié au support avancé smart-clone est acceptable à moyen terme.

## 8. Critères de succès

- FR-1 à FR-8 exécutés avec les mêmes cas de base sur Linux.
- Flash multi-cible opérationnel en mode CLI/TUI.
- Aucune régression majeure sur les fonctions existantes de flash/clone/backup.
- Couverture de code de base pour les modules clés (au minimum flash/clone/verify/io/decompress).

## 9. To-do de conformité BMad (échéancier projet)

- [ ] Compléter la branche Windows/macOS de `platform` (enumeration/unmount/system detection partielle).
- [ ] Finaliser la vue Settings de la TUI.
- [ ] Stabiliser la couverture cross-platform (compilation et validations ciblées).
