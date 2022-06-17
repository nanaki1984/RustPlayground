use crate::set::{SetEntry, Set};
use crate::fast_hash::{SetKey, KeyValuePair};
use crate::alloc::{ArrayAllocator, DefaultAllocator};

pub struct Map<Key, Value, DataAlloc = DefaultAllocator, EntriesAlloc = DefaultAllocator, TableAlloc = DefaultAllocator>
(
    Set<KeyValuePair<Key, Value>, DataAlloc, EntriesAlloc, TableAlloc>
) where
    Key: SetKey,
    Value: Unpin,
    DataAlloc: ArrayAllocator<KeyValuePair<Key, Value>>,
    EntriesAlloc: ArrayAllocator<SetEntry<KeyValuePair<Key, Value>>>,
    TableAlloc: ArrayAllocator<usize>;

impl<Key: SetKey, Value: Unpin> Map<Key, Value> {
    #[inline]
    pub fn new() -> Self {
        Map(Set::new())
    }
}

impl<Key, Value, DataAlloc, EntriesAlloc, TableAlloc> Map<Key, Value, DataAlloc, EntriesAlloc, TableAlloc> where
    Key: SetKey,
    Value: Unpin,
    DataAlloc: ArrayAllocator<KeyValuePair<Key, Value>>,
    EntriesAlloc: ArrayAllocator<SetEntry<KeyValuePair<Key, Value>>>,
    TableAlloc: ArrayAllocator<usize>
{
    #[inline]
    pub fn custom_allocators() -> Self {
        Map(Set::custom_allocators())
    }
}
