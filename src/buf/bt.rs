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
    io::{self},
    mem,
};

use super::panic_advance;
use super::Binary;

macro_rules! try_advance {
    ($flag:expr) => {
        if !$flag {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "not enough",
            ));
        }
    };
}

macro_rules! buf_get_impl {
    ($this:ident, $typ:tt::$conv:tt) => {{
        const SIZE: usize = mem::size_of::<$typ>();
        // try to convert directly from the bytes
        // this Option<ret> trick is to avoid keeping a borrow on self
        // when advance() is called (mut borrow) and to call bytes() only once
        let ret = $this
            .chunk()
            .get(..SIZE)
            .map(|src| unsafe { $typ::$conv(*(src as *const _ as *const [_; SIZE])) });

        if let Some(ret) = ret {
            // if the direct conversion was possible, advance and return
            $this.advance(SIZE);
            return ret;
        } else {
            // if not we copy the bytes in a temp buffer then convert
            let mut buf = [0; SIZE];
            $this.copy_to_slice(&mut buf); // (do the advance)
            return $typ::$conv(buf);
        }
    }};
    (le => $this:ident, $typ:tt, $len_to_read:expr) => {{
        debug_assert!(mem::size_of::<$typ>() >= $len_to_read);

        // The same trick as above does not improve the best case speed.
        // It seems to be linked to the way the method is optimised by the compiler
        let mut buf = [0; (mem::size_of::<$typ>())];
        $this.copy_to_slice(&mut buf[..($len_to_read)]);
        return $typ::from_le_bytes(buf);
    }};
    (be => $this:ident, $typ:tt, $len_to_read:expr) => {{
        debug_assert!(mem::size_of::<$typ>() >= $len_to_read);

        let mut buf = [0; (mem::size_of::<$typ>())];
        $this.copy_to_slice(&mut buf[mem::size_of::<$typ>() - ($len_to_read)..]);
        return $typ::from_be_bytes(buf);
    }};
}

macro_rules! buf_peek_impl {
    ($this:ident, $typ:tt::$conv:tt) => {{
        const SIZE: usize = mem::size_of::<$typ>();
        // try to convert directly from the bytes
        // this Option<ret> trick is to avoid keeping a borrow on self
        // when advance() is called (mut borrow) and to call bytes() only once
        let ret = $this
            .chunk()
            .get(..SIZE)
            .map(|src| unsafe { $typ::$conv(*(src as *const _ as *const [_; SIZE])) });

        if let Some(ret) = ret {
            // if the direct conversion was possible, advance and return
            $this.advance(SIZE);
            return ret;
        } else {
            // if not we copy the bytes in a temp buffer then convert
            let mut buf = [0; SIZE];
            $this.peek_to_slice(&mut buf); // (do the advance)
            return $typ::$conv(buf);
        }
    }};
    (le => $this:ident, $typ:tt, $len_to_read:expr) => {{
        debug_assert!(mem::size_of::<$typ>() >= $len_to_read);

        // The same trick as above does not improve the best case speed.
        // It seems to be linked to the way the method is optimised by the compiler
        let mut buf = [0; (mem::size_of::<$typ>())];
        $this.peek_to_slice(&mut buf[..($len_to_read)]);
        return $typ::from_le_bytes(buf);
    }};
    (be => $this:ident, $typ:tt, $len_to_read:expr) => {{
        debug_assert!(mem::size_of::<$typ>() >= $len_to_read);

        let mut buf = [0; (mem::size_of::<$typ>())];
        $this.peek_to_slice(&mut buf[mem::size_of::<$typ>() - ($len_to_read)..]);
        return $typ::from_be_bytes(buf);
    }};
}
pub trait Bt {
    /// 获取剩余数量
    fn remaining(&self) -> usize;

    /// 获取当前数据的切片引用
    fn chunk(&self) -> &[u8];

    /// 消耗掉多少字节的数据, 做指针偏移
    fn advance(&mut self, n: usize);

    /// 消耗掉多少字节的数据并返回消耗的数据
    fn advance_chunk(&mut self, n: usize) -> &[u8];

    /// 将数据转成Binary
    fn into_binary(self) -> Binary;

    /// 消耗所有的字节
    fn advance_all(&mut self) {
        self.advance(self.remaining());
    }

    /// 获取当前的值, 但不做任何偏移
    fn peek(&self) -> Option<u8> {
        if self.has_remaining() {
            let ret = self.chunk()[0] as u8;
            Some(ret)
        } else {
            None
        }
    }

    /// 是否还有数据
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    /// 获取当前的值并将偏移值+1
    fn get_next(&mut self) -> Option<u8> {
        if self.has_remaining() {
            let val = self.peek().unwrap();
            self.advance(1);
            Some(val)
        } else {
            None
        }
    }

    /// 拷贝数据 `self` into `dst`.
    ///
    /// # Examples
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"hello world"[..];
    /// let mut dst = [0; 5];
    ///
    /// buf.copy_to_slice(&mut dst);
    /// assert_eq!(&b"hello"[..], &dst);
    /// assert_eq!(6, buf.remaining());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if `self.remaining() < dst.len()`
    fn copy_to_slice(&mut self, dst: &mut [u8]) -> usize {
        assert!(self.remaining() >= dst.len());
        unsafe {
            let src = self.chunk();
            std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), dst.len());
            self.advance(dst.len())
        }
        dst.len()
    }

    fn peek_to_slice(&mut self, dst: &mut [u8]) -> usize {
        assert!(self.remaining() >= dst.len());
        unsafe {
            let src = self.chunk();
            std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), dst.len());
        }
        dst.len()
    }

    fn get_u8(&mut self) -> u8 {
        assert!(self.remaining() >= 1);
        let ret = self.chunk()[0];
        self.advance(1);
        ret
    }

    fn peek_u8(&mut self) -> u8 {
        assert!(self.remaining() >= 1);
        let ret = self.chunk()[0];
        ret
    }

    fn try_get_u8(&mut self) -> io::Result<u8> {
        try_advance!(self.remaining() >= 1);
        Ok(self.get_u8())
    }

    fn get_i8(&mut self) -> i8 {
        assert!(self.remaining() >= 1);
        let ret = self.chunk()[0] as i8;
        self.advance(1);
        ret
    }

    fn peek_i8(&mut self) -> i8 {
        assert!(self.remaining() >= 1);
        let ret = self.chunk()[0] as i8;
        ret
    }

    fn try_get_i8(&mut self) -> io::Result<i8> {
        try_advance!(self.remaining() >= 1);
        Ok(self.get_i8())
    }

    /// Gets an unsigned 16 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x08\x09 hello"[..];
    /// assert_eq!(0x0809, buf.get_u16());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u16(&mut self) -> u16 {
        buf_get_impl!(self, u16::from_be_bytes);
    }

    fn peek_u16(&mut self) -> u16 {
        buf_peek_impl!(self, u16::from_be_bytes);
    }

    fn try_get_u16(&mut self) -> io::Result<u16> {
        try_advance!(self.remaining() >= 2);
        Ok(self.get_u16())
    }
    /// Gets an unsigned 16 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x09\x08 hello"[..];
    /// assert_eq!(0x0809, buf.get_u16_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u16_le(&mut self) -> u16 {
        buf_get_impl!(self, u16::from_le_bytes);
    }

    fn peek_u16_le(&mut self) -> u16 {
        buf_peek_impl!(self, u16::from_le_bytes);
    }

    fn try_get_u16_le(&mut self) -> io::Result<u16> {
        try_advance!(self.remaining() >= 2);
        Ok(self.get_u16_le())
    }

    /// Gets an unsigned 16 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x08\x09 hello",
    ///     false => b"\x09\x08 hello",
    /// };
    /// assert_eq!(0x0809, buf.get_u16_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u16_ne(&mut self) -> u16 {
        buf_get_impl!(self, u16::from_ne_bytes);
    }

    fn peek_u16_ne(&mut self) -> u16 {
        buf_peek_impl!(self, u16::from_ne_bytes);
    }

    fn try_get_u16_ne(&mut self) -> io::Result<u16> {
        try_advance!(self.remaining() >= 2);
        Ok(self.get_u16_ne())
    }

    /// Gets a signed 16 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x08\x09 hello"[..];
    /// assert_eq!(0x0809, buf.get_i16());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i16(&mut self) -> i16 {
        buf_get_impl!(self, i16::from_be_bytes);
    }

    fn peek_i16(&mut self) -> i16 {
        buf_peek_impl!(self, i16::from_be_bytes);
    }

    fn try_get_i16(&mut self) -> io::Result<i16> {
        try_advance!(self.remaining() >= 2);
        Ok(self.get_i16())
    }

    /// Gets a signed 16 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x09\x08 hello"[..];
    /// assert_eq!(0x0809, buf.get_i16_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i16_le(&mut self) -> i16 {
        buf_get_impl!(self, i16::from_le_bytes);
    }
    fn peek_i16_le(&mut self) -> i16 {
        buf_peek_impl!(self, i16::from_le_bytes);
    }
    fn try_get_i16_le(&mut self) -> io::Result<i16> {
        try_advance!(self.remaining() >= 2);
        Ok(self.get_i16_le())
    }
    /// Gets a signed 16 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x08\x09 hello",
    ///     false => b"\x09\x08 hello",
    /// };
    /// assert_eq!(0x0809, buf.get_i16_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i16_ne(&mut self) -> i16 {
        buf_get_impl!(self, i16::from_ne_bytes);
    }

    fn peek_i16_ne(&mut self) -> i16 {
        buf_peek_impl!(self, i16::from_ne_bytes);
    }

    fn try_get_i16_ne(&mut self) -> io::Result<i16> {
        try_advance!(self.remaining() >= 2);
        Ok(self.get_i16_ne())
    }
    /// Gets an unsigned 32 bit integer from `self` in the big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x08\x09\xA0\xA1 hello"[..];
    /// assert_eq!(0x0809A0A1, buf.get_u32());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u32(&mut self) -> u32 {
        buf_get_impl!(self, u32::from_be_bytes);
    }

    fn peek_u32(&mut self) -> u32 {
        buf_peek_impl!(self, u32::from_be_bytes);
    }

    fn try_get_u32(&mut self) -> io::Result<u32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_u32())
    }
    /// Gets an unsigned 32 bit integer from `self` in the little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\xA1\xA0\x09\x08 hello"[..];
    /// assert_eq!(0x0809A0A1, buf.get_u32_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u32_le(&mut self) -> u32 {
        buf_get_impl!(self, u32::from_le_bytes);
    }
    
    fn peek_u32_le(&mut self) -> u32 {
        buf_peek_impl!(self, u32::from_le_bytes);
    }

    fn try_get_u32_le(&mut self) -> io::Result<u32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_u32_le())
    }
    /// Gets an unsigned 32 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x08\x09\xA0\xA1 hello",
    ///     false => b"\xA1\xA0\x09\x08 hello",
    /// };
    /// assert_eq!(0x0809A0A1, buf.get_u32_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u32_ne(&mut self) -> u32 {
        buf_get_impl!(self, u32::from_ne_bytes);
    }

    fn peek_u32_ne(&mut self) -> u32 {
        buf_peek_impl!(self, u32::from_ne_bytes);
    }
    
    fn try_get_u32_ne(&mut self) -> io::Result<u32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_u32_ne())
    }
    /// Gets a signed 32 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x08\x09\xA0\xA1 hello"[..];
    /// assert_eq!(0x0809A0A1, buf.get_i32());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i32(&mut self) -> i32 {
        buf_get_impl!(self, i32::from_be_bytes);
    }

    fn peek_i32(&mut self) -> i32 {
        buf_peek_impl!(self, i32::from_be_bytes);
    }

    fn try_get_i32(&mut self) -> io::Result<i32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_i32())
    }

    /// Gets a signed 32 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\xA1\xA0\x09\x08 hello"[..];
    /// assert_eq!(0x0809A0A1, buf.get_i32_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i32_le(&mut self) -> i32 {
        buf_get_impl!(self, i32::from_le_bytes);
    }

    fn peek_i32_le(&mut self) -> i32 {
        buf_peek_impl!(self, i32::from_le_bytes);
    }

    fn try_get_i32_le(&mut self) -> io::Result<i32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_i32_le())
    }

    /// Gets a signed 32 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x08\x09\xA0\xA1 hello",
    ///     false => b"\xA1\xA0\x09\x08 hello",
    /// };
    /// assert_eq!(0x0809A0A1, buf.get_i32_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i32_ne(&mut self) -> i32 {
        buf_get_impl!(self, i32::from_ne_bytes);
    }

    fn peek_i32_ne(&mut self) -> i32 {
        buf_peek_impl!(self, i32::from_ne_bytes);
    }

    fn try_get_i32_ne(&mut self) -> io::Result<i32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_i32_ne())
    }
    /// Gets an unsigned 64 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x01\x02\x03\x04\x05\x06\x07\x08 hello"[..];
    /// assert_eq!(0x0102030405060708, buf.get_u64());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u64(&mut self) -> u64 {
        buf_get_impl!(self, u64::from_be_bytes);
    }

    fn peek_u64(&mut self) -> u64 {
        buf_peek_impl!(self, u64::from_be_bytes);
    }

    fn try_get_u64(&mut self) -> io::Result<u64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_u64())
    }

    /// Gets an unsigned 64 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x08\x07\x06\x05\x04\x03\x02\x01 hello"[..];
    /// assert_eq!(0x0102030405060708, buf.get_u64_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u64_le(&mut self) -> u64 {
        buf_get_impl!(self, u64::from_le_bytes);
    }

    fn peek_u64_le(&mut self) -> u64 {
        buf_peek_impl!(self, u64::from_le_bytes);
    }

    fn try_get_u64_le(&mut self) -> io::Result<u64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_u64_le())
    }

    /// Gets an unsigned 64 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03\x04\x05\x06\x07\x08 hello",
    ///     false => b"\x08\x07\x06\x05\x04\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x0102030405060708, buf.get_u64_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u64_ne(&mut self) -> u64 {
        buf_get_impl!(self, u64::from_ne_bytes);
    }

    fn peek_u64_ne(&mut self) -> u64 {
        buf_peek_impl!(self, u64::from_ne_bytes);
    }

    fn try_get_u64_ne(&mut self) -> io::Result<u64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_u64_ne())
    }
    /// Gets a signed 64 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x01\x02\x03\x04\x05\x06\x07\x08 hello"[..];
    /// assert_eq!(0x0102030405060708, buf.get_i64());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i64(&mut self) -> i64 {
        buf_get_impl!(self, i64::from_be_bytes);
    }

    fn peek_i64(&mut self) -> i64 {
        buf_peek_impl!(self, i64::from_be_bytes);
    }

    fn try_get_i64(&mut self) -> io::Result<i64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_i64())
    }
    /// Gets a signed 64 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x08\x07\x06\x05\x04\x03\x02\x01 hello"[..];
    /// assert_eq!(0x0102030405060708, buf.get_i64_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i64_le(&mut self) -> i64 {
        buf_get_impl!(self, i64::from_le_bytes);
    }

    fn peek_i64_le(&mut self) -> i64 {
        buf_peek_impl!(self, i64::from_le_bytes);
    }

    fn try_get_i64_le(&mut self) -> io::Result<i64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_i64_le())
    }
    /// Gets a signed 64 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03\x04\x05\x06\x07\x08 hello",
    ///     false => b"\x08\x07\x06\x05\x04\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x0102030405060708, buf.get_i64_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i64_ne(&mut self) -> i64 {
        buf_get_impl!(self, i64::from_ne_bytes);
    }

    fn peek_i64_ne(&mut self) -> i64 {
        buf_peek_impl!(self, i64::from_ne_bytes);
    }

    fn try_get_i64_ne(&mut self) -> io::Result<i64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_i64_ne())
    }

    /// Gets an unsigned 128 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16 hello"[..];
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_u128());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u128(&mut self) -> u128 {
        buf_get_impl!(self, u128::from_be_bytes);
    }

    fn peek_u128(&mut self) -> u128 {
        buf_peek_impl!(self, u128::from_be_bytes);
    }

    fn try_get_u128(&mut self) -> io::Result<u128> {
        try_advance!(self.remaining() >= 16);
        Ok(self.get_u128())
    }
    /// Gets an unsigned 128 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01 hello"[..];
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_u128_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u128_le(&mut self) -> u128 {
        buf_get_impl!(self, u128::from_le_bytes);
    }

    fn peek_u128_le(&mut self) -> u128 {
        buf_peek_impl!(self, u128::from_le_bytes);
    }

    fn try_get_u128_le(&mut self) -> io::Result<u128> {
        try_advance!(self.remaining() >= 16);
        Ok(self.get_u128_le())
    }
    /// Gets an unsigned 128 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16 hello",
    ///     false => b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_u128_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u128_ne(&mut self) -> u128 {
        buf_get_impl!(self, u128::from_ne_bytes);
    }

    fn peek_u128_ne(&mut self) -> u128 {
        buf_peek_impl!(self, u128::from_ne_bytes);
    }

    fn try_get_u128_ne(&mut self) -> io::Result<u128> {
        try_advance!(self.remaining() >= 16);
        Ok(self.get_u128_ne())
    }

    /// Gets a signed 128 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16 hello"[..];
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_i128());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i128(&mut self) -> i128 {
        buf_get_impl!(self, i128::from_be_bytes);
    }

    fn peek_i128(&mut self) -> i128 {
        buf_peek_impl!(self, i128::from_be_bytes);
    }

    fn try_get_i128(&mut self) -> io::Result<i128> {
        try_advance!(self.remaining() >= 16);
        Ok(self.get_i128())
    }

    /// Gets a signed 128 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01 hello"[..];
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_i128_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i128_le(&mut self) -> i128 {
        buf_get_impl!(self, i128::from_le_bytes);
    }

    fn peek_i128_le(&mut self) -> i128 {
        buf_peek_impl!(self, i128::from_le_bytes);
    }

    fn try_get_i128_le(&mut self) -> io::Result<i128> {
        try_advance!(self.remaining() >= 16);
        Ok(self.get_i128_le())
    }
    /// Gets a signed 128 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16 hello",
    ///     false => b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_i128_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i128_ne(&mut self) -> i128 {
        buf_get_impl!(self, i128::from_ne_bytes);
    }

    fn peek_i128_ne(&mut self) -> i128 {
        buf_peek_impl!(self, i128::from_ne_bytes);
    }

    fn try_get_i128_ne(&mut self) -> io::Result<i128> {
        try_advance!(self.remaining() >= 16);
        Ok(self.get_i128_ne())
    }
    /// Gets an unsigned n-byte integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x01\x02\x03 hello"[..];
    /// assert_eq!(0x010203, buf.get_uint(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_uint(&mut self, nbytes: usize) -> u64 {
        buf_get_impl!(be => self, u64, nbytes);
    }

    fn peek_uint(&mut self, nbytes: usize) -> u64 {
        buf_peek_impl!(be => self, u64, nbytes);
    }

    fn try_get_uint(&mut self, nbytes: usize) -> io::Result<u64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_uint(nbytes))
    }
    /// Gets an unsigned n-byte integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x03\x02\x01 hello"[..];
    /// assert_eq!(0x010203, buf.get_uint_le(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_uint_le(&mut self, nbytes: usize) -> u64 {
        buf_get_impl!(le => self, u64, nbytes);
    }

    fn peek_uint_le(&mut self, nbytes: usize) -> u64 {
        buf_peek_impl!(le => self, u64, nbytes);
    }

    fn try_get_uint_le(&mut self, nbytes: usize) -> io::Result<u64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_uint_le(nbytes))
    }
    /// Gets an unsigned n-byte integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03 hello",
    ///     false => b"\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x010203, buf.get_uint_ne(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_uint_ne(&mut self, nbytes: usize) -> u64 {
        if cfg!(target_endian = "big") {
            self.get_uint(nbytes)
        } else {
            self.get_uint_le(nbytes)
        }
    }

    fn peek_uint_ne(&mut self, nbytes: usize) -> u64 {
        if cfg!(target_endian = "big") {
            self.peek_uint(nbytes)
        } else {
            self.peek_uint_le(nbytes)
        }
    }

    fn try_get_uint_ne(&mut self, nbytes: usize) -> io::Result<u64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_uint_ne(nbytes))
    }
    /// Gets a signed n-byte integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x01\x02\x03 hello"[..];
    /// assert_eq!(0x010203, buf.get_int(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_int(&mut self, nbytes: usize) -> i64 {
        buf_get_impl!(be => self, i64, nbytes);
    }

    fn peek_int(&mut self, nbytes: usize) -> i64 {
        buf_peek_impl!(be => self, i64, nbytes);
    }

    fn try_get_int(&mut self, nbytes: usize) -> io::Result<i64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_int(nbytes))
    }
    /// Gets a signed n-byte integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x03\x02\x01 hello"[..];
    /// assert_eq!(0x010203, buf.get_int_le(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_int_le(&mut self, nbytes: usize) -> i64 {
        buf_get_impl!(le => self, i64, nbytes);
    }

    fn peek_int_le(&mut self, nbytes: usize) -> i64 {
        buf_peek_impl!(le => self, i64, nbytes);
    }

    fn try_get_int_le(&mut self, nbytes: usize) -> io::Result<i64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_int_le(nbytes))
    }
    /// Gets a signed n-byte integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03 hello",
    ///     false => b"\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x010203, buf.get_int_ne(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_int_ne(&mut self, nbytes: usize) -> i64 {
        if cfg!(target_endian = "big") {
            self.get_int(nbytes)
        } else {
            self.get_int_le(nbytes)
        }
    }

    fn peek_int_ne(&mut self, nbytes: usize) -> i64 {
        if cfg!(target_endian = "big") {
            self.peek_int(nbytes)
        } else {
            self.peek_int_le(nbytes)
        }
    }

    fn try_get_int_ne(&mut self, nbytes: usize) -> io::Result<i64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_int_ne(nbytes))
    }
    /// Gets an IEEE754 single-precision (4 bytes) floating point number from
    /// `self` in big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x3F\x99\x99\x9A hello"[..];
    /// assert_eq!(1.2f32, buf.get_f32());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f32(&mut self) -> f32 {
        f32::from_bits(Self::get_u32(self))
    }

    fn peek_f32(&mut self) -> f32 {
        f32::from_bits(Self::peek_u32(self))
    }

    fn try_get_f32(&mut self) -> io::Result<f32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_f32())
    }
    /// Gets an IEEE754 single-precision (4 bytes) floating point number from
    /// `self` in little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x9A\x99\x99\x3F hello"[..];
    /// assert_eq!(1.2f32, buf.get_f32_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f32_le(&mut self) -> f32 {
        f32::from_bits(Self::get_u32_le(self))
    }

    fn peek_f32_le(&mut self) -> f32 {
        f32::from_bits(Self::peek_u32_le(self))
    }

    fn try_get_f32_le(&mut self) -> io::Result<f32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_f32_le())
    }
    /// Gets an IEEE754 single-precision (4 bytes) floating point number from
    /// `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x3F\x99\x99\x9A hello",
    ///     false => b"\x9A\x99\x99\x3F hello",
    /// };
    /// assert_eq!(1.2f32, buf.get_f32_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f32_ne(&mut self) -> f32 {
        f32::from_bits(Self::get_u32_ne(self))
    }

    fn peek_f32_ne(&mut self) -> f32 {
        f32::from_bits(Self::peek_u32_ne(self))
    }

    fn try_get_f32_ne(&mut self) -> io::Result<f32> {
        try_advance!(self.remaining() >= 4);
        Ok(self.get_f32_ne())
    }
    /// Gets an IEEE754 double-precision (8 bytes) floating point number from
    /// `self` in big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x3F\xF3\x33\x33\x33\x33\x33\x33 hello"[..];
    /// assert_eq!(1.2f64, buf.get_f64());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f64(&mut self) -> f64 {
        f64::from_bits(Self::get_u64(self))
    }

    fn peek_f64(&mut self) -> f64 {
        f64::from_bits(Self::peek_u64(self))
    }

    fn try_get_f64(&mut self) -> io::Result<f64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_f64())
    }
    /// Gets an IEEE754 double-precision (8 bytes) floating point number from
    /// `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf = &b"\x33\x33\x33\x33\x33\x33\xF3\x3F hello"[..];
    /// assert_eq!(1.2f64, buf.get_f64_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f64_le(&mut self) -> f64 {
        f64::from_bits(Self::get_u64_le(self))
    }

    fn peek_f64_le(&mut self) -> f64 {
        f64::from_bits(Self::peek_u64_le(self))
    }

    fn try_get_f64_le(&mut self) -> io::Result<f64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_f64_le())
    }
    /// Gets an IEEE754 double-precision (8 bytes) floating point number from
    /// `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::buf::Bt;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x3F\xF3\x33\x33\x33\x33\x33\x33 hello",
    ///     false => b"\x33\x33\x33\x33\x33\x33\xF3\x3F hello",
    /// };
    /// assert_eq!(1.2f64, buf.get_f64_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f64_ne(&mut self) -> f64 {
        f64::from_bits(Self::get_u64_ne(self))
    }

    fn peek_f64_ne(&mut self) -> f64 {
        f64::from_bits(Self::peek_u64_ne(self))
    }

    fn try_get_f64_ne(&mut self) -> io::Result<f64> {
        try_advance!(self.remaining() >= 8);
        Ok(self.get_f64_ne())
    }
}

impl Bt for &[u8] {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self
    }

    fn advance_chunk(&mut self, n: usize) -> &[u8] {
        let ret = &self[..n];
        *self = &self[n..];
        ret
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        if self.len() < cnt {
            panic_advance(cnt, self.len());
        }

        *self = &self[cnt..];
    }

    fn into_binary(self) -> Binary {
        Binary::from(self.to_vec())
    }
}

impl<T: AsRef<[u8]>> Bt for std::io::Cursor<T> {
    #[inline]
    fn remaining(&self) -> usize {
        self.get_ref().as_ref().len() - self.position() as usize
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        &self.get_ref().as_ref()[(self.position() as usize)..]
    }

    fn advance_chunk(&mut self, n: usize) -> &[u8] {
        let position = self.position() as usize;
        self.set_position(self.position() + n as u64);
        let ret = &self.get_ref().as_ref()[position..(position + n)];
        ret
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        if self.remaining() < cnt {
            panic_advance(cnt, self.remaining());
        }
        self.set_position(self.position() + cnt as u64);
    }

    fn into_binary(self) -> Binary {
        Binary::from(self.get_ref().as_ref()[(self.position() as usize)..].to_vec())
    }
}
