use algorithm::RoaringBitMap;
fn main() {
    let mut map = RoaringBitMap::new();
    map.add_range(9, 16);
    let mut sub_map = RoaringBitMap::new();
    sub_map.add_range(7, 12);
    let map = map.union(&sub_map);
    assert!(map.iter().collect::<Vec<_>>() == vec![7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
}