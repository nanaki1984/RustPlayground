pub mod fast_hash;

mod raw_array;
mod raw_set;

pub mod alloc;
pub mod array;

#[cfg(test)]
mod tests {
    use crate::{array::Array, alloc::InlineAllocator};

    #[test]
    fn it_works() {
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

        const inline_test_size: usize = 4;
        let mut inline_array: Array<i32, InlineAllocator<inline_test_size, i32>> = Array::custom_allocator();
        assert_eq!(std::mem::size_of_val(&inline_array), std::mem::size_of_val(&array) + std::mem::size_of::<i32>() * inline_test_size);
    }
}
