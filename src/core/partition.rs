use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom, Write};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Énumération publique `TableType`
pub enum TableType {
    /// Variante d'énumération `Gpt` du type énuméré.
    Gpt,
    /// Variante d'énumération `Mbr` du type énuméré.
    Mbr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Énumération publique `FsType`
pub enum FsType {
    /// Variante d'énumération `Ext4` du type énuméré.
    Ext4,
    /// Variante d'énumération `Fat32` du type énuméré.
    Fat32,
    /// Variante d'énumération `Ntfs` du type énuméré.
    Ntfs,
    /// Variante d'énumération `ExFat` du type énuméré.
    ExFat,
    /// Variante d'énumération `Apfs` du type énuméré.
    Apfs,
    /// Variante d'énumération `Hfs` du type énuméré.
    Hfs,
    /// Variante d'énumération `Swap` du type énuméré.
    Swap,
    /// Variante d'énumération `Unknown` du type énuméré.
    Unknown,
}

impl FsType {
    /// Fonction publique `from_str`
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ext4" => Self::Ext4,
            "fat32" | "vfat" => Self::Fat32,
            "ntfs" => Self::Ntfs,
            "exfat" => Self::ExFat,
            "apfs" => Self::Apfs,
            "hfs" | "hfs+" => Self::Hfs,
            "swap" | "linux-swap" => Self::Swap,
            _ => Self::Unknown,
        }
    }

    /// Fonction publique `as_str`
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ext4 => "ext4",
            Self::Fat32 => "fat32",
            Self::Ntfs => "ntfs",
            Self::ExFat => "exfat",
            Self::Apfs => "apfs",
            Self::Hfs => "hfs+",
            Self::Swap => "swap",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Structure publique `PartitionInfo`
pub struct PartitionInfo {
    /// Champ public `number` de la structure correspondante.
    pub number: u32,
    /// Champ public `start_sector` de la structure correspondante.
    pub start_sector: u64,
    /// Champ public `end_sector` de la structure correspondante.
    pub end_sector: u64,
    /// Champ public `size_bytes` de la structure correspondante.
    pub size_bytes: u64,
    /// Champ public `fs_type` de la structure correspondante.
    pub fs_type: FsType,
    /// Champ public `label` de la structure correspondante.
    pub label: Option<String>,
    /// Champ public `flags` de la structure correspondante.
    pub flags: Vec<String>,
    /// Champ public `uuid` de la structure correspondante.
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, Copy)]
/// Énumération publique `EraseMethod`
pub enum EraseMethod {
    /// Variante d'énumération `Zero` du type énuméré.
    Zero,
    /// Variante d'énumération `Random` du type énuméré.
    Random,
    /// Variante d'énumération `Dod` du type énuméré.
    Dod,
}

#[derive(Debug, Clone)]
/// Structure publique `EraseProgress`
pub struct EraseProgress {
    /// Champ public `bytes_erased` de la structure correspondante.
    pub bytes_erased: u64,
    /// Champ public `total_bytes` de la structure correspondante.
    pub total_bytes: u64,
    /// Champ public `pass` de la structure correspondante.
    pub pass: u32,
    /// Champ public `total_passes` de la structure correspondante.
    pub total_passes: u32,
}

/// Parse a human-readable size string into bytes.
/// Supports: "256M", "4G", "1T", "1024", "512K"
pub fn parse_size(s: &str) -> anyhow::Result<u64> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("remaining") {
        return Ok(0); // sentinel: use all remaining space
    }

    let (num_str, multiplier) = if let Some(n) = s.strip_suffix(['K', 'k']) {
        (n, 1024u64)
    } else if let Some(n) = s.strip_suffix(['M', 'm']) {
        (n, 1024 * 1024)
    } else if let Some(n) = s.strip_suffix(['G', 'g']) {
        (n, 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix(['T', 't']) {
        (n, 1024 * 1024 * 1024 * 1024)
    } else {
        (s, 1u64)
    };

    let num: u64 = num_str
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid size: {s}"))?;
    Ok(num * multiplier)
}

/// Structure publique `PartitionManager`
pub struct PartitionManager;

impl PartitionManager {
    /// Read the partition table from a device or image file.
    pub fn read_table(device: &str) -> anyhow::Result<(TableType, Vec<PartitionInfo>)> {
        tracing::info!(device, "Reading partition table");

        // Try GPT first
        let cfg = gpt::GptConfig::new().writable(false);
        match cfg.open(device) {
            Ok(disk) => {
                let lbs = match disk.logical_block_size() {
                    gpt::disk::LogicalBlockSize::Lb512 => 512u64,
                    gpt::disk::LogicalBlockSize::Lb4096 => 4096u64,
                };

                let partitions: Vec<PartitionInfo> = disk
                    .partitions()
                    .iter()
                    .map(|(&num, part)| {
                        let size = (part.last_lba - part.first_lba + 1) * lbs;
                        PartitionInfo {
                            number: num,
                            start_sector: part.first_lba,
                            end_sector: part.last_lba,
                            size_bytes: size,
                            fs_type: gpt_type_to_fs(&part.part_type_guid),
                            label: if part.name.is_empty() {
                                None
                            } else {
                                Some(part.name.clone())
                            },
                            flags: Vec::new(),
                            uuid: Some(part.part_guid.to_string()),
                        }
                    })
                    .collect();

                Ok((TableType::Gpt, partitions))
            }
            Err(_) => {
                // Try MBR fallback
                read_mbr_table(device)
            }
        }
    }

    /// Create a new partition table on a device.
    pub fn create_table(device: &str, table_type: TableType) -> anyhow::Result<()> {
        tracing::info!(device, table = ?table_type, "Creating partition table");

        match table_type {
            TableType::Gpt => {
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(device)?;

                // Get device size
                let size = file.seek(SeekFrom::End(0))?;
                file.seek(SeekFrom::Start(0))?;

                if size < 34 * 512 {
                    anyhow::bail!("Device too small for GPT table (minimum ~17 KiB)");
                }

                let lb_size = (size / 512).saturating_sub(1);
                let lb_size_u32 = u32::try_from(lb_size.min(u32::MAX as u64))?;

                // Write protective MBR
                let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(lb_size_u32);
                mbr.overwrite_lba0(&mut file)?;

                // Create GPT
                let disk_device: Box<dyn gpt::DiskDevice> = Box::new(file);
                let mut gdisk = gpt::GptConfig::default()
                    .initialized(false)
                    .writable(true)
                    .logical_block_size(gpt::disk::LogicalBlockSize::Lb512)
                    .create_from_device(disk_device, None)?;

                gdisk.update_partitions(BTreeMap::new())?;
                gdisk.write()?;

                tracing::info!("GPT table created on {device}");
                Ok(())
            }
            TableType::Mbr => {
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(device)?;

                // Write a blank MBR: 512 bytes of zeros with boot signature
                let mut mbr = [0u8; 512];
                mbr[510] = 0x55;
                mbr[511] = 0xAA;

                file.seek(SeekFrom::Start(0))?;
                file.write_all(&mbr)?;
                file.flush()?;
                file.sync_all()?;

                tracing::info!("MBR table created on {device}");
                Ok(())
            }
        }
    }

    /// Add a partition to a GPT device.
    pub fn add_partition(
        device: &str,
        fs_type: FsType,
        size_bytes: u64,
        label: Option<&str>,
        _flags: &[&str],
    ) -> anyhow::Result<()> {
        tracing::info!(device, fs = ?fs_type, size = size_bytes, "Adding partition");

        let cfg = gpt::GptConfig::new().writable(true).initialized(true);
        let mut disk = cfg.open(device)?;

        let part_type = fs_to_gpt_type(fs_type);
        let part_name = label.unwrap_or("").to_string();

        let actual_size = if size_bytes == 0 {
            // "remaining" — find max free space
            let free = disk.find_free_sectors();
            let lbs = match disk.logical_block_size() {
                gpt::disk::LogicalBlockSize::Lb512 => 512u64,
                gpt::disk::LogicalBlockSize::Lb4096 => 4096u64,
            };
            free.iter().map(|(_, len)| len * lbs).max().unwrap_or(0)
        } else {
            size_bytes
        };

        if actual_size == 0 {
            anyhow::bail!("No free space available on {device}");
        }

        let id = disk.add_partition(&part_name, actual_size, part_type, 0, None)?;
        disk.write_inplace()?;

        tracing::info!(partition_id = id, size = actual_size, "Partition added");
        Ok(())
    }

    /// Delete a partition by number from a GPT device.
    pub fn delete_partition(device: &str, number: u32) -> anyhow::Result<()> {
        tracing::info!(device, number, "Deleting partition");

        let cfg = gpt::GptConfig::new().writable(true).initialized(true);
        let mut disk = cfg.open(device)?;

        // Get the partition's GUID for removal
        let part_guid = disk
            .partitions()
            .get(&number)
            .map(|p| p.part_guid)
            .ok_or_else(|| anyhow::anyhow!("Partition {number} not found"))?;

        disk.remove_partition(Some(number), Some(part_guid))?;
        disk.write_inplace()?;

        tracing::info!(number, "Partition deleted");
        Ok(())
    }

    /// Format a partition with the given filesystem.
    pub fn format_partition(
        device: &str,
        number: u32,
        fs_type: FsType,
        label: Option<&str>,
    ) -> anyhow::Result<()> {
        tracing::info!(device, number, fs = ?fs_type, "Formatting partition");

        // Build partition path
        let part_path = partition_path(device, number);

        if !std::path::Path::new(&part_path).exists() {
            anyhow::bail!("Partition device not found: {part_path}");
        }

        let mut cmd = match fs_type {
            FsType::Ext4 => {
                let mut c = std::process::Command::new("mkfs.ext4");
                c.arg("-F"); // force
                if let Some(l) = label {
                    c.arg("-L").arg(l);
                }
                c.arg(&part_path);
                c
            }
            FsType::Fat32 => {
                let mut c = std::process::Command::new("mkfs.vfat");
                c.arg("-F").arg("32");
                if let Some(l) = label {
                    c.arg("-n").arg(l);
                }
                c.arg(&part_path);
                c
            }
            FsType::Ntfs => {
                let mut c = std::process::Command::new("mkfs.ntfs");
                c.arg("-f"); // fast format
                if let Some(l) = label {
                    c.arg("-L").arg(l);
                }
                c.arg(&part_path);
                c
            }
            FsType::ExFat => {
                let mut c = std::process::Command::new("mkfs.exfat");
                if let Some(l) = label {
                    c.arg("-n").arg(l);
                }
                c.arg(&part_path);
                c
            }
            FsType::Swap => {
                let mut c = std::process::Command::new("mkswap");
                if let Some(l) = label {
                    c.arg("-L").arg(l);
                }
                c.arg(&part_path);
                c
            }
            _ => anyhow::bail!("Cannot format as {}", fs_type.as_str()),
        };

        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("mkfs failed: {stderr}");
        }

        tracing::info!(part_path, "Partition formatted");
        Ok(())
    }

    /// Securely erase a device by overwriting all data.
    pub async fn secure_erase(
        device: &str,
        method: EraseMethod,
        progress_tx: Option<mpsc::Sender<EraseProgress>>,
    ) -> anyhow::Result<()> {
        tracing::info!(device, method = ?method, "Secure erase");

        let device_owned = device.to_string();
        let block_size = 4 * 1024 * 1024usize; // 4 MiB

        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .read(true)
                .open(&device_owned)?;

            let total_bytes = file.seek(SeekFrom::End(0))?;
            file.seek(SeekFrom::Start(0))?;

            let total_passes: u32 = match method {
                EraseMethod::Zero => 1,
                EraseMethod::Random => 1,
                EraseMethod::Dod => 3, // zeros, ones, random
            };

            for pass in 0..total_passes {
                file.seek(SeekFrom::Start(0))?;
                let mut written: u64 = 0;
                let mut buf = vec![0u8; block_size];

                // Fill buffer based on pass
                match method {
                    EraseMethod::Zero => {} // already zeros
                    EraseMethod::Random => {
                        use rand::RngCore;
                        rand::thread_rng().fill_bytes(&mut buf);
                    }
                    EraseMethod::Dod => match pass {
                        0 => {}              // zeros
                        1 => buf.fill(0xFF), // ones
                        _ => {
                            use rand::RngCore;
                            rand::thread_rng().fill_bytes(&mut buf);
                        }
                    },
                }

                while written < total_bytes {
                    let remaining = (total_bytes - written) as usize;
                    let to_write = remaining.min(block_size);

                    // Re-randomize for Random/DoD pass 2
                    if matches!(method, EraseMethod::Random)
                        || (matches!(method, EraseMethod::Dod) && pass >= 2)
                    {
                        use rand::RngCore;
                        rand::thread_rng().fill_bytes(&mut buf[..to_write]);
                    }

                    file.write_all(&buf[..to_write])?;
                    written += to_write as u64;

                    if let Some(ref tx) = progress_tx {
                        let _ = tx.blocking_send(EraseProgress {
                            bytes_erased: written,
                            total_bytes,
                            pass: pass + 1,
                            total_passes,
                        });
                    }
                }

                file.flush()?;
                file.sync_all()?;
            }

            tracing::info!(device = device_owned, total_passes, "Secure erase complete");
            Ok(())
        })
        .await??;

        Ok(())
    }
}

/// Map a GPT partition type to our FsType enum.
fn gpt_type_to_fs(t: &gpt::partition_types::Type) -> FsType {
    if *t == gpt::partition_types::LINUX_FS {
        FsType::Ext4
    } else if *t == gpt::partition_types::EFI {
        FsType::Fat32
    } else if *t == gpt::partition_types::BASIC || *t == gpt::partition_types::WINDOWS_DATA {
        FsType::Ntfs
    } else if *t == gpt::partition_types::LINUX_SWAP {
        FsType::Swap
    } else if *t == gpt::partition_types::MACOS_APFS {
        FsType::Apfs
    } else if *t == gpt::partition_types::MACOS_HFSPLUS {
        FsType::Hfs
    } else {
        FsType::Unknown
    }
}

/// Map our FsType to a GPT partition type.
fn fs_to_gpt_type(fs: FsType) -> gpt::partition_types::Type {
    match fs {
        FsType::Ext4 => gpt::partition_types::LINUX_FS,
        FsType::Fat32 => gpt::partition_types::EFI,
        FsType::Ntfs | FsType::ExFat => gpt::partition_types::BASIC,
        FsType::Swap => gpt::partition_types::LINUX_SWAP,
        FsType::Apfs => gpt::partition_types::MACOS_APFS,
        FsType::Hfs => gpt::partition_types::MACOS_HFSPLUS,
        FsType::Unknown => gpt::partition_types::BASIC,
    }
}

/// Build the partition device path (e.g., /dev/sda → /dev/sda1).
fn partition_path(device: &str, number: u32) -> String {
    // NVMe style: /dev/nvme0n1 → /dev/nvme0n1p1
    // SCSI style: /dev/sda → /dev/sda1
    if device
        .chars()
        .last()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("{device}p{number}")
    } else {
        format!("{device}{number}")
    }
}

/// Try to read an MBR partition table.
fn read_mbr_table(device: &str) -> anyhow::Result<(TableType, Vec<PartitionInfo>)> {
    let mut file = std::fs::File::open(device)?;
    let mut mbr_bytes = [0u8; 512];
    file.read_exact(&mut mbr_bytes)?;

    // Validate MBR signature
    if mbr_bytes[510] != 0x55 || mbr_bytes[511] != 0xAA {
        anyhow::bail!("No valid partition table found on {device}");
    }

    let mut partitions = Vec::new();

    // Parse 4 MBR partition entries at offset 446, each 16 bytes
    for i in 0..4u32 {
        let offset = 446 + (i as usize * 16);
        let entry = &mbr_bytes[offset..offset + 16];

        let part_type = entry[4];
        if part_type == 0 {
            continue; // empty entry
        }

        let start_lba = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as u64;
        let sectors = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as u64;

        if sectors == 0 {
            continue;
        }

        let fs_type = mbr_type_to_fs(part_type);

        partitions.push(PartitionInfo {
            number: i + 1,
            start_sector: start_lba,
            end_sector: start_lba + sectors - 1,
            size_bytes: sectors * 512,
            fs_type,
            label: None,
            flags: if entry[0] == 0x80 {
                vec!["boot".into()]
            } else {
                Vec::new()
            },
            uuid: None,
        });
    }

    Ok((TableType::Mbr, partitions))
}

fn mbr_type_to_fs(type_id: u8) -> FsType {
    match type_id {
        0x83 => FsType::Ext4,                // Linux
        0x0B | 0x0C | 0x0E => FsType::Fat32, // FAT32 variants
        0x07 => FsType::Ntfs,                // NTFS/exFAT/HPFS
        0x82 => FsType::Swap,                // Linux swap
        0xAF => FsType::Hfs,                 // HFS+
        _ => FsType::Unknown,
    }
}
