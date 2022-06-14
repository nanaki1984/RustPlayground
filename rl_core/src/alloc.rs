use std::cmp::{max};
use std::alloc::{Layout};
use std::mem::{MaybeUninit};

pub trait AllocatorBase : Default {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);

    unsafe fn grow(&self, layout: Layout, wanted_bytes: usize) -> Layout;
}

pub trait ArrayAllocator<T> : AllocatorBase { }

#[derive(Default)]
pub struct DefaultAllocator;

impl AllocatorBase for DefaultAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        std::alloc::alloc(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        std::alloc::dealloc(ptr, layout);
    }

    unsafe fn grow(&self, layout: Layout, wanted_bytes: usize) -> Layout {
        debug_assert!(wanted_bytes > layout.size());
        let new_layout_size = max(wanted_bytes, layout.size() * 2);
        Layout::from_size_align_unchecked(new_layout_size, layout.align())
    }
}

impl<T> ArrayAllocator<T> for DefaultAllocator { }

pub struct InlineAllocator<const N: usize, T> {
    inline_data: MaybeUninit<[T; N]>,
}

impl<const N: usize, T> Default for InlineAllocator<N, T> {
    fn default() -> Self {
        InlineAllocator{ inline_data: MaybeUninit::uninit() }
    }
}

impl<const N: usize, T> AllocatorBase for InlineAllocator<N, T> {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let inline_data_size = std::mem::size_of::<[T; N]>();
        if layout.size() <= inline_data_size {
            (*self.inline_data.as_mut_ptr())
                .as_mut_ptr()
                .cast::<u8>()
        } else {
            std::alloc::alloc(layout)
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let inline_data_ptr = (*self.inline_data.as_mut_ptr()).as_mut_ptr();
        if ptr.cast::<T>() != inline_data_ptr {
            std::alloc::dealloc(ptr, layout);
        }
    }

    unsafe fn grow(&self, layout: Layout, wanted_bytes: usize) -> Layout {
        debug_assert!(wanted_bytes > layout.size());
        let inline_data_size = std::mem::size_of::<[T; N]>();
        if wanted_bytes <= inline_data_size {
            Layout::from_size_align_unchecked(wanted_bytes, layout.align())
        } else {
            let new_layout_size = max(wanted_bytes, layout.size() * 2);
            Layout::from_size_align_unchecked(new_layout_size, layout.align())
        }
    }
}

impl<const N: usize, T> ArrayAllocator<T> for InlineAllocator<N, T> { }
