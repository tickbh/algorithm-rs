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
// Created Date: 2023/09/14 12:21:02

use std::fmt::Debug;
use std::io::{self, Error};
use std::marker::PhantomData;
use std::ops::Deref;
use std::{borrow::Borrow, cmp, hash, io::Read, io::Result, slice};

use super::Binary;

use super::Bt;

static EMPTY_ARRAY: &[u8] = &[];

/// 二进制引用的封装, 只针对引用
pub struct BinaryRef<'a> {
    ptr: *const u8,
    // 游标值, 可以得出当前指向的位置
    cursor: usize,
    // 标记值, 从上一次标记到现在的游标值, 可以得出偏移的对象
    mark: usize,
    // 长度值, 还剩下多少的长度
    len: usize,

    data: PhantomData<&'a ()>,
}

impl<'a> BinaryRef<'a> {
    pub fn new() -> BinaryRef<'a> {
        BinaryRef::from(EMPTY_ARRAY)
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

    pub fn to_vec(&self) -> Vec<u8> {
        unsafe { slice::from_raw_parts(self.ptr, self.len).to_vec() }
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
        // should already be asserted, but debug assert for tests
        debug_assert!(self.len >= by, "internal: inc_start out of bounds");
        self.len -= by;
        self.ptr = self.ptr.add(by);
        self.cursor += by;
    }

    // #[inline]
    // unsafe fn sub_start(&mut self, by: usize) {
    //     // should already be asserted, but debug assert for tests
    //     debug_assert!(self.cursor >= by, "internal: inc_start out of bounds");
    //     self.len += by;
    //     self.ptr = self.ptr.sub(by);
    //     self.cursor -= by;
    //     self.mark = std::cmp::min(self.mark, self.cursor);
    // }

    pub fn copy_from_slice(data: &'a [u8]) -> Self {
        data.into()
    }

    #[inline]
    pub fn into_slice_all(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl<'a> Clone for BinaryRef<'a> {
    fn clone(&self) -> Self {
        BinaryRef {
            ptr: self.ptr,
            cursor: self.cursor,
            mark: self.mark,
            len: self.len,
            data: self.data.clone(),
        }
    }
}

impl<'a> Drop for BinaryRef<'a> {
    fn drop(&mut self) {}
}

impl<'a> From<&'a str> for BinaryRef<'a> {
    fn from(value: &'a str) -> Self {
        BinaryRef::from(value.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for BinaryRef<'a> {
    fn from(value: &'a [u8]) -> Self {
        let len = value.len();
        BinaryRef {
            ptr: value.as_ptr(),
            len,
            mark: 0,
            cursor: 0,
            data: PhantomData,
        }
    }
}

impl<'a> Bt for BinaryRef<'a> {
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
        Binary::from(self.chunk().to_vec())
    }
}

impl<'a> Read for BinaryRef<'a> {
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

impl<'a> Iterator for BinaryRef<'a> {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.get_next()
    }
}

impl<'a> Deref for BinaryRef<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> Debug for BinaryRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Binary")
            .field("ptr", &self.ptr)
            .field("cursor", &self.cursor)
            .field("mark", &self.mark)
            .field("len", &self.len)
            .finish()
    }
}

impl<'a> AsRef<[u8]> for BinaryRef<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> hash::Hash for BinaryRef<'a> {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_slice().hash(state);
    }
}

impl<'a> Borrow<[u8]> for BinaryRef<'a> {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> PartialEq for BinaryRef<'a> {
    fn eq(&self, other: &BinaryRef) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<'a> PartialOrd for BinaryRef<'a> {
    fn partial_cmp(&self, other: &BinaryRef) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<'a> Ord for BinaryRef<'a> {
    fn cmp(&self, other: &BinaryRef) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<'a> Eq for BinaryRef<'a> {}

impl<'a> PartialEq<[u8]> for BinaryRef<'a> {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl<'a> PartialOrd<[u8]> for BinaryRef<'a> {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}

impl<'a> PartialEq<BinaryRef<'a>> for [u8] {
    fn eq(&self, other: &BinaryRef) -> bool {
        *other == *self
    }
}

impl<'a> PartialOrd<BinaryRef<'a>> for [u8] {
    fn partial_cmp(&self, other: &BinaryRef) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl<'a> PartialEq<str> for BinaryRef<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_slice() == other.as_bytes()
    }
}

impl<'a> PartialOrd<str> for BinaryRef<'a> {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

impl<'a> PartialEq<BinaryRef<'a>> for str {
    fn eq(&self, other: &BinaryRef) -> bool {
        *other == *self
    }
}

impl<'a> PartialOrd<BinaryRef<'a>> for str {
    fn partial_cmp(&self, other: &BinaryRef) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl<'a> PartialEq<Vec<u8>> for BinaryRef<'a> {
    fn eq(&self, other: &Vec<u8>) -> bool {
        *self == other[..]
    }
}

impl<'a> PartialOrd<Vec<u8>> for BinaryRef<'a> {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(&other[..])
    }
}

impl<'a> PartialEq<BinaryRef<'a>> for Vec<u8> {
    fn eq(&self, other: &BinaryRef) -> bool {
        *other == *self
    }
}

impl<'a> PartialOrd<BinaryRef<'a>> for Vec<u8> {
    fn partial_cmp(&self, other: &BinaryRef) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl<'a> PartialEq<String> for BinaryRef<'a> {
    fn eq(&self, other: &String) -> bool {
        *self == other[..]
    }
}

impl<'a> PartialOrd<String> for BinaryRef<'a> {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

impl<'a> PartialEq<BinaryRef<'a>> for String {
    fn eq(&self, other: &BinaryRef) -> bool {
        *other == *self
    }
}

impl<'a> PartialOrd<BinaryRef<'a>> for String {
    fn partial_cmp(&self, other: &BinaryRef) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl<'a> PartialEq<BinaryRef<'a>> for &[u8] {
    fn eq(&self, other: &BinaryRef) -> bool {
        *other == *self
    }
}

impl<'a> PartialOrd<BinaryRef<'a>> for &[u8] {
    fn partial_cmp(&self, other: &BinaryRef) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl<'a> PartialEq<BinaryRef<'a>> for &str {
    fn eq(&self, other: &BinaryRef) -> bool {
        *other == *self
    }
}

impl<'a> PartialOrd<BinaryRef<'a>> for &str {
    fn partial_cmp(&self, other: &BinaryRef) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl<'a, T: ?Sized> PartialEq<&'a T> for BinaryRef<'a>
where
    BinaryRef<'a>: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for BinaryRef<'a>
where
    BinaryRef<'a>: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(&**other)
    }
}

// impl From

impl<'a> Default for BinaryRef<'a> {
    #[inline]
    fn default() -> BinaryRef<'a> {
        BinaryRef::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{BinaryRef, Bt};

    #[test]
    fn binary_refs() {
        {
            let s = BinaryRef::from("aaaa");
            let s1 = s.clone();
            drop(s1);
        }
        {
            let v = vec![1, 2];
            let mut b = BinaryRef::from(&v[..]);
            let x = b.get_u8();
            assert!(x == 1);
            let b1 = b.clone();
            drop(b1);
        }
    }
}
