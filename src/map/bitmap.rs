use std::fmt::Display;

pub struct BitMap {
    entries: Vec<u8>,
    cap: usize,
}

impl BitMap {
    pub fn new(cap: usize) -> Self {
        let len = cap / 8 + if cap % 8 == 0 { 0 } else { 1 };
        Self {
            entries: vec![0; len],
            cap,
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn add(&mut self, val: usize) {
        let pos = val / 8;
        self.entries[pos] = self.entries[pos] | (1 << val % 8);
    }

    pub fn add_many(&mut self, val: &[usize]) {
        for v in val {
            self.add(*v);
        }
    }

    pub fn add_range(&mut self, start: usize, end: usize) {
        for pos in start..(start / 8 + 1) * 8 {
            self.add(pos)
        }
        for pos in (start / 8 + 1) .. end / 8 {
            self.entries[pos] = u8::MAX;
        }
        for pos in (end / 8) * 8..=end {
            self.add(pos)
        }
    }
    
    pub fn remove(&mut self, val: usize) {
        let pos = val / 8;
        let fix = u8::MAX - (1 << val % 8);
        self.entries[pos] = self.entries[pos] & fix;
    }

    pub fn remove_many(&mut self, val: &[usize]) {
        for v in val {
            self.remove(*v);
        }
    }

    pub fn remove_range(&mut self, start: usize, end: usize) {
        for pos in start..(start / 8 + 1) * 8 {
            self.remove(pos)
        }
        for pos in (start / 8 + 1) .. end / 8 {
            self.entries[pos] = 0;
        }
        for pos in (end / 8) * 8..=end {
            self.remove(pos)
        }
    }

    pub fn contains(&self, val: &usize) -> bool {
        let pos = val / 8;
        (self.entries[pos] & (1 << val % 8)) != 0
    }


}

impl Display for BitMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}