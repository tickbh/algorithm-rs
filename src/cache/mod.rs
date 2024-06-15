use std::borrow::Borrow;
use std::hash::Hash;

mod lfu;
mod lru;
mod lruk;
mod arc;
mod slab;

pub use lru::LruCache;
pub use lruk::LruKCache;
pub use lfu::LfuCache;
pub use arc::ArcCache;
pub use slab::{Slab, Reinit};

#[derive(Clone)]
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

impl<K: Eq> PartialOrd for KeyRef<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.k.partial_cmp(&other.k)
    }
}

impl<K: Eq> Ord for KeyRef<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.k.cmp(&other.k)
    }
}

// 确保新类型与其内部类型的内存布局完全相同
#[repr(transparent)]
struct KeyWrapper<Q: ?Sized>(Q);

impl<Q: ?Sized> KeyWrapper<Q> {
    fn from_ref(key: &Q) -> &Self {
        // 类型一致，直接内存直接做转化
        unsafe { &*(key as *const Q as *const KeyWrapper<Q>) }
    }
}

impl<Q: ?Sized + Hash> Hash for KeyWrapper<Q> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0).hash(state);
    }
}

impl<Q: ?Sized + PartialEq> PartialEq for KeyWrapper<Q> {
    fn eq(&self, other: &Self) -> bool {
        (self.0).eq(&other.0)
    }
}

impl<Q: ?Sized + Eq> Eq for KeyWrapper<Q> {}

impl<K, Q> Borrow<KeyWrapper<Q>> for KeyRef<K>
where
    K: Borrow<Q>,
    Q: ?Sized,
{
    fn borrow(&self) -> &KeyWrapper<Q> {
        let key = unsafe { &*self.k }.borrow();
        KeyWrapper::from_ref(key)
    }
}
