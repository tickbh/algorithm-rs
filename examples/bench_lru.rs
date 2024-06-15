use std::collections::hash_map::RandomState;
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
        println!("{}\t{}\t{:.2}%", $name, cost.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\t"), hit as f64 * 100.0 / all as f64);
    };
}

fn build_order_data(num: usize) -> Vec<(usize, usize)> {
    let mut data = vec![];
    for i in 0..num {
        data.push((i, i + 1));
        data.push((i, 0));
    }
    data
}

fn build_freq_data(num: usize) -> Vec<(usize, usize)> {
    let mut data = vec![];
    for i in 0..num {
        data.push((i, i + 1));
        // data.push((i+1, i + 2));
        let ridx = i / 4 + 1;
        for _ in 0..1 {
            data.push((rand::random::<usize>() % ridx, 0));
        }
    }
    data
}

fn do_bench(num: usize) {
    let evict = num * 2;
    let mut lru = LruCache::<usize, usize, RandomState>::new(num);
    let mut lruk = LruKCache::<usize, usize, RandomState>::new(num);
    let mut lfu = LfuCache::<usize, usize, RandomState>::new(num);
    let mut arc = ArcCache::<usize, usize, RandomState>::new(num / 2);
    println!("名字\t耗时\t命中率\t");
    let order_data = build_freq_data(evict);
    do_test_bench!("LruCache", lru, num, evict, &order_data);
    // do_test_bench!("LruKCache", lruk, num, evict, &order_data);
    do_test_bench!("LfuCache", lfu, num, evict, &order_data);
    // do_test_bench!("ArcCache", arc, num, evict, &order_data);
    // println!("耗时:{}", set_timer);
}

fn main() {
    do_bench(1e4 as usize);
}