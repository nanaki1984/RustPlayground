use std::slice;
use std::str;
use std::ptr::{self};

use crate::fast_hash::{self};
use crate::raw_set::RawSet;
use crate::array::Array;

static mut STRINGS_TABLE: StringsTable = StringsTable::new();

struct StringsTableEntry {
    hash: usize,
    ptr: *const u8,
    len: usize,
}

impl StringsTableEntry {
    unsafe fn as_bytes(&self) -> &[u8] {
        slice::from_raw_parts(self.ptr, self.len)
    }

    unsafe fn as_str(&self) -> &str {
        str::from_utf8_unchecked(self.as_bytes())
    }
}

struct StringsTable {
    table: RawSet,
    data: Array<u8>,
}

unsafe impl Sync for StringsTable { }

impl Drop for StringsTable {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(
                ptr::slice_from_raw_parts_mut(
                    self.table.as_mut_ptr().cast::<StringsTableEntry>(),
                    self.table.num()
                )
            );
        }
    }
}

impl StringsTable {
    const fn new() -> Self {
        Self {
            table: RawSet::for_type::<StringsTableEntry>(),
            data: Array::new(),
        }
    }

    fn get_or_add_string(&mut self, hash: usize, bytes: &[u8]) -> StringAtom {
        let mut entry_index = self.table.find_first_index(hash);
        while entry_index != usize::MAX {
            let entry_bytes = unsafe {
                (&*self.table
                    .as_ptr()
                    .cast::<StringsTableEntry>()
                    .add(entry_index))
                    .as_bytes()
            };
            if bytes.eq_ignore_ascii_case(entry_bytes) {
                break;
            }
            entry_index = self.table.find_next_index(entry_index);
        }

        if entry_index == usize::MAX {
            let new_string_offset = self.data.num();
            let new_string_len = bytes.len();

            self.data.reserve(new_string_len);
            for i in 0..new_string_len {
                self.data.push_back(bytes[i]);
            }

            let new_string_ptr = unsafe{ self.data.as_ptr().add(new_string_offset) };
            entry_index = unsafe {
                self.table.insert_data(hash, |ptr| {
                    ptr::write(ptr.cast::<StringsTableEntry>(), StringsTableEntry {
                        hash: hash,
                        ptr: new_string_ptr,
                        len: new_string_len
                    })
                })
            };
        }

        let entry = unsafe {
            &*self.table
                .as_ptr()
                .cast::<StringsTableEntry>()
                .add(entry_index)
        };
        StringAtom {
            hash,
            ptr: entry.ptr,
            len: entry.len
        }
    }
}

pub struct StringAtom {
    hash: usize,
    ptr: *const u8,
    len: usize,
}

impl StringAtom {
    fn from_str(string: &str) -> Self {
        let string_bytes = string.as_bytes();
        unsafe{ STRINGS_TABLE.get_or_add_string(fast_hash::fnv_hash(string_bytes) as usize, string_bytes) }
    }

    fn new<const N: usize>(string: &[u8; N]) -> Self {
        let hash = fast_hash::fnv_hash_const(string) as usize;
        unsafe{ STRINGS_TABLE.get_or_add_string(hash, string) }
    }

    fn as_str(&self) -> &str {
        unsafe{ str::from_utf8_unchecked(slice::from_raw_parts(self.ptr, self.len)) }
    }
}
