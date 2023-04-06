use std::ptr::{self};
use std::mem::{MaybeUninit, ManuallyDrop};
use std::any::TypeId;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::Borrow;

use crate::RawSet;
use crate::typed::TypeInfo;
use crate::{FastHash, SetItem};

struct ObjBorrowState(AtomicUsize);

const HIGH_BIT: usize = !(usize::MAX >> 1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjBorrowError {
    /// The object was borrowed when an exclusive borrow occurred.
    Unique,
    /// The object was borrowed exclusively when a shared borrow occurred.
    Shared,
}

impl ObjBorrowState {
    #[inline]
    fn new() -> Self {
        ObjBorrowState(AtomicUsize::new(0))
    }

    #[inline]
    fn try_read(&self) -> Result<ObjBorrow<'_>, ObjBorrowError> {
        let new = self.0.fetch_add(1, Ordering::Acquire) + 1;
        if new & HIGH_BIT != 0 {
            Err(ObjBorrowError::Unique)
        } else {
            Ok(ObjBorrow(self))
        }
    }

    #[inline]
    fn try_write(&self) -> Result<ObjBorrowMut<'_>, ObjBorrowError> {
        let old = match self
            .0
            .compare_exchange(0, HIGH_BIT, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(x) => x,
            Err(x) => x,
        };

        if old == 0 {
            Ok(ObjBorrowMut(self))
        } else if old & HIGH_BIT == 0 {
            Err(ObjBorrowError::Shared)
        } else {
            Err(ObjBorrowError::Unique)
        }
    }

    #[inline]
    fn try_destroy(&self) -> bool {
        let old = match self
            .0
            .compare_exchange(0, usize::MAX, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(x) => x,
            Err(x) => x,
        };
        0 == old
    }
}

pub struct ObjBorrow<'a>(&'a ObjBorrowState);

impl Drop for ObjBorrow<'_> {
    #[inline]
    fn drop(&mut self) {
        (self.0).0.fetch_sub(1, Ordering::Release);
    }
}

impl Clone for ObjBorrow<'_> {
    #[inline]
    fn clone(&self) -> Self {
        self.0.try_read().unwrap()
    }
}

pub struct ObjBorrowMut<'a>(&'a ObjBorrowState);

impl Drop for ObjBorrowMut<'_> {
    #[inline]
    fn drop(&mut self) {
        (self.0).0.store(0, Ordering::Release);
    }
}

/*pub trait Object : 'static + SetItem {
    fn get_borrow_state(&self) -> &ObjBorrowState;

    fn is_pending_drop(&self) -> bool;
    fn request_drop(&mut self);
}*/

pub trait Object : 'static + SetItem { }

struct ObjStorageEntry<T: Object> {
    value: ManuallyDrop<T>,
    borrow_state: ObjBorrowState,
    is_pending_drop: bool,
}

impl<T: Object> SetItem for ObjStorageEntry<T> {
    type KeyType = T::KeyType;

    fn get_key(&self) -> &Self::KeyType {
        self.value.get_key()
    }
}

impl<T: Object> ObjStorageEntry<T> {
    fn new(value: T) -> Self {
        ObjStorageEntry {
            value: ManuallyDrop::new(value),
            borrow_state: ObjBorrowState::new(),
            is_pending_drop: false
        }
    }
}

struct ObjStorage {
    type_info: TypeInfo,
    set_lock: RwLock<RawSet>, // use RwLock to create objects from any thread
}

impl SetItem for ObjStorage {
    type KeyType = TypeId;

    fn get_key(&self) -> &Self::KeyType {
        &self.type_info.get_id()
    }
}

impl ObjStorage {
    pub fn new<T: Object>() -> Self {
        ObjStorage{ type_info: TypeInfo::of::<T>(), set_lock: RwLock::new(RawSet::for_type::<ObjStorageEntry<T>>()) }
    }

    pub fn with_table_size<T: Object>(table_size: usize) -> Self {
        ObjStorage{ type_info: TypeInfo::of::<T>(), set_lock: RwLock::new(RawSet::for_type_with_table_size::<ObjStorageEntry<T>>(table_size)) }
    }

    pub fn insert<T: Object>(&self, value: T) -> usize {
        debug_assert!(TypeId::of::<T>() == *self.type_info.get_id());

        // ToDo: check that there are no objects with same key

        let value_hash = value.get_key().fast_hash();
        let new_entry = ObjStorageEntry::new(value);
        let mut set_write = self.set_lock.write().unwrap();
        unsafe {
            set_write.insert_data(value_hash, |ptr| {
                ptr::write(ptr.cast::<ObjStorageEntry<T>>(), new_entry)
            })
        }
    }

    fn find_obj_index<T: Object, Q: ?Sized>(&self, unique_id: &Q) -> Option<usize> where
        T::KeyType: Borrow<Q>,
        Q: FastHash + Eq
    {
        debug_assert!(TypeId::of::<T>() == *self.type_info.get_id());

        let lock_read = self.set_lock.read().unwrap(); // ToDo: manage panic?
        let set_readonly = &*lock_read;
        let set_readonly_buffer = unsafe {
            std::slice::from_raw_parts(set_readonly.as_ptr().cast::<ObjStorageEntry<T>>(), set_readonly.num())
        };

        let mut elem_index = set_readonly.find_first_index(unique_id.fast_hash());
        while elem_index != usize::MAX {
            if set_readonly_buffer[elem_index].get_key().borrow() == unique_id {
                return Some(elem_index);
            }
            elem_index = set_readonly.find_next_index(elem_index);
        }

        None
    }

    pub fn request_drop<T: Object, Q: ?Sized>(&self, unique_id: &Q) where
        T::KeyType: Borrow<Q>,
        Q: FastHash + Eq
    {

    }

    pub fn try_get<T: Object, Q: ?Sized>(&self, unique_id: &Q) -> Option<&T> where // ToDo: I should manage the borrows here
        T::KeyType: Borrow<Q>,
        Q: FastHash + Eq
    {
        debug_assert!(TypeId::of::<T>() == *self.type_info.get_id());

        let lock_read = self.set_lock.read().unwrap(); // ToDo: manage panic?
        let set_readonly = &*lock_read;
        let set_readonly_buffer = unsafe {
            std::slice::from_raw_parts(set_readonly.as_ptr().cast::<T>(), set_readonly.num())
        };

        let mut first_elem_index = set_readonly.find_first_index(unique_id.fast_hash());
        while first_elem_index != usize::MAX {
            let object = &set_readonly_buffer[first_elem_index];
            if object.get_key().borrow() == unique_id {
                return Some(object);
            }
            first_elem_index = set_readonly.find_next_index(first_elem_index);
        }

        None
    }
/*
    pub fn prune(&self)
    {
        let mut set_write = self.set_lock.write().unwrap();
        let set_num = set_write.num();
        let set_readonly_buffer = unsafe {
            std::slice::from_raw_parts(set_write.as_ptr().cast::<T>(), set_write.num())
        };

        for obj_cell_index in 0..set_num {
            let object = &set_readonly_buffer[obj_cell_index];
            if object.is_pending_drop() {
                if object.get_borrow_state().try_destroy() {
                    unsafe {
                        set_write.remove_data(obj_cell_index, |ptr| {
                            self.type_info.drop_in_place(ptr);
                        });
                    }
                }
            }
        }
    }*/
}

pub struct ObjHandle<T: Object> { // ToDo: make it an enum, to be able to have "Zero"/Null Handles (also default value)
    type_id: TypeId, // no need, I have the type
    unique_id: T::KeyType,
}
