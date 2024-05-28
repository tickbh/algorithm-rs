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
// Created Date: 2024/05/28 10:50:56

use std::{
    iter, mem,
    ops::{Index, IndexMut},
    slice, vec,
};

pub trait Reinit {
    fn reinit(&mut self) {}
}

macro_rules! impl_primitive_index {
    ($ty:ident, $zero:expr ) => {
        impl Reinit for $ty {
            #[inline(always)]
            fn reinit(&mut self) {
                *self = $zero;
            }
        }
    };
}

impl_primitive_index!(u8, 0);
impl_primitive_index!(u16, 0);
impl_primitive_index!(u32, 0);
impl_primitive_index!(u64, 0);
impl_primitive_index!(u128, 0);
impl_primitive_index!(i8, 0);
impl_primitive_index!(i16, 0);
impl_primitive_index!(i32, 0);
impl_primitive_index!(i64, 0);
impl_primitive_index!(i128, 0);
impl_primitive_index!(f32, 0.0);
impl_primitive_index!(f64, 0.0);

impl Reinit for bool {
    fn reinit(&mut self) {
        *self = false;
    }
}

impl Reinit for String {
    fn reinit(&mut self) {
        self.clear();
    }
}

impl Reinit for &str {
    fn reinit(&mut self) {
        *self = "";
    }
}

impl<T> Reinit for Vec<T> {
    fn reinit(&mut self) {
        self.clear();
    }
}

#[derive(Debug)]
struct Entry<T: Default> {
    t: T,
    next: usize,
}

impl<T: Default> Entry<T> {
    pub fn new() -> Self {
        Self {
            t: T::default(),
            next: usize::MAX,
        }
    }

    pub fn is_occupied(&self) -> bool {
        self.next == usize::MAX
    }
}

/// 一个缓存对象的实现, 类似linux中的slab
/// 将一个对象重复循环使用, 避免频繁分配数据的可能
/// 得出的对象可能未重新初始化, 为上一次的最终值, 请按需重新初始化
/// 默认的初始化依赖Default接口, 请实现Default
/// 
/// # Examples
/// 
/// ```
/// use algorithm::slabKCache;
/// use algorithm::Slab;
/// fn main() {
///     let mut slab = Slab::new();
///     for _ in 0..100 {
///         let k = slab.get_next();
///         slab[&k] = format!("{}", k);
///     }
///     assert!(slab.len() == 100);
///     for i in 0..100 {
///         let _ = slab.remove(i);
///     }
///     assert!(slab.len() == 0);
///     let k = slab.get_next();
///     assert!(k == 99);
///     assert!(slab[&k] == "99");
///     let k = slab.get_reinit_next();
///     assert!(k == 98);
///     assert!(slab[&k] == "");
/// }
/// ```
#[derive(Debug)]
pub struct Slab<T: Default> {
    entries: Vec<Entry<T>>,
    len: usize,
    next: usize,
}

impl<T: Default> Slab<T> {
    pub fn new() -> Self {
        Slab {
            entries: vec![],
            len: 0,
            next: 0,
        }
    }

    /// 获取当前长度
    pub fn len(&self) -> usize {
        self.len
    }

    /// 获取index值相对应的value值
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     let k = slab.insert("slab");
    ///     assert!(slab.get(k) == &"slab");
    /// }
    /// ```
    pub fn get(&mut self, key: usize) -> &T {
        let entry = &mut self.entries[key];
        assert!(entry.is_occupied() == true);
        &entry.t
    }

    /// 获取下一个的key值和val值
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     let k = slab.insert("slab");
    ///     assert!(slab.get_next_val() == (1, &mut ""));
    /// }
    /// ```
    pub fn get_next_val(&mut self) -> (usize, &mut T) {
        if self.entries.len() == self.len {
            let entry = Entry::new();
            self.entries.push(entry);
            self.len += 1;
            (self.len - 1, &mut self.entries[self.len - 1].t)
        } else {
            let entry = &mut self.entries[self.next];
            if entry.is_occupied() {
                unreachable!()
            }
            self.len += 1;
            let key = self.next;
            self.next = entry.next;
            entry.next = usize::MAX;
            (key, &mut entry.t)
        }
    }

    /// 获取下一个的key值
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     let k = slab.insert("slab");
    ///     assert!(slab.get_next() == 1);
    /// }
    /// ```
    pub fn get_next(&mut self) -> usize {
        if self.entries.len() == self.len {
            let entry = Entry::new();
            self.entries.push(entry);
            self.len += 1;
            self.len - 1
        } else {
            let entry = &mut self.entries[self.next];
            if entry.is_occupied() {
                unreachable!()
            }
            self.len += 1;
            let key = self.next;
            self.next = entry.next;
            entry.next = usize::MAX;
            key
        }
    }

    /// 插入一条数据进入slab缓存起来
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     let k = slab.insert("slab");
    ///     assert!(slab[&k] == "slab");
    /// }
    /// ```
    pub fn insert(&mut self, mut val: T) -> usize {
        let (key, value) = self.get_next_val();
        mem::swap(value, &mut val);
        key
    }

    /// 删除某个键值数据, 不会返回内容, 因为该内容会提供给下次复用
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     let k = slab.get_next();
    ///     let k1 = slab.get_next();
    ///     slab[&k1] = "slab";
    ///     assert!(slab.len() == 2);
    ///     slab.remove(k);
    ///     assert!(slab.len() == 1);
    ///     assert!(slab[&k1] == "slab");
    /// }
    /// ```
    pub fn remove(&mut self, key: usize) {
        if !self.try_remove(key) {
            panic!("index error")
        }
    }

    /// 试图删除某个键值数据, 不会返回内容, 因为该内容会提供给下次复用
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     let k = slab.get_next();
    ///     let k1 = slab.get_next();
    ///     slab[&k1] = "slab";
    ///     assert!(slab.len() == 2);
    ///     assert!(slab.try_remove(k) == true);
    ///     assert!(slab.try_remove(k) == false);
    ///     assert!(slab.len() == 1);
    ///     assert!(slab[&k1] == "slab");
    /// }
    /// ```
    pub fn try_remove(&mut self, key: usize) -> bool {
        let entry = &mut self.entries[key];
        if !entry.is_occupied() {
            return false;
        }
        self.len -= 1;
        entry.next = self.next;
        self.next = key;
        true
    }

    /// 是否包含某个键值
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     let k = slab.get_next();
    ///     slab[&k] = "slab";
    ///     assert!(slab.contains_key(k) == true);
    ///     assert!(slab.try_remove(k) == true);
    ///     assert!(slab.contains_key(k) == false);
    /// }
    /// ```
    pub fn contains_key(&mut self, k: usize) -> bool {
        let entry = &self.entries[k];
        entry.is_occupied()
    }

    /// 遍历当前的所有值
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     slab.insert("hello");
    ///     slab.insert("this");
    ///     let mut iter = slab.iter();
    ///     assert!(iter.next() == Some((0, &"hello")));
    ///     assert!(iter.next() == Some((1, &"this")));
    ///     assert!(iter.next() == None);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            entries: self.entries.iter().enumerate(),
            len: self.len,
        }
    }

    /// 遍历当前的所有值, 可同时修改值
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     slab.insert("slab".to_string());
    ///     slab.insert("this".to_string());
    ///     for (k, v) in slab.iter_mut() {
    ///         v.push_str(" ok")
    ///     }
    ///     assert!(slab[&0] == "slab ok");
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            entries: self.entries.iter_mut().enumerate(),
            len: self.len,
        }
    }

    /// 排除当时所有的值
    ///
    /// ```
    /// use algorithm::Slab;
    /// fn main() {
    ///     let mut slab = Slab::new();
    ///     slab.insert("slab".to_string());
    ///     slab.insert("this".to_string());
    ///     {
    ///         let mut drain = slab.drain();
    ///         assert!(drain.next()==Some("slab".to_string()));
    ///     }
    ///     assert!(slab.len() == 0);
    /// }
    /// ```
    pub fn drain(&mut self) -> Drain<'_, T> {
        let old = self.len;
        self.next = 0;
        self.len = 0;
        Drain {
            inner: self.entries.drain(..),
            len: old,
        }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, &mut T) -> bool,
    {
        for i in 0..self.entries.len() {
            let mut keep = true;
            if self.entries[i].is_occupied() {
                keep = f(i, &mut self.entries[i].t);
            }

            if !keep {
                self.remove(i);
            }
        }
    }
}

impl<T: Default + Reinit> Slab<T> {
    pub fn get_reinit_next(&mut self) -> usize {
        let key = self.get_next();
        self.entries[key].t.reinit();
        key
    }

    pub fn get_reinit_next_val(&mut self) -> (usize, &mut T) {
        let key = self.get_next();
        self.entries[key].t.reinit();
        (key, &mut self.entries[key].t)
    }
}

impl<'a, T: Default> Index<&'a usize> for Slab<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: &usize) -> &T {
        &self.entries[*index].t
    }
}

impl<'a, T: Default> IndexMut<&'a usize> for Slab<T> {
    #[inline]
    fn index_mut(&mut self, index: &usize) -> &mut T {
        &mut self.entries[*index].t
    }
}

pub struct IntoIter<T: Default> {
    entries: iter::Enumerate<vec::IntoIter<Entry<T>>>,
    len: usize,
}

pub struct Iter<'a, T: Default> {
    entries: iter::Enumerate<slice::Iter<'a, Entry<T>>>,
    len: usize,
}

impl<'a, T: Default> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            len: self.len,
        }
    }
}

pub struct IterMut<'a, T: Default> {
    entries: iter::Enumerate<slice::IterMut<'a, Entry<T>>>,
    len: usize,
}

pub struct Drain<'a, T: Default> {
    inner: vec::Drain<'a, Entry<T>>,
    len: usize,
}

impl<'a, T: Default> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        for (key, entry) in &mut self.entries {
            if entry.is_occupied() {
                self.len -= 1;
                return Some((key, &entry.t));
            }
        }

        debug_assert_eq!(self.len, 0);
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: Default> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some((key, entry)) = self.entries.next_back() {
            if entry.is_occupied() {
                self.len -= 1;
                return Some((key, &entry.t));
            }
        }
        debug_assert_eq!(self.len, 0);
        None
    }
}

impl<'a, T: Default> Iterator for IterMut<'a, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        for (key, entry) in &mut self.entries {
            if entry.is_occupied() {
                self.len -= 1;
                return Some((key, &mut entry.t));
            }
        }

        debug_assert_eq!(self.len, 0);
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: Default> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some((key, entry)) = self.entries.next_back() {
            if entry.is_occupied() {
                self.len -= 1;
                return Some((key, &mut entry.t));
            }
        }
        debug_assert_eq!(self.len, 0);
        None
    }
}

impl<T: Default> Iterator for IntoIter<T> {
    type Item = (usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        for (key, entry) in &mut self.entries {
            if entry.is_occupied() {
                self.len -= 1;
                return Some((key, entry.t));
            }
        }

        debug_assert_eq!(self.len, 0);
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T: Default> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some((key, entry)) = self.entries.next_back() {
            if entry.is_occupied() {
                self.len -= 1;
                return Some((key, entry.t));
            }
        }
        debug_assert_eq!(self.len, 0);
        None
    }
}

impl<'a, T: Default> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        for entry in &mut self.inner {
            if entry.is_occupied() {
                self.len -= 1;
                return Some(entry.t);
            }
        }

        debug_assert_eq!(self.len, 0);
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: Default> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.inner.next_back() {
            if entry.is_occupied() {
                self.len -= 1;
                return Some(entry.t);
            }
        }
        debug_assert_eq!(self.len, 0);
        None
    }
}

impl<T: Default> IntoIterator for Slab<T> {
    type Item = (usize, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            entries: self.entries.into_iter().enumerate(),
            len: self.len,
        }
    }
}
