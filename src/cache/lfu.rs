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
    collections::{hash_map::RandomState, HashMap, HashSet},
    hash::{BuildHasher, Hash},
    mem,
    ptr::NonNull,
};

use lazy_static::lazy_static;

use super::{KeyRef, KeyWrapper};

/// 避免hash表爆炸, 次数与频次映射
/// 如0:0, 1:1, 2:2, 3:3, 4:4, 5:5, 10:10, 11-20:11, 21-50:12,51-100:13, 100-500:14, 500-1000:15
/// 1001-10000:16,10001-100000:17,100001:1000000:18等
fn get_freq_by_times(times: usize) -> u8 {
    lazy_static! {
        static ref CACHE_MAP: HashMap<usize, u8> = {
            let mut cache = HashMap::new();
            for i in 0..=10 {
                cache.insert(i, i as u8);
            }
            for i in 11..=20 {
                cache.insert(i, 11);
            }
            for i in 21..=50 {
                cache.insert(i, 12);
            }
            for i in 51..=100 {
                cache.insert(i, 13);
            }
            for i in 101..=500 {
                cache.insert(i, 14);
            }
            for i in 501..=1000 {
                cache.insert(i, 15);
            }
            cache
        };
    };
    if let Some(k) = CACHE_MAP.get(&times) {
        return *k;
    }
    if times < 10000 {
        return 16;
    } else if times < 100000 {
        return 17;
    } else if times < 1000000 {
        return 18;
    } else {
        return 19;
    }
}

struct LfuEntry<K, V> {
    pub key: mem::MaybeUninit<K>,
    pub val: mem::MaybeUninit<V>,
    pub counter: usize,
}

impl<K, V> LfuEntry<K, V> {
    pub fn new_counter(k: K, v: V, counter: usize) -> Self {
        LfuEntry {
            key: mem::MaybeUninit::new(k),
            val: mem::MaybeUninit::new(v),
            counter,
        }
    }

    pub fn key_ref(&self) -> KeyRef<K> {
        KeyRef {
            k: self.key.as_ptr(),
        }
    }
}

/// 一个 lfu(least frequently used/最不经常使用页置换算法 ) 缓存的实现, 接口参照Hashmap保持一致
/// 根据元素的访问次数进行按分组进行淘汰测试
/// 在访问次数达到设定值时将全体所有的访问次数下降1处理
/// 以使高频数据在一定时间后将过期处理
///
/// # Examples
///
/// ```
/// use algorithm::LfuCache;
/// fn main() {
///     let mut lru = LfuCache::new(3);
///     lru.insert("hello", "algorithm");
///     lru.insert("this", "lru");
///     let _ = lru.get("hello");
///     let _ = lru.get("this");
///     lru.insert("now", "ok");
///     lru.insert("auth", "tickbh");
///     assert!(lru.len() == 3);
///     assert_eq!(lru.get("hello"), Some(&"algorithm"));
///     assert_eq!(lru.get("this"), Some(&"lru"));
///     assert_eq!(lru.get("now"), None);
/// }
/// ```
pub struct LfuCache<K, V, S> {
    map: HashMap<KeyRef<K>, NonNull<LfuEntry<K, V>>, S>,
    times_map: HashMap<u8, HashSet<KeyRef<K>>>,
    cap: usize,
    max_freq: u8,
    visit_count: usize,

    default_count: usize,
    reduce_count: usize,
}

impl<K: Hash + Eq, V> LfuCache<K, V, RandomState> {
    pub fn new(cap: usize) -> Self {
        LfuCache::with_hasher(cap, RandomState::new())
    }
}

impl<K, V, S> LfuCache<K, V, S> {
    /// 提供hash函数
    pub fn with_hasher(cap: usize, hash_builder: S) -> LfuCache<K, V, S> {
        let cap = cap.max(1);
        let map = HashMap::with_capacity_and_hasher(cap, hash_builder);
        Self {
            map,
            times_map: HashMap::new(),
            visit_count: 0,
            max_freq: 0,
            reduce_count: 1000000,
            default_count: 5,
            cap,
        }
    }

    /// 设定初始进入列表中默认的访问次数，防止出现一进入就权重过低的情况
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.get_visit(&"this") == Some(5));
    ///     assert!(lru.get_visit(&"hello") == Some(5));
    /// }
    /// ```
    pub fn set_default_count(&mut self, default_count: usize) {
        self.default_count = default_count;
    }

    pub fn get_default_count(&self) -> usize {
        return self.default_count;
    }

    
    /// 每多少访问存储中触发值，
    /// 如设置100次，那么将100次发生get或者put时将触发一次调整
    /// 每次衰减将进行/2进行衰减，如原来100次衰减后将变成50次
    /// 每次调整时间复杂度为O(n)
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     lru.set_reduce_count(100);
    ///     assert!(lru.get_visit(&"hello") == Some(5));
    ///     assert!(lru.get_visit(&"this") == Some(5));
    ///     for _ in 0..98 {
    ///         let _ = lru.get("this");
    ///     }
    ///     assert!(lru.get_visit(&"this") == Some(51));
    ///     assert!(lru.get_visit(&"hello") == Some(2));
    ///     let mut keys = lru.keys();
    ///     assert!(keys.next()==Some(&"this"));
    ///     assert!(keys.next()==Some(&"hello"));
    ///     assert!(keys.next() == None);
    /// }
    /// ```
    pub fn set_reduce_count(&mut self, reduce_count: usize) {
        self.reduce_count = reduce_count;
    }

    pub fn get_reduce_count(&self) -> usize {
        return self.reduce_count;
    }

    /// 获取当前容量
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// 清理当前数据
    /// # Examples
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("now", "ok");
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.len() == 3);
    ///     lru.clear();
    ///     assert!(lru.len() == 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.times_map.clear();
        self.map.drain().for_each(|(_, entry)| {
            let _node = unsafe { *Box::from_raw(entry.as_ptr()) };
        });
        self.visit_count = 0;
    }

    /// 获取当前长度
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 扩展当前容量
    pub fn reserve(&mut self, additional: usize) {
        self.cap += additional;
    }

    /// 遍历当前的所有值
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     for (k, v) in lru.iter() {
    ///         assert!(k == &"hello" || k == &"this");
    ///         assert!(v == &"algorithm" || v == &"lru");
    ///     }
    ///     assert!(lru.len() == 2);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V, S> {
        Iter::new(self)
    }

    /// 遍历当前的key值
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     let _ = lru.get("this");
    ///     let mut keys = lru.keys();
    ///     assert!(keys.next()==Some(&"this"));
    ///     assert!(keys.next()==Some(&"hello"));
    ///     assert!(keys.next() == None);
    /// }
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V, S> {
        Keys { iter: self.iter() }
    }

    /// 遍历当前的valus值
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     let _ = lru.get("this");
    ///     let mut values = lru.values();
    ///     assert!(values.next()==Some(&"lru"));
    ///     assert!(values.next()==Some(&"algorithm"));
    ///     assert!(values.next() == None);
    /// }
    /// ```
    pub fn values(&self) -> Values<'_, K, V, S> {
        Values { iter: self.iter() }
    }

    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> LfuCache<K, V, S> {
    /// 从队列中节点剥离
    fn detach(&mut self, entry: *mut LfuEntry<K, V>) {
        unsafe {
            let freq = get_freq_by_times((*entry).counter);
            self.times_map.entry(freq).and_modify(|v| {
                v.remove(&(*entry).key_ref());
            });
        }
    }

    /// 加到队列中
    fn attach(&mut self, entry: *mut LfuEntry<K, V>) {
        unsafe {
            self.visit_count += 1;
            (*entry).counter += 1;
            let freq = get_freq_by_times((*entry).counter);
            self.max_freq = self.max_freq.max(freq);
            self.times_map
                .entry(freq)
                .or_default()
                .insert((*entry).key_ref());

            self.check_reduce();
        }
    }

    fn check_reduce(&mut self) {
        if self.visit_count >= self.reduce_count {
            let mut max = 0;
            for (k, v) in self.map.iter() {
                unsafe {
                    let node = v.as_ptr();
                    let freq = get_freq_by_times((*node).counter);
                    (*node).counter /= 2;
                    let next = get_freq_by_times((*node).counter);
                    max = max.max(next);
                    if freq != next {
                        self.times_map.entry(freq).and_modify(|v| {
                            v.remove(k);
                        });
                        self.times_map
                            .entry(next)
                            .or_default()
                            .insert((*node).key_ref());
                    }
                }
            }
            self.max_freq = max;
            self.visit_count = 0;
        }
    }

    /// 加到队列中
    fn reattach(&mut self, entry: *mut LfuEntry<K, V>) {
        unsafe {
            self.visit_count += 1;
            let freq = get_freq_by_times((*entry).counter);
            (*entry).counter += 1;
            let next_freq = get_freq_by_times((*entry).counter);
            self.max_freq = self.max_freq.max(next_freq);
            if freq != next_freq {
                self.times_map.entry(freq).and_modify(|v| {
                    v.remove(&(*entry).key_ref());
                });
                self.times_map
                    .entry(next_freq)
                    .or_default()
                    .insert((*entry).key_ref());
            }


            self.check_reduce();
        }
    }
    /// 排出当前数据
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
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
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     let _ = lru.get("this");
    ///     assert!(lru.pop()==Some(("this", "lru")));
    ///     assert!(lru.len() == 1);
    /// }
    /// ```
    pub fn pop(&mut self) -> Option<(K, V)> {
        if self.len() == 0 {
            return None;
        }
        unsafe {
            for i in (0..=self.max_freq).rev() {
                if let Some(val) = self.times_map.get_mut(&i) {
                    if val.is_empty() {
                        continue;
                    }
                    let key = val.drain().next().expect("ok");
                    let value = self.map.remove(&key).expect("must ok");
                    let node = *Box::from_raw(value.as_ptr());
                    let LfuEntry { key, val, .. } = node;
                    return Some((key.assume_init(), val.assume_init()));
                    // val.take(value)
                }
            }
            None
        }
    }

    /// 弹出栈尾上的数据, 最久未使用的数据
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     let _ = lru.get("this");
    ///     assert!(lru.pop_last()==Some(("hello", "algorithm")));
    ///     assert!(lru.len() == 1);
    /// }
    /// ```
    pub fn pop_last(&mut self) -> Option<(K, V)> {
        if self.len() == 0 {
            return None;
        }

        unsafe {
            for i in 0..=self.max_freq {
                if let Some(val) = self.times_map.get_mut(&i) {
                    if val.is_empty() {
                        continue;
                    }
                    let key = val.drain().next().expect("ok");
                    let value = self.map.remove(&key).expect("must ok");
                    let node = *Box::from_raw(value.as_ptr());
                    let LfuEntry { key, val, .. } = node;
                    return Some((key.assume_init(), val.assume_init()));
                    // val.take(value)
                }
            }
            None
        }
    }


    /// 获取key值相对应的value值, 根本hash判定
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
    ///     lru.insert("hello", "algorithm");
    ///     lru.insert("this", "lru");
    ///     assert!(lru.get_visit(&"this") == Some(5));
    /// }
    /// ```
    pub fn get_visit<Q>(&mut self, k: &Q) ->  Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.get(KeyWrapper::from_ref(k)) {
            Some(l) => {
                let node = l.as_ptr();
                unsafe { Some((*node).counter) }
            }
            None => None,
        }
    }

    /// 获取key值相对应的value值, 根本hash判定
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
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
                self.reattach(node);
                unsafe { Some(&*(*node).val.as_ptr()) }
            }
            None => None,
        }
    }

    /// 获取key值相对应的key和value值, 根本hash判定
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
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
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
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
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
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
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
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

    fn replace_or_create_node(&mut self, k: K, v: V) -> (Option<(K, V)>, NonNull<LfuEntry<K, V>>) {
        if self.len() == self.cap {
            for i in 0..self.max_freq {
                if let Some(val) = self.times_map.get_mut(&i) {
                    if val.is_empty() {
                        continue;
                    }
                    let key = val.drain().next().expect("ok");
                    let old_node = self.map.remove(&key).unwrap();
                    let node_ptr: *mut LfuEntry<K, V> = old_node.as_ptr();

                    let replaced = unsafe {
                        (
                            mem::replace(&mut (*node_ptr).key, mem::MaybeUninit::new(k))
                                .assume_init(),
                            mem::replace(&mut (*node_ptr).val, mem::MaybeUninit::new(v))
                                .assume_init(),
                        )
                    };
                    unsafe {
                        (*node_ptr).counter = self.default_count.saturating_sub(1);
                    }
                    return (Some(replaced), old_node);
                }
            }
            unreachable!()
        } else {
            (None, unsafe {
                NonNull::new_unchecked(Box::into_raw(Box::new(LfuEntry::new_counter(
                    k,
                    v,
                    self.default_count.saturating_sub(1),
                ))))
            })
        }
    }

    /// 根据保留当前的元素, 返回false则表示抛弃元素
    ///
    /// ```
    /// use algorithm::LfuCache;
    /// fn main() {
    ///     let mut lru = LfuCache::new(3);
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
            let mut remove_keys = vec![];
            for (_, v) in self.map.iter() {
                let node = v.as_ptr();
                if !f(&*(*node).key.as_ptr(), &mut *(*node).val.as_mut_ptr()) {
                    remove_keys.push((*node).key_ref());
                }
            }
            for k in remove_keys {
                self.remove(&*k.k);
            }
        }
    }
}

impl<K: Clone + Hash + Eq, V: Clone, S: Clone + BuildHasher> Clone for LfuCache<K, V, S> {
    fn clone(&self) -> Self {
        let mut new_lru = LfuCache::with_hasher(self.cap, self.map.hasher().clone());

        for (key, value) in self.iter().rev() {
            new_lru.insert(key.clone(), value.clone());
        }

        new_lru
    }
}

impl<K, V, S> Drop for LfuCache<K, V, S> {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct Iter<'a, K: 'a, V: 'a, S> {
    len: usize,
    now_freq: u8,
    now_keys: Option<Vec<KeyRef<K>>>,
    base: &'a LfuCache<K, V, S>,
}

impl<'a, K, V, S> Iter<'a, K, V, S> {
    pub fn new(base: &'a LfuCache<K, V, S>) -> Self {
        Self {
            len: base.len(),
            now_freq: base.max_freq,
            now_keys: None,
            base,
        }
    }
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> Iterator for Iter<'a, K, V, S> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        if self.now_keys.is_none() {
            for i in (0..=self.now_freq).rev() {
                if let Some(s) = self.base.times_map.get(&i) {
                    if s.len() != 0 {
                        self.now_freq = i.saturating_sub(1);
                        self.now_keys = Some(s.iter().map(|s| KeyRef { k: s.k }).collect());
                        break;
                    }
                }
            }
        }

        if let Some(keys) = &mut self.now_keys {
            unsafe {
                let key = keys.pop().unwrap();
                let val = self.base.map.get(&key).unwrap();
                let node = val.as_ptr();
                if keys.len() == 0 {
                    self.now_keys = None;
                }
                return Some((&*(*node).key.as_ptr(), &*(*node).val.as_ptr()));
            }
        }
        return None;
    }
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> DoubleEndedIterator for Iter<'a, K, V, S> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        if self.now_keys.is_none() {
            for i in 0..=self.now_freq {
                if let Some(s) = self.base.times_map.get(&i) {
                    if s.len() != 0 {
                        self.now_freq = i.saturating_sub(1);
                        self.now_keys = Some(s.iter().map(|s| KeyRef { k: s.k }).collect());
                        break;
                    }
                }
            }
        }

        if let Some(keys) = &mut self.now_keys {
            unsafe {
                let key = keys.pop().unwrap();
                let val = self.base.map.get(&key).unwrap();
                let node = val.as_ptr();
                if keys.len() == 0 {
                    self.now_keys = None;
                }
                return Some((&*(*node).key.as_ptr(), &*(*node).val.as_ptr()));
            }
        }
        return None;
    }
}

pub struct Drain<'a, K: 'a + Hash + Eq, V: 'a, S: BuildHasher> {
    pub base: &'a mut LfuCache<K, V, S>,
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

pub struct Keys<'a, K, V, S> {
    iter: Iter<'a, K, V, S>,
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> Iterator for Keys<'a, K, V, S> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }
}

pub struct Values<'a, K, V, S> {
    iter: Iter<'a, K, V, S>,
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> Iterator for Values<'a, K, V, S> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}
