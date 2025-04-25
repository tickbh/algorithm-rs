use std::borrow::Borrow;

use algorithm::{KeyRef, KeyWrapper, ZSet};
fn main() {
    let mut val = ZSet::new();
    
    // let wrap = KeyWrapper::from_ref(&"bb");
    // let key_ref = KeyRef::new(&"bb");
    // println!("aaaaaaaaaaaaaa = {}",  wrap.eq(key_ref.borrow()));
    
    // val.add_or_update(11, 10);
    // val.add_or_update(22, 12);
    // assert_eq!(val.score(&22), 12);
    // assert_eq!(val.len(), 2);
    // assert_eq!(val.rank(&22), 0);
    // val.add_or_update(22, 9);
    // assert_eq!(val.rank(&22), 1);
    // assert_eq!(val.len(), 2);
    // val.add_or_update("aa", 10);
    val.add_or_update("xxx", 12);
    assert_eq!(val.score(&"xxx"), 12);
    assert_eq!(val.len(), 2);
    assert_eq!(val.rank(&"bb"), 0);
    val.add_or_update("bb", 9);
    assert_eq!(val.rank(&"bb"), 1);
    assert_eq!(val.len(), 2);
}