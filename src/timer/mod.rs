

pub trait Timer {
    /// 当时与现在的间隔，以确定插入确定的槽
    fn when(&self) -> u64;
    /// 可能需要修改对象，此处用可变值
    fn when_mut(&mut self) -> u64 {
        self.when()
    }
}

macro_rules! impl_primitive_timer {
    ($ty:ident) => {
        impl Timer for $ty {
            #[inline(always)]
            fn when(&self) -> u64 {
                *self as u64
            }
        }
    };
}

impl_primitive_timer!(u8);
impl_primitive_timer!(u16);
impl_primitive_timer!(u32);
impl_primitive_timer!(u64);
impl_primitive_timer!(u128);
impl_primitive_timer!(i8);
impl_primitive_timer!(i16);
impl_primitive_timer!(i32);
impl_primitive_timer!(i64);
impl_primitive_timer!(i128);
impl_primitive_timer!(f32);
impl_primitive_timer!(f64);
impl_primitive_timer!(usize);

mod timer_rbtree;
mod timer_wheel;

pub use timer_wheel::TimerWheel;
pub use timer_rbtree::TimerRBTree;
