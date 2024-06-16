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
    collections::hash_map::RandomState,
    fmt::{self, Debug},
    hash::{BuildHasher, Hash},
    ops::{Index, IndexMut},
};

use crate::{LfuCache, LruCache};

use super::{lfu, lru};

/// ARC(Adaptive Replacement Cache): 自适应缓存替换算法,它结合了LRU与LFU,来获得可用缓存的最佳使用。
/// 设置容量之后将最大保持该容量大小的数据
/// 后进的数据将会淘汰最久没有被访问的数据
///
/// # Examples
///
/// ```
/// use algorithm::ArcCache;
/// fn main() {
///     let mut arc = ArcCache::new(3);
///     arc.insert("now", "ok");
///     arc.insert("hello", "algorithm");
///     arc.insert("this", "arc");
///     arc.insert("auth", "tickbh");
///     assert!(arc.len() == 4);
///     assert_eq!(arc.get("hello"), Some(&"algorithm"));
///     assert_eq!(arc.get("this"), Some(&"arc"));
///     assert_eq!(arc.get("now"), Some(&"ok"));
///
/// }
/// ```
pub struct ArcCache<K, V, S> {
    main_lru: LruCache<K, V, S>,
    ghost_lru: LruCache<K, V, S>,

    main_lfu: LfuCache<K, V, S>,
    ghost_lfu: LruCache<K, V, S>,

    cap: usize,
}

impl<K: Hash + Eq, V> ArcCache<K, V, RandomState> {
    /// 因为存在四个数组, 所以实际的容量为这个的4倍
    pub fn new(cap: usize) -> Self {
        ArcCache::with_hasher(cap, RandomState::new())
    }
}

impl<K, V, S: Clone> ArcCache<K, V, S> {
    /// 提供hash函数
    pub fn with_hasher(cap: usize, hash_builder: S) -> ArcCache<K, V, S> {
        let cap = cap.max(1);
        Self {
            main_lru: LruCache::with_hasher(cap, hash_builder.clone()),
            ghost_lru: LruCache::with_hasher(cap, hash_builder.clone()),

            main_lfu: LfuCache::with_hasher(cap, hash_builder.clone()),
            ghost_lfu: LruCache::with_hasher(cap, hash_builder),

            cap,
        }
    }
}

impl<K, V, S> ArcCache<K, V, S> {
    
    /// 获取当前容量
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// 清理当前数据
    /// # Examples
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("now", "ok");
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     assert!(arc.len() == 3);
    ///     arc.clear();
    ///     assert!(arc.len() == 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.main_lru.clear();
        self.ghost_lru.clear();

        self.main_lfu.clear();
        self.ghost_lfu.clear();
    }

    /// 获取当前长度
    pub fn len(&self) -> usize {
        self.main_lru.len() + self.main_lfu.len() + self.ghost_lfu.len() + self.ghost_lru.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 扩展当前容量
    pub fn reserve(&mut self, additional: usize) {
        self.cap += additional;
        self.main_lfu.reserve(additional);
        self.main_lru.reserve(additional);
        self.ghost_lfu.reserve(additional);
        self.ghost_lru.reserve(additional);
    }

    /// 遍历当前的所有值
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     for (k, v) in arc.iter() {
    ///         assert!(k == &"hello" || k == &"this");
    ///         assert!(v == &"algorithm" || v == &"arc");
    ///     }
    ///     assert!(arc.len() == 2);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V, S> {
        Iter {
            lru_iter: self.main_lru.iter(),
            lfu_iter: self.main_lfu.iter(),
        }
    }

    /// 遍历当前的所有值, 可变
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm".to_string());
    ///     arc.insert("this", "arc".to_string());
    ///     for (k, v) in arc.iter_mut() {
    ///         v.push_str(" ok");
    ///     }
    ///     assert!(arc.len() == 2);
    ///     assert!(arc.get(&"this") == Some(&"arc ok".to_string()));
    /// assert!(arc.get(&"hello") == Some(&"algorithm ok".to_string()));
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, S> {
        IterMut { lru_iter: self.main_lru.iter_mut(), lfu_iter: self.main_lfu.iter_mut() }
    }

    /// 遍历当前的key值
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     let mut keys = arc.keys();
    ///     assert!(keys.next()==Some(&"this"));
    ///     assert!(keys.next()==Some(&"hello"));
    ///     assert!(keys.next() == None);
    /// }
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V, S> {
        Keys {
            iter: self.iter()
        }
    }

    /// 遍历当前的valus值
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let vec = vec![(1, 1), (2, 2), (3, 3)];
    ///     let mut map: ArcCache<_, _, _> = vec.into_iter().collect();
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
    pub fn values(&self) -> Values<'_, K, V, S> {
        Values {
            iter: self.iter()
        }
    }

    /// 遍历当前的valus值
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm".to_string());
    ///     arc.insert("this", "arc".to_string());
    ///     {
    ///         let mut values = arc.values_mut();
    ///         values.next().unwrap().push_str(" ok");
    ///         values.next().unwrap().push_str(" ok");
    ///         assert!(values.next() == None);
    ///     }
    ///     assert_eq!(arc.get(&"this"), Some(&"arc ok".to_string()))
    /// }
    /// ```
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V, S> {
        ValuesMut {
            iter: self.iter_mut()
        }
    }

    pub fn hasher(&self) -> &S {
        self.main_lru.hasher()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> ArcCache<K, V, S> {
    // /// 排出当前数据
    // ///
    // /// ```
    // /// use algorithm::ArcCache;
    // /// fn main() {
    // ///     let mut arc = ArcCache::new(3);
    // ///     arc.insert("hello", "algorithm");
    // ///     arc.insert("this", "arc");
    // ///     {
    // ///         let mut drain = arc.drain();
    // ///         assert!(drain.next()==Some(("hello", "algorithm")));
    // ///     }
    // ///     assert!(arc.len() == 0);
    // /// }
    // /// ```
    // pub fn drain(&mut self) -> Drain<'_, K, V, S> {
    //     Drain { base: self }
    // }

    /// 弹出栈顶上的数据, 最近使用的数据
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     assert!(arc.pop()==Some(("this", "arc")));
    ///     assert!(arc.len() == 1);
    /// }
    /// ```
    pub fn pop(&mut self) -> Option<(K, V)> {
        if self.main_lru.len() != 0 {
            return self.main_lru.pop();
        }
        self.main_lfu.pop()
    }

    /// 弹出栈尾上的数据, 最久未使用的数据
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     assert!(arc.pop_last()==Some(("hello", "algorithm")));
    ///     assert!(arc.len() == 1);
    /// }
    /// ```
    pub fn pop_last(&mut self) -> Option<(K, V)> {
        if self.main_lru.len() != 0 {
            return self.main_lru.pop_last();
        }
        self.main_lfu.pop_last()
    }

    pub fn contains_key<Q>(&mut self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.main_lru.contains_key(k) || self.main_lfu.contains_key(k)
    }

    /// 获取key值相对应的value值, 根据hash判定
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     assert!(arc.raw_get(&"this") == Some(&"arc"));
    /// }
    /// ```
    pub fn raw_get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(v) = self.main_lru.raw_get(k) {
            return Some(v);
        }
        self.main_lfu.raw_get(k)
    }

    /// 获取key值相对应的value值, 根据hash判定
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     assert!(arc.get(&"this") == Some(&"arc"));
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
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     assert!(arc.get_key_value(&"this") == Some((&"this", &"arc")));
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
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm".to_string());
    ///     arc.insert("this", "arc".to_string());
    ///     arc.get_mut(&"this").unwrap().insert_str(3, " good");
    ///     assert!(arc.get_key_value(&"this") == Some((&"this", &"arc good".to_string())));
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
        // {
        //     if let Some(v) = self.main_lfu.get_mut_key_value(k) {
        //         return Some(v)
        //     }
        // }
        if let Some((key, val)) = self.main_lru.remove(k) {
            self.main_lfu.insert(key, val);
            return self.main_lfu.get_mut_key_value(k);
        }

        if let Some((key, val)) = self.ghost_lfu.remove(k) {
            self.main_lfu.full_increase();
            self.main_lru.full_decrease();
            self.main_lfu.insert(key, val);
            return self.main_lfu.get_mut_key_value(k);
        }

        if let Some((key, val)) = self.ghost_lru.remove(k) {
            self.main_lru.full_increase();
            self.main_lfu.full_decrease();
            self.main_lru.insert(key, val);
            return self.main_lru.get_mut_key_value(k);
        }
        self.main_lfu.get_mut_key_value(k)
    }

    /// 插入值, 如果值重复将返回原来的数据
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     assert!(arc.insert("this", "arc good") == Some(&"arc"));
    /// }
    /// ```
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.capture_insert(k, v).map(|(_, v, _)| v)
    }

    pub fn capture_insert(&mut self, k: K, v: V) -> Option<(K, V, bool)> {
        if let Some((key, val, same)) = self.main_lru.capture_insert(k, v) {
            if same {
                Some((key, val, true))
            } else {
                self.ghost_lru.capture_insert(key, val)
            }
        } else {
            None
        }
    }


    pub fn get_or_insert<F>(&mut self, k: K, f: F) -> &V
    where
        F: FnOnce() -> V, {
        &*self.get_or_insert_mut(k, f)
    }

    pub fn get_or_insert_mut<F>(&mut self, k: K, f: F) -> &mut V
    where
        F: FnOnce() -> V, {

        if let Some((key, val)) = self.main_lru.remove(&k) {
            self.main_lfu.insert(key, val);
            return self.main_lfu.get_mut_key_value(&k).map(|(_, v)| v).unwrap();
        }

        if let Some((key, val)) = self.ghost_lfu.remove(&k) {
            self.main_lfu.full_increase();
            self.main_lru.full_decrease();
            self.main_lfu.insert(key, val);
            return self.main_lfu.get_mut_key_value(&k).map(|(_, v)| v).unwrap();
        }

        if let Some((key, val)) = self.ghost_lru.remove(&k) {
            self.main_lru.full_increase();
            self.main_lfu.full_decrease();
            self.main_lru.insert(key, val);
            return self.main_lru.get_mut_key_value(&k).map(|(_, v)| v).unwrap();
        }
        
        if self.main_lfu.contains_key(&k) {
            return self.main_lfu.get_mut_key_value(&k).map(|(_, v)| v).unwrap();
        }
        
        if self.main_lru.is_full() {
            let (pk, pv) = self.main_lru.pop_last().unwrap();
            self.ghost_lru.insert(pk, pv);
        }
        self.get_or_insert_mut(k, f)
    }


    /// 移除元素
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     assert!(arc.remove("this") == Some(("this", "arc")));
    ///     assert!(arc.len() == 1);
    /// }
    /// ```
    pub fn remove<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(v) = self.main_lru.remove(k) {
            return Some(v);
        }
        if let Some(v) = self.main_lfu.remove(k) {
            return Some(v);
        }
        None
    }

    /// 根据保留当前的元素, 返回false则表示抛弃元素
    ///
    /// ```
    /// use algorithm::ArcCache;
    /// fn main() {
    ///     let mut arc = ArcCache::new(3);
    ///     arc.insert("hello", "algorithm");
    ///     arc.insert("this", "arc");
    ///     arc.insert("year", "2024");
    ///     arc.retain(|_, v| *v == "2024" || *v == "arc");
    ///     assert!(arc.len() == 2);
    ///     assert!(arc.get("this") == Some(&"arc"));
    /// }
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.main_lru.retain(|k, v| f(k, v));
        self.main_lfu.retain(|k, v| f(k, v));
    }
}


impl<K: Hash + Eq, V: Default, S: BuildHasher> ArcCache<K, V, S> {
    pub fn get_or_insert_default(&mut self, k: K) -> &V {
        &*self.get_or_insert_mut(k, || V::default())
    }

    pub fn get_or_insert_default_mut(&mut self, k: K) -> &mut V {
        self.get_or_insert_mut(k, || V::default())
    }
}

impl<K: Clone + Hash + Eq, V: Clone, S: Clone + BuildHasher> Clone for ArcCache<K, V, S> {
    fn clone(&self) -> Self {
        ArcCache {
            main_lfu: self.main_lfu.clone(),
            main_lru: self.main_lru.clone(),
            ghost_lru: self.ghost_lru.clone(),
            ghost_lfu: self.ghost_lfu.clone(),
            cap: self.cap,
        }
    }
}

impl<K, V, S> Drop for ArcCache<K, V, S> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Convert ArcCache to iter, move out the tree.
pub struct IntoIter<K: Hash + Eq, V, S: BuildHasher> {
    base: ArcCache<K, V, S>,
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
        self.base.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.base.len(), Some(self.base.len()))
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> IntoIterator for ArcCache<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, S>;

    #[inline]
    fn into_iter(self) -> IntoIter<K, V, S> {
        IntoIter { base: self }
    }
}

pub struct Iter<'a, K: 'a, V: 'a, S> {
    lru_iter: lru::Iter<'a, K, V>,
    lfu_iter: lfu::Iter<'a, K, V, S>,
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> Iterator for Iter<'a, K, V, S> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.lru_iter.next() {
            return Some(v);
        }
        self.lfu_iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.lru_iter.size_hint().0 + self.lfu_iter.size_hint().0,
            None,
        )
    }
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> DoubleEndedIterator for Iter<'a, K, V, S> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.lru_iter.next_back() {
            return Some(v);
        }
        self.lfu_iter.next_back()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> DoubleEndedIterator for IntoIter<K, V, S> {
    #[inline]
    fn next_back(&mut self) -> Option<(K, V)> {
        self.base.pop_last()
    }
}

pub struct IterMut<'a, K: 'a, V: 'a, S> {
    lru_iter: lru::IterMut<'a, K, V>,
    lfu_iter: lfu::IterMut<'a, K, V, S>,
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> Iterator for IterMut<'a, K, V, S> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.lru_iter.next() {
            return Some(v);
        }
        self.lfu_iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.lru_iter.size_hint().0 + self.lfu_iter.size_hint().0,
            None,
        )
    }
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> DoubleEndedIterator for IterMut<'a, K, V, S> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.lru_iter.next_back() {
            return Some(v);
        }
        self.lfu_iter.next_back()
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

pub struct ValuesMut<'a, K, V, S> {
    iter: IterMut<'a, K, V, S>,
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> Iterator for ValuesMut<'a, K, V, S> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K: Hash + Eq, V> FromIterator<(K, V)> for ArcCache<K, V, RandomState> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> ArcCache<K, V, RandomState> {
        let mut arc = ArcCache::new(2);
        arc.extend(iter);
        arc
    }
}

impl<K: Hash + Eq, V> Extend<(K, V)> for ArcCache<K, V, RandomState> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        for (k, v) in iter {
            self.reserve(1);
            self.insert(k, v);
        }
    }
}

impl<K, V, S> PartialEq for ArcCache<K, V, S>
where
    K: Eq + Hash,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &ArcCache<K, V, S>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.raw_get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, S> Eq for ArcCache<K, V, S>
where
    K: Eq + Hash,
    V: PartialEq,
    S: BuildHasher,
{
}

impl<K, V, S> Debug for ArcCache<K, V, S>
where
    K: Eq + Hash + Debug,
    V: Debug,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'a, K, V, S> Index<&'a K> for ArcCache<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Output = V;

    #[inline]
    fn index(&self, index: &K) -> &V {
        self.raw_get(index).expect("no entry found for key")
    }
}

impl<'a, K, V, S> IndexMut<&'a K> for ArcCache<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    #[inline]
    fn index_mut(&mut self, index: &K) -> &mut V {
        self.get_mut(index).expect("no entry found for key")
    }
}

unsafe impl<K: Send, V: Send, S: Send> Send for ArcCache<K, V, S> {}
unsafe impl<K: Sync, V: Sync, S: Sync> Sync for ArcCache<K, V, S> {}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::RandomState;

    use super::ArcCache;

    #[test]
    fn test_insert() {
        let mut m = ArcCache::new(2);
        assert_eq!(m.len(), 0);
        m.insert(1, 2);
        assert_eq!(m.len(), 1);
        m.insert(2, 4);
        assert_eq!(m.len(), 2);
        m.insert(3, 6);
        assert_eq!(m.len(), 3);
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(m.len(), 3);
        assert_eq!(*m.get(&2).unwrap(), 4);
        assert_eq!(*m.get(&3).unwrap(), 6);
        assert_eq!(m.len(), 3);
        m.insert(4, 8);
        m.insert(5, 10);
        assert_eq!(m.len(), 5);
        m.insert(6, 12);
        assert_eq!(m.len(), 6);
        assert_eq!(*m.get(&6).unwrap(), 12);
        assert_eq!(m.len(), 5);
    }

    #[test]
    fn test_replace() {
        let mut m = ArcCache::new(2);
        assert_eq!(m.len(), 0);
        m.insert(2, 4);
        assert_eq!(m.len(), 1);
        m.insert(2, 6);
        assert_eq!(m.len(), 1);
        assert_eq!(*m.get(&2).unwrap(), 6);
    }

    #[test]
    fn test_clone() {
        let mut m = ArcCache::new(2);
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
        let mut m: ArcCache<isize, bool, RandomState> = ArcCache::new(2);
        assert_eq!(m.remove(&0), None);
    }

    #[test]
    fn test_empty_iter() {
        let mut m: ArcCache<isize, bool, RandomState> = ArcCache::new(2);
        assert_eq!(m.iter().next(), None);
        assert_eq!(m.iter_mut().next(), None);
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());
        assert_eq!(m.into_iter().next(), None);
    }

    #[test]
    fn test_lots_of_insertions() {
        let mut m = ArcCache::new(1000);

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
        let mut m = ArcCache::new(3);
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
        let mut m = ArcCache::new(3);
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
        let mut m = ArcCache::new(2);
        m.insert(1, 2);
        assert!(!m.is_empty());
        assert!(m.remove(&1).is_some());
        assert!(m.is_empty());
    }

    #[test]
    fn test_pop() {
        let mut m = ArcCache::new(3);
        m.insert(3, 6);
        m.insert(2, 4);
        m.insert(1, 2);
        assert_eq!(m.len(), 3);
        assert_eq!(m.pop(), Some((1, 2)));
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_last(), Some((3, 6)));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_iterate() {
        let mut m = ArcCache::new(32);
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
        let map: ArcCache<_, _, _> = vec.into_iter().collect();
        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }

    #[test]
    fn test_values() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: ArcCache<_, _, _> = vec.into_iter().collect();
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&'a'));
        assert!(values.contains(&'b'));
        assert!(values.contains(&'c'));
    }

    #[test]
    fn test_values_mut() {
        let vec = vec![(1, 1), (2, 2), (3, 3)];
        let mut map: ArcCache<_, _, _> = vec.into_iter().collect();
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
        let mut m = ArcCache::new(2);
        assert!(m.get(&1).is_none());
        m.insert(1, 2);
        match m.get(&1) {
            None => panic!(),
            Some(v) => assert_eq!(*v, 2),
        }
    }

    #[test]
    fn test_eq() {
        let mut m1 = ArcCache::new(3);
        m1.insert(1, 2);
        m1.insert(2, 3);
        m1.insert(3, 4);

        let mut m2 = ArcCache::new(3);
        m2.insert(1, 2);
        m2.insert(2, 3);

        assert!(m1 != m2);

        m2.insert(3, 4);

        assert_eq!(m1, m2);
    }

    #[test]
    fn test_from_iter() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: ArcCache<_, _, _> = xs.iter().cloned().collect();

        for &(k, v) in &xs {
            assert_eq!(map.raw_get(&k), Some(&v));
        }
    }

    #[test]
    fn test_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: ArcCache<_, _, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.size_hint(), (3, None));
    }

    #[test]
    fn test_iter_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: ArcCache<_, _, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn test_mut_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: ArcCache<_, _, _> = xs.iter().cloned().collect();

        let mut iter = map.iter_mut();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.size_hint(), (3, None));
    }

    #[test]
    fn test_iter_mut_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: ArcCache<_, _, _> = xs.iter().cloned().collect();

        let mut iter = map.iter_mut();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn test_index() {
        let mut map = ArcCache::new(2);

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        assert_eq!(map[&2], 1);
    }

    #[test]
    #[should_panic]
    fn test_index_nonexistent() {
        let mut map = ArcCache::new(2);

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        map[&4];
    }

    #[test]
    fn test_extend_iter() {
        let mut a = ArcCache::new(2);
        a.insert(1, "one");
        let mut b = ArcCache::new(2);
        b.insert(2, "two");
        b.insert(3, "three");

        a.extend(b.into_iter());

        assert_eq!(a.len(), 3);
        assert_eq!(a[&1], "one");
        assert_eq!(a[&2], "two");
        assert_eq!(a[&3], "three");
    }


    #[test]
    fn test_send() {
        use std::thread;

        let mut cache = ArcCache::new(4);
        cache.insert(1, "a");

        let handle = thread::spawn(move || {
            assert_eq!(cache.get(&1), Some(&"a"));
        });

        assert!(handle.join().is_ok());
    }
}
