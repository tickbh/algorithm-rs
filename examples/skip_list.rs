    // use algorithm::SkipList;
    // fn main() {
    //     let mut val = SkipList::new();
    //     val.insert(4);
    //     val.insert(2);
    //     val.insert(1);
    //     let mut iter = val.iter();
    //     assert_eq!(iter.next(), Some(&1));
    //     assert_eq!(iter.next(), Some(&2));
    //     assert_eq!(iter.next(), Some(&4));
    //     assert_eq!(iter.next(), None);
    //     let mut iter = val.iter().rev();
    //     assert_eq!(iter.next(), Some(&4));
    //     assert_eq!(iter.next(), Some(&2));
    //     assert_eq!(iter.next(), Some(&1));
    //     assert_eq!(iter.next(), None);
    // }

    use algorithm::SkipList;
    fn main() {
        let mut val = SkipList::new();
        val.insert(4);
        val.insert(2);
        val.insert(1);
        let mut iter = val.iter();
        assert_eq!(iter.next(), Some((&1, 0)));
        assert_eq!(iter.next(), Some((&2, 1)));
        assert_eq!(iter.next(), Some((&4, 2)));
        assert_eq!(iter.next(), None);
    }