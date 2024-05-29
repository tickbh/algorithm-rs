
use algorithm::BitMap;

fn main() {
    let mut map = BitMap::new(10240);
    map.add_many(&vec![1, 2, 3, 4, 10]);
    assert!(map.contains(&1));
    assert!(!map.contains(&5));
    assert!(map.contains(&10));
    map.add_range(7, 16);
    assert!(!map.contains(&6));
    assert!(map.contains(&7));
    assert!(map.contains(&16));
    assert!(!map.contains(&17));
}
