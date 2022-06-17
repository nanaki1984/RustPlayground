use std::ptr::{self};
use std::mem::MaybeUninit;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::{self, SliceIndex};

use crate::alloc::{ArrayAllocator, DefaultAllocator};
use crate::fast_hash::SetItem;
use crate::raw_set::RawSet;
use crate::raw_set::RawSetEntry;

pub type SetEntry<T> = RawSetEntry<<T as SetItem>::KeyType>;

pub struct Set<T, DataAlloc = DefaultAllocator, EntriesAlloc = DefaultAllocator, TableAlloc = DefaultAllocator>
(
    RawSet<T::KeyType, DataAlloc, EntriesAlloc, TableAlloc>,
    PhantomData<T>,
) where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
    pub fn num_with_key(&self, key: T::KeyType) -> usize {
        self.0.num_with_key(key)
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
    pub fn rehash(&mut self) {
        self.0.rehash();
    }

    #[inline]
    pub fn insert(&mut self, value: T) {
        unsafe {
            self.0.insert_data(value.get_key(), |ptr| {
                ptr::write(ptr.cast::<T>(), value)
            });
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        let mut tmp = MaybeUninit::<T>::uninit();
        unsafe {
            self.0.remove_data(index, |ptr| {
                tmp.write(ptr::read(ptr.cast::<T>()));
            });
            tmp.assume_init()
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
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc> Deref for Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
    TableAlloc: ArrayAllocator<usize>
{
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.num()) }
    }
}

impl<T, DataAlloc, EntriesAlloc, TableAlloc, I> Index<I> for Set<T, DataAlloc, EntriesAlloc, TableAlloc> where
    T: SetItem,
    DataAlloc: ArrayAllocator<T>,
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
    EntriesAlloc: ArrayAllocator<SetEntry<T>>,
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
