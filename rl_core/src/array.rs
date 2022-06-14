use std::ptr::{self};
use std::mem::MaybeUninit;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::{self, SliceIndex};

use crate::alloc::{AllocatorBase, DefaultAllocator};
use crate::raw_array::RawArray;

pub struct Array<T, A = DefaultAllocator>(RawArray<A>, PhantomData<T>) where
    T: Sized + Unpin,
    A: AllocatorBase;

impl<T: Sized + Unpin> Array<T> {
    #[inline]
    pub fn new() -> Self {
        Array(RawArray::for_type::<T>(), PhantomData)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Array(RawArray::for_type_with_capacity::<T>(capacity), PhantomData)
    }
}

impl<T, A> Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase
{
    #[inline]
    pub fn custom_allocator() -> Self {
        Array(RawArray::<A>::for_type::<T>(), PhantomData)
    }

    #[inline]
    pub fn custom_allocator_with_capacity(capacity: usize) -> Self {
        Array(RawArray::<A>::for_type_with_capacity::<T>(capacity), PhantomData)
    }
}

impl<T, A> Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase
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
    pub fn push_front(&mut self, value: T) {
        unsafe {
            self.0.allocate_front(|ptr| {
                ptr::write(ptr.cast::<T>(), value)
            });
        }
    }

    #[inline]
    pub fn push_back(&mut self, value: T) {
        unsafe {
            self.0.allocate_back(|ptr| {
                ptr::write(ptr.cast::<T>(), value)
            });
        };
    }

    #[inline]
    pub fn insert(&mut self, index: usize, value: T) {
        unsafe {
            self.0.allocate_at(index, |ptr| {
                ptr::write(ptr.cast::<T>(), value)
            });
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let mut tmp = MaybeUninit::<T>::uninit();
        unsafe {
            self.0.swap_remove(index, |ptr| {
                tmp.write(ptr::read(ptr.cast::<T>()));
            });
            tmp.assume_init()
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        let mut tmp = MaybeUninit::<T>::uninit();
        unsafe {
            self.0.remove(index, |ptr| {
                tmp.write(ptr::read(ptr.cast::<T>()));
            });
            tmp.assume_init()
        }
    }

    #[inline]
    pub fn pop_front(&mut self) -> T {
        self.remove(0)
    }

    #[inline]
    pub fn pop_back(&mut self) -> T {
        self.remove(self.num() - 1)
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

impl<T, A> Deref for Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase
{
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.num()) }
    }
}

impl<T, A> DerefMut for Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase
{
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.num()) }
    }
}

impl<T, A, I> Index<I> for Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase,
    I: SliceIndex<[T]>
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, A, I> IndexMut<I> for Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase,
    I: SliceIndex<[T]>
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<'a, T, A> IntoIterator for &'a Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase
{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T, A> IntoIterator for &'a mut Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase
{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T, A> Drop for Array<T, A> where
    T: Sized + Unpin,
    A: AllocatorBase
{
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.num()));
        }
    }
}

impl<T: Sized + Unpin> Default for Array<T>
{
    fn default() -> Array<T> {
        Array::new()
    }
}
