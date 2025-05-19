use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{borrow::Borrow, hash::Hash, mem, usize};

use crate::arr::SkipIter;
use crate::{KeyRef, KeyWrapper, SkipList, SkipNode};

struct Context<K: Hash> {
    key: mem::MaybeUninit<K>,
    score: isize,
    timestamp: usize,
}

impl<K: Hash> Default for Context<K> {
    fn default() -> Self {
        Self {
            key: mem::MaybeUninit::uninit(),
            score: Default::default(),
            timestamp: Default::default(),
        }
    }
}

impl<K: Hash> PartialEq for Context<K> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.timestamp == other.timestamp
    }
}

impl<K: Hash> PartialOrd for Context<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.score.partial_cmp(&other.score) {
            Some(core::cmp::Ordering::Equal) => {
                return self.timestamp.partial_cmp(&other.timestamp)
            }
            ord => return ord,
        }
    }
}

/// 一种可排序的Set类型, 可以高效的对评分进行排序, 
///
/// # Examples
///
/// ```
/// use algorithm::ZSet;
/// fn main() {
///     let mut val = ZSet::new();
///     val.add_or_update("aa", 10);
///     val.add_or_update("bb", 12);
///     assert_eq!(val.len(), 2);
///     assert_eq!(val.rank(&"bb"), 2);
///     val.add_or_update("bb", 9);
///     assert_eq!(val.rank(&"bb"), 1);
///     assert_eq!(val.len(), 2);
/// }
/// ```
pub struct ZSet<K: Hash + Eq> {
    max_count: usize,
    reverse: bool,
    zsl: SkipList<Context<K>>,
    dict: HashMap<KeyRef<K>, *mut SkipNode<Context<K>>>,
}

impl<K: Hash + Eq> ZSet<K> {
    pub fn new() -> Self {
        Self {
            max_count: usize::MAX,
            reverse: false,
            zsl: SkipList::new(),
            dict: HashMap::new(),
        }
    }

    pub fn new_with(max_count: usize, reverse: bool) -> Self {
        Self {
            max_count,
            reverse,
            zsl: SkipList::new(),
            dict: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        assert!(self.dict.len() == self.zsl.len());
        self.dict.len()
    }

    /// 清除集合
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::ZSet;
    /// fn main() {
    ///     let mut val = ZSet::new();
    ///     val.add_or_update("aa", 10);
    ///     val.add_or_update("bb", 12);
    ///     assert_eq!(val.len(), 2);
    ///     val.clear();
    ///     assert_eq!(val.len(), 0);
    /// }
    /// ```
    ///
    pub fn clear(&mut self) {
        self.dict.clear();
        self.zsl.clear();
    }

    /// 包含键值
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::ZSet;
    /// fn main() {
    ///     let mut val = ZSet::new();
    ///     val.add_or_update("aa", 10);
    ///     val.add_or_update("bb", 12);
    ///     assert_eq!(val.contains_key(&"aa"), true);
    /// }
    /// ```
    ///
    pub fn contains_key<Q>(&mut self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.dict.contains_key(KeyWrapper::from_ref(k))
    }

    /// 获取排序值
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::ZSet;
    /// fn main() {
    ///     let mut val = ZSet::new();
    ///     val.add_or_update("aa", 10);
    ///     val.add_or_update("bb", 12);
    ///     assert_eq!(val.len(), 2);
    ///
    /// }
    /// ```
    pub fn rank<Q>(&mut self, k: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(v) = self.dict.get(KeyWrapper::from_ref(k)) {
            return self.zsl.get_rank(unsafe { &(**v).score });
        }
        0
    }

    /// 删除元素
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::ZSet;
    /// fn main() {
    ///     let mut val = ZSet::new();
    ///     val.add_or_update("aa", 10);
    ///     val.add_or_update("bb", 12);
    ///     assert_eq!(val.len(), 2);
    ///     assert!(val.remove(&"bb"));
    ///     assert_eq!(val.len(), 1);
    /// }
    /// ```
    pub fn remove<Q>(&mut self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(v) = self.dict.remove(KeyWrapper::from_ref(k)) {
            self.zsl.remove(unsafe { &(*v).score })
        } else {
            false
        }
    }

    /// 添加或者更新值
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::ZSet;
    /// fn main() {
    ///     let mut val = ZSet::new();
    ///     val.add_or_update("aa", 10);
    ///     val.add_or_update("bb", 12);
    ///     assert_eq!(val.len(), 2);
    ///     val.add_or_update("bb", 14);
    ///     assert_eq!(val.len(), 2);
    ///     assert_eq!(val.score(&"bb"), 14);
    ///
    /// }
    /// ```
    pub fn add_or_update(&mut self, key: K, mut score: isize) {
        if self.max_count == 0 || self.max_count == self.dict.len() {
            return;
        }

        if self.reverse {
            score = -score;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let context = Context {
            key: mem::MaybeUninit::new(key),
            score,
            timestamp: now.as_millis() as usize,
        };

        let key_ref = KeyRef::new(context.key.as_ptr());
        if let Some(v) = self.dict.remove(&key_ref) {
            let ret = self.zsl.update(unsafe { &(*v).score }, context);
            let key_ref = KeyRef::new(unsafe { (*ret).score.key.as_ptr() });
            self.dict.insert(key_ref, ret);
        } else {
            let ret = self.zsl.insert(context);
            let key_ref = KeyRef::new(unsafe { (*ret).score.key.as_ptr() });
            self.dict.insert(key_ref, ret);
        }
    }

    /// 获取score值
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::ZSet;
    /// fn main() {
    ///     let mut val = ZSet::new();
    ///     val.add_or_update("aa", 10);
    ///     val.add_or_update("bb", 12);
    ///     assert_eq!(val.score(&"bb"), 12);
    ///
    /// }
    /// ```
    pub fn score<Q>(&mut self, k: &Q) -> isize
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(v) = self.dict.get(KeyWrapper::from_ref(k)) {
            return unsafe { (**v).score.score };
        }
        0
    }

    /// 遍历值
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::ZSet;
    /// fn main() {
    ///     let mut val = ZSet::new();
    ///     val.add_or_update("aa", 10);
    ///     val.add_or_update("bb", 12);
    ///     let mut iter = val.iter();
    ///     assert_eq!(iter.next(), Some((&"aa", 0, 10)));
    ///     assert_eq!(iter.next(), Some((&"bb", 1, 12)));
    ///     assert_eq!(iter.next(), None);
    /// }
    /// ```
    pub fn iter(&self) -> ZSetIter<K> {
        ZSetIter::new(self.zsl.iter())
    }
}

impl<K: Hash + Eq> Drop for ZSet<K> {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct ZSetIter<'a, K: 'a + Hash + Eq> {
    iter: SkipIter<'a, Context<K>>,
    data: PhantomData<&'a ()>,
}

impl<'a, T: Hash + Eq> ZSetIter<'a, T> {
    fn new(iter: SkipIter<'a, Context<T>>) -> Self {
        Self {
            iter,
            data: PhantomData,
        }
    }
}

impl<'a, T: Hash + Eq> Iterator for ZSetIter<'a, T> {
    type Item = (&'a T, usize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => return None,
            Some((v, s)) => return Some((unsafe { v.key.assume_init_ref() }, s, v.score)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: Hash + Eq> DoubleEndedIterator for ZSetIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter.next_back() {
            None => return None,
            Some((v, s)) => return Some((unsafe { v.key.assume_init_ref() }, s, v.score)),
        }
    }
}
