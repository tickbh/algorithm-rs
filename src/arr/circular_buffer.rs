use std::{marker::PhantomData, ops::{Index, IndexMut}};

/// 循环的圆结构
/// 如果数据满了之后将自动在结尾后续添加,并保持最大个数
/// 
/// # Examples
///
/// ```
/// use algorithm::CircularBuffer;
/// fn main() {
///     let mut circular = CircularBuffer::new(2);
///     circular.push_back(1);
///     circular.push_back(2);
///     circular.push_back(3);
///     assert_eq!(circular.len(), 2);
///     assert_eq!(circular[0], 2);
///     assert_eq!(circular[1], 3);
/// }
/// ```
pub struct CircularBuffer<T> {
    arr: Vec<T>,
    head: usize,
    tail: usize,
    len: usize,
    cap: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(cap: usize) -> Self {
        Self {
            arr: Vec::with_capacity(cap),
            head: 0,
            tail: cap - 1,
            len: 0,
            cap,
        }
    }

    /// 是否已经填满过所有的元素
    /// 
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     circular.push_back(1);
    ///     assert_eq!(circular.is_inited(), false);
    ///     circular.push_back(2);
    ///     assert_eq!(circular.is_inited(), true);
    ///     circular.pop_back();
    ///     assert_eq!(circular.is_inited(), true);
    /// }
    /// ```
    pub fn is_inited(&self) -> bool {
        self.cap == self.arr.len()
    }

    /// 是否元素已满
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     circular.push_back(1);
    ///     assert_eq!(circular.is_full(), false);
    ///     circular.push_back(2);
    ///     assert_eq!(circular.is_full(), true);
    ///     circular.pop_back();
    ///     assert_eq!(circular.is_full(), false);
    /// }
    /// ```
    pub fn is_full(&self) -> bool {
        self.cap == self.len
    }

    /// 是否元素为空
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     assert_eq!(circular.is_empty(), true);
    ///     circular.push_back(1);
    ///     assert_eq!(circular.is_empty(), false);
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 返回元素长度
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     assert_eq!(circular.is_empty(), true);
    ///     circular.push_back(1);
    ///     circular.push_back(2);
    ///     assert_eq!(circular.len(), 2);
    ///     circular.push_front(1);
    ///     assert_eq!(circular.len(), 2);
    ///     circular.pop_front();
    ///     assert_eq!(circular.len(), 1);
    /// }
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    fn add_fix(&self, val: usize) -> usize {
        (val + 1) % self.cap
    }

    fn sub_fix(&self, val: usize) -> usize {
        if val == 0 {
            self.cap - 1
        } else {
            val - 1
        }
    }


    /// 在元素末尾添加元素
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     assert_eq!(circular.is_empty(), true);
    ///     circular.push_back(1);
    ///     circular.push_back(2);
    ///     assert_eq!(circular[0], 1);
    /// }
    /// ```
    pub fn push_back(&mut self, val: T) {
        if self.is_inited() {
            self.tail = self.add_fix(self.tail);
            self.arr[self.tail] = val;
            self.head = self.add_fix(self.head);
        } else {
            if self.tail + 1 < self.arr.len()  {
                self.tail = self.add_fix(self.tail);
                self.arr[self.tail] = val;
            } else {
                self.arr.push(val);
                self.tail = self.arr.len() - 1;
            }
            self.len += 1;
        }
    }

    /// 在元素前面添加元素
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     assert_eq!(circular.is_empty(), true);
    ///     circular.push_front(1);
    ///     circular.push_front(2);
    ///     assert_eq!(circular[0], 2);
    /// }
    /// ```
    pub fn push_front(&mut self, val: T) {
        if self.is_inited() {
            self.head = self.sub_fix(self.head);
            self.arr[self.head] = val;
            self.tail = self.sub_fix(self.tail);
        } else {
            if self.head > 0  {
                self.head = self.sub_fix(self.head);
                self.arr[self.head] = val;
            } else {
                self.arr.insert(0, val);
                self.head = 0;
                self.tail = self.add_fix(self.tail);
            }
            self.len += 1;
        }
    }

    /// 将最前面的元素弹出
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     assert_eq!(circular.is_empty(), true);
    ///     circular.push_front(1);
    ///     circular.push_front(2);
    ///     assert_eq!(circular[0], 2);
    ///     circular.pop_front();
    ///     assert_eq!(circular[0], 1);
    /// }
    /// ```
    pub fn pop_front(&mut self) {
        debug_assert!(!self.is_empty());
        self.head = self.add_fix(self.head);
        self.len -= 1;
    }

    /// 将最后面的元素弹出
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     assert_eq!(circular.is_empty(), true);
    ///     circular.push_back(1);
    ///     circular.push_back(2);
    ///     assert_eq!(circular[0], 1);
    ///     circular.pop_back();
    ///     assert_eq!(circular[0], 1);
    /// }
    /// ```
    pub fn pop_back(&mut self) {
        debug_assert!(!self.is_empty());
        self.tail = self.sub_fix(self.tail);
        self.len -= 1;
    }

    /// 迭代器
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     assert_eq!(circular.is_empty(), true);
    ///     circular.push_back(1);
    ///     circular.push_back(2);
    ///     let val: Vec<i32> = circular.iter().map(|s| *s).collect();
    ///     assert_eq!(val, vec![1, 2]);
    ///     let val: Vec<i32> = circular.iter().rev().map(|s| *s).collect();
    ///     assert_eq!(val, vec![2, 1]);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            arr: &self.arr,
            len: self.len,
            head: self.head,
            tail: self.tail,
            cap: self.cap,
        }
    }

    /// 迭代更改器
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithm::CircularBuffer;
    /// fn main() {
    ///     let mut circular = CircularBuffer::new(2);
    ///     assert_eq!(circular.is_empty(), true);
    ///     circular.push_back(1);
    ///     circular.push_back(2);
    ///     let val: Vec<i32> = circular.iter_mut().map(|v| { *v *= 2; *v }).collect();
    ///     assert_eq!(val, vec![2, 4]);
    ///     let val: Vec<i32> = circular.iter_mut().rev().map(|v| { *v *= 2; *v }).collect();
    ///     assert_eq!(val, vec![8, 4]);
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            arr: self.arr.as_mut_ptr(),
            len: self.len,
            head: self.head,
            tail: self.tail,
            cap: self.cap,
            _marker: PhantomData,
        }
    }
}


impl<T> Index<usize> for CircularBuffer<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        if index < self.len {
            let ridx = (index + self.head) % self.cap;
            &self.arr[ridx]
        } else {
            panic!("index error");
        }
    }
}

impl<T> IndexMut<usize> for CircularBuffer<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        if index < self.len {
            let ridx = (index + self.head) % self.cap;
            &mut self.arr[ridx]
        } else {
            panic!("index error");
        }
    }
}

impl<T: Clone> Clone for CircularBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            arr: self.arr.clone(),
            head: self.head,
            tail: self.tail,
            len: self.len,
            cap: self.cap,
        }
    }
}

pub struct Iter<'a, T: 'a> {
    len: usize,
    arr: &'a Vec<T>,
    head: usize,
    tail: usize,
    cap: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let now = self.head;
        self.head = (self.head + 1) % self.cap;
        self.len -= 1;
        Some(&self.arr[now])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}


impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let now = self.tail;
        self.tail = (self.tail + self.cap - 1) % self.cap;
        self.len -= 1;
        Some(&self.arr[now])
    }
}

pub struct IterMut<'a, T: 'a> {
    len: usize,
    arr: *mut T,
    head: usize,
    tail: usize,
    cap: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let now = self.head;
        self.head = (self.head + 1) % self.cap;
        self.len -= 1;
        unsafe {
            let ptr = self.arr.add(now);
            return Some(&mut *ptr)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}


impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let now = self.tail;
        self.tail = (self.tail + self.cap - 1) % self.cap;
        self.len -= 1;
        unsafe {
            let ptr = self.arr.add(now);
            return Some(&mut *ptr)
        }
    }
}



impl<T> PartialEq for CircularBuffer<T>
    where
        T: Eq,
{
    fn eq(&self, other: &CircularBuffer<T>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().enumerate().all(|(idx, value)| &other[idx] == value)
    }
}

impl<T> Eq for CircularBuffer<T>
    where
        T: Eq,
{}

#[cfg(test)]
mod tests {
    use super::CircularBuffer;

    #[test]
    fn test_iter() {
        let mut circular = CircularBuffer::new(2);
        assert_eq!(circular.is_empty(), true);
        circular.push_back(1);
        circular.push_back(2);
        let val: Vec<i32> = circular.iter().map(|s| *s).collect();
        assert_eq!(val, vec![1, 2]);
        let val: Vec<i32> = circular.iter().rev().map(|s| *s).collect();
        assert_eq!(val, vec![2, 1]);
    }
}