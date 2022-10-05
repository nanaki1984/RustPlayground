use std::borrow::Borrow;
use std::ops::{Index, IndexMut};

use crate::raw_set::RawSetEntry;
use crate::set::Set;
use crate::array::InlineArray;
use crate::fast_hash::{SetKey, KeyValuePair, FastHash};
use crate::alloc::{ArrayAllocator, DefaultAllocator};

pub struct Map<K, V, DataAlloc = DefaultAllocator, EntriesAlloc = DefaultAllocator, TableAlloc = DefaultAllocator>
(
    Set<KeyValuePair<K, V>, DataAlloc, EntriesAlloc, TableAlloc>
) where
    K: SetKey,
    V: Unpin,
    DataAlloc: ArrayAllocator<KeyValuePair<K, V>>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
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
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
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
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
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
    pub fn contains<Q: ?Sized>(&self, key: &Q) -> bool where
        K: Borrow<Q>,
        Q: FastHash + Eq
    {
        self.0.find_first_index(key) != usize::MAX
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let existing_pair_index = self.0.find_first_index(&key);
        if existing_pair_index == usize::MAX {
            self.0.insert(KeyValuePair::new(key, value));
            Option::None
        } else {
            Option::Some(self.0[existing_pair_index].swap_value(value))
        }
    }

    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>,
        Q: FastHash + Eq
    {
        let mut pair_array: InlineArray<KeyValuePair<K, V>, 1> = self.0.remove_all(key);
        if pair_array.num() > 0 {
            Option::Some(KeyValuePair::take_value(pair_array.swap_remove(0)))
        } else {
            Option::None
        }
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V> where
        K: Borrow<Q>,
        Q: FastHash + Eq
    {
        let pair_index = self.0.find_first_index(key);
        if pair_index == usize::MAX {
            Option::None
        } else {
            Option::Some(self.0[pair_index].get_value())
        }
    }

    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V> where
        K: Borrow<Q>,
        Q: FastHash + Eq
    {
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
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
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

impl<K, V, DataAlloc, EntriesAlloc, TableAlloc, Q> Index<&Q> for Map<K, V, DataAlloc, EntriesAlloc, TableAlloc> where
    K: SetKey + Borrow<Q>,
    V: Unpin,
    DataAlloc: ArrayAllocator<KeyValuePair<K, V>>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>,
    Q: FastHash + Eq + ?Sized
{
    type Output = V;

    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, V, DataAlloc, EntriesAlloc, TableAlloc, Q> IndexMut<&Q> for Map<K, V, DataAlloc, EntriesAlloc, TableAlloc> where
    K: SetKey + Borrow<Q>,
    V: Unpin,
    DataAlloc: ArrayAllocator<KeyValuePair<K, V>>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>,
    Q: FastHash + Eq + ?Sized
{
    #[inline]
    fn index_mut(&mut self, key: &Q) -> &mut V {
        self.get_mut(key).expect("no entry found for key")
    }
}
