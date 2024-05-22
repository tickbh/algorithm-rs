use std::cmp::Ordering;
use std::cmp::Ordering::Less;
use std::time::Instant;
use rand;

use algorithm::quadsort::{quicksort, tiny_sort};

fn main() {
    println!("Hello, world!");
    let mut words = vec!["cherry", "banana", "apple", "date"];
    let mut copy_words = words.clone();
    words.sort_by(|a, b| a.cmp(b)); // 默认就是按字典序排序
    tiny_sort(&mut copy_words, &|a, b| a.cmp(b) == Ordering::Less);

    assert!(words == copy_words);
    println!("{:?}", words); // 输出: ["apple", "banana", "cherry", "date"]

    // for i in 33..63 {
    //     check_sort(i);
    // }
    // for i in 65..128 {
    //     check_sort(i);
    // }
    let mut cost_sort_time = 0;
    let mut cost_quad_time = 0;
    for i in 1..2999 {
        check_sort(i, &mut cost_sort_time, &mut cost_quad_time);
    }

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
//         rands = vec![76, 83, 127, 76, 3, 21, 15, 28, 22, 129, 2, 121, 113, 73, 22, 126, 83, 114, 24, 21, 119, 129, 114, 43, 61, 8, 119, 27, 27, 108, 41, 123, 20, 83, 13, 11, 2, 60, 68, 104, 82, 76, 122, 49, 86, 102, 24, 27, 90, 104, 55, 126, 24, 84, 44, 111, 13, 72, 24, 0, 61, 65, 37, 101, 3, 19, 15, 72, 74, 69, 29, 119, 40, 46, 61, 80, 112, 82, 52, 96, 25, 93, 56
// , 78, 32, 34, 93, 61, 94, 116, 115, 26, 9, 89, 78, 29, 74, 32, 33, 103, 84, 127, 33, 31, 17, 120, 112, 75, 127, 45, 120, 13, 33, 97, 41, 67, 71, 40, 40, 12, 76, 89, 38, 127, 112, 103, 31, 74, 31, 49];
        // for _ in 0..120 {
        //     rands.swap(rand::random::<usize>() % idx, rand::random::<usize>() % idx);
        // }
        // rands = vec![0, 4, 16, 12, 15, 2, 16, 12, 6, 1, 17, 1];
        let mut copy_rands = rands.clone();
        let now = Instant::now();
        rands.sort_by(|a, b| a.cmp(b));
        *cost_sort_time += now.elapsed().as_micros();
        let now = Instant::now();
        quicksort(&mut copy_rands);
        *cost_quad_time += now.elapsed().as_micros();
        if rands != copy_rands {
            println!("rands = {:?}", rands);
            println!("copy_rands = {:?}", copy_rands);
        }
        assert!(rands == copy_rands);
    }

}
