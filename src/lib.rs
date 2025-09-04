pub mod config;
pub mod filesystem;
pub mod io;
pub mod patterns;
pub mod security;
pub mod storage;

pub use anyhow::{Error, Result};
pub use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct AmaterasuConfig {
    pub verify: bool,
    pub progress: bool,
    pub force: bool,
    pub mode: WipeMode,
}

#[derive(Debug, Clone)]
pub enum WipeMode {
    Fast,
    Standard,
    Paranoid,
}

impl Default for AmaterasuConfig {
    fn default() -> Self {
        Self {
            verify: true,
            progress: true,
            force: false,
            mode: WipeMode::Standard,
        }
    }
}

pub struct Amaterasu {
    config: AmaterasuConfig,
}

impl Amaterasu {
    pub fn new(config: AmaterasuConfig) -> Self {
        Self { config }
    }

    pub async fn wipe_file(&self, path: &Path) -> Result<()> {
        let storage_type = storage::detector::detect_storage_type(path)?;
        let pattern_generator = patterns::create_random_generator();
        let wiper = io::FileWiper::new(&storage_type, self.config.clone());

        wiper.wipe(path, pattern_generator).await
    }

    pub async fn collect_files(&self, paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
        let mut files_to_wipe = Vec::new();

        for path in paths {
            if path.is_file() {
                files_to_wipe.push(path.clone());
            } else if path.is_dir() {
                if recursive {
                    let dir_files = self.collect_files_from_directory(path).await?;
                    files_to_wipe.extend(dir_files);
                } else if !self.config.force {
                    eprintln!("Warning: {} is a directory. Use -r/--recursive to delete directories and their contents.", path.display());
                }
            } else if !self.config.force {
                eprintln!(
                    "Warning: {} does not exist or is not a regular file/directory.",
                    path.display()
                );
            }
        }

        Ok(files_to_wipe)
    }

    async fn collect_files_from_directory(&self, dir_path: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut stack = vec![dir_path.to_path_buf()];

        while let Some(current_dir) = stack.pop() {
            let mut entries = fs::read_dir(&current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                if metadata.is_file() {
                    files.push(path);
                } else if metadata.is_dir() {
                    stack.push(path);
                }
            }
        }

        Ok(files)
    }

    pub async fn wipe_files(&self, paths: &[PathBuf]) -> Result<()> {
        for path in paths {
            if let Err(e) = self.wipe_file(path).await {
                if self.config.force {
                    eprintln!("Warning: Failed to wipe {}: {}", path.display(), e);
                } else {
                    return Err(e);
                }
            }
        }

        // After wiping all files, remove empty directories if any were processed
        if let Err(e) = self.cleanup_empty_directories(paths).await {
            if !self.config.force {
                return Err(e);
            }
        }

        Ok(())
    }

    async fn cleanup_empty_directories(&self, paths: &[PathBuf]) -> Result<()> {
        let mut dirs_to_remove = std::collections::HashSet::new();

        // Collect all parent directories of wiped files
        for path in paths {
            if let Some(parent) = path.parent() {
                dirs_to_remove.insert(parent.to_path_buf());
            }
        }

        // Sort directories by depth (deepest first) to remove them bottom-up
        let mut sorted_dirs: Vec<_> = dirs_to_remove.into_iter().collect();
        sorted_dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

        for dir in sorted_dirs {
            if let Ok(mut entries) = fs::read_dir(&dir).await {
                if entries.next_entry().await?.is_none() {
                    // Directory is empty, remove it
                    if let Err(e) = fs::remove_dir(&dir).await {
                        eprintln!(
                            "Warning: Could not remove empty directory {}: {}",
                            dir.display(),
                            e
                        );
                    } else {
                        println!("Removed empty directory: {}", dir.display());
                    }
                }
            }
        }

        Ok(())
    }
}
