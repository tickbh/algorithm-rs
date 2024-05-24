use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
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

    // pub fn from_ref<Q>(k: &Q) -> Self
    // where K: Borrow<Q> {
    //     let mk = k.borrow();
    //     Self { k }
    // }
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
        unsafe {
            (self.0).hash(state);
        }
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

impl<K, V> Drop for LruEntry<K, V> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.key.as_mut_ptr());
            ptr::drop_in_place(self.val.as_mut_ptr());
            if !self.prev.is_null() {
                drop(Box::from_raw(self.prev));
            }
            if !self.next.is_null() {
                drop(Box::from_raw(self.next));
            }
        }
    }
}
