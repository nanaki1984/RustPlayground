use std::{collections::hash_map::DefaultHasher, hash::Hasher};

pub trait FastHash {
    fn fast_hash(&self) -> usize;
}

pub trait SetKey : Copy + Eq + Unpin + FastHash { }

impl FastHash for i32 {
    fn fast_hash(&self) -> usize {
        *self as usize
    }
}

impl FastHash for u32 {
    fn fast_hash(&self) -> usize {
        *self as usize
    }
}

impl FastHash for i64 {
    fn fast_hash(&self) -> usize {
        *self as usize
    }
}

impl FastHash for u64 {
    fn fast_hash(&self) -> usize {
        *self as usize
    }
}

impl FastHash for &str {
    // TODO: copy hasher from my framework, see if possible to make it compile time like in c++
    fn fast_hash(&self) -> usize {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.as_bytes());
        hasher.finish() as usize
    }
}

impl SetKey for i32 { }
impl SetKey for u32 { }
impl SetKey for i64 { }
impl SetKey for u64 { }
impl SetKey for &str { }

pub trait SetItem : Sized + Unpin {
    const IMMUTABLE_KEY: bool;

    type KeyType : SetKey;

    fn get_key(&self) -> Self::KeyType;
}

pub struct KeyValuePair<K: SetKey, V: Unpin>(K, V);

impl<K: SetKey, V: Unpin> SetItem for KeyValuePair<K, V> {
    const IMMUTABLE_KEY: bool = true;

    type KeyType = K;

    #[inline]
    fn get_key(&self) -> Self::KeyType {
        self.0
    }
}

impl<K: SetKey, V: Unpin> KeyValuePair<K, V> {
    #[inline]
    pub fn new(key: K, value: V) -> Self {
        KeyValuePair(key, value)
    }

    #[inline]
    pub fn get_value(&self) -> &V {
        &self.1
    }

    #[inline]
    pub fn get_value_mut(&mut self) -> &mut V {
        &mut self.1
    }

    #[inline]
    pub fn swap_value(&mut self, new_value: V) -> V {
        std::mem::replace(self.get_value_mut(), new_value)
    }

    #[inline]
    pub fn take_value(pair: KeyValuePair<K, V>) -> V {
        pair.1
    }
}

impl<K: SetKey, V: Unpin + Default> KeyValuePair<K, V> {
    #[inline]
    pub fn new_with_key(key: K) -> Self {
        KeyValuePair(key, Default::default())
    }
}
