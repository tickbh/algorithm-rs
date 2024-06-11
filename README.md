## algorithm
将提供一些常用的数据结构以供使用。

# lru 全称是Least Recently Used，即最近最久未使用的意思。
每次元素访问将其更新到列表的最前，时间复杂度为O(1)。当达到容量限制时将淘汰双向列表中的链尾数据
```rust
use algorithm::LruCache;
fn main() {
    let mut lru = LruCache::new(3);
    lru.insert("now", "ok");
    lru.insert("hello", "algorithm");
    lru.insert("this", "lru");
    lru.insert("auth", "tickbh");
    assert!(lru.len() == 3);
    assert_eq!(lru.get("hello"), Some(&"algorithm"));
    assert_eq!(lru.get("this"), Some(&"lru"));
    assert_eq!(lru.get("now"), None);
}
```
# lru-k
将访问次数达到k的目标值放进到优先队列，lru-k的主要目的是为了解决LRU算法“缓存污染”的问题，其核心思想是将“最近使用过1次”的判断标准扩展为“最近使用过K次”。
相比LRU，LRU-K需要多维护一个队列，用于记录所有缓存数据被访问的历史。只有当数据的访问次数达到K次的时候，才将数据放入缓存。当需要淘汰数据时，LRU-K会淘汰第K次访问时间距当前时间最大的数据。

```rust
use algorithm::LruKCache;
fn main() {
    let mut lru = LruKCache::with_times(3, 3);
    lru.insert("this", "lru");
    for _ in 0..3 {
        let _ = lru.get("this");
    }
    lru.insert("hello", "algorithm");
    lru.insert("auth", "tickbh");
    assert!(lru.len() == 3);
    lru.insert("auth1", "tickbh");
    assert_eq!(lru.get("this"), Some(&"lru"));
    assert_eq!(lru.get("hello"), None);
    assert!(lru.len() == 3);
}
```

# lfu (least frequently used)最近频次使用
每个元素在被访问或者更新的时候将其访问次数+1，当元素满时将优先淘汰掉访问次数最少的数据。
```rust

use algorithm::LfuCache;
fn main() {
    let mut lru = LfuCache::new(3);
    lru.insert("hello", "algorithm");
    lru.insert("this", "lru");
    lru.set_reduce_count(100);
    assert!(lru.get_visit(&"hello") == Some(5));
    assert!(lru.get_visit(&"this") == Some(5));
    for _ in 0..98 {
        let _ = lru.get("this");
    }
    assert!(lru.get_visit(&"this") == Some(51));
    assert!(lru.get_visit(&"hello") == Some(2));
    let mut keys = lru.keys();
    assert!(keys.next()==Some(&"this"));
    assert!(keys.next()==Some(&"hello"));
    assert!(keys.next() == None);
}
```

# slab 缓存块组，linux中缓存对象的分配器
缓存对象需实现Default，将会使对象缓存起来，避免频繁的重复申请释放带来的开销

以下我们以简单的测试来进行对比，algorithm::Slab与slab::Slab与普通的alloc

以下测试场景相对简单，可能对`slab::Slab`较为不公平

```rust
use std::{ptr, time::Instant};

use algorithm::{Reinit, Slab};

const ARRAY_SIZE: usize = 10240;
const NUM: usize = usize::MAX - 99999;
const ZERO_ARRAY: [usize; ARRAY_SIZE] = [NUM; ARRAY_SIZE];
struct TestStruct {
    array: [usize; ARRAY_SIZE],
    size: usize,
    val: String,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self { array: [NUM; ARRAY_SIZE], size: 0, val:  "slab".to_string(), }
    }
}

impl Reinit for TestStruct {
    #[inline(always)]
    fn reinit(&mut self) {
        self.size = 0;
        self.val.clear();
        self.val.push_str("slab");
        unsafe {
            ptr::copy_nonoverlapping(&ZERO_ARRAY[0], &mut self.array[0], ARRAY_SIZE);
        }
    }
}

fn main() {
    let times = 100000;
    let now = Instant::now();
    let mut slab = Slab::<TestStruct>::new();
    let mut sum: usize = 0;
    for i in 0..times {
        let (next, test) = slab.get_reinit_next_val();
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size + test.val.len());
        slab.remove(next);
    }
    println!("algorithm: all cost times {}ms, sum = {}", now.elapsed().as_millis(), sum);


    let now = Instant::now();
    let mut slab = slab::Slab::<TestStruct>::new();
    let mut sum: usize = 0;
    for i in 0..times {
        let next = slab.insert(TestStruct::default());
        let test = &mut slab[next];
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size + test.val.len());
        slab.remove(next);
    }
    println!("tokio::slab: all cost times {}ms, sum = {}", now.elapsed().as_millis(), sum);

    let now = Instant::now();
    let mut sum: usize = 0;
    for i in 0..times {
        let mut test = TestStruct::default();
        test.array[i % 20] = test.array[i % 20].wrapping_add(i % 1024);
        sum = sum.wrapping_add(test.array[10] + test.size + test.val.len());
        drop(test);
    }
    println!("normal alloc: all cost times {}ms, sum = {}", now.elapsed().as_millis(), sum);
}
```
最终用release命令进行输出测试，结果均为一致

但是耗时algorithm避免了申请创建的开销，耗时相对较短，做的仅仅将对象重新reinit

在此场景中tokio::slab即进行了申请又开销了插入及删除，反而耗时最长
```console
algorithm: all cost times 132ms, sum = 18446744063712505088
tokio::slab: all cost times 477ms, sum = 18446744063712505088
normal alloc: all cost times 337ms, sum = 18446744063712505088
```

# 计时器轮（TimerWheel），模拟时钟格式组成的高效计时器

1. **环形数据结构**：TimerWheel，即时间轮，是一个环形的数据结构，类似于时钟的面，被等分为多个格子或槽位（slot）。

2. **槽位时间间隔**：每个槽位代表一个固定的时间间隔，例如1毫秒、1秒等。这个时间间隔决定了定时器的精度。

3. **初始化**：在算法开始时，需要初始化时间轮，包括设定时间轮的大小（即槽位的数量）和每个槽位代表的时间间隔。即当插入数据后即不允许修改时轮信息。

```rust
use algorithm::TimerWheel;

fn main() {
    let mut timer = TimerWheel::new();
    timer.append_timer_wheel(12, 60 * 60, "HourWheel");
    timer.append_timer_wheel(60, 60, "MinuteWheel");
    timer.append_timer_wheel(60, 1, "SecondWheel");

    timer.add_timer(30);
    assert_eq!(timer.get_delay_id(), 30);
    timer.add_timer(149);
    assert_eq!(timer.get_delay_id(), 30);
    let t = timer.add_timer(600);
    assert_eq!(timer.get_delay_id(), 30);
    timer.add_timer(1);
    assert_eq!(timer.get_delay_id(), 1);
    timer.del_timer(t);
    timer.add_timer(150);
    assert_eq!(timer.get_delay_id(), 1);

    let val = timer.update_deltatime(30).unwrap();
    assert_eq!(val, vec![1, 30]);

    timer.add_timer(2);

    let val = timer.update_deltatime(119).unwrap();
    assert_eq!(val, vec![2, 149]);

    let val = timer.update_deltatime(1).unwrap();
    assert_eq!(val, vec![150]);
    
    assert!(timer.is_empty());
}
```