
use algorithm::LruCache;

#[cfg(feature="ttl")]
fn run_ttl() {
    let mut lru = LruCache::new(3);
    lru.insert_with_ttl("help", "ok", 1);
    assert_eq!(lru.len(), 1);
    std::thread::sleep(std::time::Duration::from_secs(1));
    assert_eq!(lru.get("help"), None);
    assert_eq!(lru.len(), 0);
}
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

    #[cfg(feature="ttl")]
    run_ttl();
}
