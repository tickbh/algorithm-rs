
use algorithm::Slab;
fn main() {
    let mut slab = Slab::<String>::new();
    // let k = {
    //     let (k, v) = slab.get_next();
    //     *v = "ssss".to_string();
    //     println!("k = {:?}, v = {:?}", k, v);
    //     println!("k = {:?}, v = {:?}", k, slab.get(k));
    //     k
    // };
    // slab.remove(k);
    // {
    //     let (k, v) = slab.get_next();
    //     println!("k = {:?}, v = {:?}", k, v);
    // }
    // {
    //     let (k, v) = slab.get_next();
    //     println!("k = {:?}, v = {:?}", k, v);
    // }

    for _ in 0..100 {
        let k = slab.get_next_key();
        slab[&k] = format!("{}", k);
    }
    
    for i in 0..100 {
        let _ = slab.remove(i);
    }
    for _ in 0..100 {
        let k = slab.get_next_key();
        println!("k = {:?}, v = {:?}", k, slab[&k]);
    }
    // let (k1, v1) = slab.get_next();
    // println!("k = {:?}, k1 = {:?} {:?} {:?}", k, k1, v, v1);
}
