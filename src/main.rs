use std::cmp::Ordering;
use std::cmp::Ordering::Less;
use std::time::Instant;
use rand;

// use algorithm::quadsort::{quad_sort, tiny_sort};

use algorithm::quad_sort::quad_sort;

fn main() {
    println!("Hello, world!");
    // let mut words = vec!["cherry", "banana", "apple", "date"];
    // let mut copy_words = words.clone();
    // words.sort_by(|a, b| a.cmp(b)); // 默认就是按字典序排序
    // tiny_sort(&mut copy_words, &|a, b| a.cmp(b) == Ordering::Less);

    // assert!(words == copy_words);
    // println!("{:?}", words); // 输出: ["apple", "banana", "cherry", "date"]

    // for i in 33..63 {
    //     check_sort(i);
    // }
    // for i in 65..128 {
    //     check_sort(i);
    // }
    let mut cost_sort_time = 0;
    let mut cost_quad_time = 0;
    for i in 1..999 {
        check_sort(i, &mut cost_sort_time, &mut cost_quad_time);
    }
    // for i in 0..1 {
    //     check_sort(i, &mut cost_sort_time, &mut cost_quad_time);
    // }
    println!("cost sort time = {:?} cost quad time = {:?}", cost_sort_time, cost_quad_time);

    // let mut vec = Vec::with_capacity(120);
    // vec.push(1);
    // vec.push(1);
    // vec.push(1);
    // vec[2] = 10;
    // println!("ssss {:?}", vec);
    // println!("finish check");
}

fn check_sort(idx: usize, cost_sort_time: &mut u128, cost_quad_time: &mut u128) {
    for _ in 0..1 {
        let mut rands: Vec<usize> = vec![];
        for _ in 0..idx {
            rands.push(rand::random::<usize>() % idx);
        }
        // println!("start array = {:?}", rands);
        // rands = vec![4, 2, 0, 2, 2, 5, 1, 0, 0];
        // for _ in 0..120 {
        //     rands.swap(rand::random::<usize>() % idx, rand::random::<usize>() % idx);
        // }
        // rands = vec![0, 4, 16, 12, 15, 2, 16, 12, 6, 1, 17, 1];
        let mut ori = rands.clone();
        let mut copy_rands = rands.clone();
        let now = Instant::now();
        rands.sort_by(|a, b| a.cmp(b));
        // quicksort(&mut rands, |a, b| a < b);
        *cost_sort_time += now.elapsed().as_micros();
        let now = Instant::now();
        quad_sort(&mut copy_rands);
        *cost_quad_time += now.elapsed().as_micros();
        if rands != copy_rands {
            println!("rands = {:?}", rands);
            println!("copy_rands = {:?}", copy_rands);
            println!("ori = {:?}", ori);
        }
        assert!(rands == copy_rands);
    }

}
