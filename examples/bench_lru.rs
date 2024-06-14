use std::collections::hash_map::RandomState;
use std::time::Instant;
use algorithm::{ArcCache, LfuCache, LruCache, LruKCache};

macro_rules! do_test_bench {
    ($name: expr, $cache: expr, $num: expr, $evict: expr, $data1: expr, $data2: expr) => {
        let mut cost = vec![];
        let now = Instant::now();
        for i in 0..$num {
            $cache.insert($data1[i], $data1[i]);
        }
        cost.push(now.elapsed().as_micros());

        let now = Instant::now();
        for i in 0..$num {
            $cache.get(&$data1[i]);
        }
        cost.push(now.elapsed().as_micros());
        
        let now = Instant::now();
        for i in 0..$num {
            $cache.get(&$data2[i]);
        }
        cost.push(now.elapsed().as_micros());
        println!("{} 耗时:{}", $name, cost.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\t"));
    };
}

fn do_bench(num: usize, times: usize) {
    let evict = num * 2;
    let mut data1 = (0..num).collect::<Vec<_>>();
    let mut data2 = vec![];
    for _ in 0..evict {
        data2.push(rand::random::<usize>() % evict);
    }
    let mut lru = LruCache::<usize, usize, RandomState>::new(num);
    let mut lruk = LruKCache::<usize, usize, RandomState>::new(num);
    let mut lfu = LfuCache::<usize, usize, RandomState>::new(num);
    let mut arc = ArcCache::<usize, usize, RandomState>::new(num);
    do_test_bench!("LruCache", lru, num, evict, &data1, &data2);
    do_test_bench!("LruKCache", lruk, num, evict, &data1, &data2);
    do_test_bench!("LfuCache", lfu, num, evict, &data1, &data2);
    do_test_bench!("ArcCache", arc, num, evict, &data1, &data2);
    // println!("耗时:{}", set_timer);
}

fn main() {
    do_bench(1e5 as usize, 5);
}