pub mod quadsort;
pub use quadsort::{quad_sort, quad_sort_order_by};

mod arr;
pub mod buf;
mod cache;
mod key;
mod map;
mod timer;
mod tree;
mod util;

pub use arr::{CircularBuffer, FixedVec, SkipList, SkipNode};
pub use cache::{ArcCache, LfuCache, LruCache, LruKCache, Reinit, Slab};
pub use key::{KeyRef, KeyWrapper};
pub use map::{BitMap, RoaringBitMap, ZSet};
pub use timer::{StampTimer, StepTimer, Timer, TimerRBTree, TimerWheel};
pub use tree::RBTree;
pub use util::*;

#[cfg(feature = "hashbrown")]
extern crate hashbrown;

#[cfg(feature = "hashbrown")]
pub use hashbrown::{HashMap, HashSet};
#[cfg(not(feature = "hashbrown"))]
pub use std::collections::{HashMap, HashSet};

#[cfg(feature = "hashbrown")]
pub type DefaultHasher = hashbrown::DefaultHashBuilder;
#[cfg(not(feature = "hashbrown"))]
pub type DefaultHasher = std::collections::hash_map::RandomState;
