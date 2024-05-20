use std::{fmt::Debug, mem::{self, MaybeUninit}, ptr};

macro_rules! try_ex {
    ($start: expr, $end: expr) => {
        if !is_less(&v[$start], &v[$end]) {
            v.swap($start, $end);
            true
        } else {
            false
        }
    };
}


pub fn try_exchange<T, F>(src: &mut [T], is_less: &F, start: usize, end: usize) -> bool
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    // println!("起始位置:{} {:?}, 结束位置:{} {:?} 比较大小:{}", start, &src[start], end, &src[end], !is_less(&src[start], &src[end]));
    if !is_less(&src[start], &src[end]) {
        src.swap(start, end);
        true
    } else {
        false
    }
}

pub fn quad_swap_four<T, F>(src: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    try_exchange(src, &is_less, 0, 1);
    try_exchange(src, &is_less, 2, 3);
    // 中间顺序正确则表示排序完毕
    if try_exchange(src, &is_less, 1, 2) {
        try_exchange(src, &is_less, 0, 1);
        if try_exchange(src, &is_less, 2, 3) {
            try_exchange(src, &is_less, 1, 2);
        }
    }
}

pub fn head_branchless_merge<T, F>(src: &mut [T], swap: &mut [T], index: &mut usize, left: &mut usize, right: &mut usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    if is_less(&src[*left], &src[*right]) {
        mem::swap(&mut src[*left], &mut swap[*index]);
        *left += 1;
    } else {
        mem::swap(&mut src[*right], &mut swap[*index]);
        *right += 1;
    }
    *index += 1;
}

pub fn tail_branchless_merge<T, F>(src: &mut [T], swap: &mut [T], index: &mut usize, left: &mut usize, right: &mut usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    if !is_less(&src[*left], &src[*right]) {
        mem::swap(&mut src[*left], &mut swap[*index]);
        *left -= 1;
    } else {
        mem::swap(&mut src[*right], &mut swap[*index]);
        *right -= 1;
    }
    *index -= 1;
}


pub fn parity_merge_two<T, F>(src: &mut [T], swap: &mut [T], left: &mut usize, right: &mut usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    let mut index = 0;
    (*left, *right) = (0, 2);
    head_branchless_merge(src, swap, &mut index, left, right, is_less);
    if is_less(&src[*left], &src[*right]) {
        mem::swap(&mut src[*left], &mut swap[index]);
    } else {
        mem::swap(&mut src[*right], &mut swap[index]);
    }
    index = 3;
    (*left, *right) = (1, 3);
    tail_branchless_merge(src, swap, &mut index, left, right, is_less);
    if !is_less(&src[*left], &src[*right]) {
        mem::swap(&mut src[*left], &mut swap[index]);
    } else {
        mem::swap(&mut src[*right], &mut swap[index]);
    }

    println!("parity_merge_two src = {:?}", &src[..4]);
    println!("parity_merge_two swap = {:?}", &swap[..4]);

}

pub fn parity_merge_four<T, F>(src: &mut [T], swap: &mut [T], left: &mut usize, right: &mut usize, is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    println!("parity_merge_four start src = {:?}", &src[..8]);
    println!("parity_merge_four start swap = {:?}", &swap[..8]);
    let mut index = 0;
    (*left, *right) = (0, 4);
    head_branchless_merge(src, swap, &mut index, left, right, is_less);
    println!("parity_merge_four start 1 src = {:?}", &src[..8]);
    println!("parity_merge_four start 1 swap = {:?}", &swap[..8]);
    head_branchless_merge(src, swap, &mut index, left, right, is_less);
    println!("parity_merge_four start 2 src = {:?}", &src[..8]);
    println!("parity_merge_four start 2 swap = {:?}", &swap[..8]);
    head_branchless_merge(src, swap, &mut index, left, right, is_less);
    println!("parity_merge_four start 3 src = {:?}", &src[..8]);
    println!("parity_merge_four start 3 swap = {:?}", &swap[..8]);
    if is_less(&src[*left], &src[*right]) {
        mem::swap(&mut src[*left], &mut swap[index]);
    } else {
        mem::swap(&mut src[*right], &mut swap[index]);
    }
    println!("parity_merge_four start 4 src = {:?}", &src[..8]);
    println!("parity_merge_four start 4 swap = {:?}", &swap[..8]);

    println!("parity_merge_four mid src = {:?}", &src[..8]);
    println!("parity_merge_four mid swap = {:?}", &swap[..8]);
    index = 3;
    (*left, *right) = (3, 7);
    tail_branchless_merge(src, swap, &mut index, left, right, is_less);
    tail_branchless_merge(src, swap, &mut index, left, right, is_less);
    tail_branchless_merge(src, swap, &mut index, left, right, is_less);
    if !is_less(&src[*left], &src[*right]) {
        mem::swap(&mut src[*left], &mut swap[index]);
    } else {
        mem::swap(&mut src[*right], &mut swap[index]);
    }
    println!("parity_merge_four end src = {:?}", &src[..8]);
    println!("parity_merge_four end swap = {:?}", &swap[..8]);
}

pub fn parity_swap_eight<T, F>(src: &mut [T], swap: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    for i in 0..4 {
        try_exchange(src, &is_less, i * 2, i * 2 + 1);
    }
    if is_less(&src[1], &src[2]) && is_less(&src[3], &src[4]) && is_less(&src[5], &src[6]) {
        return;
    }

    let (mut left, mut right) = (0, 0);
    parity_merge_two(src, swap, &mut left, &mut right, is_less);
    parity_merge_two(&mut src[4..], &mut swap[4..], &mut left, &mut right, is_less);

    parity_merge_four(swap, src, &mut left, &mut right, is_less);
}

pub fn tiny_sort<T, F>(src: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    match src.len() {
        4 => {
            quad_swap_four(src, is_less);
        }
        3 => {
            try_exchange(src, &is_less, 0, 1);
            if try_exchange(src, &is_less, 1, 2) {
                try_exchange(src, &is_less, 0, 1);
            }
        }
        2 => {
            try_exchange(src, &is_less, 0, 1);
        }
        _ => {
            return
        }
    }
}

pub fn twice_unguarded_insert<T, F>(src: &mut [T], is_less: &F, offset: usize)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    for idx in offset..src.len() {
        if !try_exchange(src, is_less, idx - 1, idx) {
            continue;
        }

        if is_less(&src[idx - 1], &src[0]) {
            for j in (0..idx - 1).rev() {
                src.swap(j+1, j)
            }
        } else {
            for j in (0..idx - 1).rev() {
                if !try_exchange(src, is_less, j, j+1) {
                    break;
                }
            }
        }
    }
}

pub fn tail_swap<T, F>(src: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    match src.len() {
        l if l < 5 => {
            tiny_sort(src, is_less);
        }
        l if l < 8 => {
            quad_swap_four(src, is_less);
            println!("four = {:?}", &src);
            twice_unguarded_insert(src, is_less, 4);
        }
        l if l < 12 => {
            let mut swap = src[..8].to_vec();
            parity_swap_eight(src, &mut swap, is_less);
            println!("eight swap = {:?}", &swap);
            println!("eight src = {:?}", &src);
            twice_unguarded_insert(src, is_less, 4);
        }
        _ => {

        }
    }
}


pub fn quicksort<T, F>(src: &mut [T], is_less: F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug + Clone
{
    match src.len() {
        l if l < 32 => {
            tail_swap(src, &is_less);
        }
        _ => {

        }
    }

    // recurse(v, &mut is_less, None, limit);
}
