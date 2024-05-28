
use std::{ptr, time::Instant};

use algorithm::{Reinit, Slab};

const ARRAY_SIZE: usize = 102400;
// const ZERO_ARRAY: [usize; ARRAY_SIZE] = [0; ARRAY_SIZE];
struct TestStruct {
    array: [usize; ARRAY_SIZE],
    size: usize,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self { array: [0; ARRAY_SIZE], size: rand::random::<usize>() % 10 }
    }
}

impl Reinit for TestStruct {
    #[inline(always)]
    fn reinit(&mut self) {
        // self.array.fill(0);
        unsafe {
            ptr::write_bytes(&mut self.array[0], 0, ARRAY_SIZE);
        }
    }
}

fn test_speed() {

    let times = 10000;
    let now = Instant::now();
    let mut slab = Slab::<TestStruct>::new();
    let mut sum = 0;
    for i in 0..times {
        let (next, s) = slab.get_reinit_next_val();
        s.array[i % 20] += i;
        sum += s.array[10] + s.size;
        slab.remove(next);
    }
    println!("all cost times {}, sum = {}", now.elapsed().as_nanos(), sum);


    let now = Instant::now();
    let mut sum = 0;
    for i in 0..times {
        let mut test = TestStruct::default();
        test.array[i % 20] += i;
        sum += test.array[10] + test.size;
        drop(test);
    }
    println!("all cost times {}, sum = {}", now.elapsed().as_nanos(), sum);
}


fn main() {
    let mut slab = Slab::new();
    for _ in 0..100 {
        let k = slab.get_next();
        slab[&k] = format!("{}", k);
    }
    assert!(slab.len() == 100);

    for i in 0..100 {
        let _ = slab.remove(i);
    }

    assert!(slab.len() == 0);
    let k = slab.get_next();
    assert!(k == 99);
    assert!(slab[&k] == "99");
    let k = slab.get_reinit_next();
    assert!(k == 98);
    assert!(slab[&k] == "");

    test_speed();
}
