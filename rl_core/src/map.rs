use crate::set::{SetEntry, Set};
use crate::array::Array;
use crate::fast_hash::{SetKey, KeyValuePair};
use crate::alloc::{ArrayAllocator, DefaultAllocator, InlineAllocator};

pub struct Map<K, V, DataAlloc = DefaultAllocator, EntriesAlloc = DefaultAllocator, TableAlloc = DefaultAllocator>
(
    Set<KeyValuePair<K, V>, DataAlloc, EntriesAlloc, TableAlloc>
) where
    K: SetKey,
    V: Unpin,
    DataAlloc: ArrayAllocator<KeyValuePair<K, V>>,
    EntriesAlloc: ArrayAllocator<SetEntry<KeyValuePair<K, V>>>,
    TableAlloc: ArrayAllocator<usize>;

impl<K: SetKey, V: Unpin> Map<K, V> {
    #[inline]
    pub fn new() -> Self {
        Map(Set::new())
    }
}

impl<K, V, DataAlloc, EntriesAlloc, TableAlloc> Map<K, V, DataAlloc, EntriesAlloc, TableAlloc> where
    K: SetKey,
    V: Unpin,
    DataAlloc: ArrayAllocator<KeyValuePair<K, V>>,
    EntriesAlloc: ArrayAllocator<SetEntry<KeyValuePair<K, V>>>,
    TableAlloc: ArrayAllocator<usize>
{
    #[inline]
    pub fn custom_allocators() -> Self {
        Map(Set::custom_allocators())
    }
}

impl<K, V, DataAlloc, EntriesAlloc, TableAlloc> Map<K, V, DataAlloc, EntriesAlloc, TableAlloc> where
    K: SetKey,
    V: Unpin,
    DataAlloc: ArrayAllocator<KeyValuePair<K, V>>,
    EntriesAlloc: ArrayAllocator<SetEntry<KeyValuePair<K, V>>>,
    TableAlloc: ArrayAllocator<usize>
{
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    pub fn set_capacity(&mut self, capacity: usize) {
        self.0.set_capacity(capacity);
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    #[inline]
    pub fn num(&self) -> usize {
        self.0.num()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn contains(&self, key: K) -> bool {
        self.0.find_first_index(key) != usize::MAX
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let existing_pair_index = self.0.find_first_index(key);
        if existing_pair_index == usize::MAX {
            self.0.insert(KeyValuePair::new(key, value));
            Option::None
        } else {
            Option::Some(self.0[existing_pair_index].swap_value(value))
        }
    }

    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        let mut pair_array: Array<KeyValuePair<K, V>, InlineAllocator<1, KeyValuePair<K, V>>> = self.0.remove_all(key);
        if pair_array.num() > 0 {
            Option::Some(KeyValuePair::take_value(pair_array.swap_remove(0)))
        } else {
            Option::None
        }
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<&V> {
        let pair_index = self.0.find_first_index(key);
        if pair_index == usize::MAX {
            Option::None
        } else {
            Option::Some(self.0[pair_index].get_value())
        }
    }

    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let existing_pair_index = self.0.find_first_index(key);
        if existing_pair_index == usize::MAX {
            Option::None
        } else {
            Option::Some(self.0[existing_pair_index].get_value_mut())
        }
    }

    #[inline]
    pub fn get_or_insert_mut(&mut self, key: K, value: V) -> &mut V {
        let index = self.0.find_index_or_insert_mut(KeyValuePair::new(key, value));
        (&mut self.0[index]).get_value_mut()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl<K, V, DataAlloc, EntriesAlloc, TableAlloc> Map<K, V, DataAlloc, EntriesAlloc, TableAlloc> where
    K: SetKey,
    V: Unpin + Default,
    DataAlloc: ArrayAllocator<KeyValuePair<K, V>>,
    EntriesAlloc: ArrayAllocator<SetEntry<KeyValuePair<K, V>>>,
    TableAlloc: ArrayAllocator<usize>
{
    #[inline]
    pub fn get_or_insert_default_mut(&mut self, key: K) -> &mut V {
        let index = self.0.find_index_or_insert_mut(KeyValuePair::new_with_key(key));
        (&mut self.0[index]).get_value_mut()
    }
}

impl<K: SetKey, V: Unpin> Default for Map<K, V> where
{
    fn default() -> Map<K, V> {
        Map::new()
    }
}
