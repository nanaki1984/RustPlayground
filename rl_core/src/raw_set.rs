use std::alloc::Layout;

use crate::alloc::{AllocatorBase, ArrayAllocator, DefaultAllocator};
use crate::RawArray;
use crate::Array;

#[derive(Copy, Clone)]
pub struct RawSetEntry {
    hash: usize,
    index: usize,
    prev: usize,
    next: usize,
}

pub struct RawSet<DataAlloc = DefaultAllocator, EntriesAlloc = DefaultAllocator, TableAlloc = DefaultAllocator> where
    DataAlloc: AllocatorBase,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    data: RawArray<DataAlloc>,
    entries: Array<RawSetEntry, EntriesAlloc>,
    table: Array<usize, TableAlloc>,
}

impl<DataAlloc, EntriesAlloc, TableAlloc> RawSet<DataAlloc, EntriesAlloc, TableAlloc> where
    DataAlloc: AllocatorBase,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    #[inline]
    pub unsafe fn for_type_unchecked(layout: Layout) -> Self {
        Self {
            data: RawArray::<DataAlloc>::for_type_unchecked(layout),
            entries: Array::custom_allocator(),
            table: Array::custom_allocator()
        }
    }

    #[inline]
    pub unsafe fn for_type_with_table_size_unchecked(layout: Layout, table_size: usize) -> Self {
        let mut raw_set = Self {
            data: RawArray::<DataAlloc>::for_type_unchecked(layout),
            entries: Array::custom_allocator(),
            table: Array::custom_allocator_with_capacity(table_size)
        };
        raw_set.table.insert_range(0..table_size, usize::MAX);
        raw_set
    }

    #[inline]
    pub fn for_type<T>() -> Self {
        unsafe{ Self::for_type_unchecked(Layout::new::<T>()) }
    }

    #[inline]
    pub fn for_type_with_table_size<T>(table_size: usize) -> Self {
        unsafe{ Self::for_type_with_table_size_unchecked(Layout::new::<T>(), table_size) }
    }
}

impl<DataAlloc, EntriesAlloc, TableAlloc> RawSet<DataAlloc, EntriesAlloc, TableAlloc> where
    DataAlloc: AllocatorBase,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    #[inline]
    pub fn find_first_index(&self, hash: usize) -> usize {
        let mut entry_index = usize::MAX;

        if !self.table.is_empty() {
            let table_index = hash % self.table.num();
            entry_index = self.table[table_index];
            while entry_index != usize::MAX && self.entries[entry_index].hash != hash {
                entry_index = self.entries[entry_index].next;
            }
        }

        entry_index
    }

    #[inline]
    pub fn find_next_index(&self, entry_index: usize) -> usize {
        debug_assert!(entry_index != usize::MAX);

        let entry = &self.entries[entry_index];
        let next_entry_index = entry.next;

        if next_entry_index == usize::MAX || self.entries[next_entry_index].hash != entry.hash {
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
    fn setup_new_entry(&mut self, new_entry: &mut RawSetEntry) {
        let table_index = new_entry.hash % self.table.num();

        new_entry.next = self.find_first_index(new_entry.hash);
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

    pub fn rehash(&mut self, new_table_size: usize) {
        debug_assert!(new_table_size > 0);

        self.table.clear();
        self.table.set_capacity(new_table_size);
        self.table.insert_range(0..new_table_size, usize::MAX);

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

    pub fn insert_data<F>(&mut self, hash: usize, ctor: F) -> usize
        where F: FnOnce(*mut u8)
    {
        if self.table_is_full() {
            self.grow_table();
        }

        let new_entry_index = self.entries.num();
        let mut new_entry = RawSetEntry {
            hash,
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
            let table_index = removed_entry.hash % self.table.num();
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
            let table_index = moved_entry_copy.hash % self.table.num();
            self.table[table_index] = index;
        } else {
            self.entries[moved_entry_copy.prev].next = index;
        }

        if moved_entry_copy.next != usize::MAX {
            self.entries[moved_entry_copy.next].prev = index;
        }
    }

    pub unsafe fn clear<F>(&mut self, slice_dtor: F)
        where F: FnOnce(*mut u8, usize)
    {
        self.data.clear(slice_dtor);

        self.entries.clear();

        for index in &mut self.table {
            *index = usize::MAX;
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    #[inline]
    pub fn set_capacity(&mut self, capacity: usize) {
        self.entries.set_capacity(capacity);
        self.data.set_capacity(capacity);
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let wanted_capacity = self.entries.num() + additional;
        if wanted_capacity > self.capacity() {
            self.set_capacity(wanted_capacity);
        }
    }

    #[inline]
    pub fn num(&self) -> usize {
        self.entries.num()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    #[inline]
    pub fn get_hash(&self, index: usize) -> usize {
        self.entries[index].hash
    }

    #[inline]
    pub unsafe fn get_data_ptr(&self, index: usize) -> *const u8 {
        self.data.get_ptr(index)
    }

    #[inline]
    pub unsafe fn get_data_ptr_mut(&mut self, index: usize) -> *mut u8 {
        self.data.get_ptr_mut(index)
    }
}
