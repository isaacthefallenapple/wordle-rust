pub use data::Word;
pub use data::WORDS;
use std::sync::Once;

mod data;
pub mod hash;

static mut STATE: Rand = Rand(0);
static INIT: Once = Once::new();

fn get_state() -> &'static mut Rand {
    unsafe {
        INIT.call_once(|| {
            STATE = Default::default();
        });
        &mut STATE
    }
}

/// Returns a random word from [`WORDS`].
pub fn pick_random_word() -> Word {
    let index = get_state().sample() % data::WORD_COUNT as u64;
    *WORDS[index as usize]
}

/// Returns a `&[u8; 5]` as a `&str`.
pub fn to_str(word: &Word) -> &str {
    unsafe { std::str::from_utf8_unchecked(word) }
}

/// `Rand` is a simple XorShift RNG.
struct Rand(u64);

impl Rand {
    // TODO: let users pass in a seed
    #[allow(unused)]
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn sample(&mut self) -> u64 {
        let x = &mut self.0;

        *x ^= *x << 13;
        *x ^= *x >> 17;
        *x ^= *x << 5;

        return *x;
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
