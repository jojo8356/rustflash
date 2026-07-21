# Changelog

## [0.1.1] - 2026-07-21

### Added
- Comprehensive Rustdoc coverage for public API items across CLI, core, device, IO and TUI modules.
- Generated and aligned project documentation files (`DOCS.md`, `NOTICE.md`) and completion-oriented README updates.
- Added end-to-end workflow and test-related documentation assets.

### Changed
- Improved doc comments consistency with `bmad-tech-writer` aligned phrasing.
- Refined module/struct/enum/function/class coverage to satisfy strict `missing-docs` checks.

### Quality
- Verified with:
  - `RUSTDOCFLAGS='-W missing-docs' cargo doc --no-deps --document-private-items`
  - No warnings/errors reported.

