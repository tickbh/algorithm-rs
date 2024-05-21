use std::cmp::Ordering;
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

    for i in 8..32 {
        check_sort(i);
    }
    println!("finish check");
}

fn check_sort(idx: usize) {
    for _ in 0..100 {
        let mut rands: Vec<u32> = vec![];
        for _ in 0..idx {
            rands.push(rand::random::<u32>() % 20);
        }
        // rands = vec![0, 4, 16, 12, 15, 2, 16, 12, 6, 1, 17, 1];
        let mut copy_rands = rands.clone();
        rands.sort();
        quicksort(&mut copy_rands, |a, b| a < b);
        if rands != copy_rands {
            println!("rands = {:?}", rands);
            println!("copy_rands = {:?}", copy_rands);
        }
        assert!(rands == copy_rands);
    }

}
