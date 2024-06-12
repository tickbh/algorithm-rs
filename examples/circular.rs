use algorithm::CircularBuffer;
fn main() {
    let mut circular = CircularBuffer::new(2);
    circular.push_back(1);
    circular.push_back(2);
    circular.push_back(3);
    assert_eq!(circular.len(), 2);
    assert_eq!(circular[&0], 2);
    assert_eq!(circular[&1], 3);
}
