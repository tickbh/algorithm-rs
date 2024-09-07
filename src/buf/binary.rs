// Copyright 2022 - 2023 Wenmeng See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Author: tickbh
// -----
// Created Date: 2023/08/28 09:38:10

use std::fmt::Debug;
use std::io;
use std::io::Error;
use std::ops::Deref;
use std::{
    alloc::{dealloc, Layout},
    borrow::Borrow,
    cell::RefCell,
    cmp, hash,
    io::Read,
    io::Result,
    rc::Rc,
    slice,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::Bt;

static EMPTY_ARRAY: &[u8] = &[];
const STATIC_TYPE: u8 = 1;
const SHARED_TYPE: u8 = 2;

/// 二进制的封装, 包括静态引用及共享引用对象, 仅支持写操作
pub struct Binary {
    ptr: *const u8,
    // 共享引用计数
    counter: Rc<RefCell<AtomicUsize>>,
    // 游标值, 可以得出当前指向的位置
    cursor: usize,
    // 标记值, 从上一次标记到现在的游标值, 可以得出偏移的对象
    mark: usize,
    // 长度值, 还剩下多少的长度
    len: usize,
    // 对象虚表的引用函数
    vtable: &'static Vtable,
}

unsafe impl Sync for Binary {}

unsafe impl Send for Binary {}

pub struct Vtable {
    pub clone: unsafe fn(bin: &Binary) -> Binary,
    pub to_vec: unsafe fn(bin: &Binary) -> Vec<u8>,
    pub drop: unsafe fn(bin: &mut Binary),
    pub vtype: fn() -> u8,
}

const STATIC_VTABLE: Vtable = Vtable {
    clone: static_clone,
    to_vec: static_to_vec,
    drop: static_drop,
    vtype: || STATIC_TYPE,
};

unsafe fn static_clone(bin: &Binary) -> Binary {
    let slice = slice::from_raw_parts(bin.ptr, bin.len);
    Binary::from_static(slice)
}

unsafe fn static_to_vec(bin: &Binary) -> Vec<u8> {
    let slice = slice::from_raw_parts(bin.ptr, bin.len);
    slice.to_vec()
}

unsafe fn static_drop(_bin: &mut Binary) {
    // nothing to drop for &'static [u8]
}

const SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    to_vec: shared_to_vec,
    drop: shared_drop,
    vtype: || SHARED_TYPE,
};

unsafe fn shared_clone(bin: &Binary) -> Binary {
    bin.counter.borrow_mut().fetch_add(1, Ordering::Relaxed);
    Binary {
        ptr: bin.ptr,
        counter: bin.counter.clone(),
        cursor: bin.cursor,
        mark: bin.mark,
        len: bin.len,
        vtable: bin.vtable,
    }
}

unsafe fn shared_to_vec(bin: &Binary) -> Vec<u8> {
    let slice = slice::from_raw_parts(bin.ptr, bin.len);
    slice.to_vec()
}

unsafe fn shared_drop(bin: &mut Binary) {
    if (*bin.counter).borrow_mut().fetch_sub(1, Ordering::Release) == 1 {
        let ori = bin.ptr.sub(bin.cursor);
        dealloc(
            ori as *mut u8,
            Layout::from_size_align(bin.cursor + bin.len, 1).unwrap(),
        );
    }
}
impl Binary {
    pub fn new() -> Binary {
        Binary::from_static(EMPTY_ARRAY)
    }

    pub fn from_static(val: &'static [u8]) -> Binary {
        Binary {
            ptr: val.as_ptr(),
            counter: Rc::new(RefCell::new(AtomicUsize::new(0))),
            cursor: 0,
            mark: 0,
            len: val.len(),
            vtable: &STATIC_VTABLE,
        }
    }

    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Binary;
    ///
    /// let b = Binary::from(&b"hello"[..]);
    /// assert_eq!(b.len(), 5);
    /// ```
    ///
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the `Binary` has a length of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Binary;
    ///
    /// let b = Binary::new();
    /// assert!(b.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        unsafe { (self.vtable.to_vec)(self) }
    }

    /// 获取引用的数量
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Binary;
    ///
    /// let b = Binary::from(vec![1, 2, 3]);
    /// {
    /// let b1 = b.clone();
    /// assert!(b1.get_refs() == 2);
    /// drop(b1);
    /// }
    /// assert!(b.get_refs() == 1);
    /// ```
    pub fn get_refs(&self) -> usize {
        (*self.counter)
            .borrow()
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }

    #[inline]
    unsafe fn inc_start(&mut self, by: usize) {
        if by == 0 {
            return;
        }
        debug_assert!(self.len >= by, "internal: inc_start out of bounds");
        self.len -= by;
        self.ptr = self.ptr.add(by);
        self.cursor += by;
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe { self.sub_start(self.cursor) }
    }

    #[inline]
    unsafe fn sub_start(&mut self, by: usize) {
        // should already be asserted, but debug assert for tests
        debug_assert!(self.cursor >= by, "internal: inc_start out of bounds");
        self.len += by;
        self.ptr = self.ptr.sub(by);
        self.cursor -= by;
        self.mark = std::cmp::min(self.mark, self.cursor);
    }

    pub fn copy_from_slice(data: &[u8]) -> Self {
        data.to_vec().into()
    }

    #[inline]
    pub fn into_slice_all(&self) -> Vec<u8> {
        if (self.vtable.vtype)() == STATIC_TYPE {
            self.to_vec()
        } else {
            if (*self.counter).borrow().load(Ordering::SeqCst) == 1 {
                (*self.counter).borrow().fetch_add(1, Ordering::Relaxed);
                self.to_vec()
            } else {
                self.to_vec()
            }
        }
    }

    #[inline]
    pub fn into_slice(&self) -> Vec<u8> {
        if (self.vtable.vtype)() == STATIC_TYPE {
            self.to_vec()[self.cursor..(self.cursor + self.len)].to_vec()
        } else {
            if (*self.counter).borrow().load(Ordering::SeqCst) == 1 {
                (*self.counter).borrow().fetch_add(1, Ordering::Relaxed);
                self.to_vec()[self.cursor..(self.cursor + self.len)].to_vec()
            } else {
                self.to_vec()[self.cursor..(self.cursor + self.len)].to_vec()
            }
        }
    }
}

impl Clone for Binary {
    fn clone(&self) -> Self {
        unsafe { (self.vtable.clone)(self) }
    }
}

impl Drop for Binary {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self) }
    }
}

impl From<&'static str> for Binary {
    fn from(value: &'static str) -> Self {
        Binary::from_static(value.as_bytes())
    }
}

impl From<&'static [u8]> for Binary {
    fn from(value: &'static [u8]) -> Self {
        Binary::from_static(value)
    }
}

impl From<Box<[u8]>> for Binary {
    fn from(value: Box<[u8]>) -> Self {
        if value.len() == 0 {
            return Binary::new();
        }
        let len = value.len();
        let ptr = Box::into_raw(value) as *mut u8;
        Binary {
            ptr,
            len,
            mark: 0,
            cursor: 0,
            counter: Rc::new(RefCell::new(AtomicUsize::new(1))),
            vtable: &SHARED_VTABLE,
        }
    }
}

impl From<Vec<u8>> for Binary {
    fn from(value: Vec<u8>) -> Self {
        Binary::from(value.into_boxed_slice())
    }
}

impl Bt for Binary {
    fn remaining(&self) -> usize {
        self.len
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance_chunk(&mut self, n: usize) -> &[u8] {
        let ret = &unsafe { slice::from_raw_parts(self.ptr, self.len) }[..n];
        self.advance(n);
        ret
    }

    fn advance(&mut self, n: usize) {
        unsafe {
            self.inc_start(n);
        }
    }

    fn into_binary(self) -> Binary {
        self
    }
}

impl Read for Binary {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let left = self.remaining();
        if left == 0 || buf.len() == 0 {
            return Err(Error::new(io::ErrorKind::WouldBlock, ""));
        }
        let read = std::cmp::min(left, buf.len());
        unsafe {
            std::ptr::copy(&self.chunk()[0], &mut buf[0], read);
        }
        self.advance(read);
        Ok(read)
    }
}

impl Iterator for Binary {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.get_next()
    }
}

impl Deref for Binary {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Debug for Binary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Binary")
            .field("ptr", &self.ptr)
            .field("counter", &self.counter)
            .field("cursor", &self.cursor)
            .field("mark", &self.mark)
            .field("len", &self.len)
            .finish()
    }
}

impl AsRef<[u8]> for Binary {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl hash::Hash for Binary {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_slice().hash(state);
    }
}

impl Borrow<[u8]> for Binary {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl PartialEq for Binary {
    fn eq(&self, other: &Binary) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialOrd for Binary {
    fn partial_cmp(&self, other: &Binary) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl Ord for Binary {
    fn cmp(&self, other: &Binary) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl Eq for Binary {}

impl PartialEq<[u8]> for Binary {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl PartialOrd<[u8]> for Binary {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}

impl PartialEq<Binary> for [u8] {
    fn eq(&self, other: &Binary) -> bool {
        *other == *self
    }
}

impl PartialOrd<Binary> for [u8] {
    fn partial_cmp(&self, other: &Binary) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<str> for Binary {
    fn eq(&self, other: &str) -> bool {
        self.as_slice() == other.as_bytes()
    }
}

impl PartialOrd<str> for Binary {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

impl PartialEq<Binary> for str {
    fn eq(&self, other: &Binary) -> bool {
        *other == *self
    }
}

impl PartialOrd<Binary> for str {
    fn partial_cmp(&self, other: &Binary) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl PartialEq<Vec<u8>> for Binary {
    fn eq(&self, other: &Vec<u8>) -> bool {
        *self == other[..]
    }
}

impl PartialOrd<Vec<u8>> for Binary {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(&other[..])
    }
}

impl PartialEq<Binary> for Vec<u8> {
    fn eq(&self, other: &Binary) -> bool {
        *other == *self
    }
}

impl PartialOrd<Binary> for Vec<u8> {
    fn partial_cmp(&self, other: &Binary) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<String> for Binary {
    fn eq(&self, other: &String) -> bool {
        *self == other[..]
    }
}

impl PartialOrd<String> for Binary {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

impl PartialEq<Binary> for String {
    fn eq(&self, other: &Binary) -> bool {
        *other == *self
    }
}

impl PartialOrd<Binary> for String {
    fn partial_cmp(&self, other: &Binary) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl PartialEq<Binary> for &[u8] {
    fn eq(&self, other: &Binary) -> bool {
        *other == *self
    }
}

impl PartialOrd<Binary> for &[u8] {
    fn partial_cmp(&self, other: &Binary) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<Binary> for &str {
    fn eq(&self, other: &Binary) -> bool {
        *other == *self
    }
}

impl PartialOrd<Binary> for &str {
    fn partial_cmp(&self, other: &Binary) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl<'a, T: ?Sized> PartialEq<&'a T> for Binary
where
    Binary: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for Binary
where
    Binary: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(&**other)
    }
}

// impl From

impl Default for Binary {
    #[inline]
    fn default() -> Binary {
        Binary::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Binary;

    #[test]
    fn binary_refs() {
        {
            let s = Binary::from("aaaa");
            let s1 = s.clone();
            assert!(s1.get_refs() == 0);
            drop(s1);
            assert!(s.get_refs() == 0);
        }
        {
            let b = Binary::from(vec![1]);
            let b1 = b.clone();
            assert!(b1.get_refs() == 2);
            drop(b1);
            assert!(b.get_refs() == 1);
        }
    }
}
