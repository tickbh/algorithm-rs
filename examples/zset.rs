use algorithm::ZSet;
fn main() {
    let mut val = ZSet::new();
    val.add_or_update("aa", 10);
    val.add_or_update("bb", 12);
    assert_eq!(val.score(&"bb"), 12);
    assert_eq!(val.len(), 2);
    assert_eq!(val.rank(&"bb"), 2);
    val.add_or_update("bb", 9);
    assert_eq!(val.rank(&"bb"), 1);
    assert_eq!(val.len(), 2);

}
