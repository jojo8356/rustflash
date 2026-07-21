# Testing Stack for RustFlash

## Type de stack retenue

Ce projet est un outil Rust de bas niveau, sans dépendance front-end UI automatisable.
Le stack de tests retenu est:

- **Rust unit/integration tests** (`cargo test`)
- **CLI tests** via `assert_cmd` + `predicates`
- **Benchmarks IO** (`criterion`) pour perf exploratoire

## Organisation

- `tests/*.rs` : intégration CLI/I/O
- `src/**/*.rs` : modules de production

## Bonnes pratiques appliquées

- chaque test d’échec doit être explicite sur le code de sortie
- les commandes sensibles doivent être testées en mode non destructif quand possible
- éviter les effets de bord réels disque quand un stub ou un mode simulé existe

## Exécution locale

- `cargo test`
- (optionnel) `cargo test -- --nocapture`
- (optionnel) `cargo bench` si des objectifs perf ponctuels sont définis
