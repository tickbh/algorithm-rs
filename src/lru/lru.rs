use std::{
    borrow::Borrow,
    collections::{
        hash_map::RandomState,
        HashMap,
    },
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    mem,
    ptr::NonNull,
};

use super::{KeyRef, KeyWrapper, LruEntry};

pub struct LruCache<K, V, S> {
    map: HashMap<KeyRef<K>, NonNull<LruEntry<K, V>>, S>,
    cap: usize,
    head: *mut LruEntry<K, V>,
    tail: *mut LruEntry<K, V>,
}

impl<K: Hash + Eq, V> LruCache<K, V, RandomState> {
    pub fn new(cap: usize) -> Self {
        LruCache::with_hasher(cap, RandomState::new())
    }
}

impl<K, V, S> LruCache<K, V, S> {
    pub fn with_hasher(cap: usize, hash_builder: S) -> LruCache<K, V, S> {
        let map = HashMap::with_capacity_and_hasher(cap, hash_builder);
        let head = Box::into_raw(Box::new(LruEntry::new_empty()));
        let tail = Box::into_raw(Box::new(LruEntry::new_empty()));
        unsafe {
            (*head).next = tail;
            (*tail).prev = head;
        }
        Self {
            map,
            cap: cap.max(1),
            head,
            tail,
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn clear(&mut self) {
        self.map.drain().for_each(|(_, entry)| {
            let _node = unsafe { *Box::from_raw(entry.as_ptr()) };
        });
        unsafe {
            (*self.head).next = self.tail;
            (*self.tail).prev = self.head;
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

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

    pub fn reserve(&mut self, additional: usize) {
        self.cap += additional;
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter { len: self.map.len(), ptr: self.head, end: self.tail, phantom: PhantomData }
    }
    
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.iter()
        }
    }
    
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.iter()
        }
    }

    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    pub fn drain(&mut self) -> Drain<'_, K, V, S> {
        Drain { base: self }
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        if self.len() == 0 {
            return None;
        }
        unsafe {
            let node = (*self.head).next;
            self.detach(node);
            let key = KeyRef::new((*node).key.as_ptr());
            let value = self.map.remove(&key).expect("must ok");
            let node = *Box::from_raw(value.as_ptr());
            let LruEntry { key, val, .. } = node;
            Some((key.assume_init(), val.assume_init()))
        }
    }

    pub fn pop_last(&mut self) -> Option<(K, V)> {
        if self.len() == 0 {
            return None;
        }
        unsafe {
            let node = (*self.tail).prev;
            self.detach(node);
            let key = KeyRef::new((*node).key.as_ptr());
            let value = self.map.remove(&key).expect("must ok");
            let node = *Box::from_raw(value.as_ptr());
            let LruEntry { key, val, .. } = node;
            Some((key.assume_init(), val.assume_init()))
        }
    }

    pub fn get<Q>(&mut self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.get(KeyWrapper::from_ref(k)) {
            Some(l) => {
                let node = l.as_ptr();
                self.detach(node);
                self.attach(node);
                unsafe { Some(&*(*node).val.as_ptr()) }
            }
            None => None,
        }
    }

    pub fn get_key_value<Q>(&mut self, k: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.get(KeyWrapper::from_ref(k)) {
            Some(l) => {
                let node = l.as_ptr();
                self.detach(node);
                self.attach(node);
                unsafe { Some((&*(*node).key.as_ptr(), &*(*node).val.as_ptr())) }
            }
            None => None,
        }
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.get(KeyWrapper::from_ref(k)) {
            Some(l) => {
                let node = l.as_ptr();

                self.detach(node);
                self.attach(node);
                unsafe { Some(&mut *(*node).val.as_mut_ptr()) }
            }
            None => None,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.capture_insert(k, v).map(|(_, v)| v)
    }

    pub fn capture_insert(&mut self, k: K, mut v: V) -> Option<(K, V)> {
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
                    self.map
                        .insert(KeyRef::new((*entry_ptr).key.as_ptr()), entry);
                }
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.remove(KeyWrapper::from_ref(k)) {
            Some(l) => unsafe {
                self.detach(l.as_ptr());
                let node = *Box::from_raw(l.as_ptr());
                Some((node.key.assume_init(), node.val.assume_init()))
            },
            None => None,
        }
    }

    fn replace_or_create_node(&mut self, k: K, v: V) -> (Option<(K, V)>, NonNull<LruEntry<K, V>>) {
        if self.len() == self.cap {
            let old_key = KeyRef {
                k: unsafe { &(*(*(*self.tail).prev).key.as_ptr()) },
            };
            let old_node = self.map.remove(&old_key).unwrap();
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

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        unsafe {
            let mut node = (*self.head).next;
            while node != self.tail {
                if !f(&*(*node).key.as_ptr(), &mut *(*node).val.as_mut_ptr()) {
                    let next = (*node).next;
                    self.map.remove(&KeyRef { k: &*(*node).key.as_ptr()});
                    self.detach(node);
                    node = next;
                } else {
                    node = (*node).next;
                }
            }    
        }
    }
}

impl<K: Clone + Hash + Eq, V: Clone, S: Clone + BuildHasher> Clone for LruCache<K, V, S> {
    fn clone(&self) -> Self {
        
        let mut new_lru = LruCache::with_hasher(self.cap, self.map.hasher().clone());

        for (key, value) in self.iter().rev() {
            new_lru.insert(key.clone(), value.clone());
        }

        new_lru
    }
}

impl<K, V, S> Drop for LruCache<K, V, S> {
    fn drop(&mut self) {
        self.clear();

        let _head = unsafe { *Box::from_raw(self.head) };
        let _tail = unsafe { *Box::from_raw(self.tail) };
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    len: usize,
    ptr: *mut LruEntry<K, V>,
    end: *mut LruEntry<K, V>,
    phantom: PhantomData<&'a usize>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        unsafe {
            self.ptr = (*self.ptr).next;
            let node = self.ptr;
            self.len -= 1;
            Some((&*(*node).key.as_ptr(), &*(*node).val.as_ptr()))
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        unsafe {
            self.end = (*self.end).prev;
            let node = self.end;
            self.len -= 1;
            Some((&*(*node).key.as_ptr(), &*(*node).val.as_ptr()))
        }
    }
}

pub struct Drain<'a, K: 'a + Hash + Eq, V: 'a, S: BuildHasher> {
    pub base: &'a mut LruCache<K, V, S>,
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> ExactSizeIterator for Drain<'a, K, V, S> {
    fn len(&self) -> usize {
        self.base.map.len()
    }
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> Iterator for Drain<'a, K, V, S> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.base.len() == 0 {
            return None;
        }
        self.base.pop_last()
    }
}

pub struct Keys<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item=&'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {
}


pub struct Values<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item=&'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {
}