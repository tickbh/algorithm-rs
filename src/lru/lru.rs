use std::{borrow::Borrow, collections::{hash_map::{DefaultHasher, RandomState}, HashMap}, env::consts, hash::{BuildHasher, Hash}, mem, ptr::NonNull};

use super::{KeyRef, LruEntry};

pub struct LruCache<K: Hash, V, S = RandomState> {
    pub map: HashMap<KeyRef<K>, NonNull<LruEntry<K, V>> , S>,
    pub cap: usize,

    head: *mut LruEntry<K, V>,
    tail: *mut LruEntry<K, V>,
}

impl<K: Hash, V> LruCache<K, V> {

    pub fn new(cap: usize) -> Self {
        let map = HashMap::new();
        let head = Box::into_raw(Box::new(LruEntry::new_empty())); 
        let tail = Box::into_raw(Box::new(LruEntry::new_empty()));
        unsafe {
            (*head).next = tail;
            (*tail).prev = head;
        }
        Self { map, cap: cap.max(1), head, tail }
    }

}


impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {

    // pub fn get_mut<'a, Q>(&'a mut self, k: &Q) -> Option<&'a mut V>
    // where
    //     K: Borrow<Q>,
    //     Q: Hash + Eq + ?Sized,
    // {
    //     if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
    //         let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

    //         self.detach(node_ptr);
    //         self.attach(node_ptr);

    //         Some(unsafe { &mut *(*node_ptr).val.as_mut_ptr() })
    //     } else {
    //         None
    //     }
    // }

    fn detach(&mut self, entry: *mut LruEntry<K, V>) {
        unsafe {
            (*(*entry).prev).next = (*entry).next;
            (*(*entry).next).prev = (*entry).prev;
        }
    }

    
    fn attach(&mut self, entry: *mut LruEntry<K, V>) {
        unsafe {
            (*entry).next = (*self.head).next;
            (*(*entry).next).prev = entry;
            (*entry).prev = self.head;
            (*self.head).next = entry;
        }
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        match self.map.get(&KeyRef { k }) {
            Some(l) => {
                unsafe {
                    Some(&*(*l.as_ptr()).val.as_ptr())
                } 
            },
            None => None,
        }
    }
    
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        match self.map.get(&KeyRef { k }) {
            Some(l) => {
                unsafe {
                    Some(&mut *(*l.as_ptr()).val.as_mut_ptr())
                } 
            },
            None => None,
        }
    }

    pub fn put(&mut self, k: K, v: V) -> Option<V> {
        self.capture_put(k, v).map(|(_, v)| v)
    }
    
    pub fn capture_put(&mut self, k: K, mut v: V) -> Option<(K, V)> {
        let key = KeyRef::new(&k);
        match self.map.get_mut(&key) {
            Some(entry) => {
                
                let entry_ptr = entry.as_ptr();
                unsafe {
                    mem::swap(&mut *(*entry_ptr).val.as_mut_ptr(), &mut v);
                }
                self.detach(entry_ptr);
                self.attach(entry_ptr);
                
                Some((k, v))
            }
            None => {
                let (_, entry) = self.replace_or_create_node(k, v);
                let entry_ptr = entry.as_ptr();
                self.attach(entry_ptr);
                unsafe {
                    self.map.insert(KeyRef::new((*entry_ptr).key.as_ptr()) , entry);
                }
                None
            }
        }
    }
    
    fn replace_or_create_node(&mut self, k: K, v: V) -> (Option<(K, V)>, NonNull<LruEntry<K, V>>) {
        if self.len() == self.cap {
            let old_key = KeyRef {
                k: unsafe { &(*(*(*self.tail).prev).key.as_ptr()) },
            };
            let mut old_node = self.map.remove(&old_key).unwrap();
            let node_ptr: *mut LruEntry<K, V> = old_node.as_ptr();

            let replaced = unsafe {
                (
                    mem::replace(&mut (*node_ptr).key, mem::MaybeUninit::new(k)).assume_init(),
                    mem::replace(&mut (*node_ptr).val, mem::MaybeUninit::new(v)).assume_init(),
                )
            };

            self.detach(node_ptr);

            (Some(replaced), old_node)
        } else {
            (None, unsafe {
                NonNull::new_unchecked(Box::into_raw(Box::new(LruEntry::new(k, v))))
            })
        }
    }

    pub fn len(&mut self) -> usize {
        self.map.len()
    }
}