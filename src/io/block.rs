use std::path::Path;

use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct BlockWriter {
    file: File,
    block_size: usize,
    bytes_written: u64,
}

impl BlockWriter {
    pub async fn open(device_path: &str, block_size: usize) -> anyhow::Result<Self> {
        tracing::debug!(device = device_path, block_size, "Opening device for writing");

        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(device_path)
            .await?;

        Ok(Self {
            file,
            block_size,
            bytes_written: 0,
        })
    }

    pub async fn write_block(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.file.write_all(data).await?;
        self.bytes_written += data.len() as u64;
        Ok(())
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        self.file.flush().await?;
        Ok(())
    }

    pub async fn sync(&self) -> anyhow::Result<()> {
        self.file.sync_all().await?;
        Ok(())
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }
}

pub struct BlockReader {
    file: File,
    block_size: usize,
    bytes_read: u64,
}

impl BlockReader {
    pub async fn open(path: &Path, block_size: usize) -> anyhow::Result<Self> {
        let file = tokio::fs::File::open(path).await?;
        Ok(Self {
            file,
            block_size,
            bytes_read: 0,
        })
    }

    pub async fn open_device(device_path: &str, block_size: usize) -> anyhow::Result<Self> {
        let file = tokio::fs::File::open(device_path).await?;
        Ok(Self {
            file,
            block_size,
            bytes_read: 0,
        })
    }

    pub async fn read_block(&mut self, buf: &mut [u8]) -> anyhow::Result<usize> {
        let n = self.file.read(buf).await?;
        self.bytes_read += n as u64;
        Ok(n)
    }

    /// Read exactly `buf.len()` bytes, or fewer only at EOF.
    pub async fn read_exact_block(&mut self, buf: &mut [u8]) -> anyhow::Result<usize> {
        let mut total = 0;
        while total < buf.len() {
            let n = self.file.read(&mut buf[total..]).await?;
            if n == 0 {
                break;
            }
            total += n;
        }
        self.bytes_read += total as u64;
        Ok(total)
    }

    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }
}
