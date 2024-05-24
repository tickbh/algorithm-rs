use std::hash::Hash;
use std::{mem, ptr};

mod lru;

pub use lru::LruCache;

struct KeyRef<K> {
    pub k: *const K,
}

impl<K> KeyRef<K> {
    pub fn new(k: *const K) -> Self {
        Self { k }
    }
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            (*self.k).hash(state);    
        }
    }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

struct LruEntry<K, V> {
    pub key: mem::MaybeUninit<K>,
    pub val: mem::MaybeUninit<V>,
    pub prev: *mut LruEntry<K, V>,
    pub next: *mut LruEntry<K, V>,
}

impl<K, V> LruEntry<K, V> {
    pub fn new_empty() -> Self {
        LruEntry {
            key: mem::MaybeUninit::uninit(),
            val: mem::MaybeUninit::uninit(),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }

    pub fn new(k: K, v: V) -> Self {
        LruEntry {
            key: mem::MaybeUninit::new(k),
            val: mem::MaybeUninit::new(v),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}
