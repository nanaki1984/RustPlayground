use core::fmt;
use std::slice;
use std::str;
use std::ptr::{self, NonNull};
use std::mem::MaybeUninit;

use crate::fast_hash::{self, FastHash, SetKey};
use crate::raw_set::RawSet;

static mut STRINGS_TABLE: StringsTable = StringsTable::new(); // TODO multithreaded access (just use a mutex...)
// would also be nice to have a const new on StringAtom using a 'static &str and keeping that in the entry (no allocation...but gets a lot more complicated)
// const new for StringAtom is a must because of the mutex to be added (i can keep const StringAtoms around to not create them @ runtime)
// The mutex in unreal is on shards, not on the whole table (every shard is a table, e.g. shards can be last 4 bits of hash, 16 shards)

const STRINGS_TABLE_ENTRY_MAX_LEN: usize = 128;

struct StringsTableEntry {
    data: [u8; STRINGS_TABLE_ENTRY_MAX_LEN],
    len: usize,
}

impl StringsTableEntry {
    fn new(bytes: &[u8]) -> Self {
        let mut new_entry = StringsTableEntry {
            data: [0; STRINGS_TABLE_ENTRY_MAX_LEN],
            len: bytes.len()
        };
        unsafe{ ptr::copy_nonoverlapping(bytes.as_ptr(), new_entry.data.as_mut_ptr(), bytes.len()) };
        new_entry
    }

    unsafe fn as_bytes(&self) -> &[u8] {
        slice::from_raw_parts(self.data.as_ptr(), self.len)
    }

    unsafe fn as_str(&self) -> &str {
        str::from_utf8_unchecked(self.as_bytes())
    }
}

struct StringsTable {
    initialized: bool,
    table: MaybeUninit<RawSet>,
}

unsafe impl Sync for StringsTable { }

impl Drop for StringsTable {
    fn drop(&mut self) {
        unsafe {
            if self.initialized {
                let table = self.table.assume_init_mut();
                ptr::drop_in_place(
                    ptr::slice_from_raw_parts_mut(
                        table.as_mut_ptr().cast::<StringsTableEntry>(),
                        table.num()
                    )
                );
                self.table.assume_init_drop();
            }
        }
    }
}

impl StringsTable {
    const fn new() -> Self {
        Self {
            initialized: false,
            table: MaybeUninit::uninit(),
        }
    }

    fn lazy_init(&mut self) {
        if !self.initialized {
            self.table = MaybeUninit::new(RawSet::for_type::<StringsTableEntry>());
            self.initialized = true;
        }
    }

    fn get_or_add_string(&mut self, hash: usize, bytes: &[u8]) -> StringAtom {
        self.lazy_init();

        debug_assert!(bytes.len() <= STRINGS_TABLE_ENTRY_MAX_LEN);

        let mut entry_index = unsafe{ self.table.assume_init_ref().find_first_index(hash) };
        while entry_index != usize::MAX {
            let entry_bytes = unsafe {
                (&*self.table
                    .assume_init_ref()
                    .as_ptr()
                    .cast::<StringsTableEntry>()
                    .add(entry_index))
                    .as_bytes()
            };
            if bytes.eq_ignore_ascii_case(entry_bytes) {
                break;
            }
            entry_index = unsafe{ self.table.assume_init_ref().find_next_index(entry_index) };
        }

        if entry_index == usize::MAX {
            entry_index = unsafe {
                self.table
                    .assume_init_mut()
                    .insert_data(hash, |ptr| {
                        ptr::write(ptr.cast::<StringsTableEntry>(), StringsTableEntry::new(bytes))
                    })
            };
        }

        let entry = unsafe {
            &*self.table
                .assume_init_ref()
                .as_ptr()
                .cast::<StringsTableEntry>()
                .add(entry_index)
        };

        StringAtom {
            hash,
            ptr: (&entry.data[0]).into(),
            len: entry.len
        }
    }
}

pub struct StringAtom {
    hash: usize,
    ptr: NonNull<u8>,
    len: usize,
}

impl StringAtom {
    pub const fn none() -> Self {
        Self {
            hash: 0,
            ptr: NonNull::dangling(),
            len: 0
        }
    }

    pub fn new<const N: usize>(string: &[u8; N]) -> Self {
        let hash = fast_hash::fnv_hash_const(string, true) as usize;
        unsafe{ STRINGS_TABLE.get_or_add_string(hash, string) }
    }

    pub fn is_none(&self) -> bool {
        self.len == 0
    }

    pub fn as_str(&self) -> &str {
        if self.len > 0 {
            unsafe{ str::from_utf8_unchecked(slice::from_raw_parts(self.ptr.as_ptr(), self.len)) }
        } else {
            ""
        }
    }
}

impl PartialEq<Self> for StringAtom {
    fn eq(&self, other: &Self) -> bool {
        if self.is_none() {
            other.is_none()
        } else {
            !other.is_none() && self.hash == other.hash && self.ptr == other.ptr
        }
    }

    fn ne(&self, other: &Self) -> bool {
        if self.is_none() {
            !other.is_none()
        } else {
            other.is_none() || self.hash != other.hash || self.ptr != other.ptr
        }
    }
}

impl Eq for StringAtom { }

impl FastHash for StringAtom {
    fn fast_hash(&self) -> usize {
        self.hash
    }
}

impl SetKey for StringAtom { }

impl From<&str> for StringAtom {
    fn from(string: &str) -> Self {
        let string_bytes = string.as_bytes();
        unsafe{ STRINGS_TABLE.get_or_add_string(fast_hash::fnv_hash::<true>(string_bytes) as usize, string_bytes) }
    }
}

impl Into<String> for StringAtom {
    fn into(self) -> String {
        self.as_str().to_string()
    }
}

impl fmt::Debug for StringAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}