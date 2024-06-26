use std::{alloc::{self}, cmp::Ordering, fmt::Debug, ptr};


macro_rules! check_less {
    ($src: expr, $s: expr, $e: expr, $func: expr) => {
        $func(&$src[$s], &$src[$e])
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
pub fn quad_swap_four<T, F>(src: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    try_exchange!(src, &is_less, 0, 1);
    try_exchange!(src, &is_less, 2, 3);
    // 中间顺序正确则表示排序完毕
    if try_exchange!(src, &is_less, 1, 2) {
        try_exchange!(src, &is_less, 0, 1);
        if try_exchange!(src, &is_less, 2, 3) {
            try_exchange!(src, &is_less, 1, 2);
        }
    }
}

#[inline]
pub fn parity_swap_thirty_two<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    for i in 0..4 {
        parity_swap_eight(&mut src[i * 8..], &mut swap[i * 8..], &is_less);
    }
    if is_less(&src[7], &src[8]) && is_less(&src[15], &src[16]) && is_less(&src[23], &src[24]) {
        return;
    }

    parity_merge(swap, src, 8, 8, is_less);
    parity_merge(&mut swap[16..], &mut src[16..], 8, 8, is_less);
    parity_merge(src, swap, 16, 16, is_less);
}

#[inline]
pub fn quad_swap<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    let len = src.len();
    let count = len / 32;
    for i in 0..count {
        parity_swap_thirty_two(&mut src[i * 32..], swap, &is_less);
    }
    let left = len % 32;
    if left > 0 {
        tail_swap(&mut src[len - left..], swap, is_less);
    }
}

#[inline]
pub fn parity_merge<T, F>(dest: &mut [T], from: &mut [T], left: usize, right: usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    let mut dr = left + right - 1;
    if check_less!(from, left-1, left, is_less) {
        do_set_elem!(&mut from[0], &mut dest[0], left + right);
        return;
    } else if check_big!(from, 0, dr, is_less) {
        do_set_elem!(&mut from[left], &mut dest[0], right);
        do_set_elem!(&mut from[0], &mut dest[right], left);
        return;
    }
    let mut ll = 0;
    let mut lr = ll + left;
    let mut dl = 0;

    let mut rl = lr - 1;
    let mut rr = rl + right;

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
pub fn cross_merge<T, F>(dest: &mut [T], from: &mut [T], left: usize, right: usize, is_less: &F)
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
            if !check_big!(from, ll, lr, is_less) {
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

                if !(is_less)(&from[rl], &from[rr]) {
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
            while check_less!(from, ll + 7, lr, is_less) {
                do_set_elem!(&mut from[ll], &mut dest[dl], 8);
                dl += 8;
                ll += 8;
                if rl - ll <= 8 {
                    break 'outer;
                }
            }

            while check_big!(from, ll, lr + 7, is_less) {
                do_set_elem!(&mut from[lr], &mut dest[dl], 8);
                dl += 8;
                lr += 8;
                if rr - lr <= 8 {
                    break 'outer;
                }
            }
            
            while check_less!(from, rl, rr - 7, is_less) {
                dr -= 8;
                rr -= 8;
                do_set_elem!(&mut from[rr + 1], &mut dest[dr + 1], 8);
                if rr - lr <= 8 {
                    break 'outer;
                }
            }

            
            while check_big!(from, rl - 7, rr, is_less) {
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

    if is_less(&from[rl], &from[rr])  {
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
pub fn partial_backward_merge<T, F>(src: &mut [T], swap: &mut [T], block: usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    if src.len() <= block {
        return;
    }
    let mut ll = 0;
    let mut rl = block;
    if check_less!(src, rl - 1, rl, is_less) {
        return;
    }

    let mut index = 0;
    let len = src.len();
    while ll < block && rl < len {
        head_branchless_merge!(swap, src, index, ll, rl, is_less);
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
    // if is_less(&src[ll], &src[ll + 1]) {
    //     return;
    // }
    //
    // let mut lr = la - ll;
    // if src.len() <= swap.len() && lr > 64 {
    //     cross_merge(swap, src, block, lr, is_less);
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
    //     while is_less(&src[ll], &src[lr - 15]) {
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
    //     while !is_less(&src[ll - 15], &src[lr]) {
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
    //         if is_less(&src[ll], &src[lr - 1]) {
    //             for _ in 0..2 {
    //                 unsafe {
    //                     ptr::copy_nonoverlapping(&mut src[la], &mut src[lr], 1);
    //                     la -= 1;
    //                     lr -= 1;
    //                 }
    //             }
    //         } else if !is_less(&src[ll - 1], &src[lr]) {
    //             for _ in 0..2 {
    //                 unsafe {
    //                     ptr::copy_nonoverlapping(&mut src[la], &mut src[ll], 1);
    //                     la -= 1;
    //                     ll -= 1;
    //                 }
    //             }
    //         } else {
    //             if is_less(&src[ll], &src[lr]) {
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
    //             tail_branchless_merge!(src, la, src, &mut ll, swap, &mut lr, is_less);
    //         }
    //     }
    // }
    // // todo!()
}

#[inline]
pub fn tail_merge<T, F>(src: &mut [T], swap: &mut [T], mut block: usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    let len = src.len();
    let swap_len = swap.len();
    while block < len && block < swap_len {
        for idx in (0..len).step_by(block * 2) {
            if idx + block * 2 < len {
                partial_backward_merge(&mut src[idx..idx+block * 2], swap, block, is_less);
                continue;
            }
            partial_backward_merge(&mut src[idx..], swap, block, is_less);
            break;
        }

        block *= 2;
    }
}

#[inline]
pub fn quad_merge_block<T, F>(src: &mut [T], swap: &mut [T], block: usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    let block1 = block;
    let block2 = block1 + block;
    let block3 = block2 + block;
    match (is_less(&src[block1 - 1], &src[block1]), is_less(&src[block3 - 1], &src[block3])) {
        (true, true) => {
            if is_less(&src[block2 - 1], &src[block2]) {
                return;
            }
            unsafe {
                ptr::copy_nonoverlapping(&mut swap[0], &mut src[0], block * 4);
            }
        },
        (false, true) => {
            cross_merge(swap, src, block, block, is_less);
            unsafe {
                ptr::copy_nonoverlapping(&mut swap[block2], &mut src[block2], block2);
            }
        },
        (true, false) => {
            unsafe {
                ptr::copy_nonoverlapping(&mut swap[0], &mut src[0], block2);
            }
            cross_merge(&mut swap[block2..], &mut src[block2..], block, block, is_less);
        },
        (false, false) => {
            cross_merge(swap, src, block, block, is_less);
            cross_merge(&mut swap[block2..], &mut src[block2..], block, block, is_less);
        },
    }
    cross_merge(src, swap, block2, block2, is_less);
}


#[inline]
pub fn quad_merge<T, F>(src: &mut [T], swap: &mut [T], mut block: usize, is_less: &F) -> usize
where
    F: Fn(&T, &T) -> bool
{
    let len = src.len();
    let swap_len = swap.len();
    let mut index;
    block *= 4;
    while block < len && block < swap_len {
        index = 0;
        loop {
            quad_merge_block(&mut src[index..], swap, block / 4, is_less);
            index += block;
            if index + block > len {
                break;
            }
        }
        tail_merge(&mut src[index..], swap, block / 4, is_less);
		block *= 4;
    }
    tail_merge(src, swap, block / 4, is_less);
    block / 2
}

#[inline]
pub fn monobound_binary_first<T, F>(src: &mut [T], right: usize, left: usize, mut top: usize, is_less: &F) -> usize
where
    F: Fn(&T, &T) -> bool
{
    let mut end = right + top;
    while top > 1 {
        let mid = top / 2;
        if check_less!(src, left, end - mid, is_less) {
            end -= mid;
        }
        top -= mid;
    }

    if check_less!(src, left, end - 1, is_less) {
        end -= 1;
    }
    return end - left
}

#[inline]
pub fn rotate_merge_block<T, F>(src: &mut [T], swap: &mut [T], mut lblock: usize, right: usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    if check_less!(src, lblock - 1, lblock, is_less) {
        return;
    }
    let rblock = lblock / 2;
    lblock -= rblock;
    let left = monobound_binary_first(src, lblock + rblock, lblock, right, is_less);
    // right -= left;
    
    if left > 0 {
        if lblock + left < swap.len() {
            
        }
    }
	// [ lblock ] [ rblock ] [ left ] [ right ]
}

#[inline]
pub fn rotate_merge<T, F>(src: &mut [T], swap: &mut [T], block: usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    let len = src.len();
    if len <= block * 2 && len > block && len - block <= swap.len() {
        partial_backward_merge(src, swap, block, is_less);
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
pub fn parity_merge_two<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    if check_less!(src, 1, 2, is_less) {
        do_set_elem!(&mut src[0], &mut swap[0], 4);
    } else if check_big!(src, 0, 3, is_less) {
        do_set_elem!(&mut src[0], &mut swap[2], 2);
        do_set_elem!(&mut src[2], &mut swap[0], 2);
    } else {
        match check_less!(src, 1, 3, is_less) {
            true => {
                do_set_elem!(&mut src[3], &mut swap[3]);
                do_set_elem!(&mut src[1], &mut swap[2]);
            }
            false => {
                do_set_elem!(&mut src[1], &mut swap[3]);
                do_set_elem!(&mut src[3], &mut swap[2]);
            }
        }

        match check_less!(src, 0, 2, is_less) {
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
    // (*left, *right) = (0, 2);
    // for _ in 0..2 {
    //     head_branchless_merge!(swap, src, index, left, right, is_less);
    // }
    // index = 3;
    // (*left, *right) = (1, 3);
    // for _ in 0..2 {
    //     tail_branchless_merge!(swap, src, index, left, right, is_less);
    // }
    // println!("parity_merge_two src = {:?}", &src[..4]);
    // println!("parity_merge_two swap = {:?}", &swap[..4]);

}

#[inline]
pub fn parity_merge_four<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
        if check_less!(src, 3, 4, is_less) {
            do_set_elem!(&mut src[0], &mut swap[0], 8);
            return;
        }
        let mut index = 0;
        // (self.ll, self.lr) = (0, 4);
        // while true {
        //     if check_less!(src, self.ll, self.lr, self.is_less) {
        //         do_set_elem!(&mut src[self.ll], &mut swap[index]);
        //         index += 1;
        //         self.ll += 1;
        //         if self.ll >= 4 {
        //             break;
        //         }
        //     } else {
        //         do_set_elem!(&mut src[self.lr], &mut swap[index]);
        //         index += 1;
        //         self.lr += 1;
        //         if self.lr >= 8 {
        //             break;
        //         }
        //     }
        // }

        // if self.ll < 4 {
        //     do_set_elem!(&mut src[self.ll], &mut swap[index], 4 - self.ll);
        // } else if self.lr < 8 {
        //     do_set_elem!(&mut src[self.lr], &mut swap[index], 8 - self.lr);
        // }

        let (mut ll, mut lr) = (0, 4);
        for _ in 0..4 {
            head_branchless_merge!(swap, src, index, ll, lr, is_less);
        }
        index = 7;
        (ll, lr) = (3, 7);
        for _ in 0..4 {
            tail_branchless_merge!(swap, src, index, ll, lr, is_less);
        }
}

#[inline]
pub fn parity_swap_eight<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    // let mut vals = [0; 4];
    // let mut sum = 0;
    // for i in 0..4 {
    //     vals[i] = if check_less!(src, i * 2, i * 2 + 1, is_less) { 0 } else { 1 };
    //     sum += vals[i] * 2usize.pow(i as u32);
    // }
    // // if sum == 0 {
    // //     println!("000000000000");
    // // } else if sum == 15 {
    // //     println!("115555555555555555555");
    // // }

    // match &vals {
    //     &[0, 0, 0, 0] => {
    //         // return;
    //     }
    //     // [1, 1, 1, 1] => {
    //     //     src[0..8].reverse();
    //     //     return;
    //     // }
    //     _ => {
    //         for i in 0..4 {
    //             if vals[i] == 1 {
    //                 src.swap(i * 2, i * 2 + 1);
    //             }
    //         }
    //     }
    // }

    // parity_merge_two(src, swap, is_less);
    // parity_merge_two(&mut src[4..], &mut swap[4..], is_less);
    // parity_merge_four(swap, src, is_less);
    
    for i in 0..4 {
        try_exchange!(src, is_less, i * 2, i * 2 + 1);
    }
    // if is_less(&src[1], &src[2]) && is_less(&src[3], &src[4]) && is_less(&src[5], &src[6]) {
    //     return;
    // }

    parity_merge_two(src, swap, is_less);
    parity_merge_two(&mut src[4..], &mut swap[4..], is_less);
    parity_merge_four(swap, src, is_less);
}

#[inline]
pub fn parity_swap_sixteen<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    for i in 0..4 {
        quad_swap_four(&mut src[i * 4..], &is_less);
    }
    if is_less(&src[3], &src[4]) && is_less(&src[7], &src[8]) && is_less(&src[11], &src[12]) {
        return;
    }

    parity_merge_four(src, swap, is_less);
    parity_merge_four(&mut src[8..], &mut swap[8..], is_less);
    parity_merge(src, swap, 8, 8, is_less);
}

#[inline]
pub fn tiny_sort<T, F>(src: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    match src.len() {
        4 => {
            quad_swap_four(src, is_less);
        }
        3 => {
            try_exchange!(src, &is_less, 0, 1);
            if try_exchange!(src, &is_less, 1, 2) {
                try_exchange!(src, &is_less, 0, 1);
            }
        }
        2 => {
            try_exchange!(src, &is_less, 0, 1);
        }
        _ => {
            return
        }
    }
}

#[inline]
pub fn twice_unguarded_insert<T, F>(src: &mut [T], swap: &mut [T], is_less: &F, offset: usize)
where
    F: Fn(&T, &T) -> bool
{
        // index = 0;
        // for idx in offset..src.len() {
        //     index = idx;
        //     for j in 0..idx {
        //         if (self.is_less)(&src[idx], &src[j]) {
        //             index = j;
        //             break;
        //         }
        //     }
        //     if index == idx {
        //         continue;
        //     }
        //     do_set_elem!(&mut src[index], &mut swap[1], idx - index);
        //     do_set_elem!(&mut src[idx], &mut swap[0]);
        //     do_set_elem!(&mut swap[0], &mut src[index], idx - index + 1);
        // }
        let mut index;
        for idx in offset..src.len() {
            index = idx;
            for j in 0..idx {
                if (is_less)(&src[idx], &src[j]) {
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
pub fn less_24_tail_swap<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{
    match src.len() {
        l if l < 5 => {
            tiny_sort(src, is_less);
            return;
        }
        l if l < 8 => {
            quad_swap_four(src, is_less);
            twice_unguarded_insert(src, swap, is_less, 4);
            return;
        }
        l if l < 12 => {
            parity_swap_eight(src, swap, is_less);
            twice_unguarded_insert(src, swap, is_less, 8);
            return;
        }
        l if l < 16 => {
            parity_swap_eight(src, swap, is_less);
            quad_swap_four(&mut src[8..], is_less);
            twice_unguarded_insert(&mut src[8..], swap, is_less, 4);
            // twice_unguarded_insert(src, swap, is_less, 8);
            // less_24_tail_swap(&mut src[8..], swap, is_less);
            partial_backward_merge(src, swap, 8, is_less);
            return;
        }
        l if l >= 16 && l < 24 => {
            parity_swap_sixteen(src, swap, is_less);
            twice_unguarded_insert(src, swap, is_less, 16);
            return;
        }
        _ => {
                unreachable!()
            }
        }
    }
    
#[inline]
pub fn tail_swap<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool
{

	if src.len() < 24 {
	    less_24_tail_swap(src, swap, is_less);
	    return;
	}
    let half1 = src.len() / 2;
    let quad1 = half1 / 2;
    let quad2 = half1 - quad1;

    let half2 = src.len() - half1;
    let quad3 = half2 / 2;
    let quad4 = half2 - quad3;
    
    let mut index = 0;
    tail_swap(&mut src[index..index + quad1], swap, is_less);
    index += quad1;
    tail_swap(&mut src[index..index + quad2], swap, is_less);
    index += quad2;
    tail_swap(&mut src[index..index + quad3], swap, is_less);
    index += quad3;
    tail_swap(&mut src[index..index + quad4], swap, is_less);

    if is_less(&src[quad1 - 1], &src[quad1]) 
    && is_less(&src[half1 - 1], &src[half1]) 
    && is_less(&src[index - 1], &src[index]) {
		return;
    }

    parity_merge(swap, src, quad1, quad2, is_less);
    parity_merge(&mut swap[half1..], &mut src[half1..], quad3, quad4, is_less);
    parity_merge(src, swap, half1, half2, is_less);
}

#[inline]
pub fn create_swap<T>(caption: usize) -> Vec<T> {

    unsafe {
        // let mut vec = Vec::with_capacity(32);
        // vec.set_len(32);
        // return vec;
        let mem = alloc::alloc(alloc::Layout::array::<T>(caption).unwrap_unchecked()) as *mut T;
        if !mem.is_null() {
            let vec = Vec::from_raw_parts(mem, caption, caption);
            vec
        } else {
            let mut vec = Vec::with_capacity(32);
            vec.set_len(32);
            vec
        }
    }
}

#[inline]
pub fn quad_sort_order_by<T, F>(src: &mut [T], is_less: F)
    where
        F: Fn(&T, &T) -> bool,
        T: Debug
{
    match src.len() {
        l if l < 32 => {
            let mut swap = create_swap(src.len());
            tail_swap(src,  &mut swap, &is_less);
        }
        _ => {
            let mut swap = create_swap::<T>(src.len());
            quad_swap(src, &mut swap, &is_less);
            if swap.len() != src.len() {
                src.sort_by(|a, b| if is_less(a, b) { Ordering::Less } else { Ordering::Greater });
                return;
            }
            let block = quad_merge(src, &mut swap, 32, &is_less);
            rotate_merge(src, &mut swap, block, &is_less);
        }
    }
}

#[inline]
pub fn quad_sort<T>(src: &mut [T])
where
    T: Debug + Ord
{
    quad_sort_order_by(src, T::lt);
}
