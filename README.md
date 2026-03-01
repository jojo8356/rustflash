# RustFlash

A fast, cross-platform tool for flashing OS images, cloning disks, managing backups and partitions. Built in Rust with both a TUI (terminal UI) and CLI interface.

## Features

- **Flash** — Write OS images (`.img`, `.iso`, `.raw`) to USB drives and SD cards with verification
- **Multi-format decompression** — Supports `.gz`, `.xz`, `.zst`, `.bz2`, `.zip` compressed images
- **Multi-device flash** — Flash the same image to multiple targets in parallel
- **Clone** — Raw disk-to-disk or disk-to-file cloning with optional compression
- **Backup / Restore** — Custom `.rfb` backup format with block-level compression (zstd, gzip, xz) and SHA-256 integrity
- **Partition Manager** — Read/create GPT and MBR tables, add/delete/format partitions, secure erase (zero, random, DoD)
- **TUI** — Full terminal interface with step-by-step wizards, progress bars, file browser, and device detection
- **CLI** — Scriptable command-line interface for all operations

## Installation

### From source

```bash
git clone https://github.com/jojo8356/rustflash.git
cd rustflash
cargo build --release
```

The binary will be at `target/release/rustflash`.

### Requirements

- Rust 2024 edition (nightly or 1.85+)
- Linux: no extra dependencies
- Operations on real devices require root privileges

## Usage

### TUI mode (default)

```bash
rustflash
```

### CLI mode

```bash
# Flash an image
rustflash flash --image ubuntu.iso --target /dev/sdb --verify

# Clone a disk
rustflash clone --source /dev/sda --dest /dev/sdb

# Backup a device
rustflash backup --source /dev/sda --output backup.rfb --compression zstd

# Restore a backup
rustflash restore --input backup.rfb --target /dev/sdb

# Manage partitions
rustflash partition /dev/sdb show
rustflash partition /dev/sdb create gpt
rustflash partition /dev/sdb add -t ext4 -s 4G -l mydata

# List devices
rustflash list
rustflash list --json
```

## Architecture

```
src/
  cli/          # CLI commands (clap)
  config/       # TOML configuration system
  core/         # Core engines (flasher, cloner, backup, partition)
  device/       # Device detection, filtering, mount/unmount
  io/           # Block I/O, decompression, downloads
  platform/     # Platform abstraction (Linux, macOS, Windows)
  tui/          # Terminal UI (ratatui)
    ui/         # Screen renderers (home, flash, clone, backup, partition)
    app.rs      # Application state machine
    theme.rs    # Theme system (dark, light, high contrast)
tests/          # Integration tests
benches/        # Criterion benchmarks
```

## Testing

```bash
cargo test
```

## License

MIT OR Apache-2.0
