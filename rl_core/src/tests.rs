use crate::alloc::DefaultAllocator;
use crate::fast_hash::{SetItem, fnv_hash, fnv_hash_const};
use crate::{array::Array, set::Set, map::Map, alloc::InlineAllocator, strings_table::StringAtom};

#[test]
fn stringatom_test() {
    let name0 = StringAtom::new(b"nAMe tESt");
    let name1 = StringAtom::new(b"Name test");
    let name2: StringAtom = "name Test".into();
    assert_eq!(name0, name1);
    assert_eq!(name0, name2);
    assert_eq!(name1, name2);
    assert_eq!(name2.as_str(), "nAMe tESt");
}

#[test]
fn fnv_test() {
    assert_eq!(fnv_hash::<false>("Hello world!".as_bytes()), fnv_hash_const(b"Hello world!", false));
    assert_eq!(fnv_hash::<true>("hello WORLD!".as_bytes()), fnv_hash_const(b"HeLLo woRLd!", true));
}

#[test]
fn map_test() {
    let mut map: Map<String, i32> = Map::new();
    assert_eq!(map.contains("key"), false);
    map.insert("key".to_string(), 10);
    map.insert("other key".to_string(), 20);
    assert!(!map.is_empty());
    assert_eq!(map.contains("key"), true);
    assert_eq!(map.contains("other key"), true);
    assert_eq!(*map.get("other key").unwrap(), 20);
    assert_eq!(map["other key"], 20);
    *map.get_mut("other key").unwrap() = 30;
    assert_eq!(*map.get("other key").unwrap(), 30);
    assert_eq!(map["other key"], 30);
    map["other key"] = 40;
    assert_eq!(map["other key"], 40);
    assert_eq!(map.remove("key").unwrap(), 10);
    map.clear();
    assert!(map.is_empty());
    assert_eq!(*map.get_or_insert_default_mut("default key".to_string()), Default::default());
    assert_eq!(map.contains("default key"), true);
}

#[test]
fn set_test() {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    struct SetNumber(i32);
    impl SetItem for SetNumber {
        type KeyType = i32;

        fn get_key(&self) -> &Self::KeyType {
            &self.0
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

    assert_eq!(set.swap_remove(0).get_num(), 10);

    set.insert(SetNumber::new(25));
    assert_eq!(set.num(), 6);
    let elem20 = &set[set.find_first_index(&20)];
    assert_eq!(elem20.get_num(), 20);
    assert_eq!(set.find_next_index(set.get_element_index(elem20).unwrap()), usize::MAX);
    let mut elem10 = &set[set.find_first_index(&10)];
    assert_eq!(elem10.get_num(), 10);
    elem10 = &set[set.find_next_index(set.get_element_index(elem10).unwrap())];
    assert_eq!(elem10.get_num(), 10);
    assert_eq!(set.find_next_index(set.get_element_index(elem10).unwrap()), usize::MAX);
    set.clear();
    assert_eq!(set.num(), 0);

    for i in 0..10 {
        set.insert(SetNumber::new(i % 2));
    }
    assert_eq!(set.remove_all::<DefaultAllocator, i32>(&0).num(), 5);
    assert_eq!(set.remove_all::<DefaultAllocator, i32>(&1).num(), 5);
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
