
use std::{ptr, time::Instant};

use algorithm::{Reinit, Slab};

const ARRAY_SIZE: usize = 10240;
const NUM: usize = usize::MAX - 99999;
const ZERO_ARRAY: [usize; ARRAY_SIZE] = [NUM; ARRAY_SIZE];
struct TestStruct {
    array: [usize; ARRAY_SIZE],
    size: usize,
    val: String,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self { array: [NUM; ARRAY_SIZE], size: 0, val:  "slab".to_string(), }
    }
}

impl Reinit for TestStruct {
    #[inline(always)]
    fn reinit(&mut self) {
        self.size = 0;
        self.val.clear();
        self.val.push_str("slab");
        unsafe {
            ptr::copy_nonoverlapping(&ZERO_ARRAY[0], &mut self.array[0], ARRAY_SIZE);
        }
    }
}

fn main() {
    let times = 100000;
    let now = Instant::now();
    let mut slab = Slab::<TestStruct>::new();
    let mut sum: usize = 0;
    for i in 0..times {
        let (next, test) = slab.get_reinit_next_val();
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size + test.val.len());
        slab.remove(next);
    }
    println!("algorithm: all cost times {}ms, sum = {}", now.elapsed().as_millis(), sum);

    let now = Instant::now();
    let mut slab = slab::Slab::<TestStruct>::new();
    let mut sum: usize = 0;
    for i in 0..times {
        let next = slab.insert(TestStruct::default());
        let test = &mut slab[next];
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size + test.val.len());
        slab.remove(next);
    }
    println!("tokio::slab: all cost times {}ms, sum = {}", now.elapsed().as_millis(), sum);

    let now = Instant::now();
    let mut sum: usize = 0;
    for i in 0..times {
        let mut test = TestStruct::default();
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size + test.val.len());
        drop(test);
    }
    println!("normal alloc: all cost times {}ms, sum = {}", now.elapsed().as_millis(), sum);
}
