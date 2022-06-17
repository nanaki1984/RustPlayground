use std::ptr::{self};
use std::mem::MaybeUninit;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::{self, SliceIndex};
use std::option::Option;

use crate::alloc::{AllocatorBase, DefaultAllocator, ArrayAllocator};
use crate::fast_hash::SetItem;
use crate::raw_set::RawSet;
use crate::raw_set::RawSetEntry;
use crate::array::Array;

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
    pub fn insert(&mut self, value: T) {
        unsafe {
            self.0.insert_data(value.get_key(), |ptr| {
                ptr::write(ptr.cast::<T>(), value)
            });
        }
    }

    #[inline]
    pub fn remove_all<A: AllocatorBase>(&mut self, key: T::KeyType) -> Array<T, A> {
        let mut array = Array::<T, A>::custom_allocator();

        let mut index = self.0.find_first_index(key);
        while index != usize::MAX {
            if T::IMMUTABLE_KEY {
                unsafe {
                    self.0.remove_data(index, |ptr| {
                        array.push_back(ptr::read(ptr.cast::<T>()));
                    });
                }
                index = self.0.find_first_index(key);
            } else {
                let next_index = self.0.find_next_index(index);

                if self[index].get_key() == key {
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
    pub fn get_element_index<'a>(&'a self, element: &'a T) -> Option<usize> {
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

    pub fn find_first<'a>(&'a self, key: T::KeyType) -> Option<&'a T> {
        if T::IMMUTABLE_KEY {
            let first_elem_index = self.0.find_first_index(key);
            if first_elem_index != usize::MAX {
                return Option::Some(&self[first_elem_index]);
            }
        } else {
            let mut first_elem_index = self.0.find_first_index(key);
            while first_elem_index != usize::MAX {
                if self[first_elem_index].get_key() == key {
                    return Option::Some(&self[first_elem_index]);
                }
                first_elem_index = self.0.find_next_index(first_elem_index);
            }
        }
        Option::None
    }

    pub fn find_first_mut<'a>(&'a mut self, key: T::KeyType) -> Option<&'a mut T> {
        if T::IMMUTABLE_KEY {
            let first_elem_index = self.0.find_first_index(key);
            if first_elem_index != usize::MAX {
                return Option::Some(&mut self[first_elem_index]);
            }
        } else {
            let mut first_elem_index = self.0.find_first_index(key);
            while first_elem_index != usize::MAX {
                if self[first_elem_index].get_key() == key {
                    return Option::Some(&mut self[first_elem_index]);
                }
                first_elem_index = self.0.find_next_index(first_elem_index);
            }
        }
        Option::None
    }

    pub fn find_next<'a>(&'a self, current: &'a T) -> Option<&'a T> {
        let current_key = current.get_key();
        let elem_ptr = current as *const T;
        let elem_idx = unsafe{ elem_ptr.offset_from(self.as_ptr()) };
        if elem_idx >= 0 && elem_idx < self.num().try_into().unwrap() {
            if T::IMMUTABLE_KEY {
                let next_elem_idx = self.0.find_next_index(elem_idx as usize);
                if next_elem_idx != usize::MAX {
                    return Option::Some(&self[next_elem_idx]);
                }
            } else {
                let mut next_elem_idx = self.0.find_next_index(elem_idx as usize);
                while next_elem_idx != usize::MAX {
                    if self[next_elem_idx].get_key() == current_key {
                        return Option::Some(&self[next_elem_idx]);
                    }
                    next_elem_idx = self.0.find_next_index(next_elem_idx);
                }
            }
        }
        Option::None
    }

    pub fn find_next_mut<'a>(&'a mut self, current: &'a mut T) -> Option<&'a mut T> {
        let current_key = current.get_key();
        let elem_ptr = current as *mut T;
        let elem_idx = unsafe{ elem_ptr.offset_from(self.as_mut_ptr()) };
        if elem_idx >= 0 && elem_idx < self.num().try_into().unwrap() {
            if T::IMMUTABLE_KEY {
                let next_elem_idx = self.0.find_next_index(elem_idx as usize);
                if next_elem_idx != usize::MAX {
                    return Option::Some(&mut self[next_elem_idx]);
                }
            } else {
                let mut next_elem_idx = self.0.find_next_index(elem_idx as usize);
                while next_elem_idx != usize::MAX {
                    if self[next_elem_idx].get_key() == current_key {
                        return Option::Some(&mut self[next_elem_idx]);
                    }
                    next_elem_idx = self.0.find_next_index(next_elem_idx);
                }
            }
        }
        Option::None
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
