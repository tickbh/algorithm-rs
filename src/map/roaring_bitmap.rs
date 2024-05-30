use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};


const TAIL_NUM: usize = 0x10000;

#[derive(Clone)]
enum TailContainer {
    Array(Vec<u16>),
    Hash(HashSet<u16>),
}

impl TailContainer {
    fn new() -> Self {
        Self::Array(vec![])
    }

    fn try_move(&mut self) {
        let val = match self {
            TailContainer::Array(v) if v.len() >= 4096 => {
                v.drain(..).collect::<Vec<_>>()
            }
            _ => {
                return;
            }

        };
        let hash = HashSet::from_iter(val.into_iter());
        *self = TailContainer::Hash(hash);
    }

    pub fn add(&mut self, val: u16) -> bool {
        self.try_move();
        match self {
            TailContainer::Array(vec) => {
                if let Err(s) = vec.binary_search(&val) {
                    vec.insert(s, val);
                    true
                } else {
                    false
                }
            },
            TailContainer::Hash(hash) => hash.insert(val)
        }
    }

    pub fn remove(&mut self, val: u16) -> bool {
        match self {
            TailContainer::Array(vec) => {
                if let Ok(s) = vec.binary_search(&val) {
                    vec.remove(s);
                    true
                } else {
                    false
                }
            },
            TailContainer::Hash(hash) => hash.remove(&val)
        }
    }

    pub fn next(&self, val: u16) -> Option<u16> {
        match self {
            TailContainer::Array(vec) => {
                match vec.binary_search(&val) {
                    Ok(s) => { return Some(vec[s]) },
                    Err(s) => {
                        if s == vec.len() {
                            return None;
                        }
                        return Some(vec[s]);
                    }
                }
            },
            TailContainer::Hash(hash) => {
                for i in val..=65535u16 {
                    if hash.contains(&i) {
                        return Some(i);
                    }
                }
                return None;
            }
        }
    }


    pub fn next_back(&self, val: u16) -> Option<u16> {
        match self {
            TailContainer::Array(vec) => {
                match vec.binary_search(&val) {
                    Ok(s) => { return Some(vec[s]) },
                    Err(s) => {
                        if s == 0 {
                            return None;
                        }
                        return Some(vec[s - 1]);
                    }
                }
            },
            TailContainer::Hash(hash) => {
                for i in (0..=val).rev() {
                    if hash.contains(&i) {
                        return Some(i);
                    }
                }
                return None;
            }
        }
    }

    pub fn contains(&self, val: u16) -> bool {
        match self {
            TailContainer::Array(vec) => {
                if let Ok(_) = vec.binary_search(&val) {
                    true
                } else {
                    false
                }
            },
            TailContainer::Hash(hash) => hash.contains(&val)
        }
    }
}
/// 位图类RoaringBitMap，根据访问的位看是否被占用
/// 解决经典的是否被占用的问题，不会一次性分配大内存
/// 头部以val / 65536做为索引键值, 尾部分为Array及HashSet结构
/// 当元素个数小于4096时以有序array做为索引, 当>4096以HashSet做为存储
///
/// # Examples
///
/// ```
/// use algorithm::RoaringBitMap;
/// fn main() {
///     let mut map = RoaringBitMap::new();
///     map.add_many(&vec![1, 2, 3, 4, 10]);
///     assert!(map.contains(&1));
///     assert!(!map.contains(&5));
///     assert!(map.contains(&10));
///     map.add_range(7, 16);
///     assert!(!map.contains(&6));
///     assert!(map.contains(&7));
///     assert!(map.contains(&16));
///     assert!(!map.contains(&17));
/// }
/// ```
pub struct RoaringBitMap {
    map: HashMap<usize, TailContainer>,
    len: usize,
    max_key: usize,
    min_key: usize,
}

impl RoaringBitMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            len: 0,
            max_key: 0,
            min_key: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn clear(&mut self) {
        self.map.clear();
        self.len = 0;
    }

    /// 添加新的元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add(1);
    ///     assert!(map.contains(&1));
    ///     assert!(map.len() == 1);
    /// }
    /// ```
    pub fn add(&mut self, val: usize) {
        let head = val >> 16;
        let tail = (val % TAIL_NUM) as u16;
        if self.map.entry(head).or_insert(TailContainer::new()).add(tail) {
            self.len += 1;
            self.min_key = self.min_key.min(val);
            self.max_key = self.max_key.max(val);
        }
    }

    /// 添加许多新的元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_many(&vec![1, 2, 3, 4, 10]);
    ///     assert!(map.contains(&1));
    ///     assert!(map.contains(&10));
    ///     assert!(map.len() == 5);
    /// }
    /// ```
    pub fn add_many(&mut self, val: &[usize]) {
        for v in val {
            self.add(*v);
        }
    }

    /// 添加范围内的元素(包含头与结果)，批量添加增加效率
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_range(7, 16);
    ///     assert!(!map.contains(&6));
    ///     assert!(map.contains(&7));
    ///     assert!(map.contains(&16));
    ///     assert!(!map.contains(&17));
    ///     assert!(map.len() == 10);
    /// }
    /// ```
    pub fn add_range(&mut self, start: usize, end: usize) {
        for i in start..=end {
            self.add(i);
        }
    }

    /// 删除元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_range(7, 16);
    ///     assert!(map.len() == 10);
    ///     assert!(map.contains(&7));
    ///     assert!(map.remove(7));
    ///     assert!(!map.contains(&7));
    ///     assert!(map.len() == 9);
    /// }
    /// ```
    pub fn remove(&mut self, val: usize) -> bool {
        let head = val >> 16;
        let tail = (val % TAIL_NUM) as u16;
        if let Some(map) = self.map.get_mut(&head) {
            if map.remove(tail) {
                self.len -= 1;
                return true;
            }
        }
        false
    }

    /// 删除列表中元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_range(7, 16);
    ///     assert!(map.len() == 10);
    ///     assert!(map.contains(&7));
    ///     assert!(map.remove(7));
    ///     assert!(!map.contains(&7));
    ///     assert!(map.len() == 9);
    /// }
    /// ```
    pub fn remove_many(&mut self, val: &[usize]) {
        for v in val {
            self.remove(*v);
        }
    }

    /// 删除范围元素（包含头与尾）
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_range(7, 16);
    ///     assert!(map.len() == 10);
    ///     map.remove_range(7, 15);
    ///     assert!(map.len() == 1);
    ///     assert!(map.contains(&16));
    /// }
    /// ```
    pub fn remove_range(&mut self, start: usize, end: usize) {
        for i in start..=end {
            self.remove(i);
        }
    }

    /// 醒看是否包含
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add(7);
    ///     assert!(map.contains(&7));
    /// }
    /// ```
    pub fn contains(&self, val: &usize) -> bool {
        let head = val >> 16;
        let tail = (val % TAIL_NUM) as u16;
        if let Some(map) = self.map.get(&head) {
            map.contains(tail)
        } else {
            false
        }
    }

    /// 迭代器，通过遍历进行循环，如果位图的容量非常大，可能效率相当低
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add(7);
    ///     map.add_range(9, 12);
    ///     map.add_many(&vec![20, 100, 300]);
    ///     assert!(map.iter().collect::<Vec<_>>() == vec![7, 9, 10, 11, 12, 20, 100, 300]);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            base: self,
            len: self.len,
            min_val: self.min_key,
            max_val: self.max_key,
        }
    }


    /// 是否保留，通过遍历进行循环，如果位图的容量非常大，可能效率相当低
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_range(9, 16);
    ///     map.retain(|v| v % 2 == 0);
    ///     assert!(map.iter().collect::<Vec<_>>() == vec![10, 12, 14, 16]);
    /// }
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
        where
            F: FnMut(&usize) -> bool,
    {
        let mut oper = self.len;
        for i in self.min_key..=self.max_key {
            if oper == 0 {
                break;
            }
            if self.contains(&i) {
                oper -= 1;
                if !f(&i) {
                    self.remove(i);
                }
            }
        }
    }

    /// 是否为子位图
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_range(9, 16);
    ///     let mut sub_map = RoaringBitMap::new();
    ///     sub_map.add_range(9, 12);
    ///     assert!(map.contains_sub(&sub_map));
    /// }
    /// ```
    pub fn contains_sub(&self, other: &RoaringBitMap) -> bool {
        other.iter().all(|k| self.contains(&k))
    }

    /// 取两个位图间的交集
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_range(9, 16);
    ///     let mut sub_map = RoaringBitMap::new();
    ///     sub_map.add_range(7, 12);
    ///     let map = map.intersect(&sub_map);
    ///     assert!(map.iter().collect::<Vec<_>>() == vec![9, 10, 11, 12]);
    /// }
    /// ```
    pub fn intersect(&self, other: &RoaringBitMap) -> RoaringBitMap {
        let mut map = RoaringBitMap::new();
        let min = self.min_key.max(other.min_key);
        let max = self.max_key.min(other.max_key);
        for i in min..=max {
            if self.contains(&i) && other.contains(&i) {
                map.add(i);
            }
        }
        map
    }

    /// 取两个位图间的交集
    /// # Examples
    ///
    /// ```
    /// use algorithm::RoaringBitMap;
    /// fn main() {
    ///     let mut map = RoaringBitMap::new();
    ///     map.add_range(9, 16);
    ///     let mut sub_map = RoaringBitMap::new();
    ///     sub_map.add_range(7, 12);
    ///     let map = map.union(&sub_map);
    ///     assert!(map.iter().collect::<Vec<_>>() == vec![7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    /// }
    /// ```
    pub fn union(&self, other: &RoaringBitMap) -> RoaringBitMap {
        let mut map = RoaringBitMap::new();
        let min = self.min_key.min(other.min_key);
        let max = self.max_key.max(other.max_key);
        for i in min..=max {
            if self.contains(&i) || other.contains(&i) {
                map.add(i);
            }
        }
        map
    }
}

impl Clone for RoaringBitMap {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            len: self.len,
            max_key: self.max_key,
            min_key: self.min_key,
        }
    }
}


impl FromIterator<usize> for RoaringBitMap {
    fn from_iter<T: IntoIterator<Item=usize>>(iter: T) -> RoaringBitMap {
        let vec = iter.into_iter().collect::<Vec<_>>();
        let mut cap = 1024;
        for v in &vec {
            cap = cap.max(*v);
        }
        let mut map = RoaringBitMap::new();
        map.extend(vec);
        map
    }
}

impl PartialEq for RoaringBitMap {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().all(|k| other.contains(&k))
    }
}

impl Eq for RoaringBitMap {}

impl Extend<usize> for RoaringBitMap {
    fn extend<T: IntoIterator<Item=usize>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        for v in iter {
            self.add(v);
        }
    }
}

pub struct Iter<'a> {
    base: &'a RoaringBitMap,
    len: usize,
    min_val: usize,
    max_val: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        while self.min_val <= self.base.max_key {
            let head = self.min_val >> 16;
            if !self.base.map.contains_key(&head) {
                self.min_val = (head + 1) * TAIL_NUM;
                continue;
            }
            let tail = (self.min_val % TAIL_NUM) as u16;
            let container = self.base.map.get(&head).expect("ok");
            if let Some(i) = container.next(tail) {
                self.min_val = head * TAIL_NUM + i as usize + 1;
                self.len -= 1;
                return Some(head * TAIL_NUM + i as usize);
            } else {
                self.min_val = (head + 1) * TAIL_NUM;
                continue;
            }
        }
        unreachable!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        loop {
            let head = self.max_val >> 16;
            if !self.base.map.contains_key(&head) {
                self.max_val = (head * TAIL_NUM).saturating_sub(1);
                continue;
            }
            let tail = (self.max_val % TAIL_NUM) as u16;
            let container = self.base.map.get(&head).expect("ok");
            if let Some(i) = container.next_back(tail) {
                self.max_val = (head * TAIL_NUM + i as usize).saturating_sub(1);
                self.len -= 1;
                return Some(head * TAIL_NUM + i as usize);
            } else {
                self.max_val = (head * TAIL_NUM).saturating_sub(1);
                continue;
            }
        }
    }
}

impl Display for RoaringBitMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("len:{}-val:{{", self.len))?;
        let mut iter = self.iter();
        if let Some(v) = iter.next() {
            f.write_str(&v.to_string())?;
        }
        let mut sum = 1;
        while let Some(v) = iter.next() {
            f.write_fmt(format_args!(",{}", v))?;
            sum += 1;
            if sum > 0x100000 {
                break;
            }
        }
        f.write_str("}")
    }
}

impl Debug for RoaringBitMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self))
    }
}



#[cfg(test)]
mod tests {

    use super::RoaringBitMap;

    #[test]
    fn test_display() {
        let mut m = RoaringBitMap::new();
        m.add_many(&vec![1, 3, 9, 10240000111]);
        assert_eq!(format!("{}", m), "len:4-val:{1,3,9,10240000111}".to_string());
    }

    #[test]
    fn test_nextback() {
        let mut m = RoaringBitMap::new();
        m.add_many(&vec![1, 3, 9, 10240000111]);
        let vec = m.iter().rev().collect::<Vec<_>>();
        assert_eq!(vec, vec![10240000111, 9, 3, 1]);
    }
}