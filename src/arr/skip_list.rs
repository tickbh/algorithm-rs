use std::{marker::PhantomData, ptr, usize::MAX};

struct LevelType<T> {
    pub forward: *mut SkipNode<T>,
    pub span: usize,
}

impl<T> Clone for LevelType<T> {
    fn clone(&self) -> Self {
        Self {
            forward: self.forward.clone(),
            span: self.span.clone(),
        }
    }
}

impl<T> LevelType<T> {
    pub fn new() -> Self {
        LevelType {
            forward: ptr::null_mut(),
            span: 0,
        }
    }
}

pub struct SkipNode<T> {
    pub score: T,
    pub backward: *mut SkipNode<T>,
    levels: Vec<LevelType<T>>,
}

pub struct SkipList<T: Default + PartialEq + PartialOrd> {
    length: usize,
    level: usize,

    header: *mut SkipNode<T>,
    tail: *mut SkipNode<T>,
}

const MAX_LEVEL: usize = 32;
const PERCENT: u16 = 25;

impl<T: Default + PartialEq + PartialOrd> SkipList<T> {
    pub fn new() -> Self {
        let mut sl = SkipList {
            length: 0,
            level: 1,
            header: ptr::null_mut(),
            tail: ptr::null_mut(),
        };
        sl.clear();
        sl
    }

    pub fn free_all(&mut self) {
        while !self.header.is_null() {
            unsafe {
                let next = (*self.header).levels[0].forward;
                Self::free_node(self.header);
                self.header = next;
            }
        }
        self.header = ptr::null_mut();
    }

    pub fn clear(&mut self) {
        self.free_all();

        self.header = Self::make_node(MAX_LEVEL, T::default());

        unsafe {
            for i in 0..MAX_LEVEL {
                (*self.header).levels[i].forward = ptr::null_mut();
                (*self.header).levels[i].span = 0;
            }
            (*self.header).backward = ptr::null_mut();
        }
        self.level = 1;
        self.length = 0;
    }

    fn make_node(level: usize, score: T) -> *mut SkipNode<T> {
        assert!(level > 0);

        let levels = vec![LevelType::new(); level];
        let node = SkipNode {
            score,
            backward: ptr::null_mut(),
            levels,
        };
        Box::into_raw(Box::new(node))
    }

    fn free_node(node: *mut SkipNode<T>) {
        unsafe {
            let _ = Box::from_raw(node);
        }
    }

    fn rand_level() -> usize {
        let mut level = 1;
        while rand::random::<u16>() % 100 < PERCENT {
            level += 1;
        }
        level
    }

    pub fn insert(&mut self, score: T) -> *mut SkipNode<T> {
        let mut update: [*mut SkipNode<T>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        let mut rank = [0; MAX_LEVEL];
        let mut x = self.header;
        for i in (0..self.level as usize).rev() {
            rank[i] = if i == (self.level - 1) as usize {
                0
            } else {
                rank[i + 1]
            };

            unsafe {
                while (*x).levels[i].forward != ptr::null_mut()
                    && (*(*x).levels[i].forward).score < score
                {
                    rank[i] += (*x).levels[i].span;
                    x = (*x).levels[i].forward;
                }
                update[i] = x;
            }
        }

        let level = Self::rand_level();
        if level > self.level {
            for i in level as usize..self.level as usize {
                rank[i] = 0;
                update[i] = self.header;
                unsafe {
                    (*update[i]).levels[i].span = self.length;
                }
            }
            self.level = level;
        }

        x = Self::make_node(level, score);
        unsafe {
            for i in 0..level {
                (*x).levels[i].forward = (*update[i]).levels[i].forward;
                (*update[i]).levels[i].forward = x;

                (*x).levels[i].span = (*update[i]).levels[i].span - (rank[0] - rank[i]);
                (*update[i]).levels[i].span = (rank[0] - rank[i]) + 1;
            }

            for i in level..self.level {
                (*update[i]).levels[i].span += 1;
            }

            (*x).backward = if update[0] == self.header {
                ptr::null_mut()
            } else {
                update[0]
            };
            if (*x).levels[0].forward != ptr::null_mut() {
                (*(*x).levels[0].forward).backward = x;
            } else {
                self.tail = x;
            }

            self.length += 1;
        }
        x
    }

    pub fn update(&mut self, cur_score: &T, new_score: T) -> *mut SkipNode<T> {
        let mut update: [*mut SkipNode<T>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        let mut rank = [0; MAX_LEVEL];
        let mut x = self.header;

        unsafe {
            for i in (0..self.level).rev() {
                while (*x).levels[i].forward != ptr::null_mut()
                    && &(*(*x).levels[i].forward).score < cur_score
                {
                    rank[i] += (*x).levels[i].span;
                    x = (*x).levels[i].forward;
                }
                update[i] = x;
            }
            x = (*x).levels[0].forward;

            assert!(x != ptr::null_mut() && cur_score == &(*x).score);

            if ((*x).backward != ptr::null_mut() || (*(*x).backward).score < new_score)
                && ((*x).levels[0].forward != ptr::null_mut()
                    || (*(*x).levels[0].forward).score < new_score)
            {
                (*x).score = new_score;
                return x;
            }

            self.remove_node(x, update);
            let ret = self.insert(new_score);
            Self::free_node(x);
            ret
        }
    }

    fn remove_node(&mut self, x: *mut SkipNode<T>, update: [*mut SkipNode<T>; MAX_LEVEL]) {
        unsafe {
            for i in 0..self.level {
                if (*update[i]).levels[i].forward == x {
                    (*update[i]).levels[i].span += (*x).levels[i].span - 1;
                    (*update[i]).levels[i].forward = (*x).levels[i].forward;
                } else {
                    (*update[i]).levels[i].span -= 1;
                }
            }

            if (*x).levels[0].forward != ptr::null_mut() {
                (*(*x).levels[0].forward).backward = (*x).backward;
            } else {
                self.tail = (*x).backward;
            }

            while self.level > 1 && (*self.header).levels[self.level - 1].forward.is_null() {
                self.level -= 1;
            }
            self.length -= 1;
        }
    }

    pub fn get_rank(&mut self, score: &T) -> usize {
        let mut x = self.header;
        let mut rank = 0;
        for i in (0..self.level).rev() {
            unsafe {
                while !(*x).levels[i].forward.is_null() && &(*(*x).levels[i].forward).score <= score
                {
                    rank += (*x).levels[i].span;
                    x = (*x).levels[i].forward;
                }

                if &(*x).score == score {
                    return rank;
                }
            }
        }
        0
    }

    fn find_by_rank(&mut self, rank: usize) -> *mut SkipNode<T> {
        let mut x = self.header;
        let mut traversed = 0;
        for i in (0..self.level).rev() {
            unsafe {
                while !(*x).levels[i].forward.is_null() && (traversed + (*x).levels[i].span) <= rank
                {
                    traversed += (*x).levels[i].span;
                    x = (*x).levels[i].forward;
                }
                if traversed == rank {
                    return x;
                }
            }
        }
        ptr::null_mut()
    }

    pub fn erase(&mut self, score: &T) -> bool {
        let mut update: [*mut SkipNode<T>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        let mut x = self.header;
        unsafe {
            for i in (0..self.level).rev() {
                while (*x).levels[i].forward != ptr::null_mut()
                    && &(*(*x).levels[i].forward).score < score
                {
                    x = (*x).levels[i].forward;
                }
                update[i] = x;
            }
            x = (*x).levels[0].forward;

            if !x.is_null() && score == &(*x).score {
                self.remove_node(x, update);
                Self::free_node(x);
                return true;
            }
        }
        return false;
    }

    pub fn len(&self) -> usize {
        self.length
    }
}
