use algorithm::FixedVec;
fn main() {
    let mut val = FixedVec::new(5);
    val.insert_head(1);
    val.insert_head(2);
    val.insert_head(3);
    let _ = val.iter_mut().map(|(_, v)| *v = *v * 2).collect::<Vec<_>>();
    assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<_>>(), vec![6, 4, 2]);
}