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

use std::{
    cmp,
    fmt::{self, Debug},
    hash,
    io::{self, Error, Read, Result, Write},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr, slice, usize,
};

use super::{Binary, Bt};

use super::BtMut;

/// 100k，当数据大于100k时，可以尝试重排当前的结构
static RESORT_MEMORY_SIZE: usize = 102400;

/// 二进制的封装, 可写可读
pub struct BinaryMut {
    vec: Vec<u8>,
    // 游标值, 可以得出当前指向的位置
    cursor: usize,
    // 当前写入的位置
    wpos: usize,
    // 标记值, 从上一次标记到现在的游标值, 可以得出偏移的对象
    mark: usize,
    // 尝试重排的大小
    resort: usize,
}

impl BinaryMut {
    #[inline]
    pub fn with_capacity(n: usize) -> BinaryMut {
        BinaryMut::from_vec(Vec::with_capacity(n))
    }

    /// 新建对象
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::BinaryMut;
    ///
    /// let mut bytes = BinaryMut::new();
    /// assert_eq!(0, bytes.len());
    /// bytes.reserve(2);
    /// bytes.put_slice(b"xy");
    /// assert_eq!(&b"xy"[..], &bytes[..]);
    /// ```
    #[inline]
    pub fn new() -> BinaryMut {
        BinaryMut::with_capacity(0)
    }

    #[inline]
    pub(crate) fn from_vec(vec: Vec<u8>) -> BinaryMut {
        BinaryMut {
            wpos: vec.len(),
            vec,
            cursor: 0,
            mark: usize::MAX,
            resort: RESORT_MEMORY_SIZE,
        }
    }

    #[inline]
    pub fn into_slice_all(self) -> Vec<u8> {
        self.vec[self.cursor..self.wpos].into()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.vec[self.cursor..self.wpos]
    }

    #[inline]
    fn as_slice_mut(&mut self) -> &mut [u8] {
        &mut self.vec[self.cursor..self.wpos]
    }

    #[inline]
    fn inc_start(&mut self, by: usize) {
        // should already be asserted, but debug assert for tests
        debug_assert!(self.remaining() >= by, "internal: inc_start out of bounds");
        self.cursor += by;
    }

    /// 判断对象的长度
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::BinaryMut;
    ///
    /// let b = BinaryMut::from(&b"hello"[..]);
    /// assert_eq!(b.len(), 5);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.wpos - self.cursor
    }

    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    #[inline]
    pub fn clear(&mut self) {
        self.cursor = 0;
        self.wpos = 0;
    }
    /// 判断对象是否为空
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::BinaryMut;
    ///
    /// let b = BinaryMut::with_capacity(64);
    /// assert!(b.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 返回对象大小的容量
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::BinaryMut;
    ///
    /// let b = BinaryMut::with_capacity(64);
    /// assert_eq!(b.capacity(), 64);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        let len = self.vec.len();
        let rem = self.vec.capacity() - len;
        if rem >= additional {
            return;
        }
        self.vec.reserve(additional)
    }

    pub fn put<T: super::Bt>(&mut self, mut src: T)
    where
        Self: Sized,
    {
        while src.has_remaining() {
            let s = src.chunk();
            let l = s.len();
            self.extend_from_slice(s);
            src.advance(l);
        }
    }

    pub fn put_slice(&mut self, src: &[u8]) -> usize {
        self.extend_from_slice(src);
        src.len()
    }

    /// 将当前的数据转成不可写的对象Binary
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::BinaryMut;
    ///
    /// let mut buf = BinaryMut::with_capacity(0);
    /// buf.extend_from_slice(b"aaabbb");
    /// let bin = buf.freeze();
    ///
    /// assert_eq!(b"aaabbb", &bin[..]);
    /// ```
    #[inline]
    pub fn freeze(self) -> Binary {
        Binary::from(self.into_slice_all())
    }

    pub fn copy_to_binary(&mut self) -> Binary {
        let binary = Binary::from(self.chunk().to_vec());
        self.advance_all();
        binary
    }

    /// 扩展bytes到`BinaryMut`, 将会自动扩展容量空间
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::BinaryMut;
    ///
    /// let mut buf = BinaryMut::with_capacity(0);
    /// buf.extend_from_slice(b"aaabbb");
    /// buf.extend_from_slice(b"cccddd");
    ///
    /// assert_eq!(b"aaabbbcccddd", &buf[..]);
    /// ```
    #[inline]
    pub fn extend_from_slice(&mut self, extend: &[u8]) {
        let cnt = extend.len();
        self.reserve(cnt);

        unsafe {
            let dst = self.chunk_mut();
            // Reserved above
            debug_assert!(dst.len() >= cnt);

            ptr::copy_nonoverlapping(extend.as_ptr(), dst.as_mut_ptr().cast(), cnt);
        }

        unsafe {
            self.advance_mut(cnt);
        }
    }

    pub fn get_resort(&self) -> usize {
        self.resort
    }

    pub fn set_resort(&mut self, resort: usize) {
        self.resort = resort;
    }

    /// 标记当前游标的位置, 如果在需要的情况下进行回退
    /// 注: 如果当前数据被写入的情况下游标可能失效
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::{BinaryMut, Bt, BtMut};
    ///
    /// let mut buf = BinaryMut::with_capacity(0);
    /// buf.put_u8(1);
    /// buf.put_u8(2);
    /// buf.put_u8(3);
    /// buf.put_u8(4);
    /// assert_eq!(1, buf.get_u8());
    /// buf.mark();
    /// assert_eq!(2, buf.get_u8());
    /// assert_eq!(3, buf.get_u8());
    /// buf.rewind_mark();
    /// assert_eq!(2, buf.get_u8());
    /// assert_eq!(3, buf.get_u8());
    /// assert_eq!(4, buf.get_u8());
    ///
    /// ```
    pub fn mark(&mut self) {
        self.mark = self.cursor;
    }

    #[inline(always)]
    pub fn clear_mark(&mut self) {
        self.mark = usize::MAX;
    }

    pub fn rewind_mark(&mut self) -> bool {
        if self.mark == usize::MAX {
            false
        } else {
            if self.mark > self.wpos {
                self.clear_mark();
                false
            } else {
                self.cursor = self.mark;
                true
            }
        }
    }

    /// 获取可读且已初始化的数据
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::{BinaryMut, Bt, BtMut};
    ///
    /// let mut buf = BinaryMut::with_capacity(0);
    /// assert!(buf.data_mut().len() != 0);
    ///
    /// ```
    pub fn data_mut(&mut self) -> &mut [u8] {
        if self.wpos + 128 > self.vec.len() {
            self.vec.resize(self.wpos + 128, 0);
        }
        &mut self.vec[self.wpos..]
    }

    #[inline]
    pub unsafe fn try_resort_memory(&mut self) {
        if self.vec.len() < self.resort || self.cursor < self.resort / 2 {
            return;
        }
        let left = self.remaining();
        // 当时数据占一半, 不做处理
        if left * 2 > self.vec.len() {
            return;
        }
        if left == 0 {
            self.clear();
        } else {
            std::ptr::copy(
                self.vec.as_ptr().add(self.cursor),
                self.vec.as_mut_ptr(),
                left,
            );
            self.cursor = 0;
            self.wpos = left;
            self.clear_mark();
        }
    }
}

impl From<Vec<u8>> for BinaryMut {
    fn from(value: Vec<u8>) -> Self {
        BinaryMut::from_vec(value)
    }
}

impl Drop for BinaryMut {
    fn drop(&mut self) {}
}

impl Bt for BinaryMut {
    fn remaining(&self) -> usize {
        std::cmp::min(self.wpos, self.vec.len()) - self.cursor
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, n: usize) {
        self.inc_start(n);
    }

    fn advance_chunk(&mut self, n: usize) -> &[u8] {
        let cursor = self.cursor;
        self.inc_start(n);
        let ret = &{
            let end = std::cmp::min(self.wpos, cursor + n);
            &self.vec[cursor..end]
        }[..n];
        ret
    }

    fn into_binary(self) -> Binary {
        Binary::from(self.chunk().to_vec())
    }
}

unsafe impl BtMut for BinaryMut {
    fn remaining_mut(&self) -> usize {
        usize::MAX - self.len()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.wpos += cnt;
        if self.wpos > self.vec.len() {
            self.vec.set_len(self.wpos);
        }
        self.try_resort_memory();
    }

    fn chunk_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        if self.wpos == self.vec.capacity() {
            self.reserve(128);
        }
        unsafe {
            slice::from_raw_parts_mut(
                self.vec.as_mut_ptr().add(self.wpos) as *mut MaybeUninit<u8>,
                self.vec.capacity() - self.wpos,
            )
        }
    }
}



impl AsRef<[u8]> for BinaryMut {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for BinaryMut {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsMut<[u8]> for BinaryMut {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl DerefMut for BinaryMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl<'a> From<&'a [u8]> for BinaryMut {
    fn from(src: &'a [u8]) -> BinaryMut {
        BinaryMut::from_vec(src.to_vec())
    }
}

impl<'a> From<&'a str> for BinaryMut {
    fn from(src: &'a str) -> BinaryMut {
        BinaryMut::from(src.as_bytes())
    }
}

impl From<String> for BinaryMut {
    fn from(src: String) -> BinaryMut {
        BinaryMut::from_vec(src.into_bytes())
    }
}

impl From<BinaryMut> for Binary {
    fn from(src: BinaryMut) -> Binary {
        src.freeze()
    }
}

impl From<Binary> for BinaryMut {
    fn from(src: Binary) -> BinaryMut {
        BinaryMut::from(src.into_slice())
    }
}

impl PartialEq for BinaryMut {
    fn eq(&self, other: &BinaryMut) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialOrd for BinaryMut {
    fn partial_cmp(&self, other: &BinaryMut) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl Ord for BinaryMut {
    fn cmp(&self, other: &BinaryMut) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl Eq for BinaryMut {}

impl Default for BinaryMut {
    #[inline]
    fn default() -> BinaryMut {
        BinaryMut::new()
    }
}

impl hash::Hash for BinaryMut {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let s: &[u8] = self.as_ref();
        s.hash(state);
    }
}

impl Iterator for BinaryMut {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.get_next()
    }
}

impl fmt::Write for BinaryMut {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.remaining_mut() >= s.len() {
            self.put_slice(s.as_bytes());
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    #[inline]
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::write(self, args)
    }
}

impl TryInto<String> for BinaryMut {
    type Error = io::Error;

    fn try_into(self) -> std::result::Result<String, Self::Error> {
        Ok(String::from_utf8_lossy(&self.chunk()).to_string())
    }
}

impl Read for BinaryMut {
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

impl Write for BinaryMut {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.put_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// impl Write for &mut BinaryMut {
//     #[inline(always)]
//     fn write(&mut self, buf: &[u8]) -> Result<usize> {
//         self.put_slice(buf);
//         Ok(buf.len())
//     }

//     fn flush(&mut self) -> Result<()> {
//         Ok(())
//     }
// }

impl Debug for BinaryMut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryMut")
            .field("ptr", &self.vec)
            .field("cursor", &self.cursor)
            .field("wpos", &self.wpos)
            .field("mark", &self.mark)
            .finish()
    }
}

unsafe impl Sync for BinaryMut {}

unsafe impl Send for BinaryMut {}

#[cfg(test)]
mod tests {}
