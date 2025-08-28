use super::{ReadbackVerifier, VerificationMethod};
use crate::Result;
use std::path::Path;

pub struct WipeVerifier {
    method: Box<dyn VerificationMethod>,
}

impl WipeVerifier {
    pub fn new() -> Self {
        Self {
            method: Box::new(ReadbackVerifier),
        }
    }

    pub fn verify_zero_fill(&self, path: &Path) -> Result<bool> {
        self.method.verify(path, None)
    }

    pub fn verify_pattern(&self, path: &Path, pattern: &[u8]) -> Result<bool> {
        self.method.verify(path, Some(pattern))
    }
}

impl Default for WipeVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_zero_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(&[0u8; 1024])?;
        temp_file.flush()?;

        let verifier = WipeVerifier::new();
        let result = verifier.verify_zero_fill(temp_file.path())?;
        assert!(result);

        Ok(())
    }

    #[test]
    fn test_pattern_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let pattern = [0xAA, 0x55];
        let data = pattern.repeat(512);
        temp_file.write_all(&data)?;
        temp_file.flush()?;

        let verifier = WipeVerifier::new();
        let result = verifier.verify_pattern(temp_file.path(), &pattern)?;
        assert!(result);

        Ok(())
    }
}