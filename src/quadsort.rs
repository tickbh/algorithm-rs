use std::fmt::Debug;

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


pub fn try_exchange<T, F>(v: &mut [T], is_less: &F, start: usize, end: usize) -> bool
where
    F: Fn(&T, &T) -> bool,
    T: Debug
{
    println!("起始位置:{} {:?}, 结束位置:{} {:?} 比较大小:{}", start, &v[start], end, &v[end], !is_less(&v[start], &v[end]));
    if !is_less(&v[start], &v[end]) {
        v.swap(start, end);
        true
    } else {
        false
    }
}

pub fn quad_swap_four<T, F>(v: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug
{
    try_exchange(v, &is_less, 0, 1);
    try_exchange(v, &is_less, 2, 3);
    // 中间顺序正确则表示排序完毕
    if try_exchange(v, &is_less, 1, 2) {
        try_exchange(v, &is_less, 0, 1);
        if try_exchange(v, &is_less, 2, 3) {
            try_exchange(v, &is_less, 1, 2);
        }
    }
}

pub fn tiny_sort<T, F>(v: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug
{
    match v.len() {
        4 => {
            quad_swap_four(v, is_less);
        }
        3 => {
            try_exchange(v, &is_less, 0, 1);
            if try_exchange(v, &is_less, 1, 2) {
                try_exchange(v, &is_less, 0, 1);
            }
        }
        2 => {
            try_exchange(v, &is_less, 0, 1);
        }
        _ => {
            return
        }
    }
}

pub fn twice_unguarded_insert<T, F>(v: &mut [T], is_less: &F, offset: usize)
where
    F: Fn(&T, &T) -> bool,
    T: Debug
{
    for idx in offset..v.len() {
        if !try_exchange(v, is_less, idx - 1, idx) {
            continue;
        }

        if is_less(&v[idx - 1], &v[0]) {
            for j in (0..idx - 1).rev() {
                v.swap(j+1, j)
            }
        } else {
            for j in (0..idx - 1).rev() {
                if !try_exchange(v, is_less, j, j+1) {
                    break;
                }
            }
        }
    }
}

pub fn tail_swap<T, F>(v: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug
{
    match v.len() {
        l if l < 5 => {
            tiny_sort(v, is_less);
        }
        l if l < 8 => {
            quad_swap_four(v, is_less);
            println!("four = {:?}", &v);
            twice_unguarded_insert(v, is_less, 4);
        }
        _ => {

        }
    }
}


pub fn quicksort<T, F>(v: &mut [T], is_less: F)
where
    F: Fn(&T, &T) -> bool,
    T: Debug
{
    match v.len() {
        l if l < 32 => {
            tail_swap(v, &is_less);
        }
        _ => {

        }
    }

    // recurse(v, &mut is_less, None, limit);
}
