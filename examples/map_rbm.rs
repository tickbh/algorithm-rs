
use algorithm::SkipList;
fn main() {
    let mut val = SkipList::new();
    val.insert(1);
    assert_eq!(val.len(), 1);
}