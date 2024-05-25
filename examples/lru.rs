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
