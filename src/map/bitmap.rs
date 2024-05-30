use std::fmt::{Debug, Display};

/// 位图类，根据访问的位看是否被占用
/// 解决经典的是否被占用的问题，但是相对占用大小会较大
/// 
/// # Examples
/// 
/// ```
/// use algorithm::BitMap;
/// fn main() {
///     let mut map = BitMap::new(10240);
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
pub struct BitMap {
    entries: Vec<u8>,
    cap: usize,
    len: usize,
    max_key: usize,
    min_key: usize,
}

impl BitMap {
    pub fn new(cap: usize) -> Self {
        let len = cap / 8 + if cap % 8 == 0 { 0 } else { 1 };
        Self {
            entries: vec![0; len],
            cap,
            len: 0,
            max_key: 0,
            min_key: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn clear(&mut self) {
        self.entries.fill(0);
        self.len = 0;
    }

    /// 添加新的元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
    ///     map.add(1);
    ///     assert!(map.contains(&1));
    ///     assert!(map.len() == 1);
    /// }
    /// ```
    pub fn add(&mut self, val: usize) {
        let pos = val / 8;
        let mask = 1 << val % 8;
        if self.entries[pos] & mask == 0 {
            self.len += 1;
            self.max_key = self.max_key.max(val);
            self.min_key = self.min_key.min(val);
        }
        self.entries[pos] = self.entries[pos] | mask;
    }

    /// 添加许多新的元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
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
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
    ///     map.add_range(7, 16);
    ///     assert!(!map.contains(&6));
    ///     assert!(map.contains(&7));
    ///     assert!(map.contains(&16));
    ///     assert!(!map.contains(&17));
    ///     assert!(map.len() == 10);
    /// }
    /// ```
    pub fn add_range(&mut self, start: usize, end: usize) {
        for pos in start..end.min((start / 8 + 1) * 8) {
            self.add(pos)
        }
        for pos in (start / 8 + 1)..end / 8 {
            self.len += 8 - self.entries[pos].count_ones() as usize;
            self.entries[pos] = u8::MAX;
        }
        for pos in start.max((end / 8) * 8)..=end {
            self.add(pos)
        }

        self.min_key = self.min_key.max(start);
        self.max_key = self.max_key.max(end);
    }

    /// 删除元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
    ///     map.add_range(7, 16);
    ///     assert!(map.len() == 10);
    ///     assert!(map.contains(&7));
    ///     assert!(map.remove(7));
    ///     assert!(!map.contains(&7));
    ///     assert!(map.len() == 9);
    /// }
    /// ```
    pub fn remove(&mut self, val: usize) -> bool {
        let pos = val / 8;
        let mask = 1 << val % 8;
        let mut success = false;
        if self.entries[pos] & mask != 0 {
            self.len -= 1;
            success = true;
        }
        self.entries[pos] = self.entries[pos] & (u8::MAX - mask);
        success
    }

    /// 删除列表中元素
    /// # Examples
    ///
    /// ```
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
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
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
    ///     map.add_range(7, 16);
    ///     assert!(map.len() == 10);
    ///     map.remove_range(7, 15);
    ///     assert!(map.len() == 1);
    ///     assert!(map.contains(&16));
    /// }
    /// ```
    pub fn remove_range(&mut self, start: usize, end: usize) {
        for pos in start..end.min((start / 8 + 1) * 8) {
            self.remove(pos);
        }
        for pos in (start / 8 + 1)..end / 8 {
            self.len -= self.entries[pos].count_ones() as usize;
            self.entries[pos] = 0;
        }
        for pos in start.max((end / 8) * 8)..=end {
            self.remove(pos);
        }
    }

    /// 醒看是否包含
    /// # Examples
    ///
    /// ```
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
    ///     map.add(7);
    ///     assert!(map.contains(&7));
    /// }
    /// ```
    pub fn contains(&self, val: &usize) -> bool {
        let pos = val / 8;
        (self.entries[pos] & (1 << val % 8)) != 0
    }

    /// 迭代器，通过遍历进行循环，如果位图的容量非常大，可能效率相当低
    /// # Examples
    ///
    /// ```
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
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
            val: self.min_key,
        }
    }


    /// 是否保留，通过遍历进行循环，如果位图的容量非常大，可能效率相当低
    /// # Examples
    ///
    /// ```
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
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
        for i in 0..self.cap {
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
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
    ///     map.add_range(9, 16);
    ///     let mut sub_map = BitMap::new(10240);
    ///     sub_map.add_range(9, 12);
    ///     assert!(map.contains_sub(&sub_map));
    /// }
    /// ```
    pub fn contains_sub(&self, other: &BitMap) -> bool {
        other.iter().all(|k| self.contains(&k))
    }

    /// 取两个位图间的交集
    /// # Examples
    ///
    /// ```
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
    ///     map.add_range(9, 16);
    ///     let mut sub_map = BitMap::new(10240);
    ///     sub_map.add_range(7, 12);
    ///     let map = map.intersect(&sub_map);
    ///     assert!(map.iter().collect::<Vec<_>>() == vec![9, 10, 11, 12]);
    /// }
    /// ```
    pub fn intersect(&self, other: &BitMap) -> BitMap {
        let mut map = BitMap::new(other.cap.max(self.cap));
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
    /// use algorithm::BitMap;
    /// fn main() {
    ///     let mut map = BitMap::new(10240);
    ///     map.add_range(9, 16);
    ///     let mut sub_map = BitMap::new(10240);
    ///     sub_map.add_range(7, 12);
    ///     let map = map.union(&sub_map);
    ///     assert!(map.iter().collect::<Vec<_>>() == vec![7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    /// }
    /// ```
    pub fn union(&self, other: &BitMap) -> BitMap {
        let mut map = BitMap::new(other.cap.max(self.cap));
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

impl Clone for BitMap {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            cap: self.cap,
            len: self.len,
            max_key: self.max_key,
            min_key: self.min_key,
        }
    }
}


impl FromIterator<usize> for BitMap {
    fn from_iter<T: IntoIterator<Item=usize>>(iter: T) -> BitMap {
        let vec = iter.into_iter().collect::<Vec<_>>();
        let mut cap = 1024;
        for v in &vec {
            cap = cap.max(*v);
        }
        let mut lru = BitMap::new(cap);
        lru.extend(vec);
        lru
    }
}

impl PartialEq for BitMap {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().all(|k| other.contains(&k))
    }
}

impl Eq for BitMap {}

impl Extend<usize> for BitMap {
    fn extend<T: IntoIterator<Item=usize>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        for v in iter {
            self.add(v);
        }
    }
}

pub struct Iter<'a> {
    base: &'a BitMap,
    len: usize,
    val: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        for i in self.val..=self.base.max_key {
            if self.base.contains(&i) {
                self.len -= 1;
                self.val = i + 1;
                return Some(i);
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
        for i in (0..=(self.base.max_key - self.val)).rev() {
            if self.base.contains(&i) {
                self.len -= 1;
                self.val = self.base.cap - i;
                return Some(i);
            }
        }
        unreachable!()
    }
}

impl Display for BitMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("len:{}-val:{{", self.len))?;
        let mut iter = self.iter();
        if let Some(v) = iter.next() {
            f.write_str(&v.to_string())?;
        }
        let mut sum = 1;
        while let Some(v) = iter.next() {
            f.write_fmt(format_args!(",{} ", v))?;
            sum += 1;
            if sum > 0x100000 {
                break;
            }
        }
        f.write_str("}")
    }
}

impl Debug for BitMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self))
    }
}
