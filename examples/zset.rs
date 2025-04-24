use algorithm::ZSet;
fn main() {
    let mut val = ZSet::new();
    val.add_or_update("key", 1);
    assert_eq!(val.len(), 1);
    println!("ok!!!!!!!!!!");
}