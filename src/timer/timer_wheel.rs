use std::ptr;

struct Entry<T: Timer> {
    val: T,
    when: usize,
    id: usize,
}

pub trait Timer {
    /// 当时与现在的间隔，以确定插入确定的槽
    fn when(&self) -> usize;
    /// 可能需要修改对象，此处用可变值
    fn when_mut(&mut self) -> usize {
        self.when()
    }
}

macro_rules! impl_primitive_timer {
    ($ty:ident) => {
        impl Timer for $ty {
            #[inline(always)]
            fn when(&self) -> usize {
                *self as usize
            }
        }
    };
}

impl_primitive_timer!(u8);
impl_primitive_timer!(u16);
impl_primitive_timer!(u32);
impl_primitive_timer!(u64);
impl_primitive_timer!(u128);
impl_primitive_timer!(i8);
impl_primitive_timer!(i16);
impl_primitive_timer!(i32);
impl_primitive_timer!(i64);
impl_primitive_timer!(i128);
impl_primitive_timer!(f32);
impl_primitive_timer!(f64);
impl_primitive_timer!(usize);

/// 单轮结构
pub struct OneTimerWheel<T: Timer> {
    /// 当时指针指向的位置，如秒针指向3点钟方向
    index: usize,
    /// 当前结构的容量，如表示秒的为60的容量
    capation: usize,
    /// 当前结构步长，如分钟就表示60s的
    step: usize,
    /// 当前槽位容纳的元素
    slots: Vec<Vec<Entry<T>>>,
    /// 当前轮结构的父轮，如当前是分的，那父轮为时轮
    parent: *mut OneTimerWheel<T>,
    /// 当前轮结构的子轮，如当前是分的，那父轮为秒轮
    child: *mut OneTimerWheel<T>,
    /// 当前轮的名字，辅助定位
    name: &'static str,
}

impl<T: Timer> OneTimerWheel<T> {
    pub fn new(capation: usize, step: usize, name: &'static str) -> Self {
        let mut slots = vec![];
        for _ in 0..capation {
            slots.push(Vec::new());
        }
        Self {
            index: 0,
            capation,
            step,
            slots,
            parent: ptr::null_mut(),
            child: ptr::null_mut(),
            name,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn clear(&mut self) {
        for idx in 0..self.capation {
            self.slots[idx].clear();
        }
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

    fn add_timer(&mut self, mut entry: Entry<T>) {
        let offset = entry.when;
        self.add_timer_with_offset(entry, offset);
    }

    fn del_timer(&mut self, timer_id: usize) -> Option<T> {
        for i in 0..self.capation {
            let mut found_idx = None;
            for (idx, val) in self.slots[i].iter().enumerate() {
                if val.id == timer_id {
                    found_idx = Some(idx);
                    break;
                }
            }
            if let Some(idx) = found_idx {
                return Some(self.slots[i].remove(idx).val);
            }
        }
        None
    }

    fn get_timer(&self, timer_id: &usize) -> Option<&T> {
        for i in 0..self.capation {
            for (idx, val) in self.slots[i].iter().enumerate() {
                if &val.id == timer_id {
                    return Some(&val.val);
                }
            }
        }
        None
    }

    fn get_mut_timer(&mut self, timer_id: &usize) -> Option<&mut T> {
        for i in 0..self.capation {
            let mut found_idx = None;
            let v = &mut self.slots[i];
            for (idx, val) in v.iter().enumerate() {
                if &val.id == timer_id {
                    found_idx = Some(idx);
                    break;
                }
            }
            if let Some(idx) = found_idx {
                return Some(&mut self.slots[i][idx].val);
            }
        }
        None
    }

    fn add_step_timer(&mut self, mut entry: Entry<T>) {
        let offset = entry.when % self.capation;
        self.add_timer_with_offset(entry, offset);
    }

    fn add_timer_with_offset(&mut self, entry: Entry<T>, offset: usize) {
        if offset > self.capation * self.step {
            let index = (self.index + self.capation - 1) % self.capation;
            self.slots[index].push(entry);
        } else if offset < self.step && !self.child.is_null() {
            unsafe {
                (*self.child).add_timer_with_offset(entry, offset);
            }
        } else {
            // 当前偏差值还在自己的容纳范围之前，做容错，排在最后处理位
            let index = (offset - 1) / self.step;
            let index = (index + self.index) % self.capation;
            self.slots[index].push(entry);
        }
    }

    pub fn update_index(&mut self, offset: usize, remainder: usize, result: &mut Vec<T>) -> (usize, usize) {
        let next = self.index + offset;
        let mut all = 0;
        for idx in self.index..next {
            if all > self.capation {
                break;
            }
            all += 1;
            let idx = idx % self.capation;
            let list = &mut self.slots[idx];
            for val in list.drain(..) {
                result.push(val.val);
            }
        }
        self.index = next % self.capation;
        if !self.child.is_null() {
            unsafe {
                let full = offset * self.step;
                let list = &mut self.slots[self.index];
                let step = (*self.child).step;

                for mut val in list.drain(..) {
                    val.when = val.when.saturating_sub(full);

                    if val.when <= remainder {
                        result.push(val.val);
                    } else {
                        (*self.child).add_step_timer(val);
                    }
                }
            }
        }
        (next / self.capation, next % self.capation + remainder)
    }
}

/// 计时器轮，模拟时钟格式组成的高效计时器
///
/// 时间轮是一个环形的数据结构，可以想象成一个时钟的面，被分成多个格子
///
/// 每个格子代表一段时间，这段时间越短，定时器的精度就越高。
///
/// 每个格子用一个Vec存储放在该格子上的延时任务。
///
/// Mark: 在Rust中双向链表中暂未提供元素关键列表的接口，这里改用Vec，删除时会额外移动Vec值
///
pub struct TimerWheel<T: Timer> {
    /// 时轮的最大轮，以时钟为例就是时针
    greatest: *mut OneTimerWheel<T>,
    /// 时轮的最小轮，以时钟为例就是秒针
    lessest: *mut OneTimerWheel<T>,
    /// 时轮的最小间隔，以时间为例就是秒
    min_step: usize,
    /// 维护定时器id
    next_timer_id: usize,
    /// 离的最近的id
    delay_id: usize,
    /// 总共的递进步长，缓存优化触发
    all_deltatime: usize,
    /// 当时时轮里的元素个数
    len: usize,
}

impl<T: Timer> TimerWheel<T> {
    pub fn new() -> Self {
        Self {
            greatest: ptr::null_mut(),
            lessest: ptr::null_mut(),
            next_timer_id: 0,
            delay_id: 0,
            min_step: 0,
            all_deltatime: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        let mut wheel = self.lessest;
        while !wheel.is_null() {
            unsafe {
                (*wheel).clear();
                wheel = (*wheel).parent;
            }
        }
    }

    pub fn append_timer_wheel(&mut self, slots: usize, step: usize, name: &'static str) {
        debug_assert!(self.len == 0, "必须时轮为空才可改变时轮");
        let one = Box::into_raw(Box::new(OneTimerWheel::new(slots, step, name)));
        self.delay_id = self.delay_id.max(slots * step);
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

    pub fn update_deltatime(&mut self, delta: usize) -> Option<Vec<T>> {
        debug_assert!(self.min_step > 0);
        self.all_deltatime += delta;
        let mut offset = self.all_deltatime / self.min_step;
        if offset < self.delay_id {
            return None;
        }

        self.all_deltatime -= offset * self.min_step;
        let mut remainder = 0;
        let mut result = vec![];
        let mut wheel = self.lessest;
        while !wheel.is_null() {
            unsafe {
                (offset, remainder) = (*wheel).update_index(offset, remainder, &mut result);
                if offset == 0 {
                    break;
                }
                wheel = (*wheel).parent;
            }
        }
        self.calc_delay_id();
        self.len -= result.len();
        Some(result)
    }

    pub fn update_deltatime_with_callback<F>(&mut self, delta: usize, f: &mut F)
    where
        F: FnMut(&mut Self, T),
    {
        debug_assert!(self.min_step > 0);
        if let Some(result) = self.update_deltatime(delta) {
            for r in result.into_iter() {
                (*f)(self, r);
            }
        }
    }

    /// 计算下一个delay_id, 根据容器的密度稀疏有关
    /// 密度高的基本为O(1)的复杂度, 最差情况为O(n)的复杂度
    /// 总刻度数以时钟为计秒轮遍历60次,分轮遍历60次,时轮遍历12次,即最高遍历132次
    pub fn calc_delay_id(&mut self) {
        let mut next_delay_id = 0;
        let mut wheel = self.lessest;
        'outer: while !wheel.is_null() {
            unsafe {
                let (step, index, cap) = ((*wheel).step, (*wheel).index, (*wheel).capation);
                for i in 0..cap {
                    let index = (index + i) % cap;
                    if !(*wheel).slots[index].is_empty() {
                        next_delay_id = (i + 1) * step;
                        break 'outer;
                    }
                }
                next_delay_id = cap * step;
                wheel = (*wheel).parent;
            }
        }
        self.delay_id = next_delay_id;
    }

    /// 删除指定的定时器，时间复杂度为O(n)，
    /// 该模型删除不具备优势，需要频繁删除请选用其它时间框架
    pub fn del_timer(&mut self, timer_id: usize) -> Option<T> {
        let mut wheel = self.lessest;
        while !wheel.is_null() {
            unsafe {
                if let Some(v) = (*wheel).del_timer(timer_id) {
                    self.len -= 1;
                    return Some(v);
                }
                wheel = (*wheel).parent;
            }
        }
        None
    }

    /// 获取指定的定时器，时间复杂度为O(n)
    pub fn get_timer(&self, timer_id: &usize) -> Option<&T> {
        let mut wheel = self.lessest;
        while !wheel.is_null() {
            unsafe {
                if let Some(v) = (*wheel).get_timer(timer_id) {
                    return Some(v);
                }
                wheel = (*wheel).parent;
            }
        }
        None
    }

    /// 获取指定的定时器，时间复杂度为O(n)
    pub fn get_mut_timer(&mut self, timer_id: &usize) -> Option<&mut T> {
        let mut wheel = self.lessest;
        while !wheel.is_null() {
            unsafe {
                if let Some(v) = (*wheel).get_mut_timer(timer_id) {
                    return Some(v);
                }
                wheel = (*wheel).parent;
            }
        }
        None
    }

    pub fn add_timer(&mut self, mut val: T) -> usize {
        debug_assert!(!self.greatest.is_null(), "必须设置时轮才能添加元素");
        let timer_id = self.next_timer_id;
        self.next_timer_id += 1;
        let entry = Entry { when: val.when_mut().max(1), val, id: timer_id };
        self.delay_id = self.delay_id.min(entry.when);
        unsafe {
            (*self.greatest).add_timer(entry);
        }
        self.len += 1;
        timer_id
    }

    pub fn get_delay_id(&self) -> usize {
        self.delay_id
    }
}
