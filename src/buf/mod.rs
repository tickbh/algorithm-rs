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

// copy a large content from bytes.

mod binary;
mod binary_mut;
mod binary_ref;
mod bt;
mod bt_mut;

pub use binary::Binary;
pub use binary_mut::BinaryMut;
pub use binary_ref::BinaryRef;
pub use bt::Bt;
pub use bt_mut::BtMut;

fn panic_advance(cnt: usize, left: usize) {
    panic!("当前只剩余:{},无法消耗:{}", left, cnt);
}
