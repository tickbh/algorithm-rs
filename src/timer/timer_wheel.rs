use std::{collections::LinkedList, ptr};

struct Entry<T:Timer> {
    val: T,
    id: usize,
}

pub trait Timer {
    fn when(&self) -> usize;
}

pub struct OneTimerWheel<T:Timer> {
    index: usize,
    capation: usize,
    step: usize,
    real_step: usize,
    slots: Vec<LinkedList<Entry<T>>>,
    parent: *mut OneTimerWheel<T>,
    child: *mut OneTimerWheel<T>,
    name: &'static str,
}

impl<T:Timer> OneTimerWheel<T> {
    pub fn new(capation: usize, step: usize, name: &'static str) -> Self {
        let mut slots = vec![];
        for _ in 0..capation {
            slots.push(LinkedList::new());
        }
        Self {
            index: 0,
            capation,
            step,
            real_step: step,
            slots,
            parent: ptr::null_mut(),
            child: ptr::null_mut(),
            name,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn append(&mut self, next: *mut OneTimerWheel<T>) {
        if self.child.is_null() {
            unsafe {
                (*next).parent = self;
                self.child = next;
            }
        } else {
            unsafe {
                (*self.child).append(next);
            }
        }
    }

    fn add_timer(&mut self, entry: Entry<T>) {
        // entry.val.when();
        let offset = entry.val.when();
        self.add_timer_with_offset(entry, offset);
    }

    fn add_timer_with_offset(&mut self, entry: Entry<T>, offset: usize) {
        if offset < self.step && !self.child.is_null() {
            unsafe {
                (*self.child).add_timer_with_offset(entry, offset);
            }
        } else {
            // 当前偏差值还在自己的容纳范围之前，做容错，排在最后处理位
            let index = (offset / self.step).max(1).min(self.capation);
            let index = (index + self.index) % self.capation;
            self.slots[index].push_back(entry); 
        }
    }

    pub fn update_index(&mut self, offset: usize, result: &mut Vec<T>) -> usize {
        let next = self.index + offset;
        let mut all = 0;
        for idx in self.index..next {
            if all > self.capation {
                break;
            }
            all += 1;
            let idx = idx % self.capation;
            let list = &mut self.slots[idx];
            while let Some(val) = list.pop_back() {
                result.push(val.val);
            }
        }
        self.index = next % self.capation;
        if !self.child.is_null() {
            unsafe {
                let list = &mut self.slots[self.index];
                while let Some(val) = list.pop_back() {
                    (*self.child).add_timer(val);
                }
            }
        }
        next / self.step
    }
}

pub struct TimerWheel<T:Timer> {
    greatest: *mut OneTimerWheel<T>,
    lessest: *mut OneTimerWheel<T>,
    min_step: usize,
    next_timer_id: usize,
    delay_id: usize,
    all_deltatime: usize,
}

impl<T:Timer> TimerWheel<T> {
    pub fn new() -> Self {
        Self {
            greatest: ptr::null_mut(),
            lessest: ptr::null_mut(),
            next_timer_id: 0,
            delay_id: 0,
            min_step: 0,
            all_deltatime: 0,
        }
    }

    pub fn append_timer_wheel(&mut self, slots: usize, step: usize, name: &'static str) {
        let one = Box::into_raw(Box::new(OneTimerWheel::new(slots, step, name)));
        self.lessest = one;
        if self.greatest.is_null() {
            self.greatest = one;
        } else {
            unsafe {
                (*self.greatest).append(one);
            }
        }
        self.min_step = step;
    }

    pub fn update_deltatime<F>(&mut self, delta: usize, f: &mut F)
    where F: FnMut(T) {
        debug_assert!(self.min_step > 0);
        self.all_deltatime += delta;
        let mut offset = self.all_deltatime / self.min_step;
        if offset < self.delay_id {
            return;
        }

        let mut result = vec![];
        let mut wheel = self.lessest;
        while !wheel.is_null() {
            unsafe {
                offset = (*wheel).update_index(offset, &mut result);
                if offset == 0 {
                    break;
                }
                wheel = (*wheel).parent;
            }
        }

        for r in result.into_iter() {
            (*f)(r);
        }
    }

    pub fn add_timer(&mut self, val: T) -> usize {
        let timer_id = self.next_timer_id;
        self.next_timer_id += 1;
        let entry = Entry { val, id: timer_id };
        unsafe {
            (*self.greatest).add_timer(entry);
        }
        timer_id
    }
}
