use std::{collections::LinkedList, ptr};

struct Entry<T> {
    val: T,
    id: usize,
}

pub trait Timer {
    
}

pub struct OneTimerWheel<T> {
    index: usize,
    capation: usize,
    step: usize,
    real_step: usize,
    slots: Vec<LinkedList<Entry<T>>>,
    parent: *mut OneTimerWheel<T>,
    child: *mut OneTimerWheel<T>,
    name: &'static str,
}

impl<T> OneTimerWheel<T> {
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
                    // (*self.child).add_timer()
                }
            }
        }
        next / self.step
    }
}

pub struct TimerWheel<T> {
    greatest: *mut OneTimerWheel<T>,
    lessest: *mut OneTimerWheel<T>,
    min_step: usize,
    next_timer_id: usize,
    delay_id: usize,
    all_deltatime: usize,
}

impl<T> TimerWheel<T> {
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
}
