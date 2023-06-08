pub use data::Word;
pub use data::WORDS;

use rand::{seq::SliceRandom, Rng, RngCore};

mod data;
pub mod hash;

/// Returns a random word from [`WORDS`].
pub fn pick_random_word<R: Rng + ?Sized>(random_state: &mut R) -> Word {
    **WORDS.choose(random_state).expect("WORDS has no elements")
}

/// Returns a `&[u8; 5]` as a `&str`.
pub fn to_str(word: &Word) -> &str {
    unsafe { std::str::from_utf8_unchecked(word) }
}

/// `Rand` is a simple XorShift RNG.
pub struct Rand(u64);

impl RngCore for Rand {
    fn next_u64(&mut self) -> u64 {
        self.sample().to_le()
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl Rand {
    // TODO: let users pass in a seed
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn sample(&mut self) -> u64 {
        let x = &mut self.0;

        *x ^= *x << 13;
        *x ^= *x >> 17;
        *x ^= *x << 5;

        *x
    }
}

impl Default for Rand {
    /// Seeds the random state with the current time.
    fn default() -> Self {
        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos() as u64;
        Self(seed)
    }
}
