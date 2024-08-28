use std::{thread, time::{Duration, Instant}};

use algorithm::LruCache;
use algorithm_macro::cache;

#[cache(LruCache : LruCache::new(20))]
#[cache_cfg(ignore_args = call_count)]
#[cache_cfg(thread)]
fn fib(x: u64, call_count: &mut u32) -> u64 {
    *call_count += 1;
    if x <= 1 {
        1
    } else {
        fib(x - 1, call_count) + fib(x - 2, call_count)
    }
}

#[cache(LruCache : LruCache::new(20))]
fn slow_func(u: u64) -> u64 {
    thread::sleep(Duration::from_secs(1));
    u * 10
}

fn slow_func_not_cache(u: u64) -> u64 {
    thread::sleep(Duration::from_secs(1));
    u * 10
}
fn main() {
    let now = Instant::now();
    let cache_ret: u64 = (0..21).map(|v| slow_func(v % 3)).into_iter().sum();
    let cache_elapsed = now.elapsed();
    
    let now = Instant::now();
    let normal_ret: u64 = (0..21).map(|v| slow_func_not_cache(v % 3)).into_iter().sum();
    let normal_elapsed = now.elapsed();

    assert_eq!(cache_ret, normal_ret);
    assert!(normal_elapsed.as_secs() > cache_elapsed.as_secs() * 6);

    println!("cache_elapsed = {}ms", cache_elapsed.as_millis());
    println!("normal_elapsed = {}ms", normal_elapsed.as_millis());
    // let mut call_count = 0;
    // assert_eq!(fib(39, &mut call_count), 102_334_155);
    // assert_eq!(call_count, 40);
    // const CALC_VALUE: u128 = 99;
    // let now = Instant::now();
    // let cache_ret = cache_fib(CALC_VALUE);
    // let cache_elapsed = now.elapsed();
    
    // let now = Instant::now();
    // let normal_ret = fibonacci(CALC_VALUE);
    // let normal_elapsed = now.elapsed();

    // assert_eq!(cache_ret, normal_ret);
    // println!("cache_elapsed = {}ms", cache_elapsed.as_millis());

    // println!("normal_elapsed = {}ms", normal_elapsed.as_millis());

}
