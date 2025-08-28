pub mod verification;

use crate::Result;
use std::path::Path;

pub trait VerificationMethod {
    fn verify(&self, path: &Path, expected_pattern: Option<&[u8]>) -> Result<bool>;
}

pub struct ReadbackVerifier;

impl VerificationMethod for ReadbackVerifier {
    fn verify(&self, path: &Path, expected_pattern: Option<&[u8]>) -> Result<bool> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; 8192];
        let file_size = std::fs::metadata(path)?.len();
        let mut bytes_read = 0u64;

        while bytes_read < file_size {
            let bytes_to_read = std::cmp::min(buffer.len(), (file_size - bytes_read) as usize);
            let chunk = &mut buffer[..bytes_to_read];
            
            let n = file.read(chunk)?;
            if n == 0 {
                break;
            }

            if let Some(expected) = expected_pattern {
                let pattern_len = expected.len();
                let start_offset = (bytes_read % pattern_len as u64) as usize;
                
                for (i, &byte) in chunk[..n].iter().enumerate() {
                    let pattern_idx = (start_offset + i) % pattern_len;
                    if byte != expected[pattern_idx] {
                        return Ok(false);
                    }
                }
            } else {
                for &byte in &chunk[..n] {
                    if byte != 0 {
                        return Ok(false);
                    }
                }
            }

            bytes_read += n as u64;
        }

        Ok(true)
    }
}