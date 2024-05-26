## algorithm
将提供一些常用的数据结构以供使用。

# lru 全称是Least Recently Used，即最近最久未使用的意思。
每次元素访问将其更新到列表的最前，时间复杂度为O(1)。当达到容量限制时将淘汰双向列表中的链尾数据
```rust
use algorithm::LruCache;
fn main() {
    let mut lru = LruCache::new(3);
    lru.insert("now", "ok");
    lru.insert("hello", "algorithm");
    lru.insert("this", "lru");
    lru.insert("auth", "tickbh");
    assert!(lru.len() == 3);
    assert_eq!(lru.get("hello"), Some(&"algorithm"));
    assert_eq!(lru.get("this"), Some(&"lru"));
    assert_eq!(lru.get("now"), None);
}
```
# lru-k
将访问次数达到k的目标值放进到优先队列，lru-k的主要目的是为了解决LRU算法“缓存污染”的问题，其核心思想是将“最近使用过1次”的判断标准扩展为“最近使用过K次”。
相比LRU，LRU-K需要多维护一个队列，用于记录所有缓存数据被访问的历史。只有当数据的访问次数达到K次的时候，才将数据放入缓存。当需要淘汰数据时，LRU-K会淘汰第K次访问时间距当前时间最大的数据。

```rust
use algorithm::LruTimeskCache;
fn main() {
    let mut lru = LruTimeskCache::new(3, 3);
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
```

# lfu (least frequently used)最近频次使用
每个元素在被访问或者更新的时候将其访问次数+1，当元素满时将优先淘汰掉访问次数最少的数据。
```rust

use algorithm::LfuCache;
fn main() {
    let mut lru = LfuCache::new(3);
    lru.insert("hello", "algorithm");
    lru.insert("this", "lru");
    lru.set_reduce_count(100);
    assert!(lru.get_visit(&"hello") == Some(5));
    assert!(lru.get_visit(&"this") == Some(5));
    for _ in 0..98 {
        let _ = lru.get("this");
    }
    assert!(lru.get_visit(&"this") == Some(51));
    assert!(lru.get_visit(&"hello") == Some(2));
    let mut keys = lru.keys();
    assert!(keys.next()==Some(&"this"));
    assert!(keys.next()==Some(&"hello"));
    assert!(keys.next() == None);
}
```
