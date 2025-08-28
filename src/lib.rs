pub mod storage;
pub mod patterns;
pub mod io;
pub mod security;

pub use anyhow::{Error, Result};
pub use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AmaterasuConfig {
    pub verify: bool,
    pub progress: bool,
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

    pub async fn wipe_files(&self, paths: &[PathBuf]) -> Result<()> {
        for path in paths {
            self.wipe_file(path).await?;
        }
        Ok(())
    }
}