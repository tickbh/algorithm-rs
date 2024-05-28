
use algorithm::Slab;
fn main() {
    let mut slab = Slab::new();
    for _ in 0..100 {
        let k = slab.get_next();
        slab[&k] = format!("{}", k);
    }
    assert!(slab.len() == 100);

    for i in 0..100 {
        let _ = slab.remove(i);
    }

    assert!(slab.len() == 0);
    let k = slab.get_next();
    assert!(k == 99);
    assert!(slab[&k] == "99");
    let k = slab.get_reinit_next();
    assert!(k == 98);
    assert!(slab[&k] == "");
}
