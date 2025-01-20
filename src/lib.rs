
pub mod quadsort;
pub use quadsort::{quad_sort, quad_sort_order_by};

mod util;
mod cache;
mod tree;
mod map;
mod timer;
mod arr;
pub mod buf;

pub use cache::{LruCache, LruKCache, LfuCache, ArcCache, Slab, Reinit};
pub use tree::RBTree;
pub use map::{BitMap, RoaringBitMap};
pub use timer::{TimerWheel, Timer, TimerRBTree, StampTimer, StepTimer};
pub use arr::{CircularBuffer, FixedVec};
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