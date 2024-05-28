use std::{mem, ops::{Index, IndexMut}};

pub trait Reinit {
    fn reinit(&mut self) {}
}

macro_rules! impl_primitive_index {
    ($ty:ident, $zero:expr ) => {
        impl Reinit for $ty {
            #[inline(always)]
            fn reinit(&mut self) {
                *self = $zero;
            }
        }
    }
}

impl_primitive_index!(u8, 0);
impl_primitive_index!(u16, 0);
impl_primitive_index!(u32, 0);
impl_primitive_index!(u64, 0);
impl_primitive_index!(u128, 0);
impl_primitive_index!(i8, 0);
impl_primitive_index!(i16, 0);
impl_primitive_index!(i32, 0);
impl_primitive_index!(i64, 0);
impl_primitive_index!(i128, 0);
impl_primitive_index!(f32, 0.0);
impl_primitive_index!(f64, 0.0);

impl Reinit for bool {
    fn reinit(&mut self) {
        *self = false;
    }
}

impl Reinit for String {
    fn reinit(&mut self) {
        self.clear();
    }
}

impl Reinit for &str {
    fn reinit(&mut self) {
        *self = "";
    }
}

impl<T> Reinit for Vec<T> {
    fn reinit(&mut self) {
        self.clear();
    }
}

struct Entry<T: Default> {
    t: T,
    next: usize,
}

impl<T: Default> Entry<T> {

    pub fn new() -> Self {
        Self {
            t: T::default(),
            next: usize::MAX,
        }
    }

    pub fn is_occupied(&self) -> bool {
        self.next == usize::MAX
    }

}

pub struct Slab<T: Default> {
    entries: Vec<Entry<T>>,
    len: usize,
    next: usize,
}

impl<T: Default> Slab<T> {
    
    pub fn new() -> Self {
        Slab { entries: vec![], len: 0, next: 0 }
    }

    
    pub fn get(&mut self, key: usize) -> &T {
        let entry = &mut self.entries[key];
        assert!(entry.is_occupied() == true);
        &entry.t
    }

    pub fn get_next(&mut self) -> (usize, &mut T) {
        if self.entries.len() == self.len {
            let entry = Entry::new();
            self.entries.push(entry);
            self.len += 1;
            (self.len - 1, &mut self.entries[self.len - 1].t)
        } else {
            let entry = &mut self.entries[self.next];
            if entry.is_occupied() {
                unreachable!()
            }
            self.len += 1;
            let key = self.next;
            self.next = entry.next;
            entry.next = usize::MAX;
            (key, &mut entry.t)
        }
    }
    
    pub fn get_next_key(&mut self) -> usize {
        if self.entries.len() == self.len {
            let entry = Entry::new();
            self.entries.push(entry);
            self.len += 1;
            self.len - 1
        } else {
            let entry = &mut self.entries[self.next];
            if entry.is_occupied() {
                unreachable!()
            }
            self.len += 1;
            let key = self.next;
            self.next = entry.next;
            entry.next = usize::MAX;
            key
        }
    }

    pub fn remove(&mut self, key: usize) {
        if !self.try_remove(key) {
            panic!("index error")
        }
    }

    pub fn try_remove(&mut self, key: usize) -> bool {
        let entry = &mut self.entries[key];
        if !entry.is_occupied() {
            return false;
        }
        self.len -= 1;
        entry.next = self.next;
        self.next = key;
        true
    }
}

impl<T: Default + Reinit> Slab<T> {
    pub fn get_reinit_next_key(&mut self) -> usize {
        let key = self.get_next_key();
        self.entries[key].t.reinit();
        key
    }
}


impl<'a, T: Default> Index<&'a usize> for Slab<T>
{
    type Output = T;

    #[inline]
    fn index(&self, index: &usize) -> &T {
        &self.entries[*index].t
    }
}


impl<'a, T: Default> IndexMut<&'a usize> for Slab<T>
{
    #[inline]
    fn index_mut(&mut self, index: &usize) -> &mut T {
        &mut self.entries[*index].t
    }
}

