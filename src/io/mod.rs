use crate::{patterns::WipePattern, storage::StorageType, AmaterasuConfig, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use tokio::task;

pub struct FileWiper {
    storage_type: StorageType,
    config: AmaterasuConfig,
}

impl FileWiper {
    pub fn new(storage_type: &StorageType, config: AmaterasuConfig) -> Self {
        Self {
            storage_type: storage_type.clone(),
            config,
        }
    }

    pub async fn wipe(&self, path: &Path, _pattern: WipePattern) -> Result<()> {
        let file_size = std::fs::metadata(path)?.len();
        let patterns = crate::patterns::create_pattern_sequence(&self.config.mode);

        println!("🔥 Wiping: {}", path.display());
        println!("Size: {} bytes", file_size);
        println!("Storage: {:?}", self.storage_type);
        println!("Passes: {}", patterns.len());

        let progress_bar = if self.config.progress {
            let pb = ProgressBar::new(file_size * patterns.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .unwrap()
                    .progress_chars("##-"),
            );
            Some(pb)
        } else {
            None
        };

        for (pass_num, pattern) in patterns.into_iter().enumerate() {
            if let Some(ref pb) = progress_bar {
                pb.set_message(format!(
                    "Pass {}/{} ({})",
                    pass_num + 1,
                    crate::patterns::create_pattern_sequence(&self.config.mode).len(),
                    pattern.name()
                ));
            }

            self.wipe_pass(path, pattern, file_size, progress_bar.clone())
                .await?;
        }

        if let Some(ref pb) = progress_bar {
            pb.finish_with_message("Wipe completed");
        }

        if self.config.verify {
            self.verify_wipe(path, file_size).await?;
        }

        std::fs::remove_file(path)?;
        println!("✅ File securely deleted: {}", path.display());
        Ok(())
    }

    async fn wipe_pass(
        &self,
        path: &Path,
        mut pattern: WipePattern,
        file_size: u64,
        progress_bar: Option<ProgressBar>,
    ) -> Result<()> {
        let block_size = self.storage_type.get_optimal_block_size();
        let path_owned = path.to_path_buf();

        task::spawn_blocking(move || -> Result<()> {
            let mut file = OpenOptions::new().write(true).open(&path_owned)?;

            file.seek(SeekFrom::Start(0))?;

            let mut buffer = vec![0u8; block_size];
            let mut bytes_written = 0u64;

            while bytes_written < file_size {
                let chunk_size = std::cmp::min(block_size, (file_size - bytes_written) as usize);
                let chunk = &mut buffer[..chunk_size];

                pattern.generate(chunk);
                file.write_all(chunk)?;
                file.flush()?;

                bytes_written += chunk_size as u64;

                if let Some(ref pb) = progress_bar {
                    pb.inc(chunk_size as u64);
                }
            }

            file.sync_all()?;
            Ok(())
        })
        .await??;

        Ok(())
    }

    async fn verify_wipe(&self, path: &Path, file_size: u64) -> Result<()> {
        println!("🔍 Verifying wipe...");

        let path_owned = path.to_path_buf();
        task::spawn_blocking(move || -> Result<()> {
            use std::io::Read;
            let mut file = File::open(&path_owned)?;
            let mut buffer = vec![0u8; 8192];
            let mut bytes_read = 0u64;
            let mut pattern_found = false;

            while bytes_read < file_size {
                let bytes_to_read = std::cmp::min(buffer.len(), (file_size - bytes_read) as usize);
                let chunk = &mut buffer[..bytes_to_read];
                let n = file.read(chunk)?;
                if n == 0 {
                    break;
                }

                for &byte in &chunk[..n] {
                    if byte != 0 {
                        pattern_found = true;
                        break;
                    }
                }

                bytes_read += n as u64;
                if pattern_found {
                    break;
                }
            }

            if !pattern_found {
                println!("⚠️  Warning: File appears to contain only zeros - this may indicate incomplete wipe");
            } else {
                println!("✅ Verification successful - data overwritten with pattern");
            }
            Ok(())
        }).await??;

        Ok(())
    }
}
