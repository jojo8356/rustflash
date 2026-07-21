---
stepsCompleted:
  - "1"
  - "2"
  - "3"
inputDocuments:
  - "planning-artifacts/prd.md"
  - "planning-artifacts/architecture.md"
  - "CDC_RustEtcher.md"
---

# rustflash - Epic Breakdown

## Overview

Ce découpage transforme le PRD, l’architecture et les exigences techniques du CDC en éléments livrables pour lancer la prochaine étape de dev.  
Le périmètre de ce cycle est priorisé sur la **gestion des périphériques (découverte + sécurité de garde-fou)**, qui bloque aujourd’hui les opérations de flash/clonage/copy de manière cross-platform.

## Requirements Inventory

### Functional Requirements

- FR-1 — Lister les périphériques disponibles (CLI/TUI) de façon fiable sur Linux/Windows/macOS.
- FR-2 — Séparer explicitement les périphériques amovibles des disques système.
- FR-3 — Préparer un mode “tout inclure” (`--all`) avec confirmation explicite.
- FR-4 — Préparer et appliquer la logique d’unmount/détachement automatique avant opération.
- FR-5 — Détecter les chemins de périphérique déjà montés pour prévention des actions destructrices.
- FR-6 — Exposer un écran/chemin de configuration cohérent (ou explicite “TODO not implemented”) côté TUI.

### NonFunctional Requirements

- NFR-1 — Le comportement doit être stable par défaut: système = protégé hors mode expert.
- NFR-2 — Les messages d’erreur doivent être explicites et journalisés.
- NFR-3 — Le code doit rester non bloquant pour la CLI et compatible avec le runtime existant.

### Additional Requirements

- S’assurer que les TODO actifs ne cassent pas le workflow de runtime principal.
- Traiter Windows/macOS avant une release multi-plateforme.

### UX Design Requirements

- Vue cohérente dans la TUI/CLI avec indication claire des périphériques détectés, taille, modèle et point de montage.
- Messages de confirmation préventifs pour les opérations à risque.

### FR Coverage Map

| FR | Coverage |
| --- | --- |
| FR-1 | Epic 1 |
| FR-2 | Story 1.1 + Story 1.2 |
| FR-3 | Story 1.1 |
| FR-4 | Story 1.1 |
| FR-5 | Story 1.1 |
| FR-6 | Epic 1 (technique, note d’UI) |

## Epic List

## Epic 1: Support matériel cross-platform et sécurité système

**État global Epic 1:** 🔶 *Backlog* (pas encore stable, dépend des stories)

### Objectif

Sécuriser la couche `platform` et la logique de validation de périphérique afin que flash/clonage/restauration puissent fonctionner de manière fiable sur Linux/Windows/macOS, sans risque sur disque système.

### Story 1.1: Initialiser la couche de support cross-platform des périphériques

**État Story 1.1:** 🟠 *Backlog*  
**Réalisation actuelle:** Linux déjà opérationnel; Windows/macOS partiellement couverts (macOS unmount + is_system_disk, mais list_devices partiel; Windows entièrement TODO).

As a développeur CLI/TUI,
I want une implémentation commune de liste des périphériques + détection sécurité,
So that les actions de flash/clonage/reprise se basent sur une source fiable et uniforme.

**Acceptance Criteria:**

**Given** un périphérique amovible non-système connecté,  
**When** `rustflash list --all=false` est exécuté,  
**Then** il apparaît dans la liste avec `path`, `size`, `model` et `mount_point` s’il est monté.

**Given** `--all=false`,  
**When** un disque système existe,  
**Then** il est exclu de la sortie standard.

**Given** un périphérique déjà utilisé pour le système,  
**When** `unmount_device` est appelé,  
**Then** l’action échoue avec un message explicite si le démontage automatique est impossible.

**Given** une configuration multi-plateforme,
**When** la commande `rustflash list` s’exécute,
**Then** le comportement n’introduit pas de panique/erreur fatale liée aux stubs non implémentés.

**Tâches:**

- [ ] T1: Compléter l’implémentation de `windows.rs` (`list_devices`, `unmount_device`, `is_system_disk`) avec un comportement non-heuristique minimalement robuste.
- [ ] T2: Ajouter les règles de filtrage “system vs removable” alignées avec Linux (incluses dans le trait `DeviceEnumerator`).
- [ ] T3: Ajouter/mettre à jour la journalisation et les cas d’erreur retourables (failures explicites, pas de warning silencieux).
- [ ] T4: Documenter la stratégie de sécurité (exclusion système + mode expert `include_system`) dans la PRD/architecture de suivi.

### Story 1.2: Implémenter l’énumération macOS et finaliser l’exhaustivité cross-platform

**État Story 1.2:** 🔶 *Backlog*  
**Réalisation actuelle:** `unmount_device`/`is_system_disk` actifs, `list_devices` TODO.

As a développeur mainteneur,
I want une énumération des périphériques macOS cohérente avec Linux et Windows,
So that la liste CLI/TUI affiche les mêmes attributs et respecte le filtre système.

**Acceptance Criteria:**

**Given** un périphérique USB branché sur macOS,  
**When** `rustflash list` est exécuté,  
**Then** le périphérique est listé avec chemin et capacité.

**Given** un disque système en mode normal,  
**When** `rustflash list` est exécuté,  
**Then** ce disque ne s’affiche pas.

**Given** mode `--all`,  
**When** la liste est demandée,  
**Then** le disque système est visible avec indication claire.

**Given** la TUI demande la liste des périphériques,  
**When** `DeviceInfo` est inégal en champs entre plateformes,  
**Then** le mapping normalisé produit des valeurs cohérentes (fallback “Unknown” si non disponibles).

**Tâches:**

- [ ] T1: Remplacer le `TODO` dans `src/platform/macos.rs::list_devices` par une implémentation fonctionnelle.
- [ ] T2: Alignement des règles de filtrage système/removable avec la stratégie définie dans Story 1.1.
- [ ] T3: Ajouter tests ciblés (unitaires/invocation contrôlée) pour la conversion DeviceInfo sur macOS.
- [ ] T4: Ajouter une note de fallback propre si l’environnement macOS ne permet pas une lecture complète.
