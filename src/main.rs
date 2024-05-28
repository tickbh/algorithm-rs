

use algorithm::LruKCache;
fn main() {
    let mut lru = LruKCache::with_times(3, 3);
    lru.insert("this", "lru");
    for _ in 0..3 {
        let _ = lru.get("this");
    }
    lru.insert("hello", "algorithm");
    lru.insert("auth", "tickbh");
    assert!(lru.len() == 3);
    lru.insert("auth1", "tickbh");
    assert_eq!(lru.get("this"), Some(&"lru"));
    assert_eq!(lru.get("hello"), None);
    assert!(lru.len() == 3);
}

// use std::time::Instant;
// use rand;

// // use algorithm::quadsort::{quad_sort, tiny_sort};

// // use algorithm::quad_sort::quad_sort;

// use algorithm::LruCache;

// fn main() {

//     // let x: &i32 = unsafe { std::mem::MaybeUninit::uninit().assume_init() }; // undefined behavior! ⚠️

//     // println!("Hello, world! {:?} ===", x);
//     let mut lru = LruCache::new(3);
//     lru.insert(1, 2);
//     lru.insert(3, 3);
//     lru.insert(4, 4);
//     lru.get(&3);
//     lru.insert(9999, 2);
//     lru.insert(9999, 3);
//     println!("vvvvvvvvvvv len = {}", lru.len());
//     println!("vvvvvvvvvvv len = {:?}", lru.get(&9999));
//     println!("vvvvvvvvvvv len = {:?}", lru.get(&4));
//     println!("vvvvvvvvvvv len = {:?}", lru.get(&3));

//     for (k, v) in lru.iter() {
//         println!("k = {:?}", k);
//         println!("v = {:?}", v);
//     }
//     lru.retain(|_, v| *v % 2 == 0);
    
//     for (k, v) in lru.iter() {
//         println!("k = {:?}", k);
//         println!("v = {:?}", v);
//     }
//     // for k in lru.keys() {
//     //     println!("keys = {:?}", k);
//     // }
//     // for k in lru.values() {
//     //     println!("values = {:?}", k);
//     // }
//     return;
//     // v.drain()


//     // let mut words = vec!["cherry", "banana", "apple", "date"];
//     // let mut copy_words = words.clone();
//     // words.sort_by(|a, b| a.cmp(b)); // 默认就是按字典序排序
//     // tiny_sort(&mut copy_words, &|a, b| a.cmp(b) == Ordering::Less);

//     // assert!(words == copy_words);
//     // println!("{:?}", words); // 输出: ["apple", "banana", "cherry", "date"]

//     // for i in 33..63 {
//     //     check_sort(i);
//     // }
//     // for i in 65..128 {
//     //     check_sort(i);
//     // }
//     let mut cost_sort_vec = vec![];
//     let mut cost_quad_vec = vec![];
//     let times = 10;
//     for _ in 0..times {
//         let mut cost_sort_time = 0;
//         let mut cost_quad_time = 0;
//         // for i in 100000..100001 {
//         for i in 1..1999 {
//             check_sort(i, &mut cost_sort_time, &mut cost_quad_time);
//         }
//         cost_sort_vec.push(cost_sort_time);
//         cost_quad_vec.push(cost_quad_time);
//         // for i in 0..1 {
//         //     check_sort(i, &mut cost_sort_time, &mut cost_quad_time);
//         // }
//         // println!("cost sort time = {:?} cost quad time = {:?}", cost_sort_time, cost_quad_time);
//     }
//     fn aver(val: Vec<u128>) -> u128 {
//         let mut sum = 0;
//         for v in &val {
//             sum += v;
//         }
//         return sum / (val.len() as u128);
//     }
//     let aver_sort = aver(cost_sort_vec).max(1);
//     let aver_quad = aver(cost_quad_vec).max(1);

//     println!("time = {:?} cost sort time = {:?}ms cost quad time = {:?}ms, ratio = {:?}%", Instant::now(), aver_sort as f64 / 1000000f64, aver_quad as f64 / 1000000f64, (aver_quad - aver_sort) as f64 / aver_sort as f64 * 100f64);


//     // let mut vec = Vec::with_capacity(120);
//     // vec.push(1);
//     // vec.push(1);
//     // vec.push(1);
//     // vec[2] = 10;
//     // println!("ssss {:?}", vec);
//     // println!("finish check");
// }

// fn check_sort(idx: usize, cost_sort_time: &mut u128, cost_quad_time: &mut u128) {
//     for _ in 0..1 {
//         let mut rands: Vec<usize> = vec![];
//         for _i in 0..idx {
//             rands.push(rand::random::<usize>() % (idx * 10));
//             // rands.push(i * 10);
//         }
//         // println!("start array = {:?}", rands);
//         // rands = vec![26, 139, 38, 29, 48, 203, 132, 97, 190, 117, 244, 159, 212, 174, 50, 209, 113, 206, 144, 153, 43, 95, 165, 244, 169];
//         // for _ in 0..120 {
//         //     rands.swap(rand::random::<usize>() % idx, rand::random::<usize>() % idx);
//         // }
//         rands = vec![6, 47, 33, 67, 0, 74, 0, 91, 87, 85, 10];
//         let _ori = rands.clone();
//         // println!("ori = {:?}", ori);
//         let mut copy_rands = rands.clone();
//         let now = Instant::now();
//         // rands.sort_by(|a, b| a.cmp(b));
//         rands.sort();
//         // quicksort(&mut rands, |a, b| a < b);
//         *cost_sort_time += now.elapsed().as_nanos();
//         let now = Instant::now();
//         // quad_sort(&mut copy_rands);
//         // algorithm::quad_sort::quad_sort(&mut copy_rands);
//         algorithm::quadsort::quad_sort(&mut copy_rands);
//         *cost_quad_time += now.elapsed().as_nanos();
//         // if rands != copy_rands {
//         //     println!("rands = {:?}", rands);
//         //     println!("copy_rands = {:?}", copy_rands);
//         //     println!("ori = {:?}", ori);
//         // }
//         assert!(rands == copy_rands);
//     }

// }
