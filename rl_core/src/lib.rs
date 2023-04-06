mod fast_hash;

pub use fast_hash::fnv_hash_const;
pub use fast_hash::fnv_hash;

pub use fast_hash::FastHash;
pub use fast_hash::SetKey;
pub use fast_hash::SetItem;
pub use fast_hash::KeyValuePair;

pub mod alloc;

mod strings_table;

pub use strings_table::StringAtom;

mod raw_array;
mod raw_set;

pub use raw_array::RawArray;
pub use raw_set::RawSet;
pub use raw_set::RawSetEntry;

mod array;
mod set;
mod map;
// ToDo: multimap

pub use array::Array;
pub use array::InlineArray;
pub use set::Set;
pub use map::Map;

//mod typed;
//mod object;

#[cfg(test)]
mod tests;
