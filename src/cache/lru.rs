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
    borrow::Borrow, fmt::{self, Debug}, hash::{BuildHasher, Hash}, marker::PhantomData, mem, ops::{Index, IndexMut}, ptr::{self, NonNull}
};

use crate::{HashMap, DefaultHasher};
use super::{KeyRef, KeyWrapper};

#[cfg(feature = "ttl")]
use crate::get_milltimestamp;
#[cfg(feature = "ttl")]
const DEFAULT_CHECK_STEP: u64 = 120;
/// Lru节点数据
pub(crate) struct LruEntry<K, V> {
    /// 头部节点及尾部结点均未初始化值
    pub key: mem::MaybeUninit<K>,
    /// 头部节点及尾部结点均未初始化值
    pub val: mem::MaybeUninit<V>,
    pub prev: *mut LruEntry<K, V>,
    pub next: *mut LruEntry<K, V>,
    /// 带ttl的过期时间，单位秒
    /// 如果为u64::MAX，则表示不过期
    #[cfg(feature = "ttl")]
    pub expire: u64,
}

impl<K, V> LruEntry<K, V> {
    pub fn new_empty() -> Self {
        LruEntry {
            key: mem::MaybeUninit::uninit(),
            val: mem::MaybeUninit::uninit(),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
            #[cfg(feature = "ttl")]
            expire: u64::MAX,
        }
    }

    pub fn new(k: K, v: V) -> Self {
        LruEntry {
            key: mem::MaybeUninit::new(k),
            val: mem::MaybeUninit::new(v),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
            #[cfg(feature = "ttl")]
            expire: u64::MAX,
        }
    }

    #[cfg(feature = "ttl")]
    #[allow(dead_code)]
    pub fn new_expire(k: K, v: V, expire: u64) -> Self {
        LruEntry {
            key: mem::MaybeUninit::new(k),
            val: mem::MaybeUninit::new(v),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
            expire,
        }
    }


    #[cfg(feature = "ttl")]
    #[inline(always)]
    pub fn is_expire(&self) -> bool {
        get_milltimestamp() >= self.expire
    }


    #[cfg(feature = "ttl")]
    #[inline(always)]
    pub fn is_little(&self, time: &u64) -> bool {
        time >= &self.expire
    }
    
    #[cfg(feature = "ttl")]
    #[inline(always)]
    pub fn get_ttl(&self) -> u64 {
        if self.expire == u64::MAX {
            self.expire
        } else {
            self.expire.saturating_sub(get_milltimestamp()) / 1000
        }
    }
}


/// LRU 全称是Least Recently Used，即最近最久未使用的意思
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
    /// 存储数据结构
    map: HashMap<KeyRef<K>, NonNull<LruEntry<K, V>>, S>,
    /// 缓存的总容量
    cap: usize,
    /// 双向列表的头
    head: *mut LruEntry<K, V>,
    /// 双向列表的尾
    tail: *mut LruEntry<K, V>,
    /// 下一次检查的时间点，如果大于该时间点则全部检查是否过期
    #[cfg(feature = "ttl")]
    check_next: u64,
    /// 每次大检查点的时间间隔，如果不想启用该特性，可以将该值设成u64::MAX
    #[cfg(feature = "ttl")]
    check_step: u64,
    /// 所有节点中是否存在带ttl的结点，如果均为普通的元素，则过期的将不进行检查
    #[cfg(feature = "ttl")]
    has_ttl: bool,
}

impl<K: Hash + Eq, V> Default for LruCache<K, V, DefaultHasher> {
    fn default() -> Self {
        LruCache::new(100 )
    }
}

impl<K: Hash + Eq, V> LruCache<K, V, DefaultHasher> {
    pub fn new(cap: usize) -> Self {
        LruCache::with_hasher(cap, DefaultHasher::default())
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
            #[cfg(feature = "ttl")]
            check_step: DEFAULT_CHECK_STEP,
            #[cfg(feature = "ttl")]
            check_next: get_milltimestamp()+DEFAULT_CHECK_STEP * 1000,
            #[cfg(feature = "ttl")]
            has_ttl: false,
        }
    }

    /// 获取当前检查lru的间隔
    #[cfg(feature="ttl")]
    pub fn get_check_step(&self) -> u64 {
        self.check_step
    }

    /// 设置当前检查lru的间隔
    /// 单位为秒，意思就是每隔多少秒会清理一次数据
    /// 如果数据太大的话遍历一次可能会比较久的时长
    /// 一次清理时间复杂度O(n)
    /// 仅仅在插入时触发检查，获取时仅检查当前元素
    #[cfg(feature="ttl")]
    pub fn set_check_step(&mut self, check_step: u64) {
        self.check_step = check_step;
        self.check_next = get_milltimestamp() + self.check_step * 1000;
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

    pub fn is_full(&self) -> bool {
        self.map.len() == self.cap
    }

    pub fn is_empty(&self) -> bool {
        self.map.len() == 0
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
    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        self.cap += additional;
        self
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


    /// 遍历当前的所有值, 可变
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm".to_string());
    ///     lru.insert("this", "lru".to_string());
    ///     for (k, v) in lru.iter_mut() {
    ///         v.push_str(" ok");
    ///     }
    ///     assert!(lru.len() == 2);
    ///     assert!(lru.get(&"this") == Some(&"lru ok".to_string()));
    ///     assert!(lru.get(&"hello") == Some(&"algorithm ok".to_string()));
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut { len: self.map.len(), ptr: self.head, end: self.tail, phantom: PhantomData }
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
    ///     let vec = vec![(1, 1), (2, 2), (3, 3)];
    ///     let mut map: LruCache<_, _, _> = vec.into_iter().collect();
    ///     for value in map.values_mut() {
    ///     *value = (*value) * 2
    ///     }
    ///     let values: Vec<_> = map.values().cloned().collect();
    ///     assert_eq!(values.len(), 3);
    ///     assert!(values.contains(&2));
    ///     assert!(values.contains(&4));
    ///     assert!(values.contains(&6));
    /// }
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.iter()
        }
    }


    /// 遍历当前的valus值
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm".to_string());
    ///     lru.insert("this", "lru".to_string());
    ///     {
    ///         let mut values = lru.values_mut();
    ///         values.next().unwrap().push_str(" ok");
    ///         values.next().unwrap().push_str(" ok");
    ///         assert!(values.next() == None);
    ///     }
    ///     assert_eq!(lru.get(&"this"), Some(&"lru ok".to_string()))
    /// }
    /// ```
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.iter_mut()
        }
    }

    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    pub fn full_increase(&mut self) {
        if self.cap == self.len() {
            self.cap += 1;
        }
    }
    
    pub fn full_decrease(&mut self) -> Option<(K, V)> {
        if self.cap == self.len() {
            let ret = self.pop_unusual();
            self.cap = self.cap.saturating_sub(1);
            ret
        } else {
            None
        }
    }
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
    ///     assert!(lru.len() == 0);
    /// }
    /// ```
    pub fn drain(&mut self) -> Drain<'_, K, V, S> {
        Drain { base: self }
    }


    /// 弹出栈顶上的数据, 最常使用的数据
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.pop_usual()==Some(("this", "lru")));
    ///     assert!(lru.len() == 1);
    /// }
    /// ```
    pub fn pop_usual(&mut self) -> Option<(K, V)> {
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
    ///     assert!(lru.pop_unusual()==Some(("hello", "algorithm")));
    ///     assert!(lru.len() == 1);
    /// }
    /// ```
    pub fn pop_unusual(&mut self) -> Option<(K, V)> {
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

    
    /// 取出栈顶上的数据, 最近使用的数据
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.peek_usual()==Some((&"this", &"lru")));
    ///     assert!(lru.len() == 2);
    /// }
    /// ```
    pub fn peek_usual(&mut self) -> Option<(&K, &V)> {
        if self.len() == 0 {
            return None;
        }
        unsafe {
            let node = (*self.head).next;
            Some((&*(*node).key.as_ptr(), &*(*node).val.as_ptr()))
        }
    }

    /// 取出栈尾上的数据, 最久未使用的数据
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.peek_unusual()==Some((&"hello", &"algorithm")));
    ///     assert!(lru.len() == 2);
    /// }
    /// ```
    pub fn peek_unusual(&mut self) -> Option<(&K, &V)> {
        if self.len() == 0 {
            return None;
        }
        unsafe {
            let node = (*self.tail).prev;
            Some((&*(*node).key.as_ptr(), &*(*node).val.as_ptr()))
        }
    }

    pub fn contains_key<Q>(&mut self, k: &Q) -> bool
        where
            K: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(KeyWrapper::from_ref(k))
    }


    /// 获取key值相对应的value值, 根据hash判定
    ///
    /// ```
    /// use algorithm::LruCache;
    /// fn main() {
    ///     let mut lru = LruCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.raw_get(&"this") == Some(&"lru"));
    /// }
    /// ```
    pub fn raw_get<Q>(&self, k: &Q) -> Option<&V>
        where
            K: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        match self.map.get(KeyWrapper::from_ref(k)) {
            Some(l) => {
                let node = l.as_ptr();
                unsafe { Some(&*(*node).val.as_ptr()) }
            }
            None => None,
        }
    }

    /// 获取key值相对应的value值, 根据hash判定
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
        self.get_key_value(k).map(|(_, v)| v)
    }

    /// 获取key值相对应的key和value值, 根据hash判定
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
        self.get_mut_key_value(k).map(|(k, v)| (k, &*v))
    }

    /// 获取key值相对应的value值, 根据hash判定, 可编辑被改变
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
        self.get_mut_key_value(k).map(|(_, v)| v)
    }

    
    pub fn get_mut_key_value<Q>(&mut self, k: &Q) -> Option<(&K, &mut V)>
        where
            K: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        match self.get_node(k) {
            Some(node) => {
                unsafe { Some(( &*(*node).key.as_mut_ptr(), &mut *(*node).val.as_mut_ptr())) }
            }
            None => None,
        }
    }

    pub(crate) fn get_node<Q>(&mut self, k: &Q) -> Option<*mut LruEntry<K, V>>
        where
            K: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        match self.map.get(KeyWrapper::from_ref(k)) {
            Some(l) => {
                let node = l.as_ptr();
                self.detach(node);
                #[cfg(feature = "ttl")]
                unsafe {
                    if self.has_ttl && (*node).is_expire() {
                        self.map.remove(KeyWrapper::from_ref(k));
                        let _ = *Box::from_raw(node);
                        return None;
                    }
                }
                
                self.attach(node);
                Some(node)
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
    #[inline(always)]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.capture_insert(k, v).map(|(_, v, _)| v)
    }

    /// 插入带有生存时间的元素
    /// 每次获取像redis一样，并不会更新生存时间
    /// 如果需要更新则需要手动的进行重新设置
    #[cfg(feature="ttl")]
    #[inline(always)]
    pub fn insert_with_ttl(&mut self, k: K, v: V, ttl: u64) -> Option<V> {
        self.capture_insert_with_ttl(k, v, ttl).map(|(_, v, _)| v)
    }

    #[inline(always)]
    pub fn capture_insert(&mut self, k: K, v: V) -> Option<(K, V, bool)> {
        self._capture_insert_with_ttl(k, v, u64::MAX)
    }

    #[cfg(feature = "ttl")]
    #[inline(always)]
    pub fn capture_insert_with_ttl(&mut self, k: K, v: V, ttl: u64) -> Option<(K, V, bool)> {
        if ttl == 0 { return None };
        self.has_ttl = true;
        self._capture_insert_with_ttl(k, v, ttl)
    }

    #[allow(unused_variables)]
    fn _capture_insert_with_ttl(&mut self, k: K, mut v: V, ttl: u64) -> Option<(K, V, bool)> {
        #[cfg(feature="ttl")]
        self.clear_expire();

        let key = KeyRef::new(&k);
        match self.map.get_mut(&key) {
            Some(entry) => {
                let entry_ptr = entry.as_ptr();
                unsafe {
                    mem::swap(&mut *(*entry_ptr).val.as_mut_ptr(), &mut v);
                }
                #[cfg(feature="ttl")]
                unsafe {
                    (*entry_ptr).expire = ttl.saturating_mul(1000).saturating_add(get_milltimestamp());
                }
                self.detach(entry_ptr);
                self.attach(entry_ptr);

                Some((k, v, true))
            }
            None => {
                let (val, entry) = self.replace_or_create_node(k, v);
                let entry_ptr = entry.as_ptr();
                self.attach(entry_ptr);
                #[cfg(feature="ttl")]
                unsafe {
                    (*entry_ptr).expire = ttl.saturating_mul(1000).saturating_add(get_milltimestamp());
                }
                unsafe {
                    self.map
                        .insert(KeyRef::new((*entry_ptr).key.as_ptr()), entry);
                }
                val.map(|(k, v)| (k, v, false))
            }
        }
    }

    pub fn get_or_insert<F>(&mut self, k: K, f: F) -> &V
    where
        F: FnOnce() -> V, {
        &*self.get_or_insert_mut(k, f)
    }

    pub fn get_or_insert_mut<'a, F>(&'a mut self, k: K, f: F) -> &mut V
    where
        F: FnOnce() -> V, {
        if let Some(v) = self.get_node(&k) {
            return unsafe {
                &mut *(*v).val.as_mut_ptr()
            };
        } else {
            let v = f();

            let (_, node) = self.replace_or_create_node(k, v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ptr() };
            self.map.insert(KeyRef { k: keyref }, node);
            unsafe { &mut *(*node_ptr).val.as_mut_ptr() }
        }
    }

    #[cfg(feature="ttl")]
    pub fn clear_expire(&mut self) {
        if !self.has_ttl {
            return;
        }
        let now = get_milltimestamp();
        if now < self.check_next {
            return;
        }
        self.check_next = now + self.check_step;
        unsafe {
            let mut ptr = self.tail;
            while ptr != self.head {
                if (*ptr).is_little(&now) {
                    let next = (*ptr).prev;
                    self.detach(ptr);
                    self.map.remove(&KeyRef::new(&*(*ptr).key.as_ptr()));
                    let _ = *Box::from_raw(ptr);
                    ptr = next;
                } else {
                    ptr = (*ptr).prev;
                }
            }
        }
    }

    #[cfg(feature="ttl")]
    #[inline(always)]
    pub fn del_ttl<Q>(&mut self, k: &Q)
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized, {
        self.set_ttl(k, u64::MAX);
    }

    #[cfg(feature="ttl")]
    pub fn set_ttl<Q>(&mut self, k: &Q, expire: u64) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized, {
        if let Some(v) = self.get_node(&k) {
            self.has_ttl = true;
            unsafe {
                (*v).expire = get_milltimestamp().saturating_add(expire.saturating_mul(1000));
            }
            true
        } else {
            false
        }
    }

    #[cfg(feature="ttl")]
    pub fn get_ttl<Q>(&mut self, k: &Q) -> Option<u64>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized, {
        if let Some(v) = self.get_node(&k) {
            unsafe {
                Some((*v).get_ttl())
            }
        } else {
            None
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
        if let Some(node) = self.remove_node(k) {
            unsafe {
                Some((node.key.assume_init(), node.val.assume_init()))   
            }
        } else {
            None
        }
    }

    
    #[cfg(feature="ttl")]
    pub fn remove_with_ttl<Q>(&mut self, k: &Q) -> Option<(K, V, u64)>
        where
            K: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.remove_node(k) {
            unsafe {
                let ttl = node.get_ttl();
                Some((node.key.assume_init(), node.val.assume_init(), ttl))
            }
        } else {
            None
        }
    }
    
    fn remove_node<Q>(&mut self, k: &Q) -> Option<LruEntry<K, V>>
        where
            K: Borrow<Q>,
            Q: Hash + Eq + ?Sized,
    {
        match self.map.remove(KeyWrapper::from_ref(k)) {
            Some(l) => unsafe {
                self.detach(l.as_ptr());
                let node = *Box::from_raw(l.as_ptr());
                Some(node)
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
                    self.map.remove(&KeyRef { k: &*(*node).key.as_ptr() });
                    self.detach(node);
                    node = next;
                } else {
                    node = (*node).next;
                }
            }
        }
    }
}


impl<K: Hash + Eq, V: Default, S: BuildHasher> LruCache<K, V, S> {
    pub fn get_or_insert_default(&mut self, k: K) -> &V {
        &*self.get_or_insert_mut(k, || V::default())
    }

    pub fn get_or_insert_default_mut(&mut self, k: K) -> &mut V {
        self.get_or_insert_mut(k, || V::default())
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


/// Convert LruCache to iter, move out the tree.
pub struct IntoIter<K: Hash + Eq, V, S: BuildHasher> {
    base: LruCache<K, V, S>,
}

// Drop all owned pointers if the collection is dropped
impl<K: Hash + Eq, V, S: BuildHasher> Drop for IntoIter<K, V, S> {
    #[inline]
    fn drop(&mut self) {
        for (_, _) in self {}
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> Iterator for IntoIter<K, V, S> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        self.base.pop_usual()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.base.len(), Some(self.base.len()))
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> IntoIterator for LruCache<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, S>;

    #[inline]
    fn into_iter(self) -> IntoIter<K, V, S> {
        IntoIter {
            base: self
        }
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
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


impl<K: Hash + Eq, V, S: BuildHasher> DoubleEndedIterator for IntoIter<K, V, S> {
    #[inline]
    fn next_back(&mut self) -> Option<(K, V)> {
        self.base.pop_unusual()
    }
}

pub struct IterMut<'a, K: 'a, V: 'a> {
    len: usize,
    ptr: *mut LruEntry<K, V>,
    end: *mut LruEntry<K, V>,
    phantom: PhantomData<&'a usize>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        unsafe {
            self.ptr = (*self.ptr).next;
            let node = self.ptr;
            self.len -= 1;
            Some((&*(*node).key.as_ptr(), &mut *(*node).val.as_mut_ptr()))
        }
    }

    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        unsafe {
            self.end = (*self.end).prev;
            let node = self.end;
            self.len -= 1;
            Some((&*(*node).key.as_ptr(), &mut *(*node).val.as_mut_ptr()))
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
        self.base.pop_unusual()
    }
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> Drop for Drain<'a, K, V, S> {
    fn drop(&mut self) {
        self.base.clear();
    }
}

pub struct Keys<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }

    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.len, Some(self.iter.len))
    }
}

pub struct Values<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.len, Some(self.iter.len))
    }
}


pub struct ValuesMut<'a, K, V> {
    iter: IterMut<'a, K, V>,
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.len, Some(self.iter.len))
    }
}


impl<K: Hash + Eq, V> FromIterator<(K, V)> for LruCache<K, V, DefaultHasher> {
    fn from_iter<T: IntoIterator<Item=(K, V)>>(iter: T) -> LruCache<K, V, DefaultHasher> {
        let mut lru = LruCache::new(2);
        lru.extend(iter);
        lru
    }
}

impl<K: Hash + Eq, V> Extend<(K, V)> for LruCache<K, V, DefaultHasher> {
    fn extend<T: IntoIterator<Item=(K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        for (k, v) in iter {
            self.reserve(1);
            self.insert(k, v);
        }
    }
}

impl<K, V, S> PartialEq for LruCache<K, V, S>
    where
        K: Eq + Hash,
        V: PartialEq,
        S: BuildHasher
{
    fn eq(&self, other: &LruCache<K, V, S>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.raw_get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, S> Eq for LruCache<K, V, S>
    where
        K: Eq + Hash,
        V: PartialEq,
        S: BuildHasher
{}

impl<K, V, S> Debug for LruCache<K, V, S>
where
    K: Ord + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}


impl<'a, K, V, S> Index<&'a K> for LruCache<K, V, S>
where
    K: Hash+Eq,
    S: BuildHasher
{
    type Output = V;

    #[inline]
    fn index(&self, index: &K) -> &V {
        self.raw_get(index).expect("no entry found for key")
    }
}


impl<'a, K, V, S> IndexMut<&'a K> for LruCache<K, V, S>
where
    K: Hash+Eq,
    S: BuildHasher
{
    #[inline]
    fn index_mut(&mut self, index: &K) -> &mut V {
        self.get_mut(index).expect("no entry found for key")
    }
}

unsafe impl<K: Send, V: Send, S: Send> Send for LruCache<K, V, S> {}
unsafe impl<K: Sync, V: Sync, S: Sync> Sync for LruCache<K, V, S> {}


#[cfg(test)]
mod tests {
    use crate::DefaultHasher;

    use super::LruCache;

    #[test]
    fn test_insert() {
        let mut m = LruCache::new(2);
        assert_eq!(m.len(), 0);
        m.insert(1, 2);
        assert_eq!(m.len(), 1);
        m.insert(2, 4);
        assert_eq!(m.len(), 2);
        m.insert(3, 6);
        assert_eq!(m.len(), 2);
        assert_eq!(m.get(&1), None);
        assert_eq!(*m.get(&2).unwrap(), 4);
        assert_eq!(*m.get(&3).unwrap(), 6);
    }

    #[test]
    fn test_replace() {
        let mut m = LruCache::new(2);
        assert_eq!(m.len(), 0);
        m.insert(2, 4);
        assert_eq!(m.len(), 1);
        m.insert(2, 6);
        assert_eq!(m.len(), 1);
        assert_eq!(*m.get(&2).unwrap(), 6);
    }

    #[test]
    fn test_clone() {
        let mut m = LruCache::new(2);
        assert_eq!(m.len(), 0);
        m.insert(1, 2);
        assert_eq!(m.len(), 1);
        m.insert(2, 4);
        assert_eq!(m.len(), 2);
        let mut m2 = m.clone();
        m.clear();
        assert_eq!(*m2.get(&1).unwrap(), 2);
        assert_eq!(*m2.get(&2).unwrap(), 4);
        assert_eq!(m2.len(), 2);
    }

    #[test]
    fn test_empty_remove() {
        let mut m: LruCache<isize, bool, DefaultHasher> = LruCache::new(2);
        assert_eq!(m.remove(&0), None);
    }

    #[test]
    fn test_empty_iter() {
        let mut m: LruCache<isize, bool, DefaultHasher> = LruCache::new(2);
        assert_eq!(m.iter().next(), None);
        assert_eq!(m.iter_mut().next(), None);
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());
        assert_eq!(m.into_iter().next(), None);
    }

    #[test]
    fn test_lots_of_insertions() {
        let mut m = LruCache::new(1000);

        // Try this a few times to make sure we never screw up the hashmap's
        // internal state.
        for _ in 0..10 {
            assert!(m.is_empty());

            for i in 1..101 {
                m.insert(i, i);

                for j in 1..i + 1 {
                    let r = m.get(&j);
                    assert_eq!(r, Some(&j));
                }

                for j in i + 1..101 {
                    let r = m.get(&j);
                    assert_eq!(r, None);
                }
            }

            for i in 101..201 {
                assert!(!m.contains_key(&i));
            }

            // remove forwards
            for i in 1..101 {
                assert!(m.remove(&i).is_some());

                for j in 1..i + 1 {
                    assert!(!m.contains_key(&j));
                }

                for j in i + 1..101 {
                    assert!(m.contains_key(&j));
                }
            }

            for i in 1..101 {
                assert!(!m.contains_key(&i));
            }

            for i in 1..101 {
                m.insert(i, i);
            }

            // remove backwards
            for i in (1..101).rev() {
                assert!(m.remove(&i).is_some());

                for j in i..101 {
                    assert!(!m.contains_key(&j));
                }

                for j in 1..i {
                    assert!(m.contains_key(&j));
                }
            }
        }
    }

    #[test]
    fn test_find_mut() {
        let mut m = LruCache::new(3);
        m.insert(1, 12);
        m.insert(2, 8);
        m.insert(5, 14);
        let new = 100;
        match m.get_mut(&5) {
            None => panic!(),
            Some(x) => *x = new,
        }
        assert_eq!(m.get(&5), Some(&new));
    }

    #[test]
    fn test_remove() {
        let mut m = LruCache::new(3);
        m.insert(1, 2);
        assert_eq!(*m.get(&1).unwrap(), 2);
        m.insert(5, 3);
        assert_eq!(*m.get(&5).unwrap(), 3);
        m.insert(9, 4);
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&5).unwrap(), 3);
        assert_eq!(*m.get(&9).unwrap(), 4);
        assert_eq!(m.remove(&1).unwrap(), (1, 2));
        assert_eq!(m.remove(&5).unwrap(), (5, 3));
        assert_eq!(m.remove(&9).unwrap(), (9, 4));
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let mut m = LruCache::new(2);
        m.insert(1, 2);
        assert!(!m.is_empty());
        assert!(m.remove(&1).is_some());
        assert!(m.is_empty());
    }

    #[test]
    fn test_pop() {
        let mut m = LruCache::new(3);
        m.insert(3, 6);
        m.insert(2, 4);
        m.insert(1, 2);
        assert_eq!(m.len(), 3);
        assert_eq!(m.pop_usual(), Some((1, 2)));
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_unusual(), Some((3, 6)));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_iterate() {
        let mut m = LruCache::new(32);
        for i in 0..32 {
            m.insert(i, i * 2);
        }
        assert_eq!(m.len(), 32);

        let mut observed: u32 = 0;

        for (k, v) in m.iter() {
            assert_eq!(*v, *k * 2);
            observed |= 1 << *k;
        }
        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_keys() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: LruCache<_, _, _> = vec.into_iter().collect();
        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }

    #[test]
    fn test_values() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: LruCache<_, _, _> = vec.into_iter().collect();
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&'a'));
        assert!(values.contains(&'b'));
        assert!(values.contains(&'c'));
    }

    #[test]
    fn test_values_mut() {
        let vec = vec![(1, 1), (2, 2), (3, 3)];
        let mut map: LruCache<_, _, _> = vec.into_iter().collect();
        for value in map.values_mut() {
            *value = (*value) * 2
        }
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&2));
        assert!(values.contains(&4));
        assert!(values.contains(&6));
    }

    #[test]
    fn test_find() {
        let mut m = LruCache::new(2);
        assert!(m.get(&1).is_none());
        m.insert(1, 2);
        match m.get(&1) {
            None => panic!(),
            Some(v) => assert_eq!(*v, 2),
        }
    }

    #[test]
    fn test_eq() {
        let mut m1 = LruCache::new(3);
        m1.insert(1, 2);
        m1.insert(2, 3);
        m1.insert(3, 4);

        let mut m2 = LruCache::new(3);
        m2.insert(1, 2);
        m2.insert(2, 3);

        assert!(m1 != m2);

        m2.insert(3, 4);

        assert_eq!(m1, m2);
    }

    #[test]
    fn test_from_iter() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: LruCache<_, _, _> = xs.iter().cloned().collect();

        for &(k, v) in &xs {
            assert_eq!(map.raw_get(&k), Some(&v));
        }
    }

    #[test]
    fn test_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: LruCache<_, _, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_iter_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: LruCache<_, _, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn test_mut_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: LruCache<_, _, _> = xs.iter().cloned().collect();

        let mut iter = map.iter_mut();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_iter_mut_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: LruCache<_, _, _> = xs.iter().cloned().collect();

        let mut iter = map.iter_mut();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn test_index() {
        let mut map = LruCache::new(2);

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        assert_eq!(map[&2], 1);
    }

    #[test]
    #[should_panic]
    fn test_index_nonexistent() {
        let mut map = LruCache::new(2);

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        map[&4];
    }

    #[test]
    fn test_extend_iter() {
        let mut a = LruCache::new(2);
        a.insert(1, "one");
        let mut b = LruCache::new(2);
        b.insert(2, "two");
        b.insert(3, "three");

        a.extend(b.into_iter());

        assert_eq!(a.len(), 3);
        assert_eq!(a[&1], "one");
        assert_eq!(a[&2], "two");
        assert_eq!(a[&3], "three");
    }

    #[test]
    fn test_drain() {
        let mut a = LruCache::new(3);
        a.insert(1, 1);
        a.insert(2, 2);
        a.insert(3, 3);

        assert_eq!(a.len(), 3);
        {
            let mut drain = a.drain();
            assert_eq!(drain.next().unwrap(), (1, 1));
            assert_eq!(drain.next().unwrap(), (2, 2));
        }
        assert_eq!(a.len(), 0);
    }


    #[test]
    fn test_send() {
        use std::thread;

        let mut cache = LruCache::new(4);
        cache.insert(1, "a");

        let handle = thread::spawn(move || {
            assert_eq!(cache.get(&1), Some(&"a"));
        });

        assert!(handle.join().is_ok());
    }

    #[test]
    #[cfg(feature="ttl")]
    fn test_ttl_cache() {
        let mut lru = LruCache::new(3);
        lru.insert_with_ttl("help", "ok", 1);
        lru.insert_with_ttl("author", "tickbh", 2);
        assert_eq!(lru.len(), 2);
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(lru.get("help"), None);
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(lru.get("author"), None);
        assert_eq!(lru.len(), 0);
    }

    #[test]
    #[cfg(feature="ttl")]
    fn test_ttl_check_cache() {
        let mut lru = LruCache::new(3);
        lru.set_check_step(1);
        lru.insert_with_ttl("help", "ok", 1);
        lru.insert("now", "algorithm");
        assert_eq!(lru.len(), 2);
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(lru.len(), 2);
        lru.insert_with_ttl("author", "tickbh", 3);
        assert_eq!(lru.len(), 2);
        assert_eq!(lru.get("help"), None);
        assert_eq!(lru.len(), 2);
    }

    #[test]
    #[cfg(feature="ttl")]
    fn test_ttl_del() {
        let mut lru = LruCache::new(3);
        lru.insert_with_ttl("help", "ok", 1);
        lru.insert_with_ttl("author", "tickbh", 2);
        assert_eq!(lru.len(), 2);
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(lru.get("help"), None);
        lru.del_ttl(&"author");
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(lru.get("author"), Some(&"tickbh"));
        assert_eq!(lru.len(), 1);
    }

    #[test]
    #[cfg(feature="ttl")]
    fn test_ttl_set() {
        let mut lru = LruCache::new(3);
        lru.insert_with_ttl("help", "ok", 1);
        lru.insert_with_ttl("author", "tickbh", 2);
        lru.set_ttl(&"help", 3);
        assert_eq!(lru.len(), 2);
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(lru.get("help"), Some(&"ok"));
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(lru.get("author"), None);
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(lru.get("help"), None);
        assert_eq!(lru.len(), 0);
    }


    #[test]
    #[cfg(feature="ttl")]
    fn test_ttl_get() {
        let mut lru = LruCache::new(3);
        lru.insert_with_ttl("help", "ok", 1);
        lru.insert_with_ttl("author", "tickbh", 2);
        lru.insert("now", "algorithm");
        assert!(lru.get_ttl(&"help").unwrap() <= 1);
        assert!(lru.get_ttl(&"author").unwrap() <= 2);
        assert_eq!(lru.get_ttl(&"now"), Some(u64::MAX));
    }
}