use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::Path;

use anyhow::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Raw,
    Gzip,
    Xz,
    Zstd,
    Bzip2,
    Zip,
}

pub fn detect_format(path: &Path) -> ImageFormat {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check double extensions like .img.gz
    let full_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if full_name.ends_with(".img.gz") || ext == "gz" {
        ImageFormat::Gzip
    } else if full_name.ends_with(".img.xz") || ext == "xz" {
        ImageFormat::Xz
    } else if full_name.ends_with(".img.zst") || ext == "zst" {
        ImageFormat::Zstd
    } else if full_name.ends_with(".img.bz2") || ext == "bz2" {
        ImageFormat::Bzip2
    } else if ext == "zip" {
        ImageFormat::Zip
    } else {
        ImageFormat::Raw
    }
}

/// Image file extensions considered flashable inside a ZIP archive.
const IMAGE_EXTENSIONS: &[&str] = &["img", "iso", "raw", "bin", "dmg", "wic"];

pub fn open_image(path: &Path) -> anyhow::Result<Box<dyn Read + Send>> {
    let format = detect_format(path);
    let file = File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;

    match format {
        ImageFormat::Raw => Ok(Box::new(BufReader::new(file))),
        ImageFormat::Gzip => {
            let decoder = flate2::read::GzDecoder::new(BufReader::new(file));
            Ok(Box::new(decoder))
        }
        ImageFormat::Xz => {
            let decoder = xz2::read::XzDecoder::new(BufReader::new(file));
            Ok(Box::new(decoder))
        }
        ImageFormat::Zstd => {
            let decoder = zstd::stream::read::Decoder::new(BufReader::new(file))?;
            Ok(Box::new(decoder))
        }
        ImageFormat::Bzip2 => {
            let decoder = bzip2::read::BzDecoder::new(BufReader::new(file));
            Ok(Box::new(decoder))
        }
        ImageFormat::Zip => open_zip_image(file),
    }
}

/// Open the best image file from inside a ZIP archive.
/// Strategy: find the largest file with an image extension, or the largest file overall.
fn open_zip_image(file: File) -> anyhow::Result<Box<dyn Read + Send>> {
    let mut archive =
        zip::ZipArchive::new(BufReader::new(file)).context("Failed to read ZIP archive")?;

    // Find best candidate: prefer files with image extensions, pick the largest
    let mut best_image: Option<(usize, u64)> = None;
    let mut best_any: Option<(usize, u64)> = None;

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }

        let size = entry.size();
        let name = entry.name().to_lowercase();

        let has_image_ext = IMAGE_EXTENSIONS
            .iter()
            .any(|ext| name.ends_with(&format!(".{ext}")));

        if has_image_ext {
            if best_image.map_or(true, |(_, s)| size > s) {
                best_image = Some((i, size));
            }
        }
        if best_any.map_or(true, |(_, s)| size > s) {
            best_any = Some((i, size));
        }
    }

    let (idx, _) = best_image
        .or(best_any)
        .ok_or_else(|| anyhow::anyhow!("ZIP archive is empty"))?;

    // Extract the file into memory so we can return a Send reader.
    // ZIP entries borrow the archive, so we can't return them directly.
    let mut entry = archive.by_index(idx)?;
    let entry_name = entry.name().to_string();
    let entry_size = entry.size();

    tracing::info!(
        name = entry_name,
        size = entry_size,
        "Extracting image from ZIP"
    );

    let mut buf = Vec::with_capacity(entry_size as usize);
    entry
        .read_to_end(&mut buf)
        .context("Failed to extract image from ZIP")?;

    Ok(Box::new(Cursor::new(buf)))
}
