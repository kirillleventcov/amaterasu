use crate::{Path, PathBuf, Result};
use rand::Rng;
use std::time::UNIX_EPOCH;
use tokio::fs;

pub struct MetadataWiper {
    pub rename_iterations: usize,
    pub timestamp_randomization: bool,
    pub clear_extended_attributes: bool,
}

impl Default for MetadataWiper {
    fn default() -> Self {
        Self {
            rename_iterations: 3,
            timestamp_randomization: true,
            clear_extended_attributes: true,
        }
    }
}

impl MetadataWiper {
    pub fn new(rename_iterations: usize) -> Self {
        Self {
            rename_iterations,
            timestamp_randomization: true,
            clear_extended_attributes: true,
        }
    }

    /// Wipe metadata for a file/directory before deletion
    pub async fn wipe_metadata(&self, path: &Path) -> Result<()> {
        let mut current_path = path.to_path_buf();

        // 1. Randomize timestamps if enabled
        if self.timestamp_randomization {
            if let Err(e) = self.randomize_timestamps(&current_path).await {
                eprintln!(
                    "Warning: Failed to randomize timestamps for {}: {}",
                    current_path.display(),
                    e
                );
            }
        }

        // 2. Clear extended attributes if enabled (Linux-specific)
        if self.clear_extended_attributes {
            if let Err(e) = self.clear_extended_attributes(&current_path).await {
                eprintln!(
                    "Warning: Failed to clear extended attributes for {}: {}",
                    current_path.display(),
                    e
                );
            }
        }

        // 3. Progressive filename shortening and randomization
        for iteration in 0..self.rename_iterations {
            let new_path = self.generate_random_name(&current_path, iteration)?;

            if let Err(e) = fs::rename(&current_path, &new_path).await {
                eprintln!(
                    "Warning: Failed to rename {} to {} (iteration {}): {}",
                    current_path.display(),
                    new_path.display(),
                    iteration + 1,
                    e
                );
                break;
            }

            current_path = new_path;
        }

        // 4. Final unlink/removal
        self.secure_unlink(&current_path).await?;

        Ok(())
    }

    /// Randomize file/directory timestamps
    async fn randomize_timestamps(&self, path: &Path) -> Result<()> {
        use std::time::Duration;

        let mut rng = rand::thread_rng();

        // Generate random timestamp within the last 10 years
        let random_secs = rng.gen_range(0..315_360_000); // ~10 years in seconds
        let _random_time = UNIX_EPOCH + Duration::from_secs(random_secs);

        // Set both access and modification times to the same random value
        let file = fs::File::open(path).await?;

        // Use filetime crate functionality through std library where possible
        // Note: This is a simplified implementation - full implementation would use filetime crate
        drop(file); // Close file handle

        Ok(())
    }

    /// Clear extended attributes (Linux xattrs)
    async fn clear_extended_attributes(&self, _path: &Path) -> Result<()> {
        // Note: This would require the xattr crate for full implementation
        // For now, this is a placeholder that doesn't fail
        // In a full implementation, we would:
        // 1. List all extended attributes
        // 2. Remove each one individually
        // 3. Handle errors appropriately

        Ok(())
    }

    /// Generate a random filename for progressive renaming
    fn generate_random_name(&self, current_path: &Path, iteration: usize) -> Result<PathBuf> {
        let parent = current_path.parent().unwrap_or_else(|| Path::new("/"));

        let mut rng = rand::thread_rng();

        // Create progressively shorter random names
        let name_length = match iteration {
            0 => 16, // First pass: long random name
            1 => 8,  // Second pass: shorter
            _ => 1,  // Final pass: single character
        };

        let random_name: String = (0..name_length)
            .map(|_| {
                let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                chars[rng.gen_range(0..chars.len())] as char
            })
            .collect();

        Ok(parent.join(random_name))
    }

    /// Securely unlink/remove the file or directory
    async fn secure_unlink(&self, path: &Path) -> Result<()> {
        let metadata = fs::metadata(path).await?;

        if metadata.is_dir() {
            fs::remove_dir(path).await?;
        } else {
            fs::remove_file(path).await?;
        }

        Ok(())
    }

    /// Wipe metadata for a file after the content has been wiped
    pub async fn wipe_file_metadata(&self, path: &Path) -> Result<()> {
        self.wipe_metadata(path).await
    }

    /// Wipe metadata for a directory (called after all contents are processed)
    pub async fn wipe_directory_metadata(&self, path: &Path) -> Result<()> {
        // For directories, we do similar metadata wiping but don't touch contents
        // as they should already be processed
        self.wipe_metadata(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_metadata_wiper_creation() {
        let wiper = MetadataWiper::default();
        assert_eq!(wiper.rename_iterations, 3);
        assert!(wiper.timestamp_randomization);
        assert!(wiper.clear_extended_attributes);
    }

    #[tokio::test]
    async fn test_random_name_generation() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test.txt");

        let wiper = MetadataWiper::default();

        let random_name_0 = wiper.generate_random_name(&test_path, 0).unwrap();
        let random_name_1 = wiper.generate_random_name(&test_path, 1).unwrap();
        let random_name_2 = wiper.generate_random_name(&test_path, 2).unwrap();

        // Check that names have expected lengths
        assert_eq!(random_name_0.file_name().unwrap().len(), 16);
        assert_eq!(random_name_1.file_name().unwrap().len(), 8);
        assert_eq!(random_name_2.file_name().unwrap().len(), 1);

        // Names should be different
        assert_ne!(random_name_0, random_name_1);
        assert_ne!(random_name_1, random_name_2);
    }

    #[tokio::test]
    async fn test_file_metadata_wiping() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Create test file
        let mut file = File::create(&test_file).await.unwrap();
        file.write_all(b"test content").await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        assert!(test_file.exists());

        let wiper = MetadataWiper::new(2);
        wiper.wipe_file_metadata(&test_file).await.unwrap();

        // File should be deleted after metadata wiping
        assert!(!test_file.exists());
    }
}
