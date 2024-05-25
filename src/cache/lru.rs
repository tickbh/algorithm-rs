// Copyright 2022 - 2024 Wenmeng See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Author: tickbh
// -----
// Created Date: 2024/05/24 03:04:11

use std::{
    borrow::Borrow,
    collections::{
        hash_map::RandomState,
        HashMap,
    },
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
};

use super::{KeyRef, KeyWrapper};


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


/// 一个 LRU 缓存普通级的实现, 接口参照Hashmap保持一致
/// 设置容量之后将最大保持该容量大小的数据
/// 后进的数据将会淘汰最久没有被访问的数据
/// 
/// # Examples
/// 
/// ```
/// use algorithm::LruCache;
/// fn main() {
///     let mut lru = LruCache::new(3);
///     lru.insert("now", "ok");
///     lru.insert("hello", "algorithm");
///     lru.insert("this", "lru");
///     lru.insert("auth", "tickbh");
///     assert!(lru.len() == 3);
///     assert_eq!(lru.get("hello"), Some(&"algorithm"));
///     assert_eq!(lru.get("this"), Some(&"lru"));
///     assert_eq!(lru.get("now"), None);
/// }
/// ```
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
    /// 提供hash函数
    pub fn with_hasher(cap: usize, hash_builder: S) -> LruCache<K, V, S> {
        let cap = cap.max(1);
        let map = HashMap::with_capacity_and_hasher(cap, hash_builder);
        let head = Box::into_raw(Box::new(LruEntry::new_empty()));
        let tail = Box::into_raw(Box::new(LruEntry::new_empty()));
        unsafe {
            (*head).next = tail;
            (*tail).prev = head;
        }
        Self {
            map,
            cap,
            head,
            tail,
        }
    }

    /// 获取当前容量
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// 清理当前数据
    /// # Examples
    /// 
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("now", "ok");
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.len() == 3);
    ///     lru.clear();
    ///     assert!(lru.len() == 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.map.drain().for_each(|(_, entry)| {
            let _node = unsafe { *Box::from_raw(entry.as_ptr()) };
        });
        unsafe {
            (*self.head).next = self.tail;
            (*self.tail).prev = self.head;
        }
    }

    /// 获取当前长度
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 从队列中节点剥离
    fn detach(&mut self, entry: *mut LruEntry<K, V>) {
        unsafe {
            (*(*entry).prev).next = (*entry).next;
            (*(*entry).next).prev = (*entry).prev;
        }
    }

    /// 加到队列中
    fn attach(&mut self, entry: *mut LruEntry<K, V>) {
        unsafe {
            (*entry).next = (*self.head).next;
            (*(*entry).next).prev = entry;
            (*entry).prev = self.head;
            (*self.head).next = entry;
        }
    }

    /// 扩展当前容量
    pub fn reserve(&mut self, additional: usize) {
        self.cap += additional;
    }

    /// 遍历当前的所有值
    /// 
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     for (k, v) in lru.iter() {
    ///         assert!(k == &"hello" || k == &"this");
    ///         assert!(v == &"algorithm" || v == &"lru");
    ///     }
    ///     assert!(lru.len() == 2);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter { len: self.map.len(), ptr: self.head, end: self.tail, phantom: PhantomData }
    }
    
    /// 遍历当前的key值
    /// 
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     let mut keys = lru.keys();
    ///     assert!(keys.next()==Some(&"this"));
    ///     assert!(keys.next()==Some(&"hello"));
    ///     assert!(keys.next() == None);
    /// }
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.iter()
        }
    }
    
    /// 遍历当前的valus值
    /// 
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     let mut values = lru.values();
    ///     assert!(values.next()==Some(&"lru"));
    ///     assert!(values.next()==Some(&"algorithm"));
    ///     assert!(values.next() == None);
    /// }
    /// ```
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

    /// 排出当前数据
    /// 
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     {
    ///         let mut drain = lru.drain();
    ///         assert!(drain.next()==Some(("hello", "algorithm")));
    ///     }
    ///     assert!(lru.len() == 1);
    /// }
    /// ```
    pub fn drain(&mut self) -> Drain<'_, K, V, S> {
        Drain { base: self }
    }


    /// 弹出栈顶上的数据, 最近使用的数据
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.pop()==Some(("this", "lru")));
    ///     assert!(lru.len() == 1);
    /// }
    /// ```
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

    /// 弹出栈尾上的数据, 最久未使用的数据
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.pop_last()==Some(("hello", "algorithm")));
    ///     assert!(lru.len() == 1);
    /// }
    /// ```
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

    /// 获取key值相对应的value值, 根本hash判定
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.get(&"this") == Some(&"lru"));
    /// }
    /// ```
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

    /// 获取key值相对应的key和value值, 根本hash判定
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.get_key_value(&"this") == Some((&"this", &"lru")));
    /// }
    /// ```
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

    /// 获取key值相对应的value值, 根本hash判定, 可编辑被改变
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm".to_string());
    ///     lru.insert("this", "lru".to_string());
    ///     lru.get_mut(&"this").unwrap().insert_str(3, " good");
    ///     assert!(lru.get_key_value(&"this") == Some((&"this", &"lru good".to_string())));
    /// }
    /// ```
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

    /// 插入值, 如果值重复将返回原来的数据
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.insert("this", "lru good") == Some(&"lru"));
    /// }
    /// ```
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

    /// 移除元素
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.remove("this") == Some(("this", "lru")));
    ///     assert!(lru.len() == 1);
    /// }
    /// ```
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

    /// 根据保留当前的元素, 返回false则表示抛弃元素
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     lru.insert("year", "2024");
    ///     lru.retain(|_, v| *v == "2024" || *v == "lru");
    ///     assert!(lru.len() == 2);
    ///     assert!(lru.get("this") == Some(&"lru"));
    /// }
    /// ```
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