pub mod fast_hash;
pub mod alloc;
pub mod strings_table;

mod raw_array;
mod raw_set;

pub mod array;
pub mod set;
pub mod map;
// ToDo: multimap

#[cfg(test)]
mod tests;
