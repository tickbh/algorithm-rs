use crate::HashMap;

use crate::RBTree;
use std::cmp::Ordering;
use std::vec;

use super::Timer;

#[derive(PartialEq, Eq)]
struct TreeKey(u64, u64);

impl Ord for TreeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 != other.0 {
            return self.0.cmp(&other.0);
        }
        other.1.cmp(&self.1)
    }
}

impl PartialOrd for TreeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
/// use algorithm::TimerRBTree;
/// fn main() {
///     let mut timer = TimerRBTree::new();
///     timer.add_timer(30);
///     assert_eq!(timer.tick_first(), Some(30));
///     timer.add_timer(149);
///     assert_eq!(timer.tick_first(), Some(30));
///     let t = timer.add_timer(600);
///     assert_eq!(timer.tick_first(), Some(30));
///     timer.add_timer(1);
///     assert_eq!(timer.tick_first(), Some(1));
///     timer.del_timer(t);
///     timer.add_timer(150);
///     assert_eq!(timer.tick_first(), Some(1));
///     let val = timer.update_deltatime(30).unwrap();
///     assert_eq!(val, vec![1, 30]);
///     timer.add_timer(2);
///     let val = timer.update_deltatime(119).unwrap();
///     assert_eq!(val, vec![2, 149]);
///     let val = timer.update_deltatime(1).unwrap();
///     assert_eq!(val, vec![150]);
///     assert!(timer.is_empty());
/// }
/// ```
pub struct TimerRBTree<T: Timer> {
    tree: RBTree<TreeKey, T>,

    map: HashMap<u64, u64>,

    /// 当时记录的时序
    cur_step: u64,

    /// id记录
    next_timer_id: u64,
}

impl<T: Timer> TimerRBTree<T> {
    pub fn new() -> Self {
        Self {
            tree: RBTree::new(),
            map: HashMap::new(),
            cur_step: 0,
            next_timer_id: 0,
        }
    }

    /// 获取定时器的长度
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::<u64>::new();
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
        self.tree.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// 清除所有的槽位
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::<u64>::new();
    ///     assert!(timer.is_empty());
    ///     timer.add_timer(1);
    ///     timer.add_timer(2);
    ///     assert_eq!(timer.len(), 2);
    ///     timer.clear();
    ///     assert_eq!(timer.len(), 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.tree.clear();
        self.map.clear();
        self.cur_step = 0;
        self.next_timer_id = 0;
    }

    /// 添加定时器元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::new();
    ///     timer.add_timer(30);
    ///     assert_eq!(timer.len(), 1);
    /// }
    pub fn add_timer(&mut self, val: T) -> u64 {
        let timer_id = self.next_timer_id;
        self.next_timer_id = self.next_timer_id.wrapping_add(1);
        let when = val.when();
        self.tree.insert(TreeKey(when, timer_id), val);
        self.map.insert(timer_id, when);
        timer_id
    }

    /// 删除指定的定时器，时间复杂度为O(logn)，
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::new();
    ///     let t = timer.add_timer(30);
    ///     timer.del_timer(t);
    ///     assert_eq!(timer.len(), 0);
    /// }
    pub fn del_timer(&mut self, timer_id: u64) -> Option<T> {
        if let Some(when) = self.map.remove(&timer_id) {
            let tree = TreeKey(when, timer_id);
            self.tree.remove(&tree).map(|e| e)
        } else {
            None
        }
    }

    /// 获取指定的定时器，时间复杂度为O(log(n))
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::new();
    ///     let t = timer.add_timer(30);
    ///     assert_eq!(timer.get_timer(&t), Some(&30));
    /// }
    pub fn get_timer(&self, timer_id: &u64) -> Option<&T> {
        if let Some(when) = self.map.get(timer_id) {
            let tree = TreeKey(*when, *timer_id);
            self.tree.get(&tree).map(|e| e)
        } else {
            None
        }
    }

    /// 获取指定的定时器，时间复杂度为O(log(n))
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::new();
    ///     let t = timer.add_timer(30);
    ///     *timer.get_mut_timer(&t).unwrap() = 33;
    ///     let val = timer.update_deltatime(30).unwrap();
    ///     assert_eq!(val, vec![33]);
    /// }
    pub fn get_mut_timer(&mut self, timer_id: &u64) -> Option<&mut T> {
        if let Some(when) = self.map.get(timer_id) {
            let tree = TreeKey(*when, *timer_id);
            self.tree.get_mut(&tree).map(|e| e)
        } else {
            None
        }
    }

    /// 取出时间轴最小的一个值
    pub fn tick_first(&self) -> Option<u64> {
        self.tree
            .get_first()
            .map(|(key, _)| Some(key.0))
            .unwrap_or(None)
    }

    /// 判断到指定时间是否有小于该指定值的实例
    pub fn tick_time(&mut self, tm: u64) -> Option<T> {
        if tm < self.tick_first().unwrap_or(tm + 1) {
            return None;
        }
        self.tree.pop_first().map(|(_, e)| e)
    }

    /// 计时器轮的递进时间
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::new();
    ///     timer.add_timer(30);
    ///     let val = timer.update_deltatime(30).unwrap();
    ///     assert_eq!(val, vec![30]);
    /// }
    pub fn update_now(&mut self, now: u64) -> Option<Vec<T>> {
        self.cur_step = now;
        let mut result = vec![];
        loop {
            if let Some(val) = self.tick_first() {
                if self.cur_step < val {
                    break;
                }
                result.push(self.tree.pop_first().map(|(_, e)| e).unwrap());
            } else {
                break;
            }
        }
        Some(result)
    }
    /// 计时器轮的递进时间
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::new();
    ///     timer.add_timer(30);
    ///     let val = timer.update_deltatime(30).unwrap();
    ///     assert_eq!(val, vec![30]);
    /// }
    pub fn update_deltatime(&mut self, delta: u64) -> Option<Vec<T>> {
        self.update_now(self.cur_step.wrapping_add(delta))
    }

    /// 计时器轮的递进时间
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::TimerRBTree;
    /// fn main() {
    ///     let mut timer = TimerRBTree::new();
    ///     timer.add_timer(30);
    ///     let mut idx = 0;
    ///     timer.update_deltatime_with_callback(30, &mut |_, v| {
    ///         idx = v;
    ///         None
    ///     });
    ///     assert_eq!(idx, 30);
    /// }
    pub fn update_deltatime_with_callback<F>(&mut self, delta: u64, f: &mut F)
    where
        F: FnMut(&mut Self, T) -> Option<T>,
    {
        self.update_now_with_callback(self.cur_step.wrapping_add(delta), f)
    }

    pub fn update_now_with_callback<F>(&mut self, now: u64, f: &mut F)
    where
        F: FnMut(&mut Self, T) -> Option<T>,
    {
        if let Some(result) = self.update_now(now) {
            let mut collect_result = vec![];
            for r in result.into_iter() {
                if let Some(v) = (*f)(self, r) {
                    collect_result.push(v);
                }
            }
            for v in collect_result.drain(..) {
                self.add_timer(v);
            }
        }
    }
}
