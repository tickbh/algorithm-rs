use std::cmp::Ordering;

use algorithm::quadsort::tiny_sort;

fn main() {
    println!("Hello, world!");
    let mut words = vec!["cherry", "banana", "apple", "date"];
    let mut copy_words = words.clone();
    words.sort_by(|a, b| a.cmp(b)); // 默认就是按字典序排序
    tiny_sort(&mut copy_words, &|a, b| a.cmp(b) == Ordering::Less);

    assert!(words == copy_words);
    println!("{:?}", words); // 输出: ["apple", "banana", "cherry", "date"]
}
