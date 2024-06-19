
pub mod quadsort;
pub use quadsort::{quad_sort, quad_sort_order_by};

mod cache;
mod tree;
mod map;
mod timer;
mod arr;
pub use cache::{LruCache, LruKCache, LfuCache, ArcCache, Slab, Reinit};
pub use tree::RBTree;
pub use map::{BitMap, RoaringBitMap};
pub use timer::{TimerWheel, Timer};
pub use arr::{CircularBuffer, FixedVec};

#[cfg(feature = "hashbrown")]
extern crate hashbrown;


#[cfg(feature = "hashbrown")]
pub use hashbrown::{HashMap, HashSet};
#[cfg(not(feature = "hashbrown"))]
pub use std::collections::{HashMap, HashSet};

#[cfg(feature = "hashbrown")]
pub type DefaultHasher = hashbrown::hash_map::DefaultHashBuilder;
#[cfg(not(feature = "hashbrown"))]
pub type DefaultHasher = std::collections::hash_map::RandomState;