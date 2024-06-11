pub struct CircularBuffer<T: Default> {
    arr: Vec<T>,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T: Default> CircularBuffer<T> {
    pub fn new(cap: usize) -> Self {
        let mut arr = Vec::with_capacity(cap);
        for _ in 0..cap {
            arr.push(T::default());
        }
        Self {
            arr,
            head: 0,
            tail: cap,
            size: 0,
        }
    }
}
