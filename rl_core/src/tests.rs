use crate::fast_hash::{FastHash, SetKey, SetItem};
use crate::{array::Array, set::Set, alloc::InlineAllocator};

#[test]
fn set_test() {
    #[derive(Copy, Clone)]
    struct SetNumber(i32);
    impl SetItem for SetNumber {
        type KeyType = i32;

        fn get_key(&self) -> Self::KeyType {
            self.0
        }
    }
    impl SetNumber {
        fn new(n: i32) -> Self {
            Self(n)
        }
    }

    let mut set = Set::new();
    set.insert(SetNumber::new(10));
    set.insert(SetNumber::new(10));
    set.insert(SetNumber::new(10));
    set.insert(SetNumber::new(15));
    set.insert(SetNumber::new(15));
    set.insert(SetNumber::new(20));
    assert_eq!(set.num_with_key(10), 3);
    assert_eq!(set.num_with_key(15), 2);
    assert_eq!(set.num_with_key(20), 1);
    assert_eq!(set.num_with_key(25), 0);
    set.remove(0);
    set.insert(SetNumber::new(25));
    assert_eq!(set.num_with_key(10), 2);
    assert_eq!(set.num_with_key(15), 2);
    assert_eq!(set.num_with_key(20), 1);
    assert_eq!(set.num_with_key(25), 1);
    assert_eq!(set.num(), 6);
    assert_eq!(set.find_first(20), 0);
    assert_eq!(set.find_next(0), usize::MAX);
    assert_eq!(set.find_first(10), 2);
    assert_eq!(set.find_next(2), 1);
    assert_eq!(set.find_next(1), usize::MAX);
    set.clear();
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
    let mut inline_array: Array<i32, InlineAllocator<INLINE_TEST_SIZE, i32>> = Array::custom_allocator();
    assert_eq!(std::mem::size_of_val(&inline_array), std::mem::size_of_val(&array) + std::mem::size_of::<i32>() * INLINE_TEST_SIZE);
}
