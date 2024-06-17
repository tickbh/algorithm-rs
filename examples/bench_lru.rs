use std::time::Instant;
use algorithm::{ArcCache, LfuCache, LruCache, LruKCache};

macro_rules! do_test_bench {
    ($name: expr, $cache: expr, $num: expr, $evict: expr, $data: expr) => {
        let mut cost = vec![];
        let now = Instant::now();
        let mut all = 0;
        let mut hit = 0;
        for v in $data {
            if v.1 == 0 {
                all += 1;
                if $cache.get(&v.0).is_some() {
                    hit += 1;
                }
            } else {
                $cache.insert(v.0, v.1);
            }
        }
        cost.push(now.elapsed().as_micros());
        println!("|{}|{}|{:.2}%|", $name, cost.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\t"), hit as f64 * 100.0 / all as f64);
    };
}

#[allow(dead_code)]
fn build_order_data(num: usize) -> Vec<(usize, usize)> {
    let mut data = vec![];
    for i in 0..num {
        data.push((i, i + 1));
        data.push((i, 0));
    }
    data
}

#[allow(dead_code)]
fn build_freq_data(num: usize) -> Vec<(usize, usize)> {
    let mut data = vec![];
    for i in 0..num {
        data.push((i, i + 1));
        let ridx = i / 4 + 1;
        for _ in 0..1 {
            data.push((rand::random::<usize>() % ridx, 0));
        }
    }
    data
}


#[allow(dead_code)]
fn build_high_freq_data(num: usize) -> Vec<(usize, usize)> {
    let mut data = vec![];
    for i in 0..num {
        data.push((i, i + 1));
        let ridx = (i / 4 + 1).min(1000);
        for _ in 0..10 {
            data.push((rand::random::<usize>() % ridx, 0));
        }
        for _ in 0..5 {
            data.push((i + num + rand::random::<usize>() % num, i + 1));
        }
    }
    data
}

fn do_bench(num: usize) {
    let evict = num * 2;
    let mut lru = LruCache::new(num);
    let mut lruk = LruKCache::new(num);
    let mut lfu = LfuCache::new(num);
    let mut arc = ArcCache::new(num / 2);
    println!("|名字|耗时|命中率|");
    println!("|---|---|---|");
    // let data = build_freq_data(evict);
    let data = build_high_freq_data(evict);
    // let data = build_order_data(evict);
    do_test_bench!("LruCache", lru, num, evict, &data);
    do_test_bench!("LruKCache", lruk, num, evict, &data);
    do_test_bench!("LfuCache", lfu, num, evict, &data);
    do_test_bench!("ArcCache", arc, num, evict, &data);
}

fn main() {
    do_bench(1e5 as usize);
}