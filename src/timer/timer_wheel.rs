use std::{
    fmt::{self, Display},
    ptr, u64,
};

use super::Timer;

struct Entry<T: Timer> {
    val: T,
    when: u64,
    id: u64,
}
/// 单轮结构
pub struct OneTimerWheel<T: Timer> {
    /// 当时指针指向的位置，如秒针指向3点钟方向
    index: u64,
    /// 当前结构的容量，比如60s可能有30个槽,每个都是2秒
    capation: u64,
    /// 当前槽的个数
    num: u64,
    /// 当前结构步长，如分钟就表示60s的
    step: u64,
    /// 修正步长，当前的步长*基础步长
    fix_step: u64,
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
    pub fn new(num: u64, step: u64, one_step: u64, name: &'static str) -> Self {
        let mut slots = vec![];
        for _ in 0..num {
            slots.push(Vec::new());
        }
        Self {
            index: 0,
            capation: num * step,
            num,
            step,
            fix_step: one_step * step,
            slots,
            parent: ptr::null_mut(),
            child: ptr::null_mut(),
            name,
        }
    }

    pub fn clear(&mut self) {
        for idx in 0..self.num as usize {
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

    fn add_timer(&mut self, entry: Entry<T>) {
        let offset = entry.when;
        self.add_timer_with_offset(entry, offset);
    }

    fn del_timer(&mut self, timer_id: u64) -> Option<T> {
        for i in 0..self.num as usize {
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

    fn get_timer(&self, timer_id: &u64) -> Option<&T> {
        for i in 0..self.num as usize {
            for val in self.slots[i].iter() {
                if &val.id == timer_id {
                    return Some(&val.val);
                }
            }
        }
        None
    }

    fn get_mut_timer(&mut self, timer_id: &u64) -> Option<&mut T> {
        for i in 0..self.num as usize {
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

    fn add_step_timer(&mut self, entry: Entry<T>) {
        let offset = entry.when % self.num;
        self.add_timer_with_offset(entry, offset);
    }

    fn add_timer_with_offset(&mut self, entry: Entry<T>, offset: u64) {
        if offset > self.capation {
            let index = (self.index + self.num - 1) % self.num;
            self.slots[index as usize].push(entry);
        } else if offset < self.fix_step && !self.child.is_null() {
            unsafe {
                (*self.child).add_timer_with_offset(entry, offset);
            }
        } else {
            // 当前偏差值还在自己的容纳范围之前，做容错，排在最后处理位
            let index = (offset - 1) / self.step;
            let index = (index + self.index) % self.num;
            self.slots[index as usize].push(entry);
        }
    }

    pub fn update_index(
        &mut self,
        offset: u64,
        remainder: u64,
        result: &mut Vec<(u64, T)>,
    ) -> (u64, u64) {
        let next = self.index + offset;
        let mut all = 0;
        for idx in self.index..next {
            if all > self.num {
                break;
            }
            all += 1;
            let idx = idx % self.num;
            let list = &mut self.slots[idx as usize];
            for val in list.drain(..) {
                result.push((val.id, val.val));
            }
        }
        self.index = next % self.num;
        if !self.child.is_null() {
            unsafe {
                let list = &mut self.slots[self.index as usize];
                for mut val in list.drain(..) {
                    val.when = (val.when % self.step).saturating_sub(remainder);
                    if val.when == 0 {
                        result.push((val.id, val.val));
                    } else {
                        (*self.child).add_step_timer(val);
                    }
                }
            }
        }
        (next / self.num, next % self.num + remainder)
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
/// # Examples
///
/// ```
/// use algorithm::TimerWheel;
/// fn main() {
///     let mut timer = TimerWheel::new();
///     timer.append_timer_wheel(60, "SecondWheel");
///     timer.append_timer_wheel(60, "MinuteWheel");
///     timer.append_timer_wheel(12, "HourWheel");
///     timer.add_timer(30);
///     assert_eq!(timer.get_delay_id(), 30);
///     timer.add_timer(149);
///     assert_eq!(timer.get_delay_id(), 30);
///     let t = timer.add_timer(600);
///     assert_eq!(timer.get_delay_id(), 30);
///     timer.add_timer(1);
///     assert_eq!(timer.get_delay_id(), 1);
///     timer.del_timer(t);
///     timer.add_timer(150);
///     assert_eq!(timer.get_delay_id(), 1);
///     let val = timer.update_deltatime(30).unwrap();
///     assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<usize>>(), vec![1, 30]);
///     timer.add_timer(2);
///     let val = timer.update_deltatime(119).unwrap();
///     assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<usize>>(), vec![2, 149]);
///     let val = timer.update_deltatime(1).unwrap();
///     assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<usize>>(), vec![150]);
///     assert!(timer.is_empty());
/// }
/// ```
pub struct TimerWheel<T: Timer> {
    /// 时轮的最大轮，以时钟为例就是时针
    greatest: *mut OneTimerWheel<T>,
    /// 时轮的最小轮，以时钟为例就是秒针
    lessest: *mut OneTimerWheel<T>,
    /// 时轮的最小间隔，以时间为例就是秒
    one_step: u64,
    /// 维护定时器id
    next_timer_id: u64,
    /// 限制最大的timer id
    max_timer_id: u64,
    /// 离的最近的id
    delay_id: u64,
    /// 总共的递进步长，缓存优化触发
    all_deltatime: u64,
    /// 当时时轮里的元素个数
    len: usize,
}

impl<T: Timer> TimerWheel<T> {
    /// 创建一个计时器轮
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::<u64>::new();
    ///     assert!(timer.is_empty());
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            greatest: ptr::null_mut(),
            lessest: ptr::null_mut(),
            next_timer_id: 1,
            max_timer_id: u64::MAX,
            delay_id: 0,
            one_step: 1,
            all_deltatime: 0,
            len: 0,
        }
    }

    /// 获取计时器轮的长度
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::<u64>::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     assert!(timer.is_empty());
    ///     timer.add_timer(1);
    ///     assert_eq!(timer.len(), 1);
    ///     let t = timer.add_timer(2);
    ///     assert_eq!(timer.len(), 2);
    ///     timer.del_timer(t);
    ///     assert_eq!(timer.len(), 1);
    /// }
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// 是否为空
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::<u64>::new();
    ///     assert!(timer.is_empty());
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 清除所有的槽位
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::<u64>::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     assert!(timer.is_empty());
    ///     timer.add_timer(1);
    ///     timer.add_timer(2);
    ///     assert_eq!(timer.len(), 2);
    ///     timer.clear();
    ///     assert_eq!(timer.len(), 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        let mut wheel = self.lessest;
        while !wheel.is_null() {
            unsafe {
                (*wheel).clear();
                wheel = (*wheel).parent;
            }
        }
        self.len = 0;
    }

    pub fn get_one_step(&self) -> u64 {
        self.one_step
    }

    pub fn set_one_step(&mut self, step: u64) {
        self.one_step = step.max(1);
    }
    /// 添加计时器轮, 设置槽位和精度值, 名字用来辅助
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     timer.append_timer_wheel(60, "MinuteWheel");
    ///     timer.append_timer_wheel(12, "HourWheel");
    ///     timer.add_timer(30);
    /// }
    pub fn append_timer_wheel(&mut self, slots: u64, name: &'static str) {
        debug_assert!(self.len == 0, "必须时轮为空才可改变时轮");
        let step = if self.greatest.is_null() {
            self.one_step
        } else {
            let mut now_step = 1;
            let mut node = self.greatest;
            unsafe {
                while !node.is_null() {
                    now_step *= (*node).capation;
                    node = (*node).child;
                }
            }
            now_step / self.one_step
        };
        let one = Box::into_raw(Box::new(OneTimerWheel::new(slots, step, self.one_step, name)));
        self.delay_id = self.delay_id.max(slots * step);
        if self.lessest.is_null() {
            self.lessest = one;
            self.greatest = one;
        } else {
            unsafe {
                let child = self.greatest;
                (*one).append(child);
                self.greatest = one;
            }
        }
    }

    /// 计时器轮的递进时间
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     timer.add_timer(30);
    ///     let val = timer.update_deltatime(30).unwrap();
    ///     assert_eq!(val, vec![(1, 30)]);
    /// }
    pub fn update_deltatime(&mut self, delta: u64) -> Option<Vec<(u64, T)>> {
        debug_assert!(self.one_step > 0);
        self.update_now(self.all_deltatime.wrapping_add(delta))
    }

    /// 计时器轮的递进时间
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     timer.add_timer(30);
    ///     let val = timer.update_deltatime(30).unwrap();
    ///     assert_eq!(val, vec![(1, 30)]);
    /// }
    pub fn update_now(&mut self, now: u64) -> Option<Vec<(u64, T)>> {
        debug_assert!(self.one_step > 0);
        self.all_deltatime = now;
        let mut offset = self.all_deltatime / self.one_step;
        if offset < self.delay_id {
            return None;
        }

        self.all_deltatime -= offset * self.one_step;
        let mut remainder = 0;
        let mut result = vec![];
        let mut wheel = self.lessest;
        while !wheel.is_null() {
            unsafe {
                (offset, remainder) = (*wheel).update_index(offset, remainder, &mut result);
                // println!("offset = {offset} remainder = {remainder}, wheel = {}", (*wheel).name);
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

    /// 计时器轮的递进时间
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     timer.add_timer(30);
    ///     let mut idx = 0;
    ///     timer.update_deltatime_with_callback(30, &mut |_, id, v| {
    ///         idx = v;
    ///         None
    ///     });
    ///     assert_eq!(idx, 30);
    /// }
    pub fn update_deltatime_with_callback<F>(&mut self, delta: u64, f: &mut F)
    where
        F: FnMut(&mut Self, u64, T) -> Option<(u64, T)>,
    {
        debug_assert!(self.one_step > 0);
        self.update_now_with_callback(self.all_deltatime.wrapping_add(delta), f);
    }

    /// 计时器轮的递进时间
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     timer.add_timer(30);
    ///     let mut idx = 0;
    ///     timer.update_deltatime_with_callback(30, &mut |_, _, v| {
    ///         idx = v;
    ///         None
    ///     });
    ///     assert_eq!(idx, 30);
    /// }
    pub fn update_now_with_callback<F>(&mut self, now: u64, f: &mut F)
    where
        F: FnMut(&mut Self, u64, T) -> Option<(u64, T)>,
    {
        debug_assert!(self.one_step > 0);
        if let Some(result) = self.update_now(now) {
            let mut collect_result = vec![];
            for r in result.into_iter() {
                if let Some(v) = (*f)(self, r.0, r.1) {
                    collect_result.push(v);
                }
            }
            for (timer_id, val) in collect_result.drain(..) {
                self.add_timer_by_id(timer_id, val);
            }
        }
    }

    /// 计算下一个delay_id, 根据容器的密度稀疏有关
    /// 密度高的基本为O(1)的复杂度, 最差情况为O(n)的复杂度
    /// 总刻度数以时钟为计秒轮遍历60次,分轮遍历60次,时轮遍历12次,即最高遍历132次
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     timer.add_timer(30);
    ///     assert_eq!(timer.get_delay_id(), 30);
    /// }
    pub fn calc_delay_id(&mut self) {
        let mut next_delay_id = 0;
        let mut wheel = self.lessest;
        'outer: while !wheel.is_null() {
            unsafe {
                let (step, index, cap) = ((*wheel).step, (*wheel).index, (*wheel).num);
                for i in 0..cap {
                    let index = (index + i) % cap;
                    if !(*wheel).slots[index as usize].is_empty() {
                        next_delay_id = (i + 1) * step;
                        break 'outer;
                    }
                }
                next_delay_id = cap * step;
                wheel = (*wheel).parent;
            }
        }
        self.delay_id = next_delay_id / self.one_step;
    }

    /// 删除指定的定时器，时间复杂度为O(n)，
    /// 该模型删除不具备优势，需要频繁删除请选用其它时间框架
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     let t = timer.add_timer(30);
    ///     timer.del_timer(t);
    ///     assert_eq!(timer.len(), 0);
    /// }
    pub fn del_timer(&mut self, timer_id: u64) -> Option<T> {
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
    /// 该模型获取不具备优势，需要频繁获取请选用其它时间框架
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     let t = timer.add_timer(30);
    ///     assert_eq!(timer.get_timer(&t), Some(&30));
    /// }
    pub fn get_timer(&self, timer_id: &u64) -> Option<&T> {
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
    /// 该模型获取不具备优势，需要频繁获取请选用其它时间框架
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     let t = timer.add_timer(30);
    ///     *timer.get_mut_timer(&t).unwrap() = 33;
    ///     let val = timer.update_deltatime(30).unwrap();
    ///     assert_eq!(val, vec![(1, 33)]);
    /// }
    pub fn get_mut_timer(&mut self, timer_id: &u64) -> Option<&mut T> {
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

    pub fn get_max_timerid(&self) -> u64 {
        self.max_timer_id
    }

    pub fn set_max_timerid(&mut self, max: u64) {
        self.max_timer_id = max;
    }

    fn get_next_timerid(&mut self) -> u64 {
        let timer_id = self.next_timer_id;
        if self.next_timer_id >= self.max_timer_id {
            self.next_timer_id = 1;
        } else {
            self.next_timer_id = self.next_timer_id + 1;
        }
        timer_id
    }

    /// 添加定时器元素, 时间复杂度为O(1)
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     timer.add_timer(30);
    /// }
    pub fn add_timer(&mut self, val: T) -> u64 {
        debug_assert!(!self.greatest.is_null(), "必须设置时轮才能添加元素");
        let timer_id: u64 = self.get_next_timerid();
        self.add_timer_by_id(timer_id, val);
        timer_id
    }

    pub fn add_timer_by_id(&mut self, timer_id: u64, mut val: T) {
        let entry = Entry {
            when: val.when_mut().max(1),
            val,
            id: timer_id,
        };
        self.delay_id = self.delay_id.min(entry.when / self.one_step);
        unsafe {
            (*self.greatest).add_timer(entry);
        }
        self.len += 1;
    }

    /// 获取下一个延时
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerWheel;
    /// fn main() {
    ///     let mut timer = TimerWheel::new();
    ///     timer.append_timer_wheel(60, "SecondWheel");
    ///     timer.add_timer(30);
    ///     assert_eq!(timer.get_delay_id(), 30);
    /// }
    pub fn get_delay_id(&self) -> u64 {
        self.delay_id
    }
}

impl<T: Timer> Display for TimerWheel<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TimerWheel {\r\n")?;
        let mut wheel = self.greatest;
        while !wheel.is_null() {
            unsafe {
                f.write_fmt(format_args!(
                    "{}, slots: {}, step: {}",
                    (*wheel).name,
                    (*wheel).slots.len(),
                    (*wheel).step
                ))?;
                wheel = (*wheel).child;
            }
        }
        f.write_str("}")
    }
}

impl<T: Timer> Drop for TimerWheel<T> {
    fn drop(&mut self) {
        let mut wheel = self.greatest;
        while !wheel.is_null() {
            unsafe {
                let val = *Box::from_raw(wheel);
                wheel = val.child;
            }
        }
    }
}
