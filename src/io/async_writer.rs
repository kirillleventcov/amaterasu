use crate::{patterns::WipePattern, Result};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::task;

#[derive(Clone)]
pub struct BufferPool {
    buffers: Arc<Mutex<VecDeque<Vec<u8>>>>,
    buffer_size: usize,
    max_buffers: usize,
}

impl BufferPool {
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(VecDeque::new())),
            buffer_size,
            max_buffers,
        }
    }

    pub fn get_buffer(&self) -> Vec<u8> {
        let mut buffers = self.buffers.lock().unwrap();
        buffers
            .pop_front()
            .unwrap_or_else(|| vec![0u8; self.buffer_size])
    }

    pub fn return_buffer(&self, buffer: Vec<u8>) {
        let mut buffers = self.buffers.lock().unwrap();
        if buffers.len() < self.max_buffers && buffer.len() == self.buffer_size {
            buffers.push_back(buffer);
        }
    }
}

pub struct AsyncWiper {
    buffer_pool: BufferPool,
    concurrency_limit: Arc<Semaphore>,
}

impl AsyncWiper {
    pub fn new(buffer_size: usize) -> Self {
        let max_buffers = 16; // Keep up to 16 buffers cached
        let concurrency_limit = num_cpus::get().max(4); // At least 4 concurrent tasks

        Self {
            buffer_pool: BufferPool::new(buffer_size, max_buffers),
            concurrency_limit: Arc::new(Semaphore::new(concurrency_limit)),
        }
    }

    pub async fn wipe_chunk(
        &self,
        path: &Path,
        mut pattern: WipePattern,
        start_offset: u64,
        chunk_size: usize,
    ) -> Result<()> {
        let _permit = self.concurrency_limit.acquire().await.unwrap();
        let path_owned = path.to_path_buf();
        let buffer_pool = self.buffer_pool.clone();

        task::spawn_blocking(move || -> Result<()> {
            let mut buffer = buffer_pool.get_buffer();
            let buffer_len = buffer.len();
            let chunk = &mut buffer[..chunk_size.min(buffer_len)];

            // Generate pattern data
            pattern.generate(chunk);

            // Write to file
            let mut file = File::options().write(true).open(&path_owned)?;
            file.seek(SeekFrom::Start(start_offset))?;
            file.write_all(chunk)?;
            file.sync_data()?; // Use sync_data for better performance than sync_all

            buffer_pool.return_buffer(buffer);
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub async fn parallel_wipe(
        &self,
        path: &Path,
        pattern: WipePattern,
        file_size: u64,
        chunk_size: usize,
    ) -> Result<()> {
        let chunks = (file_size as usize + chunk_size - 1) / chunk_size;
        let mut tasks = Vec::new();

        for i in 0..chunks {
            let start_offset = (i * chunk_size) as u64;
            let current_chunk_size = if i == chunks - 1 {
                (file_size - start_offset) as usize
            } else {
                chunk_size
            };

            let task = self.wipe_chunk(path, pattern.clone(), start_offset, current_chunk_size);
            tasks.push(task);
        }

        // Execute all tasks in parallel
        futures::future::try_join_all(tasks).await?;

        Ok(())
    }
}

// Add Clone trait to WipePattern
impl Clone for WipePattern {
    fn clone(&self) -> Self {
        match self {
            WipePattern::Random(_) => {
                // Create a new random generator for each clone
                use rand::SeedableRng;
                WipePattern::Random(rand_chacha::ChaCha20Rng::from_entropy())
            }
            WipePattern::Fixed(byte) => WipePattern::Fixed(*byte),
            WipePattern::Zeros => WipePattern::Zeros,
            WipePattern::Ones => WipePattern::Ones,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as StdWrite;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_buffer_pool() {
        let pool = BufferPool::new(1024, 4);

        let buf1 = pool.get_buffer();
        let buf2 = pool.get_buffer();

        assert_eq!(buf1.len(), 1024);
        assert_eq!(buf2.len(), 1024);

        pool.return_buffer(buf1);
        pool.return_buffer(buf2);

        // Should reuse buffers
        let buf3 = pool.get_buffer();
        assert_eq!(buf3.len(), 1024);
    }

    #[tokio::test]
    async fn test_async_wiper() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(&[0u8; 1024])?;
        temp_file.flush()?;

        let wiper = AsyncWiper::new(256);
        let pattern = WipePattern::Fixed(0xAA);

        wiper
            .parallel_wipe(temp_file.path(), pattern, 1024, 256)
            .await?;

        // Verify the file was wiped
        let content = std::fs::read(temp_file.path())?;
        assert!(content.iter().all(|&b| b == 0xAA));

        Ok(())
    }
}
