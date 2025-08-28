use super::PatternGenerator;
use rand::RngCore;
use rand_chacha::ChaCha20Rng;

pub struct SecureRandomGenerator {
    rng: ChaCha20Rng,
}

impl SecureRandomGenerator {
    pub fn new() -> Self {
        use rand::SeedableRng;
        Self {
            rng: ChaCha20Rng::from_entropy(),
        }
    }
}

impl PatternGenerator for SecureRandomGenerator {
    fn generate(&mut self, buffer: &mut [u8]) {
        self.rng.fill_bytes(buffer);
    }

    fn name(&self) -> &str {
        "secure_random"
    }
}

impl Default for SecureRandomGenerator {
    fn default() -> Self {
        Self::new()
    }
}