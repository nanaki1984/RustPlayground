use std::ptr::{self};
use std::mem::MaybeUninit;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::{self, SliceIndex};
use std::option::Option;
use std::borrow::Borrow;

use crate::alloc::{AllocatorBase, DefaultAllocator, ArrayAllocator};
use crate::{SetItem, FastHash};
use crate::RawSet;
use crate::RawSetEntry;
use crate::Array;

pub struct Set<T, DataAlloc = DefaultAllocator, EntriesAlloc = DefaultAllocator, TableAlloc = DefaultAllocator>
(
    RawSet<DataAlloc, EntriesAlloc, TableAlloc>,
    PhantomData<T>,
) where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>;

impl<T: SetItem> Set<T> {
    #[inline]
    pub fn new() -> Self {
        Set(RawSet::for_type::<T>(), PhantomData)
    }

    #[inline]
    pub fn with_table_size(table_size: usize) -> Self {
        Set(RawSet::for_type_with_table_size::<T>(table_size), PhantomData)
    }
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc> Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    #[inline]
    pub fn custom_allocators() -> Self {
        Set(RawSet::for_type::<T>(), PhantomData)
    }

    #[inline]
    pub fn custom_allocators_with_table_size(table_size: usize) -> Self {
        Set(RawSet::for_type_with_table_size::<T>(table_size), PhantomData)
    }
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc> Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
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
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr().cast::<T>()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr().cast::<T>()
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    #[inline]
    pub fn insert(&mut self, value: T) -> usize {
        unsafe {
            self.0.insert_data(value.get_key().fast_hash(), |ptr| {
                ptr::write(ptr.cast::<T>(), value)
            })
        }
    }

    #[inline]
    pub fn remove_all<A: AllocatorBase, Q: ?Sized>(&mut self, key: &Q) -> Array<T, A> where
        T::KeyType: Borrow<Q>,
        Q: FastHash + Eq
    {
        let mut array = Array::<T, A>::custom_allocator();

        let mut index = self.0.find_first_index(key.fast_hash());
        while index != usize::MAX {
            let next_index = self.0.find_next_index(index);

            if self[index].get_key().borrow() == key {
                unsafe {
                    self.0.remove_data(index, |ptr| {
                        array.push_back(ptr::read(ptr.cast::<T>()));
                    });
                }    
            }

            // Change index to next_index only if is valid
            // (next_index == self.num() means next_index was the last element that has been moved on the same index now)
            if next_index != self.0.num() {
                index = next_index;
            }
        }

        array
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let mut tmp = MaybeUninit::<T>::uninit();
        unsafe {
            self.0.remove_data(index, |ptr| {
                tmp.write(ptr::read(ptr.cast::<T>()));
            });
            tmp.assume_init()
        }
    }

    #[inline]
    pub fn get_element_index(&self, element: &T) -> Option<usize> {
        let elem_ptr = element as *const T;
        let elem_idx = unsafe{ elem_ptr.offset_from(self.as_ptr()) };
        if elem_idx >= 0 && elem_idx < self.num().try_into().unwrap() {
            Option::Some(elem_idx as usize)
        } else {
            Option::None
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            self.0.clear(|ptr, num| {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut(ptr.cast::<T>(), num));
            });
        }
    }

    pub fn find_first_index<Q: ?Sized>(&self, key: &Q) -> usize where
        T::KeyType: Borrow<Q>,
        Q: FastHash + Eq
    {
        let mut first_elem_index = self.0.find_first_index(key.fast_hash());
        while first_elem_index != usize::MAX {
            if self[first_elem_index].get_key().borrow() == key {
                return first_elem_index;
            }
            first_elem_index = self.0.find_next_index(first_elem_index);
        }
        usize::MAX
    }

    pub fn find_next_index(&self, index: usize) -> usize {
        debug_assert!(index != usize::MAX && index < self.num());
        let key = self[index].get_key();
        let mut next_elem_index = self.0.find_next_index(index);
        while next_elem_index != usize::MAX {
            if self[next_elem_index].get_key() == key {
                return next_elem_index;
            }
            next_elem_index = self.0.find_next_index(next_elem_index);
        }
        usize::MAX
    }

    pub fn find_index_or_insert_mut(&mut self, value: T) -> usize {
        let key = value.get_key();
        let mut first_elem_index = self.0.find_first_index(key.fast_hash());
        while first_elem_index != usize::MAX {
            if self[first_elem_index].get_key() == key {
                return first_elem_index;
            }
            first_elem_index = self.0.find_next_index(first_elem_index);
        }
        self.insert(value)
    }
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc> Deref for Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.num()) }
    }
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc> DerefMut for Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.num()) }
    }
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc, I> Index<I> for Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>,
    I: SliceIndex<[T]>
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc, I> IndexMut<I> for Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>,
    I: SliceIndex<[T]>
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<'a, T, DataAlloc, EntriesAlloc, TableAlloc> IntoIterator for &'a Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T, DataAlloc, EntriesAlloc, TableAlloc> IntoIterator for &'a mut Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc> Drop for Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<RawSetEntry>,
    TableAlloc: ArrayAllocator<usize>
{
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.num()));
        }
    }
}

impl<T: SetItem> Default for Set<T>
{
    fn default() -> Set<T> {
        Set::new()
    }
}
