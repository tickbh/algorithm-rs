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

pub fn quicksort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    match v.len() {
        l if l < 32 => {

        }
        _ => {

        }
    }

    // recurse(v, &mut is_less, None, limit);
}


pub fn try_exchange<T, F>(v: &mut [T], is_less: &F, start: usize, end: usize) -> bool
where
    F: Fn(&T, &T) -> bool,
{
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
{
    let mut l = v.as_mut_ptr();
    let mut block_l = BLOCK;
    let mut start_l = ptr::null_mut();
    let mut end_l = ptr::null_mut();
    for i in offset..v.len() {

    }
}


pub fn tail_swap<T, F>(v: &mut [T], is_less: &F)
where
    F: Fn(&T, &T) -> bool,
{
    match v.len() {
        l if l < 5 => {
            tiny_sort(v, is_less);
        }
        l if l < 8 => {
            quad_swap_four(v, is_less);

        }
        _ => {

        }
    }
}