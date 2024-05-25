
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
