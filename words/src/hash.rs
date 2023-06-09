use super::Word;
use std::hash::{BuildHasher, Hasher};

pub fn perfect_hash(w: &Word) -> u64 {
    let mut hash = 0;
    for (i, byte) in w.iter().copied().enumerate() {
        // set the ith bit of the nth six-bit group in hash to the nth bit of the ith byte.
        for bit_idx in 0..5 {
            let bit = ((byte & (1 << bit_idx)) >> bit_idx) << i;
            let bit = bit as u64;
            hash |= bit << (bit_idx * 6);
        }
    }
    // shift more entropy into the top 7 bits because swiss table uses those to short circuit
    // hash lookups (I believe).
    hash << 2
}

#[derive(Default)]
pub struct WordHasher(Word);

pub struct WordHashBuilder;

impl Hasher for WordHasher {
    fn finish(&self) -> u64 {
        perfect_hash(&self.0)
    }

    fn write(&mut self, bytes: &[u8]) {
        // `[T; _]` will write its len as part of its `hash` impl
        // don't need to do that, because all our data has len 5.
        if bytes.len() == std::mem::size_of::<usize>() {
            return;
        }
        // assert `bytes` are a `Word`
        debug_assert!(
            bytes.is_ascii(),
            "word is not ascii, non ascii byte: {}",
            bytes.iter().find(|b| !b.is_ascii()).unwrap()
        );
        self.0 = bytes.try_into().expect("word does not have length 5");
    }
}

impl BuildHasher for WordHashBuilder {
    type Hasher = WordHasher;

    fn build_hasher(&self) -> Self::Hasher {
        WordHasher::default()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::data::WORDS;

    use super::*;

    #[test]
    fn test_is_perfect() {
        let mut hashes = HashSet::with_capacity(WORDS.len());
        for word in WORDS {
            let hash = perfect_hash(word);
            assert!(hashes.replace(hash).is_none());
        }
    }

    #[test]
    fn test_hashmap() {
        let mut hashes = HashMap::with_capacity_and_hasher(WORDS.len(), WordHashBuilder);
        for (i, word) in WORDS.iter().enumerate() {
            eprintln!("hashing: {}@{i}", std::str::from_utf8(&word[..]).unwrap());
            hashes.insert(word, i);
        }
        for (i, word) in WORDS.iter().enumerate() {
            let got = hashes[word];
            assert_eq!(
                got,
                i,
                "{} hashed to {got} instead of {i}",
                std::str::from_utf8(&word[..]).unwrap(),
            );
        }
    }
}
