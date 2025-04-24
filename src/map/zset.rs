use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
    mem, usize,
};

use hashbrown::HashMap;

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

    pub fn len(&mut self) -> usize {
        assert!(self.dict.len() == self.zsl.len());
        self.dict.len()
    }

    pub fn clear(&mut self) {
        self.dict.clear();
        self.zsl.clear();
    }

    pub fn contains_key<Q>(&mut self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.dict.contains_key(KeyWrapper::from_ref(k))
    }

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

    pub fn erase<Q>(&mut self, k: &Q) -> bool
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

    pub fn add_or_update(&mut self, key: K, mut score: isize) {
        if self.max_count == 0 || self.max_count == self.dict.len() {
            return;
        }

        if self.reverse {
            score = -score;
        }

        let context = Context {
            key: mem::MaybeUninit::new(key),
            score,
            timestamp: 0,
        };

        let key_ref = KeyRef::new(unsafe { &*context.key.as_ptr() });

        if let Some(v) = self.dict.remove(&key_ref) {
            let ret = self.zsl.update(unsafe { &(*v).score }, context);
            self.dict.insert(key_ref, ret);
        } else {
            let ret = self.zsl.insert(context);
            self.dict.insert(key_ref, ret);
        }
    }

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
}

impl<K: Hash + Eq> Drop for ZSet<K> {
    fn drop(&mut self) {
        self.clear();
    }
}
