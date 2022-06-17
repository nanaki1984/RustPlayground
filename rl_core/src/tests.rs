use crate::alloc::DefaultAllocator;
use crate::fast_hash::SetItem;
use crate::{array::Array, set::Set, alloc::InlineAllocator};

#[test]
fn set_test() {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    struct SetNumber(i32);
    impl SetItem for SetNumber {
        const IMMUTABLE_KEY: bool = false;

        type KeyType = i32;

        fn get_key(&self) -> Self::KeyType {
            self.0
        }
    }
    impl SetNumber {
        fn new(n: i32) -> Self {
            Self(n)
        }

        fn get_num(&self) -> i32 {
            self.0
        }
    }

    let mut set = Set::new();
    set.insert(SetNumber::new(10));
    set.insert(SetNumber::new(10));
    set.insert(SetNumber::new(10));
    set.insert(SetNumber::new(15));
    set.insert(SetNumber::new(15));
    set.insert(SetNumber::new(20));
    //assert_eq!(set.num_with_key(10), 3);
    //assert_eq!(set.num_with_key(15), 2);
    //assert_eq!(set.num_with_key(20), 1);
    //assert_eq!(set.num_with_key(25), 0);

    //assert_eq!(set.swap_remove(0).get_num(), 10);
    let test_ref_index;
    {
        let test_ref = unsafe{ set.get_unchecked(0) };
        test_ref_index = set.get_element_index(test_ref).unwrap();
        assert_eq!(test_ref_index, 0);
    }
    assert_eq!(set.swap_remove(test_ref_index).get_num(), 10);

    set.insert(SetNumber::new(25));
    //assert_eq!(set.num_with_key(10), 2);
    //assert_eq!(set.num_with_key(15), 2);
    //assert_eq!(set.num_with_key(20), 1);
    //assert_eq!(set.num_with_key(25), 1);
    assert_eq!(set.num(), 6);
    let elem20 = set.find_first(20).unwrap();
    assert_eq!(elem20.get_num(), 20);
    assert_eq!(set.find_next(elem20), Option::None);
    let mut elem10 = set.find_first(10).unwrap();
    assert_eq!(elem10.get_num(), 10);
    elem10 = set.find_next(elem10).unwrap();
    assert_eq!(elem10.get_num(), 10);
    assert_eq!(set.find_next(elem10), Option::None);
    set.clear();
    assert_eq!(set.num(), 0);

    for i in 0..10 {
        set.insert(SetNumber::new(i % 2));
    }
    assert_eq!(set.remove_all::<DefaultAllocator>(0).num(), 5);
    assert_eq!(set.remove_all::<DefaultAllocator>(1).num(), 5);
    assert_eq!(set.num(), 0);
}

#[test]
fn array_test() {
    let mut array = Array::new();
    array.push_back(2);
    array.push_front(1);
    array.push_back(3);
    assert_eq!(array.num(), 3);
    assert_eq!(array.capacity(), 4);
    assert_eq!(array[0], 1);
    assert_eq!(array[1], 2);
    assert_eq!(array[2], 3);
    array.set_capacity(4);
    assert_eq!(array.capacity(), 4);
    assert_eq!(array.remove(1), 2);
    assert_eq!(array.swap_remove(0), 1);
    assert_eq!(array.num(), 1);
    assert_eq!(array[0], 3);
    for num in &mut array {
        *num += 1;
    }
    assert_eq!(array[0], 4);
    array.insert(0, 2);
    array.insert(1, 3);
    array.insert(3, 5);
    assert_eq!(array[0], 2);
    assert_eq!(array[1], 3);
    assert_eq!(array[2], 4);
    assert_eq!(array[3], 5);
    assert!(!array.is_empty());
    array.clear();
    assert!(array.is_empty());

    array.insert_range(0..10, 0);
    assert_eq!(array.num(), 10);
    for i in 0..10 {
        assert_eq!(array[i], 0);
    }
    array.insert_range(1..3, 1);
    assert_eq!(array[0], 0);
    assert_eq!(array[1], 1);
    assert_eq!(array[2], 1);
    assert_eq!(array[3], 0);
    array.insert_range(array.num()..20, 2);
    assert_eq!(array[19], 2);

    const INLINE_TEST_SIZE: usize = 4;
    let inline_array: Array<i32, InlineAllocator<INLINE_TEST_SIZE, i32>> = Array::custom_allocator();
    assert_eq!(std::mem::size_of_val(&inline_array), std::mem::size_of_val(&array) + std::mem::size_of::<i32>() * INLINE_TEST_SIZE);
}
