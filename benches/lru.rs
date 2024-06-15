// bench.rs
#![feature(test)]

extern crate test;

use algorithm::{ArcCache, LfuCache, LruCache, LruKCache};
use test::Bencher;

static BENCH_SIZE: usize = 10000;

macro_rules! do_test_bench {
    ($cache: expr) => {
        for i in 0..BENCH_SIZE {
            $cache.insert(i, i);
            $cache.get(&i);
        }
    };
}
#[bench]
fn calc_lru(b: &mut Bencher) {
    b.iter(|| {
        let mut lru = LruCache::new(BENCH_SIZE / 2);
        do_test_bench!(lru);
    })
}


#[bench]
fn calc_lruk(b: &mut Bencher) {
    b.iter(|| {
        let mut lruk = LruKCache::new(BENCH_SIZE / 2);
        do_test_bench!(lruk);
    })
}

#[bench]
fn calc_lfu(b: &mut Bencher) {
    b.iter(|| {
        let mut lfu = LfuCache::new(BENCH_SIZE / 2);
        do_test_bench!(lfu);
    })
}

#[bench]
fn calc_arc(b: &mut Bencher) {
    b.iter(|| {
        let mut arc = ArcCache::new(BENCH_SIZE / 2);
        do_test_bench!(arc);
    })
}

