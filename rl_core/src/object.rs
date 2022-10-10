use std::ptr::{self};
use std::any::TypeId;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::Borrow;

use crate::raw_set::RawSet;
use crate::typed::{Typed, TypeInfo};
use crate::fast_hash::{FastHash, SetItem};

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

pub struct Obj<T: Typed + SetItem> {
    value: T,
    borrow_state: ObjBorrowState,
    pending_destroy: bool,
}

impl<T: Typed + SetItem> Obj<T> {
    pub fn new(value: T) -> Self {
        Obj{ value, borrow_state: ObjBorrowState::new(), pending_destroy: false }
    }

    pub fn get_unique_id(&self) -> &T::KeyType {
        self.value.get_key()
    }

    pub fn is_pending_destroy(&self) -> bool {
        self.pending_destroy
    }

    pub fn destroy(&mut self) {
        self.pending_destroy = true;
    }
}

struct ObjStorage {
    type_info: TypeInfo, // type id is enough!
    set_lock: RwLock<RawSet>, // use RwLock to create objects from any thread
}

impl SetItem for ObjStorage {
    type KeyType = TypeId;

    fn get_key(&self) -> &Self::KeyType {
        self.type_info.get_id()
    }
}

impl ObjStorage {
    pub fn new<T: Typed + SetItem>() -> Self {
        ObjStorage { type_info: TypeInfo::of::<T>(), set_lock: RwLock::new(RawSet::for_type::<T>()) }
    }

    pub fn with_table_size<T: Typed + SetItem>(table_size: usize) -> Self {
        ObjStorage { type_info: TypeInfo::of::<T>(), set_lock: RwLock::new(RawSet::for_type_with_table_size::<T>(table_size)) }
    }

    fn find_obj_index<T: Typed + SetItem, Q: ?Sized>(&self, unique_id: &Q) -> Option<usize> where
        T::KeyType: Borrow<Q>,
        Q: FastHash + Eq
    {
        let set_read = self.set_lock.read()?;
        let mut first_elem_index = set_read.find_first_index(unique_id.fast_hash());
        while first_elem_index != usize::MAX {
            if (*set_read)[first_elem_index].get_unique_id().borrow() == unique_id {
                return Some(first_elem_index);
            }
            first_elem_index = set_read.find_next_index(first_elem_index);
        }
        None
    }

    pub fn insert<T: Typed + SetItem>(&self, value: T) -> usize {
        //debug_assert!(TypeId::of::<T>() == self.type_info.get_id()); or use Option<usize> and ? instead of unwrap
        let new_object = Obj::<T>::new(value);
        let mut set_write = self.set_lock.write().unwrap();
        unsafe {
            set_write.insert_data(new_object.get_unique_id().fast_hash(), |ptr| {
                ptr::write(ptr.cast::<Obj<T>>(), new_object)
            })
        }
    }

    pub fn get<T: Typed + SetItem>(&self, unique_id: T::KeyType) -> &T {
        let set_read = self.set_lock.read().unwrap();
        
    }
}

pub struct ObjHandle<T: Typed + SetItem> {
    type_id: TypeId, // no need, I have the type
    unique_id: T::KeyType,
}
