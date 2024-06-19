use std::cmp::Ordering;
use std::ptr::NonNull;

#[derive(Debug)]
struct FixedVecNode<T> {
    prev: usize,
    next: usize,
    data: T,
}

/// 指定位置的序列, 每个位置上指向上向的位置, 相当于模拟指针
/// 可以根据坐标位置获取相应数据, 亦可以操作上级及下级位置
/// 
/// # Examples
///
/// ```
/// use algorithm::FixedVec;
/// fn main() {
///     let mut val = FixedVec::new(2);
///     val.insert_head(1);
///     val.insert_head(2);
///     assert_eq!(val.len(), 2);
///     assert_eq!(val.head(), Some(&2));
///     assert_eq!(val.tail(), Some(&1));
///     assert_eq!(val.insert_head(3), None);
/// }
/// ```
#[derive(Debug)]
pub struct FixedVec<T> {
    capacity: usize,
    nodes: Vec<Option<FixedVecNode<T>>>,
    // 存储空闲位置, 用O(1)的时间复杂度取出空闲位置
    free: Vec<usize>,
    head: usize,
    tail: usize,
}

impl<T> FixedVec<T> {
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            nodes: Vec::new(),
            free: Vec::new(),
            head: usize::MAX,
            tail: usize::MAX,
        }
    }

    #[inline]
    pub fn with_memory(capacity: usize, mut reserve: usize) -> Self {
        reserve = reserve.min(capacity);
        Self {
            capacity,
            nodes: Vec::with_capacity(reserve),
            free: Vec::new(),
            head: usize::MAX,
            tail: usize::MAX,
        }
    }

    /// 获取容量
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 返回长度
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len() - self.free.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// 清除数据
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_head(1);
    ///     assert_eq!(val.len(), 1);
    ///     val.clear();
    ///     assert_eq!(val.len(), 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.free.clear();
        self.head = usize::MAX;
        self.tail = usize::MAX;
    }

    fn next(&mut self) -> Option<usize> {
        if self.is_full() {
            None
        } else if self.free.is_empty() {
            let len = self.len();
            self.nodes.push(None);
            Some(len)
        } else {
            self.free.pop()
        }
    }

    #[inline]
    fn node_ref(&self, idx: usize) -> Option<&FixedVecNode<T>> {
        self.nodes.get(idx).and_then(|node| node.as_ref())
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.node_ref(idx).map(|node| &node.data)
    }

    #[inline]
    fn node_mut(&mut self, idx: usize) -> Option<&mut FixedVecNode<T>> {
        self.nodes.get_mut(idx).and_then(|node| node.as_mut())
    }

    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.node_mut(idx).map(|node| &mut node.data)
    }
    /// 获取头部的坐标位置
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_tail(1);
    ///     assert_eq!(val.head_idx(), 0);
    /// }
    /// ```
    #[inline]
    pub fn head_idx(&self) -> usize {
        self.head
    }
    /// 获取头部首位数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_tail(1);
    ///     assert_eq!(val.head(), Some(&1));
    /// }
    /// ```
    #[inline]
    pub fn head(&self) -> Option<&T> {
        self.node_ref(self.head).map(|node| &node.data)
    }

    /// 获取头部首位可变数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_tail(1);
    ///     assert_eq!(val.head_mut(), Some(&mut 1));
    /// }
    /// ```
    #[inline]
    pub fn head_mut(&mut self) -> Option<&mut T> {
        self.node_mut(self.head).map(|node| &mut node.data)
    }

    /// 获取尾部的坐标位置
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_tail(1);
    ///     assert_eq!(val.tail_idx(), 0);
    /// }
    /// ```
    #[inline]
    pub fn tail_idx(&self) -> usize {
        self.tail
    }

    /// 获取尾部首位数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_head(1);
    ///     assert_eq!(val.tail(), Some(&1));
    /// }
    /// ```
    #[inline]
    pub fn tail(&self) -> Option<&T> {
        self.node_ref(self.tail).map(|node| &node.data)
    }

    /// 获取尾部首位可变数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_head(1);
    ///     assert_eq!(val.tail_mut(), Some(&mut 1));
    /// }
    /// ```
    #[inline]
    pub fn tail_mut(&mut self) -> Option<&mut T> {
        self.node_mut(self.tail).map(|node| &mut node.data)
    }

    /// 从头部插入新的数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     assert_eq!(val.insert_head(1), Some((0, &mut 1)));
    ///     assert_eq!(val.insert_head(2), Some((1, &mut 2)));
    ///     assert_eq!(val.insert_head(3), None);
    /// }
    /// ```
    pub fn insert_head(&mut self, data: T) -> Option<(usize, &mut T)> {
        let idx = self.next()?;
        if let Some(head) = self.node_mut(self.head) {
            head.prev = idx;
        }
        if self.node_ref(self.tail).is_none() {
            self.tail = idx;
        }
        let node = self.nodes.get_mut(idx).unwrap().insert(FixedVecNode {
            prev: usize::MAX,
            next: self.head,
            data,
        });
        self.head = idx;
        Some((idx, &mut node.data))
    }
    /// 从头部插入新的数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     assert_eq!(val.insert_tail(1), Some((0, &mut 1)));
    ///     assert_eq!(val.insert_tail(2), Some((1, &mut 2)));
    ///     assert_eq!(val.insert_tail(3), None);
    /// }
    /// ```
    pub fn insert_tail(&mut self, data: T) -> Option<(usize, &mut T)> {
        let idx = self.next()?;
        if let Some(tail) = self.node_mut(self.tail) {
            tail.next = idx;
        }
        if self.node_ref(self.head).is_none() {
            self.head = idx;
        }
        let node = self.nodes.get_mut(idx).unwrap().insert(FixedVecNode {
            prev: self.tail,
            next: usize::MAX,
            data,
        });
        self.tail = idx;
        Some((idx, &mut node.data))
    }

    /// 从头部弹出数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_tail(1);
    ///     val.insert_tail(2);
    ///     assert_eq!(val.pop_head(), Some(1));
    /// }
    /// ```
    #[inline]
    pub fn pop_head(&mut self) -> Option<T> {
        self.remove(self.head)
    }
    /// 从尾部弹出数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_head(1);
    ///     val.insert_head(2);
    ///     assert_eq!(val.pop_tail(), Some(1));
    /// }
    /// ```
    #[inline]
    pub fn pop_tail(&mut self) -> Option<T> {
        self.remove(self.tail)
    }

    /// 删除指定位置数据
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(2);
    ///     val.insert_head(1);
    ///     val.insert_head(2);
    ///     assert_eq!(val.remove(1), Some(2));
    ///     assert_eq!(val.len(), 1);
    ///     assert_eq!(val.tail_idx(), 0);
    /// }
    /// ```
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        let node = self.nodes.get_mut(idx)?.take()?;
        if let Some(prev) = self.node_mut(node.prev) {
            prev.next = node.next;
        } else {
            self.head = node.next;
        }
        if let Some(next) = self.node_mut(node.next) {
            next.prev = node.prev;
        } else {
            self.tail = node.prev;
        }
        self.free.push(idx);
        Some(node.data)
    }

    
    /// 迭代器
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(5);
    ///     val.insert_head(1);
    ///     val.insert_head(2);
    ///     val.insert_head(3);
    ///     assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<_>>(), vec![3, 2, 1]);
    /// }
    /// ```
    #[inline]
    pub fn iter(&self) -> FixedVecIter<'_, T> {
        FixedVecIter {
            list: self,
            head: self.head,
            tail: self.tail,
            len: self.len(),
        }
    }

    /// 迭代器
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(5);
    ///     val.insert_head(1);
    ///     val.insert_head(2);
    ///     val.insert_head(3);
    ///     let _ = val.iter_mut().map(|(_, v)| *v = *v * 2).collect::<Vec<_>>();
    ///     assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<_>>(), vec![6, 4, 2]);
    /// }
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> FixedVecIterMut<'_, T> {
        let head = self.head;
        let tail = self.tail;
        let len = self.len();
        FixedVecIterMut::new(&mut self.nodes, head, tail, len)
    }

    fn reorder(&mut self) {
        if self.is_empty() {
            return;
        }

        let len = self.len();
        let mut current = 0;
        while current < len {
            let head = self.head;
            let head_data = self.pop_head().unwrap();
            if head != current {
                debug_assert!(current < head, "{} < {}", current, head);
                if let Some(current_node) = self.nodes[current].take() {
                    if let Some(node) = self.node_mut(current_node.prev) {
                        node.next = head;
                    } else {
                        self.head = head;
                    }
                    if let Some(node) = self.node_mut(current_node.next) {
                        node.prev = head;
                    } else {
                        self.tail = head;
                    }
                    self.nodes[head] = Some(current_node);
                }
            }
            self.nodes[current] = Some(FixedVecNode {
                prev: current.wrapping_sub(1),
                next: current + 1,
                data: head_data,
            });
            current += 1;
        }
        self.head = 0;
        self.nodes[len - 1].as_mut().unwrap().next = usize::MAX;
        self.tail = len - 1;
        self.free.clear();
        self.free.extend((len..self.nodes.len()).rev());
    }

    
    /// 重置设置大小
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(5);
    ///     val.insert_head(1);
    ///     val.insert_head(2);
    ///     val.resize(3);
    ///     assert_eq!(val.len(), 2);
    ///     assert_eq!(val.head_idx(), 0);
    ///     assert_eq!(val.tail_idx(), 1);
    ///     assert_eq!(val.tail(), Some(&1));
    /// }
    /// ```
    pub fn resize(&mut self, capacity: usize) {
        let len = self.len();
        let cap = self.capacity();
        if capacity < len {
            return;
        }
        match capacity.cmp(&cap) {
            Ordering::Less => {
                self.reorder();
                self.nodes.truncate(capacity);
                self.free.clear();
                self.free.extend(len..self.nodes.len());
                self.capacity = capacity;
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                self.capacity = capacity;
            }
        };
        debug_assert_eq!(self.len(), len);
        debug_assert_eq!(self.capacity(), capacity);
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let mut head = self.head;
        while head != usize::MAX {
            let node = self.node_ref(head).unwrap();
            let next = node.next;
            if !f(&node.data) {
                self.remove(head);
            }
            head = next;
        }
    }

    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let mut head = self.head;
        while head != usize::MAX {
            let node = self.node_mut(head).unwrap();
            let next = node.next;
            if !f(&mut node.data) {
                self.remove(head);
            }
            head = next;
        }
    }

    /// 将指定位置挪到最前面
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(5);
    ///     val.insert_head(1);
    ///     val.insert_head(2);
    ///     val.insert_head(3);
    ///     assert_eq!(val.get(1), Some(&2));
    ///     assert_eq!(val.head(), Some(&3));
    ///     val.move_head(1);
    ///     assert_eq!(val.head(), Some(&2));
    ///     assert_eq!(val.get(1), Some(&2));
    ///     assert_eq!(val.tail(), Some(&1));
    /// }
    /// ```
    #[inline]
    pub fn move_head(&mut self, idx: usize) -> Option<&mut T> {
        let node = self.nodes.get_mut(idx)?.take()?;
        if let Some(prev) = self.node_mut(node.prev) {
            prev.next = node.next;
        } else {
            self.head = node.next;
        }
        if let Some(next) = self.node_mut(node.next) {
            next.prev = node.prev;
        } else {
            self.tail = node.prev;
        }

        if let Some(head) = self.node_mut(self.head) {
            head.prev = idx;
        }
        if self.node_ref(self.tail).is_none() {
            self.tail = idx;
        }

        let node = self.nodes.get_mut(idx).unwrap().insert(FixedVecNode {
            prev: usize::MAX,
            next: self.head,
            data: node.data,
        });
        self.head = idx;
        Some(&mut node.data)
    }

    
    /// 将指定位置挪到最后面
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::FixedVec;
    /// fn main() {
    ///     let mut val = FixedVec::new(5);
    ///     val.insert_tail(1);
    ///     val.insert_tail(2);
    ///     val.insert_tail(3);
    ///     assert_eq!(val.get(1), Some(&2));
    ///     assert_eq!(val.tail(), Some(&3));
    ///     val.move_tail(1);
    ///     assert_eq!(val.tail(), Some(&2));
    ///     assert_eq!(val.get(1), Some(&2));
    ///     assert_eq!(val.head(), Some(&1));
    /// }
    /// ```
    #[inline]
    pub fn move_tail(&mut self, idx: usize) -> Option<&mut T> {
        let node = self.nodes.get_mut(idx)?.take()?;
        if let Some(prev) = self.node_mut(node.prev) {
            prev.next = node.next;
        } else {
            self.head = node.next;
        }
        if let Some(next) = self.node_mut(node.next) {
            next.prev = node.prev;
        } else {
            self.tail = node.prev;
        }

        if let Some(tail) = self.node_mut(self.tail) {
            tail.next = idx;
        }
        if self.node_ref(self.head).is_none() {
            self.head = idx;
        }

        let node = self.nodes.get_mut(idx).unwrap().insert(FixedVecNode {
            prev: self.tail,
            next: usize::MAX,
            data: node.data,
        });
        self.tail = idx;
        Some(&mut node.data)
    }
}

#[derive(Debug)]
pub struct FixedVecIter<'a, T> {
    list: &'a FixedVec<T>,
    head: usize,
    tail: usize,
    len: usize,
}

impl<'a, T> Clone for FixedVecIter<'a, T> {
    fn clone(&self) -> Self {
        Self {
            list: self.list,
            head: self.head,
            tail: self.tail,
            len: self.len,
        }
    }
}

impl<'a, T> Iterator for FixedVecIter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let head = self.head;
            let node = self.list.node_ref(head).unwrap();
            self.head = node.next;
            self.len -= 1;
            Some((head, &node.data))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for FixedVecIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let tail = self.tail;
            let node = self.list.node_ref(tail).unwrap();
            self.tail = node.prev;
            self.len -= 1;
            Some((tail, &node.data))
        } else {
            None
        }
    }
}

pub struct FixedVecIterMut<'a, T> {
    ptr: NonNull<Option<FixedVecNode<T>>>,
    head: usize,
    tail: usize,
    len: usize,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> FixedVecIterMut<'a, T> {
    #[allow(unsafe_code)]
    fn new(
        slice: &'a mut [Option<FixedVecNode<T>>],
        head: usize,
        tail: usize,
        len: usize,
    ) -> Self {
        let ptr = slice.as_mut_ptr();
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            head,
            tail,
            len,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for FixedVecIterMut<'a, T> {
    type Item = (usize, &'a mut T);

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let head = self.head;
            let node_ref = unsafe {
                let ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(head)).as_ptr();
                &mut *ptr
            };

            let node = node_ref.as_mut().unwrap();
            self.head = node.next;
            self.len -= 1;
            Some((head, &mut node.data))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for FixedVecIterMut<'a, T> {
    #[allow(unsafe_code)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let tail = self.tail;
            let node_ref = unsafe {
                let ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(tail)).as_ptr();
                &mut *ptr
            };

            let node = node_ref.as_mut().unwrap();
            self.tail = node.prev;
            self.len -= 1;
            Some((tail, &mut node.data))
        } else {
            None
        }
    }
}
