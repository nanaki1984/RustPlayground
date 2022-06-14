use std::cmp::{max};
use std::alloc::{Layout};
use std::ptr::{self, NonNull};

use crate::alloc::AllocatorBase;

pub(crate) struct RawArray<A: AllocatorBase> {
    data: NonNull<u8>,
    items_layout: Layout,
    items_num: usize,
    items_cap: usize,
    allocator: A,
}

impl<A: AllocatorBase> Drop for RawArray<A> {
    fn drop(&mut self) {
        if self.items_cap > 0 {
            unsafe{ self.allocator.dealloc(self.data.as_ptr(), self.current_layout()) };
        }
    }
}

impl<A: AllocatorBase> RawArray<A> {
    #[inline]
    pub unsafe fn for_type_unchecked(layout: Layout) -> Self {
        RawArray {
            data: NonNull::dangling(),
            items_layout: layout,
            items_num: 0,
            items_cap: 0,
            allocator: Default::default(),
        }
    }

    #[inline]
    pub unsafe fn for_type_with_capacity_unchecked(layout: Layout, capacity: usize) -> Self {
        let mut raw_array = Self::for_type_unchecked(layout);
        raw_array.set_capacity(capacity);
        raw_array
    }

    #[inline]
    pub fn for_type<T: Sized>() -> Self {
        unsafe{ Self::for_type_unchecked(Layout::new::<T>()) }
    }

    #[inline]
    pub fn for_type_with_capacity<T: Sized>(capacity: usize) -> Self {
        unsafe{ Self::for_type_with_capacity_unchecked(Layout::new::<T>(), capacity) }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.items_cap
    }

    #[inline]
    pub fn current_layout(&self) -> Layout {
        let alloc_size = self.items_layout.size() * self.items_cap;
        unsafe{ Layout::from_size_align_unchecked(alloc_size, self.items_layout.align()) }
    }

    #[inline]
    fn change_layout(&mut self, new_layout: Layout) {
        debug_assert!(new_layout.size() % self.items_layout.size() == 0);

        let new_ptr = unsafe{ self.allocator.alloc(new_layout) };
        if let Some(new_data) = NonNull::new(new_ptr) {
            if self.items_num > 0 {
                unsafe{ ptr::copy_nonoverlapping(self.data.as_ptr(), new_data.as_ptr(), self.items_layout.size() * self.items_num) };
            }

            if self.items_cap > 0 {
                unsafe{ self.allocator.dealloc(self.data.as_ptr(), self.current_layout()) };
            }

            self.data = new_data;
            self.items_cap = new_layout.size() / self.items_layout.size();
        }
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        let new_capacity = max(self.items_num, capacity);
        if new_capacity != self.items_cap {
            if let Some(new_alloc_size) = self.items_layout.size().checked_mul(new_capacity) {
                let new_layout = unsafe{ Layout::from_size_align_unchecked(new_alloc_size, self.items_layout.align()) };
                self.change_layout(new_layout);
            }
        }
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let wanted_capacity = self.items_num + additional;
        if wanted_capacity > self.items_cap {
            self.set_capacity(wanted_capacity);
        }
    }

    #[inline]
    pub fn num(&self) -> usize {
        self.items_num
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.num() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_ptr()
    }

    #[inline]
    fn grow_if_needed(&mut self, new_items_num: usize) {
        if new_items_num > self.items_cap {
            let new_layout = unsafe {
                self.allocator.grow(self.current_layout(), new_items_num * self.items_layout.size())
            };
            self.change_layout(new_layout);
        }
    }

    pub unsafe fn allocate_front<F>(&mut self, ctor: F)
        where F: FnOnce(*mut u8)
    {
        //if self.items_num == self.items_cap {
        //    self.set_capacity(1 + self.items_cap * 2);
        //}
        self.grow_if_needed(self.items_num + 1);

        let single_item_size = self.items_layout.size();
        ptr::copy(
            self.data.as_ptr(),
            self.data.as_ptr().add(single_item_size),
            self.items_num * single_item_size);

        ctor(self.data.as_ptr());

        self.items_num += 1;
    }

    pub unsafe fn allocate_back<F>(&mut self, ctor: F)
        where F: FnOnce(*mut u8)
    {
        //if self.items_num == self.items_cap {
        //    self.set_capacity(1 + self.items_cap * 2);
        //}
        self.grow_if_needed(self.items_num + 1);

        ctor(self.data
                .as_ptr()
                .add(self.items_num * self.items_layout.size()));

        self.items_num += 1;
    }

    pub unsafe fn allocate_at<F>(&mut self, index: usize, ctor: F)
        where F: FnOnce(*mut u8)
    {
        debug_assert!(index <= self.items_num);

        //if self.items_num == self.items_cap {
        //    self.set_capacity(1 + self.items_cap * 2);
        //}
        self.grow_if_needed(self.items_num + 1);

        if index == 0 {
            self.allocate_front(ctor);
        } else if index == self.items_num {
            self.allocate_back(ctor);
        } else {
            let single_item_size = self.items_layout.size();

            let ptr = self.data.as_ptr().add(index * single_item_size);
            ptr::copy(
                ptr,
                ptr.add(single_item_size),
                (self.items_num - index) * single_item_size);    

            ctor(ptr);

            self.items_num += 1;
        }
    }

    pub unsafe fn swap_remove<F>(&mut self, index: usize, dtor: F)
        where F: FnOnce(*mut u8)
    {
        debug_assert!(index < self.items_num);

        let single_item_size = self.items_layout.size();
        self.items_num -= 1;

        let ptr_index = self.data.as_ptr().add(index * single_item_size);
        let ptr_end = self.data.as_ptr().add(self.items_num * single_item_size);
        dtor(ptr_index);

        if ptr_index != ptr_end {
            ptr::swap_nonoverlapping(ptr_index, ptr_end, single_item_size);
        }
    }

    pub unsafe fn remove<F>(&mut self, index: usize, dtor: F)
        where F: FnOnce(*mut u8)
    {
        debug_assert!(index < self.items_num);

        let single_item_size = self.items_layout.size();
        self.items_num -= 1;

        let ptr = self.data.as_ptr().add(index * single_item_size);
        dtor(ptr);

        if self.items_num > 0 {
            ptr::copy(
                ptr.add(single_item_size),
                ptr,
                self.items_num * single_item_size);
        }
    }

    pub unsafe fn clear<F>(&mut self, slice_dtor: F)
        where F: FnOnce(*mut u8, usize)
    {
        let old_items_num = self.items_num;
        self.items_num = 0;
        slice_dtor(self.data.as_ptr(), old_items_num);
    }
}
