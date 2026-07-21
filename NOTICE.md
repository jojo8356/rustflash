# Notice — RustFlash

RustFlash est un logiciel libre distribué sous licence **MIT OR Apache-2.0**.

Ce projet inclut des dépendances open source provenant de l’écosystème Rust (Rust crate ecosystem).  
Les licences applicables aux dépendances sont listées dans les fichiers `Cargo.toml` et `Cargo.lock`.

## Licence du projet

Copyright (c) 2026 - RustFlash contributors.

Le fichier `Cargo.toml` déclare :

```
license = "MIT OR Apache-2.0"
```

En pratique :

- toute redistribution peut suivre la licence MIT,
- ou la licence Apache-2.0,
- ou une compatibilité conforme aux termes de l’une des deux.

## Notices de dépendances

Le détail des dépendances de compilation et d’exécution (noms, versions, licences, checksums) se trouve dans `Cargo.lock`.  
Les dépendances principales incluent notamment :

- `ratatui`
- `crossterm`
- `clap`
- `tokio`
- `reqwest`
- `serde`
- `sha2`
- `tracing`
- `sysinfo`
- `criterion` (dépendance de benchmark)

Référence complète des notices : `Cargo.lock` et métadonnées officielles publiées de chaque crate.

## Remarque

Aucun composant tiers n’a été ajouté sans dépendance explicite dans `Cargo.toml`.

