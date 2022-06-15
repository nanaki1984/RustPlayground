use std::alloc::Layout;
//use std::option::{Option};

use crate::fast_hash::SetKey;
use crate::alloc::AllocatorBase;
use crate::alloc::ArrayAllocator;
use crate::raw_array::RawArray;
use crate::array::Array;

struct SetEntry<K: SetKey> {
    key: K,
    index: usize,
    prev: usize,
    next: usize,
}

struct RawSet<Key, DataAlloc, EntriesAlloc, TableAlloc> where
    Key: SetKey,
    DataAlloc: AllocatorBase,
    EntriesAlloc: ArrayAllocator<SetEntry<Key>>,
    TableAlloc: ArrayAllocator<usize>
{
    data: RawArray<DataAlloc>,
    entries: Array<SetEntry<Key>, EntriesAlloc>,
    table: Array<usize, TableAlloc>,
}

impl<Key,DataAlloc, EntriesAlloc, TableAlloc> RawSet<Key, DataAlloc, EntriesAlloc, TableAlloc> where
    Key: SetKey,
    DataAlloc: AllocatorBase,
    EntriesAlloc: ArrayAllocator<SetEntry<Key>>,
    TableAlloc: ArrayAllocator<usize>
{
    pub(crate) unsafe fn for_type_unchecked(layout: Layout) -> Self {
        Self {
            data: RawArray::<DataAlloc>::for_type_unchecked(layout),
            entries: Array::custom_allocator(),
            table: Array::custom_allocator()
        }
    }

    //pub(crate) fn with_table_size(table_size: usize) -> Self {
    //}

    fn find_first_entry_index(&self, key: Key) -> usize {
        let mut entry_index = usize::MAX;

        if !self.table.is_empty() {
            let table_index = key.fast_hash() % self.table.num();
            entry_index = self.table[table_index];
            while entry_index != usize::MAX && self.entries[entry_index].key != key {
                entry_index = self.entries[entry_index].next;
            }
        }

        entry_index
    }

    fn find_next_entry_index(&self, entry_index: usize) -> usize {
        debug_assert!(entry_index != usize::MAX);

        let entry = &self.entries[entry_index];
        let next_entry_index = entry.next;

        if next_entry_index == usize::MAX || self.entries[next_entry_index].key != entry.key {
            return usize::MAX;
        }

        next_entry_index
    }

    fn insert<F>(&mut self, key: Key, ctor: F) -> usize
        where F: FnOnce(*mut u8)
    {
        let new_entry_index = self.entries.num();
        let table_index = key.fast_hash() % self.table.num();

        let mut new_entry = SetEntry {
            key,
            index: new_entry_index,
            prev: usize::MAX,
            next: usize::MAX
        };

        let next_entry_index = self.find_first_entry_index(key);

        new_entry_index
    }
}
