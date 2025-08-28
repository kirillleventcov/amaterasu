pub mod random;

use rand_chacha::ChaCha20Rng;

pub trait PatternGenerator {
    fn generate(&mut self, buffer: &mut [u8]);
    fn name(&self) -> &str;
}

pub enum WipePattern {
    Random(ChaCha20Rng),
    Fixed(u8),
    Zeros,
    Ones,
}

impl WipePattern {
    pub fn generate(&mut self, buffer: &mut [u8]) {
        match self {
            WipePattern::Random(rng) => {
                use rand::RngCore;
                rng.fill_bytes(buffer);
            }
            WipePattern::Fixed(byte) => {
                buffer.fill(*byte);
            }
            WipePattern::Zeros => {
                buffer.fill(0x00);
            }
            WipePattern::Ones => {
                buffer.fill(0xFF);
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            WipePattern::Random(_) => "random",
            WipePattern::Fixed(byte) => match *byte {
                0x55 => "0x55",
                0xAA => "0xAA",
                _ => "fixed",
            }
            WipePattern::Zeros => "zeros",
            WipePattern::Ones => "ones",
        }
    }
}

pub fn create_random_generator() -> WipePattern {
    use rand::SeedableRng;
    let rng = ChaCha20Rng::from_entropy();
    WipePattern::Random(rng)
}

pub fn create_pattern_sequence(mode: &crate::WipeMode) -> Vec<WipePattern> {
    match mode {
        crate::WipeMode::Fast => vec![create_random_generator()],
        crate::WipeMode::Standard => vec![
            create_random_generator(),
            WipePattern::Zeros,
            create_random_generator(),
        ],
        crate::WipeMode::Paranoid => vec![
            create_random_generator(),
            WipePattern::Fixed(0x55),
            WipePattern::Fixed(0xAA),
            create_random_generator(),
            WipePattern::Ones,
            WipePattern::Zeros,
            create_random_generator(),
        ],
    }
}