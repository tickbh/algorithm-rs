
use std::{mem, ptr, time::Instant};

use algorithm::{Reinit, Slab};

const ARRAY_SIZE: usize = 10240;
const NUM: usize = usize::MAX;
const ZERO_ARRAY: [usize; ARRAY_SIZE] = [NUM; ARRAY_SIZE];
struct TestStruct {
    array: [usize; ARRAY_SIZE],
    size: usize,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self { array: [NUM; ARRAY_SIZE], size: 0 }
    }
}

impl Reinit for TestStruct {
    #[inline(always)]
    fn reinit(&mut self) {
        // self.array.fill(0);
        unsafe {
            // ptr::write_bytes(&mut self.array[0], u8::MAX, ARRAY_SIZE);
            ptr::copy_nonoverlapping(&ZERO_ARRAY[0], &mut self.array[0], ARRAY_SIZE);
            // libc::memset(&mut self.array[0] as *mut usize as *mut libc::c_void, 0, ARRAY_SIZE * mem::size_of::<usize>());
        }
    }
}

fn test_speed() {

    let times = 10000;
    let now = Instant::now();
    let mut slab = Slab::<TestStruct>::new();
    let mut sum: usize = 0;
    for i in 0..times {
        let (next, test) = slab.get_reinit_next_val();
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size);
        slab.remove(next);
    }
    println!("all cost times {}, sum = {}", now.elapsed().as_nanos(), sum);


    let now = Instant::now();
    let mut slab = slab::Slab::<TestStruct>::new();
    let mut sum: usize = 0;
    for i in 0..times {
        let next = slab.insert(TestStruct::default());
        let test = &mut slab[next];
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size);
        slab.remove(next);
    }
    println!("all cost times {}, sum = {}", now.elapsed().as_nanos(), sum);

    let now = Instant::now();
    let mut sum: usize = 0;
    for i in 0..times {
        let mut test = TestStruct::default();
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size);
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
