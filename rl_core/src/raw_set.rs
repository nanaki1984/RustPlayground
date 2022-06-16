use std::alloc::Layout;
//use std::option::{Option};

use crate::fast_hash::SetKey;
use crate::alloc::AllocatorBase;
use crate::alloc::ArrayAllocator;
use crate::raw_array::RawArray;
use crate::array::Array;

#[derive(Copy, Clone)]
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

    #[inline]
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

    #[inline]
    fn find_next_entry_index(&self, entry_index: usize) -> usize {
        debug_assert!(entry_index != usize::MAX);

        let entry = &self.entries[entry_index];
        let next_entry_index = entry.next;

        if next_entry_index == usize::MAX || self.entries[next_entry_index].key != entry.key {
            return usize::MAX;
        }

        next_entry_index
    }

    #[inline]
    fn table_is_full(&self) -> bool {
        const MAX_LOAD_FACTOR: f32 = 0.7;
        self.table.num() == 0 || self.entries.num() > (self.table.num() as f32 * MAX_LOAD_FACTOR).round() as usize
    }

    #[inline]
    fn grow_table(&mut self) {
        self.rehash(self.table.num() * 2 + 8);
    }

    #[inline]
    fn setup_new_entry(&mut self, new_entry: &mut SetEntry<Key>) {
        let table_index = new_entry.key.fast_hash() % self.table.num();

        new_entry.next = self.find_first_entry_index(new_entry.key);
        if new_entry.next == usize::MAX {
            new_entry.next = self.table[table_index];
        }

        let new_entry_index = new_entry.index;
        if new_entry.next == usize::MAX {
            self.table[table_index] = new_entry_index;
        } else {
            let next_entry = &mut self.entries[new_entry.next];

            new_entry.prev = next_entry.prev;
            next_entry.prev = new_entry_index;

            if new_entry.prev == usize::MAX {
                self.table[table_index] = new_entry_index;
            } else {
                self.entries[new_entry.prev].next = new_entry_index;
            }
        }
    }

    pub fn rehash(&mut self, table_size: usize) {
        debug_assert!(table_size > 0);

        self.table.clear();
        self.table.set_capacity(table_size);
        self.table.insert_range(0..table_size, usize::MAX);

        for entry in &mut self.entries {
            entry.prev = usize::MAX;
            entry.next = usize::MAX;
        }

        let entries_num = self.entries.num();
        for index in 0..entries_num {
            let mut new_entry = self.entries[index];
            self.setup_new_entry(&mut new_entry);
            self.entries[index] = new_entry;
        }
    }

    fn insert_data<F>(&mut self, key: Key, ctor: F) -> usize
        where F: FnOnce(*mut u8)
    {
        if self.table_is_full() {
            self.grow_table();
        }

        let new_entry_index = self.entries.num();
        let mut new_entry = SetEntry {
            key,
            index: new_entry_index,
            prev: usize::MAX,
            next: usize::MAX
        };
        self.setup_new_entry(&mut new_entry);

        self.entries.push_back(new_entry);

        unsafe{ self.data.allocate_back(ctor) };

        new_entry_index
    }

    pub unsafe fn remove_data<F>(&mut self, index: usize, dtor: F)
        where F: FnOnce(*mut u8)
    {
        debug_assert!(index < self.entries.num());

        let last_entry_index = self.entries.num() - 1;

        // Fix removed entry prev & next indices after swap operation
        let mut removed_entry = self.entries.swap_remove(index);
        removed_entry.prev = if removed_entry.prev == last_entry_index {
            index
        } else {
            removed_entry.prev
        };
        removed_entry.next = if removed_entry.next == last_entry_index {
            index
        } else {
            removed_entry.next
        };

        // Fix prev & next indices after removing removed_entry
        if removed_entry.prev == usize::MAX {
            let table_index = removed_entry.key.fast_hash() % self.table.num();
            self.table[table_index] = removed_entry.next;
        } else {
            self.entries[removed_entry.prev].next = removed_entry.next;
        }

        if removed_entry.next != usize::MAX {
            self.entries[removed_entry.next].prev = removed_entry.prev;
        }

        self.data.swap_remove(index, dtor);

        if index == last_entry_index {
            return;
        }

        self.entries[index].index = index;

        // Fix prev & next indices after moving last entry in index
        let moved_entry_copy = self.entries[index];

        if moved_entry_copy.prev == usize::MAX {
            let table_index = moved_entry_copy.key.fast_hash() % self.table.num();
            self.table[table_index] = index;
        } else {
            self.entries[moved_entry_copy.prev].next = index;
        }

        if moved_entry_copy.prev != usize::MAX {
            self.entries[moved_entry_copy.next].prev = index;
        }
    }
}
