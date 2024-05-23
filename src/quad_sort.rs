use std::{alloc::{self, Layout}, fmt::Debug, marker::PhantomData, mem::{self, MaybeUninit}, ops::Range, ptr};
use std::process::id;

struct QuadSort<T, F: Fn(&T, &T) -> bool> {
    is_less: F,
    index: usize,
    block: usize,
    mark: PhantomData<T>,
}

macro_rules! check_less {
    ($src: expr, $s: expr, $e: expr, $func: expr) => {
        $func(&$src[$s], &$src[$e])
        // unsafe { $func($src.get_unchecked($s), $src.get_unchecked($e)) }
    };
    ($src: expr, $s: expr, $dst: expr, $d: expr, $func: expr) => {
        $func(&$src[$s], &$dst[$d])
    };
}


macro_rules! check_big {
    ($src: expr, $s: expr, $e: expr, $func: expr) => {
        !check_less!($src, $s, $e, $func)
    };
    ($src: expr, $s: expr, $dst: expr, $d: expr, $func: expr) => {
        !check_less!($src, $s, $dst, $d, $func)
    };
}

macro_rules! do_set_elem {
    ($src: expr, $dst: expr) => {
        unsafe {
            ptr::copy_nonoverlapping($src, $dst, 1);
        }
    };
    ($src: expr, $dst: expr, $num: expr) => {
        unsafe {
            ptr::copy_nonoverlapping($src, $dst, $num);
        }
    };
}

macro_rules! head_branchless_merge {
    ($dest: expr, $src: expr, $index: expr, $left: expr, $right: expr, $func: expr) => {
        head_branchless_merge!($dest, $index, $src, $left, $src, $right, $func)
    };

    ($dest: expr, $di: expr, $array_a: expr, $ai: expr, $array_b: expr, $bi: expr, $func: expr) => {
        if check_less!($array_a, $ai, $array_b, $bi, $func) {
            do_set_elem!(&mut $array_a[$ai], &mut $dest[$di]);
            $ai += 1;
        } else {
            do_set_elem!(&mut $array_b[$bi], &mut $dest[$di]);
            $bi += 1;
        }
        $di += 1;
    };
}

macro_rules! tail_branchless_merge {
    ($dest: expr, $src: expr, $index: expr, $left: expr, $right: expr, $func: expr) => {
        tail_branchless_merge!($dest, $index, $src, $left, $src, $right, $func)
    };

    ($dest: expr, $di: expr, $array_a: expr, $ai: expr, $array_b: expr, $bi: expr, $func: expr) => {
        if check_big!($array_a, $ai, $array_b, $bi, $func) {
            do_set_elem!(&mut $array_a[$ai], &mut $dest[$di]);
            $ai = ($ai).max(1) - 1;
        } else {
            do_set_elem!(&mut $array_b[$bi], &mut $dest[$di]);
            $bi = ($bi).max(1) - 1;
        }
        $di = $di.max(1) - 1;
    };
}

macro_rules! try_exchange {
    ($src: expr, $func: expr, $start: expr, $end: expr) => {
        if check_big!($src, $start, $end, $func) {
            $src.swap($start, $end);
            true
        } else {
            false
        }
    };
}


#[inline]
pub fn create_swap<T>(caption: usize) -> Vec<T> {

    unsafe {
        // let mut vec = Vec::with_capacity(32);
        // vec.set_len(32);
        // return vec;
        let mem = alloc::alloc(alloc::Layout::array::<T>(caption).unwrap_unchecked()) as *mut T;
        if !mem.is_null() {
            let mut vec = Vec::from_raw_parts(mem, caption, caption);
            vec
        } else {
            let mut vec = Vec::with_capacity(32);
            vec.set_len(32);
            vec
        }
    }
}

impl<T: Debug, F: Fn(&T, &T) -> bool> QuadSort<T, F> {
    pub fn new(is_less: F) -> Self {
        Self { is_less, index: 0, block: 0, mark: PhantomData }
    }


    #[inline]
    pub fn quad_swap_four(&mut self, src: &mut [T])
    {
        try_exchange!(src, &self.is_less, 0, 1);
        try_exchange!(src, &self.is_less, 2, 3);
        // 中间顺序正确则表示排序完毕
        if try_exchange!(src, &self.is_less, 1, 2) {
            try_exchange!(src, &self.is_less, 0, 1);
            if try_exchange!(src, &self.is_less, 2, 3) {
                try_exchange!(src, &self.is_less, 1, 2);
            }
        }
    }

    #[inline]
    pub fn parity_swap_thirty_two(&mut self, src: &mut [T], swap: &mut [T])
    {
        for i in 0..4 {
            self.parity_swap_eight(&mut src[i * 8..], &mut swap[i * 8..]);
        }
        // if is_less(&src[7], &src[8]) && is_less(&src[15], &src[16]) && is_less(&src[23], &src[24]) {
        //     return;
        // }

        self.parity_merge(swap, src, 8, 8);
        self.parity_merge(&mut swap[16..], &mut src[16..], 8, 8);
        self.parity_merge(src, swap, 16, 16);
    }

    #[inline]
    pub fn quad_swap(&mut self, src: &mut [T], swap: &mut [T])
    {
        let len = src.len();
        let count = len / 32;
        for i in 0..count {
            self.parity_swap_thirty_two(&mut src[i * 32..], swap);
        }
        let left = len % 32;
        if left > 0 {
            self.tail_swap(&mut src[len - left..], swap);
        }
    }

    #[inline]
    pub fn parity_merge(&mut self, dest: &mut [T], from: &mut [T], mut left: usize, mut right: usize)
    where
        F: Fn(&T, &T) -> bool
    {
        let mut dr = left + right - 1;
        if check_less!(from, left-1, left, self.is_less) {
            do_set_elem!(&mut from[0], &mut dest[0], left + right);
            return;
        } else if check_big!(from, 0, dr, self.is_less) {
            do_set_elem!(&mut from[left], &mut dest[0], right);
            do_set_elem!(&mut from[0], &mut dest[right], left);
            return;
        }
        let mut ll = 0;
        let mut lr = ll + left;
        let mut dl = 0;

        let mut rl = lr - 1;
        let mut rr = rl + right;

        let is_less = &self.is_less;

        macro_rules! compare_to_next {
            (true) => {
                // println!("left 起始位置:{} {:?}, 结束位置:{} {:?} 比较大小:{}", ll, &from[ll], lr, &from[lr], is_less(&from[ll], &from[lr]));
                if check_less!(from, ll, lr, is_less) {
                    do_set_elem!(&mut from[ll], &mut dest[dl]);
                    ll += 1;
                } else {
                    do_set_elem!(&mut from[lr], &mut dest[dl]);
                    lr += 1;
                }
                dl += 1;
            };
            (false) => {
                // println!("right 起始位置:{} {:?}, 结束位置:{} {:?} 比较大小:{}/{}", rl, &from[rl], rr, &from[rr], dr, is_less(&from[rl], &from[rr]));

                if !is_less(&from[rl], &from[rr]) {
                    do_set_elem!(&mut from[rl], &mut dest[dr]);
                    rl-=1;
                } else {
                    do_set_elem!(&mut from[rr], &mut dest[dr]);
                    rr-=1;
                }
                dr -= 1;
            };
        }
        
        if left < right {
            compare_to_next!(true);
        }
        // println!("left = {}, right = {} dr = {}, rr = {}, rl = {}", left, right, dr, self.rr, self.rl);
        // while left > 0 {
        //     // println!("in repeat left = {}, right = {} dr = {}, rr = {}, rl = {}", left, right, dr, self.rr, self.rl);
        //     // if self.rl == 0 {
        //     //     println!("src = {:?}, swap = {:?}", dest, from);
        //     // }
        //     compare_to_next!(true);
        //     compare_to_next!(false);
        //     left -= 1;
        // }
        for _ in 0..left {

            compare_to_next!(true);
            compare_to_next!(false);
        }
    }

    #[inline]
    pub fn cross_merge(&mut self, dest: &mut [T], from: &mut [T], left: usize, right: usize)
    where
        F: Fn(&T, &T) -> bool
    {
        let mut ll = 0;
        let mut lr = ll + left;

        let mut rl = lr - 1;
        let mut rr = rl + right;

        // if left + 1 >= right && right + 1 >= left && left >= 32
        // {
            // if (cmp(ptl + 15, ptr) > 0 && cmp(ptl, ptr + 15) <= 0 && cmp(tpl, tpr - 15) > 0 && cmp(tpl - 15, tpr) <= 0)
            // {
            // 	parity_merge(dest, from, left, right, cmp);
            // 	return;
            // }
        // }

        let mut dl = 0;
        let mut dr = left + right - 1;
        
        macro_rules! compare_to_next {
            (true) => {
                // println!("left 起始位置:{} {:?}, 结束位置:{} {:?} 比较大小:{}", ll, &from[ll], lr, &from[lr], is_less(&from[ll], &from[lr]));
                if !check_big!(from, ll, lr, self.is_less) {
                    do_set_elem!(&mut from[ll], &mut dest[dl]);
                    ll += 1;
                } else {
                    do_set_elem!(&mut from[lr], &mut dest[dl]);
                    lr += 1;
                }
                dl += 1;
            };
            (false) => {
                if !(self.is_less)(&from[rl], &from[rr]) {
                    do_set_elem!(&mut from[rl], &mut dest[dr]);
                    rl -= 1;
                } else {
                    do_set_elem!(&mut from[rr], &mut dest[dr]);
                    rr -= 1;
                }
                dr -= 1;
            };
        }

        'outer: while rl.saturating_sub(ll) > 8 && rr.saturating_sub(lr) > 8 {
            while check_less!(from, ll + 7, lr, self.is_less) {
                do_set_elem!(&mut from[ll], &mut dest[dl], 8);
                dl += 8;
                ll += 8;
                if rl - ll <= 8 {
                    break 'outer;
                }
            }

            while check_big!(from, ll, lr + 7, self.is_less) {
                do_set_elem!(&mut from[lr], &mut dest[dl], 8);
                dl += 8;
                lr += 8;
                if rr - lr <= 8 {
                    break 'outer;
                }
            }
            
            while check_less!(from, rl, rr - 7, self.is_less) {
                dr -= 8;
                rr -= 8;
                do_set_elem!(&mut from[rr + 1], &mut dest[dr + 1], 8);
                if rr - lr <= 8 {
                    break 'outer;
                }
            }

            
            while check_big!(from, rl - 7, rr, self.is_less) {
                dr -= 8;
                rl -= 8;
                do_set_elem!(&mut from[rl + 1], &mut dest[dr + 1], 8);
                if rl - ll <= 8 {
                    break 'outer;
                }
            }

            for _ in 0..8 {
                compare_to_next!(true);
                compare_to_next!(false);
            }
        }

        if (self.is_less)(&from[rl], &from[rr])  {
            while ll <= rl {
                compare_to_next!(true);
            }
            while lr <= rr {
                do_set_elem!(&mut from[lr], &mut dest[dl]);
                lr += 1;
                dl += 1;
            }
        } else {
            while lr <= rr {
                compare_to_next!(true);
            }
            while ll <= rl {
                do_set_elem!(&mut from[ll], &mut dest[dl]);
                ll += 1;
                dl += 1;
            }
        }
    }

    #[inline]
    pub fn partial_backward_merge(&mut self, src: &mut [T], swap: &mut [T], block: usize)
    where
        F: Fn(&T, &T) -> bool
    {
        if src.len() <= block {
            return;
        }
        let mut ll = 0;
        let mut rl = block;
        if check_less!(src, rl - 1, rl, self.is_less) {
            return;
        }

        let mut index = 0;
        let len = src.len();
        while ll < block && rl < len {
            head_branchless_merge!(swap, src, index, ll, rl, self.is_less);
        }

        if ll < block {
            do_set_elem!(&mut src[ll], &mut swap[index], block - ll);
        } else if rl < len {
            do_set_elem!(&mut src[rl], &mut swap[index], len - rl);
        }

        do_set_elem!(&mut swap[0], &mut src[0], len);

        //
        // let mut ll = block - 1;
        // let mut la = src.len() - 1;
        // if (self.is_less)(&src[ll], &src[ll + 1]) {
        //     return;
        // }
        //
        // let mut lr = la - ll;
        // if src.len() <= swap.len() && lr > 64 {
        //     self.cross_merge(swap, src, block, lr);
        //     unsafe {
        //         ptr::copy_nonoverlapping(&mut src[0], &mut swap[0], src.len());
        //     }
        //     return;
        // }
        // unsafe {
        //     ptr::copy_nonoverlapping(&mut swap[0], &mut src[block], lr);
        // }
        // lr -= 1;
        // while ll > 16 && lr > 16 {
        //     while check_less!(src, ll, lr - 15, self.is_less) {
        //         for _ in 0..16 {
        //             do_set_elem!(&mut src[la], &mut swap[lr]);
        //             la -= 1;
        //             lr -= 1;
        //         }
        //         if lr <= 16 {
        //             break;
        //         }
        //     }
        //
        //     while !check_less!(src, ll - 15, lr, self.is_less) {
        //         for _ in 0..16 {
        //             unsafe {
        //                 ptr::copy_nonoverlapping(&mut src[la], &mut src[ll], 1);
        //             }
        //             la -= 1;
        //             lr -= 1;
        //         }
        //         if ll <= 16 {
        //             break;
        //         }
        //     }
        //
        //     for _ in 0..8 {
        //         if check_less!(src, ll, lr - 1, self.is_less) {
        //             for _ in 0..2 {
        //                 unsafe {
        //                     ptr::copy_nonoverlapping(&mut src[la], &mut src[lr], 1);
        //                     la -= 1;
        //                     lr -= 1;
        //                 }
        //             }
        //         } else if !check_less!(src, ll - 1,  lr, self.is_less) {
        //             for _ in 0..2 {
        //                 unsafe {
        //                     ptr::copy_nonoverlapping(&mut src[la], &mut src[ll], 1);
        //                     la -= 1;
        //                     ll -= 1;
        //                 }
        //             }
        //         } else {
        //             if check_less!(src, ll, lr, self.is_less) {
        //                 unsafe {
        //                     ptr::copy_nonoverlapping(&mut src[la], &mut src[lr], 1);
        //                     ptr::copy_nonoverlapping(&mut src[la-1], &mut src[ll], 1);
        //                     la -= 2;
        //                     ll -= 1;
        //                     lr -= 1;
        //                 }
        //             } else {
        //                 unsafe {
        //                     ptr::copy_nonoverlapping(&mut src[la], &mut src[ll], 1);
        //                     ptr::copy_nonoverlapping(&mut src[la-1], &mut src[lr], 1);
        //                     la -= 2;
        //                     ll -= 1;
        //                     lr -= 1;
        //                 }
        //             }
        //
        //             tail_branchless_merge!(src, la, src, ll, swap, lr, self.is_less);
        //         }
        //     }
        // }
        // while lr > 1 && ll > 1 {
        //     if check_less!(src, ll, swap, lr - 1, self.is_less) {
        //         do_set_elem!(&mut swap[lr-1], &mut src[la-1], 2);
        //         la -= 2;
        //         lr -= 2;
        //     } else if check_less!(src, ll - 1, swap, lr, self.is_less) {
        //         do_set_elem!(&mut src[ll-1], &mut src[la-1], 2);
        //         la -= 2;
        //         ll -= 2;
        //     } else {
        //         if check_less!(src, ll, swap, lr, self.is_less) {
        //             do_set_elem!(&mut swap[lr], &mut src[la]);
        //             do_set_elem!(&mut src[ll], &mut src[la - 1]);
        //         } else {
        //             do_set_elem!(&mut swap[lr], &mut src[la - 1]);
        //             do_set_elem!(&mut src[ll], &mut src[la]);
        //         }
        //         la -= 2;
        //         lr -= 1;
        //         ll -= 1;
        //         tail_branchless_merge!(src, la, src, ll, swap, lr, self.is_less);
        //     }
        // }
        // while lr > 0 && ll > 0 {
        //     if check_less!(src, ll, swap, lr, self.is_less) {
        //         do_set_elem!(&mut swap[lr], &mut src[la]);
        //         lr -= 1;
        //     } else {
        //         do_set_elem!(&mut src[ll], &mut src[la]);
        //         ll -= 1;
        //     }
        //     la -= 1;
        // }
        // if lr > 0 {
        //     do_set_elem!(&mut swap[0], &mut src[la - lr], lr);
        // }
        // // // todo!()
    }

    #[inline]
    pub fn tail_merge(&mut self, src: &mut [T], swap: &mut [T], mut block: usize)
    where
        F: Fn(&T, &T) -> bool
    {
        let len = src.len();
        let swap_len = swap.len();
        while block < len && block < swap_len {
            for idx in (0..len).step_by(block * 2) {
                if idx + block * 2 < len {
                    self.partial_backward_merge(&mut src[idx..idx+block * 2], swap, block);
                    continue;
                }
                self.partial_backward_merge(&mut src[idx..], swap, block);
                break;
            }

            block *= 2;
        }
    }

    #[inline]
    pub fn quad_merge_block(&mut self, src: &mut [T], swap: &mut [T], block: usize)
    where
        F: Fn(&T, &T) -> bool
    {
        let block1 = block;
        let block2 = block1 + block;
        let block3 = block2 + block;
        match ((self.is_less)(&src[block1 - 1], &src[block1]), (self.is_less)(&src[block3 - 1], &src[block3])) {
            (true, true) => {
                if (self.is_less)(&src[block2 - 1], &src[block2]) {
                    return;
                }
                unsafe {
                    ptr::copy_nonoverlapping(&mut swap[0], &mut src[0], block * 4);
                }
            },
            (false, true) => {
                self.cross_merge(swap, src, block, block);
                unsafe {
                    ptr::copy_nonoverlapping(&mut swap[block2], &mut src[block2], block2);
                }
            },
            (true, false) => {
                unsafe {
                    ptr::copy_nonoverlapping(&mut swap[0], &mut src[0], block2);
                }
                self.cross_merge(&mut swap[block2..], &mut src[block2..], block, block);
            },
            (false, false) => {
                self.cross_merge(swap, src, block, block);
                self.cross_merge(&mut swap[block2..], &mut src[block2..], block, block);
            },
        }
        self.cross_merge(src, swap, block2, block2);
    }


    #[inline]
    pub fn quad_merge(&mut self, src: &mut [T], swap: &mut [T], mut block: usize) -> usize
    {
        let len = src.len();
        let swap_len = swap.len();
        block *= 4;
        while block < len && block < swap_len {
            self.index = 0;
            loop {
                self.quad_merge_block(&mut src[self.index..], swap, block / 4);
                self.index += block;
                if self.index + block > len {
                    break;
                }
            }
            self.tail_merge(&mut src[self.index..], swap, block / 4);
            block *= 4;
        }
        self.tail_merge(src, swap, block / 4);
        block / 2
    }

    #[inline]
    pub fn monobound_binary_first(&mut self, src: &mut [T], right: usize, left: usize, mut top: usize) -> usize
    {
        let mut end = right + top;
        while top > 1 {
            let mid = top / 2;
            if check_less!(src, left, end - mid, self.is_less) {
                end -= mid;
            }
            top -= mid;
        }

        if check_less!(src, left, end - 1, self.is_less) {
            end -= 1;
        }
        return end - left
    }

    #[inline]
    pub fn rotate_merge_block(&mut self, src: &mut [T], swap: &mut [T], mut lblock: usize, mut right: usize)
    {
        if check_less!(src, lblock - 1, lblock, self.is_less) {
            return;
        }
        let mut rblock = lblock / 2;
        lblock -= rblock;
        let left = self.monobound_binary_first(src, lblock + rblock, lblock, right);
        right -= left;
        
        if left > 0 {
            if lblock + left < swap.len() {
                
            }
        }
        // [ lblock ] [ rblock ] [ left ] [ right ]
    }

    #[inline]
    pub fn rotate_merge(&mut self, src: &mut [T], swap: &mut [T], mut block: usize)
    {
        let len = src.len();
        if len <= block * 2 && len > block && len - block <= swap.len() {
            self.partial_backward_merge(src, swap, block);
            return;
        }
        // while block < len {
        //     for i in (0..len).step_by(block * 2) {
        //         if i + block * 2 < len {
        //             rotate_merge_block(&mut src[i..], swap, block, block, is_less);
        //             continue;
        //         }
        //         rotate_merge_block(&mut src[i..], swap, block, len - i - block, is_less);
        //         break;
        //     }
        // 	block *= 2;
        // }
    }

    #[inline]
    pub fn parity_merge_two(&mut self, src: &mut [T], swap: &mut [T])
    where
        F: Fn(&T, &T) -> bool
    {
        if check_less!(src, 1, 2, self.is_less) {
            do_set_elem!(&mut src[0], &mut swap[0], 4);
        } else if check_big!(src, 0, 3, self.is_less) {
            do_set_elem!(&mut src[0], &mut swap[2], 2);
            do_set_elem!(&mut src[2], &mut swap[0], 2);
        } else {
            match check_less!(src, 1, 3, self.is_less) {
                true => {
                    do_set_elem!(&mut src[3], &mut swap[3]);
                    do_set_elem!(&mut src[1], &mut swap[2]);
                }
                false => {
                    do_set_elem!(&mut src[1], &mut swap[3]);
                    do_set_elem!(&mut src[3], &mut swap[2]);
                }
            }

            match check_less!(src, 0, 2, self.is_less) {
                true => {
                    do_set_elem!(&mut src[2], &mut swap[1]);
                    do_set_elem!(&mut src[0], &mut swap[0]);
                }
                false => {
                    do_set_elem!(&mut src[0], &mut swap[1]);
                    do_set_elem!(&mut src[2], &mut swap[0]);
                }
            }
        }
        // let mut index = 0;
        // (self.ll, self.lr) = (0, 2);
        // for _ in 0..2 {
        //     head_branchless_merge!(swap, src, index, self.ll, self.lr, self.is_less);
        // }
        // index = 3;
        // (self.ll, self.lr) = (1, 3);
        // for _ in 0..2 {
        //     tail_branchless_merge!(swap, src, index, self.ll, self.lr, self.is_less);
        // }
        // println!("parity_merge_two src = {:?}", &src[..4]);
        // println!("parity_merge_two swap = {:?}", &swap[..4]);

    }

    #[inline]
    pub fn parity_merge_four(&mut self, src: &mut [T], swap: &mut [T])
    where
        F: Fn(&T, &T) -> bool
    {
        if check_less!(src, 3, 4, self.is_less) {
            do_set_elem!(&mut src[0], &mut swap[0], 8);
            return;
        }
        self.index = 0;
        // (self.ll, self.lr) = (0, 4);
        // while true {
        //     if check_less!(src, self.ll, self.lr, self.is_less) {
        //         do_set_elem!(&mut src[self.ll], &mut swap[self.index]);
        //         self.index += 1;
        //         self.ll += 1;
        //         if self.ll >= 4 {
        //             break;
        //         }
        //     } else {
        //         do_set_elem!(&mut src[self.lr], &mut swap[self.index]);
        //         self.index += 1;
        //         self.lr += 1;
        //         if self.lr >= 8 {
        //             break;
        //         }
        //     }
        // }

        // if self.ll < 4 {
        //     do_set_elem!(&mut src[self.ll], &mut swap[self.index], 4 - self.ll);
        // } else if self.lr < 8 {
        //     do_set_elem!(&mut src[self.lr], &mut swap[self.index], 8 - self.lr);
        // }

        let (mut ll, mut lr) = (0, 4);
        for _ in 0..4 {
            head_branchless_merge!(swap, src, self.index, ll, lr, self.is_less);
        }
        self.index = 7;
        (ll, lr) = (3, 7);
        for _ in 0..4 {
            tail_branchless_merge!(swap, src, self.index, ll, lr, self.is_less);
        }
    }

    #[inline]
    pub fn parity_swap_eight(&mut self, src: &mut [T], swap: &mut [T])
    {
        // self.quad_swap_four(src);
        // self.quad_swap_four(&mut src[4..]);
        // do_set_elem!(&mut src[0], &mut swap[0], 8);

        for i in 0..4 {
            try_exchange!(src, &self.is_less, i * 2, i * 2 + 1);
        }
        // if (self.is_less)(&src[1], &src[2]) && (self.is_less)(&src[3], &src[4]) && (self.is_less)(&src[5], &src[6]) {
        //     return;
        // }
        
        self.parity_merge_two(src, swap);
        self.parity_merge_two(&mut src[4..], &mut swap[4..]);

        self.parity_merge_four(swap, src);
    }

    #[inline]
    pub fn parity_swap_sixteen(&mut self, src: &mut [T], swap: &mut [T])
    where
        F: Fn(&T, &T) -> bool
    {
        for i in 0..4 {
            self.quad_swap_four(&mut src[i * 4..]);
        }
        // if is_less(&src[3], &src[4]) && is_less(&src[7], &src[8]) && is_less(&src[11], &src[12]) {
        //     return;
        // }

        self.parity_merge_four(src, swap);
        self.parity_merge_four(&mut src[8..], &mut swap[8..]);
        self.parity_merge(src, swap, 8, 8);
    }

    #[inline]
    pub fn tiny_sort(&mut self, src: &mut [T])
    {
        match src.len() {
            4 => {
                self.quad_swap_four(src);
            }
            3 => {
                try_exchange!(src, &self.is_less, 0, 1);
                if try_exchange!(src, &self.is_less, 1, 2) {
                    try_exchange!(src, &self.is_less, 0, 1);
                }
            }
            2 => {
                try_exchange!(src, &self.is_less, 0, 1);
            }
            _ => {
                return
            }
        }
    }

    #[inline]
    pub fn twice_unguarded_insert(&mut self, src: &mut [T], swap: &mut [T], offset: usize)
    where
        F: Fn(&T, &T) -> bool
    {
        // self.index = 0;
        // for idx in offset..src.len() {
        //     self.index = idx;
        //     for j in 0..idx {
        //         if (self.is_less)(&src[idx], &src[j]) {
        //             self.index = j;
        //             break;
        //         }
        //     }
        //     if self.index == idx {
        //         continue;
        //     }
        //     do_set_elem!(&mut src[self.index], &mut swap[1], idx - self.index);
        //     do_set_elem!(&mut src[idx], &mut swap[0]);
        //     do_set_elem!(&mut swap[0], &mut src[self.index], idx - self.index + 1);
        // }
        let mut index = 0;
        for idx in offset..src.len() {
            index = idx;
            for j in 0..idx {
                if (self.is_less)(&src[idx], &src[j]) {
                    index = j;
                    break;
                }
            }
            if index == idx {
                continue;
            }
            do_set_elem!(&mut src[index], &mut swap[1], idx - index);
            do_set_elem!(&mut src[idx], &mut swap[0]);
            do_set_elem!(&mut swap[0], &mut src[index], idx - index + 1);
        }
        // let mut index = 0;
        // for idx in offset..src.len() {
        //     if !try_exchange!(src, self.is_less, idx - 1, idx) {
        //         continue;
        //     }
            
        //     for j in (0..idx - 1).rev() {
        //         if (self.is_less)(&src[idx - 1], &src[j]) {
        //             index = j;
        //             break;
        //         }
        //         if !try_exchange!(src, self.is_less, j, j+1) {
        //             break;
        //         }
        //     }
        //     if (self.is_less)(&src[idx - 1], &src[0]) {
        //         for j in (0..idx - 1).rev() {
        //             src.swap(j+1, j)
        //         }
        //     } else {
        //         for j in (0..idx - 1).rev() {
        //             if !try_exchange!(src, self.is_less, j, j+1) {
        //                 break;
        //             }
        //         }
        //     }
        // }
    }

    #[inline]
    pub fn less_24_tail_swap(&mut self, src: &mut [T], swap: &mut [T]) {
        match src.len() {
            l if l < 5 => {
                self.tiny_sort(src);
                return;
            }
            l if l < 8 => {
                self.quad_swap_four(src);
                self.twice_unguarded_insert(src, swap, 4);
                return;
            }
            l if l < 12 => {
                self.parity_swap_eight(src, swap);
                self.twice_unguarded_insert(src, swap, 8);
                return;
            }
            l if l < 16 => {
                self.parity_swap_eight(src, swap);
                self.less_24_tail_swap(&mut src[8..], swap);
                self.partial_backward_merge(src, swap, 8);
                return;
            }
            l if l >= 16 && l < 24 => {
                self.parity_swap_sixteen(src, swap);
                self.twice_unguarded_insert(src, swap, 16);
                return;
            }
            _ => {
                unreachable!()
            }
        }
    }

    #[inline]
    pub fn tail_swap(&mut self, src: &mut [T], swap: &mut [T])
    {
        if src.len() < 24 {
            self.less_24_tail_swap(src, swap);
            return;
        }
        let half1 = src.len() / 2;
        let quad1 = half1 / 2;
        let quad2 = half1 - quad1;

        let half2 = src.len() - half1;
        let quad3 = half2 / 2;
        let quad4 = half2 - quad3;
        
        let mut index = 0;
        self.less_24_tail_swap(&mut src[index..index + quad1], swap);
        index += quad1;
        self.less_24_tail_swap(&mut src[index..index + quad2], swap);
        index += quad2;
        self.less_24_tail_swap(&mut src[index..index + quad3], swap);
        index += quad3;
        self.less_24_tail_swap(&mut src[index..index + quad4], swap);

        // if (self.is_less)(&src[quad1 - 1], &src[quad1]) 
        // && (self.is_less)(&src[half1 - 1], &src[half1]) 
        // && (self.is_less)(&src[index - 1], &src[index]) {
        //     return;
        // }

        self.parity_merge(swap, src, quad1, quad2);
        self.parity_merge(&mut swap[half1..], &mut src[half1..], quad3, quad4);
        self.parity_merge(src, swap, half1, half2);
    }

    #[inline]
    pub fn quad_sort_order_by(&mut self, src: &mut [T])
    {
        match src.len() {
            l if l < 32 => {
                let mut swap = create_swap(l);
                self.tail_swap(src,  &mut swap);
            }
            _ => {
                let mut swap = create_swap(src.len());
                self.quad_swap(src, &mut swap);
                // if swap.len() != src.len() {
                //     tail_merge(src, &mut swap[..32], 32, &is_less);
                //     rotate_merge(src, &mut swap[..32], 64, &is_less);
                //     return;
                // }
                let block = self.quad_merge(src, &mut swap, 32);
                self.rotate_merge(src, &mut swap, block);
                // Vec::from_raw_parts(ptr, length, capacity)
                // Vec::with_capacity(capacity)
            }
        }

        // recurse(v, &mut is_less, None, limit);
    }

}



#[inline]
pub fn quad_sort_order_by<T, F>(src: &mut [T], is_less: F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug
{
    let mut quad = QuadSort::new(is_less);
    quad.quad_sort_order_by(src);
}

#[inline]
pub fn quad_sort<T>(src: &mut [T])
where
    T: Debug + Ord
{
    quad_sort_order_by(src, T::lt);
}
