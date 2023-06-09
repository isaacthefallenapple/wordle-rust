#![feature(test)]

use std::collections::HashSet;
use words::WORDS;

use rustc_hash::FxHashSet;

extern crate test;

#[bench]
fn bench_hashmap_default(b: &mut test::Bencher) {
    let mut table = HashSet::with_capacity(WORDS.len());
    for word in WORDS {
        table.insert(word);
    }

    b.iter(|| {
        for word in WORDS {
            assert!(table.contains(word));
        }
    })
}

#[bench]
fn bench_hashmap_perfect(b: &mut test::Bencher) {
    let mut table = HashSet::with_capacity_and_hasher(WORDS.len(), words::hash::WordHashBuilder);
    for word in WORDS {
        table.insert(word);
    }

    b.iter(|| {
        for word in WORDS {
            assert!(table.contains(word));
        }
    })
}

#[bench]
fn bench_fxhashmap(b: &mut test::Bencher) {
    let mut table = FxHashSet::default();
    for word in WORDS {
        table.insert(word);
    }

    b.iter(|| {
        for word in WORDS {
            assert!(table.contains(word));
        }
    })
}

#[bench]
fn bench_binary_search(b: &mut test::Bencher) {
    b.iter(|| {
        for word in WORDS {
            assert!(WORDS.binary_search(&word).is_ok());
        }
    })
}
