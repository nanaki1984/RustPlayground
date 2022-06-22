pub const fn fnv_hash_const<const N: usize>(bytes: &[u8; N], lowercase: bool) -> u32 {
    let mut hash = 2166136261u32;
    let mut i = 0;
    while i < N {
        let byte = if lowercase { bytes[i].to_ascii_lowercase() } else { bytes[i] } as u32;
        hash = u32::wrapping_mul(hash ^ byte, 16777619u32);
        i += 1;
    }
    hash
}

pub fn fnv_hash<const LOWERCASE: bool>(bytes: &[u8]) -> u32 {
    let bytes_len = bytes.len();
    let mut hash = 2166136261u32;
    for i in 0..bytes_len {
        let byte = if LOWERCASE { bytes[i].to_ascii_lowercase() } else { bytes[i] } as u32;
        hash = u32::wrapping_mul(hash ^ byte, 16777619u32);
    }
    hash
}

pub trait FastHash {
    fn fast_hash(&self) -> usize;
}

pub trait SetKey : Eq + Unpin + FastHash { }

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

impl FastHash for str {
    fn fast_hash(&self) -> usize {
        fnv_hash::<false>(self.as_bytes()) as usize
    }
}

impl FastHash for String {
    fn fast_hash(&self) -> usize {
        fnv_hash::<false>(self.as_bytes()) as usize
    }
}

impl SetKey for i32 { }
impl SetKey for u32 { }
impl SetKey for i64 { }
impl SetKey for u64 { }
impl SetKey for str { }
impl SetKey for String { }

pub trait SetItem : Sized + Unpin {
    type KeyType : SetKey;

    fn get_key(&self) -> &Self::KeyType;
}

pub struct KeyValuePair<K: SetKey, V: Unpin>(K, V);

impl<K: SetKey, V: Unpin> SetItem for KeyValuePair<K, V> {
    type KeyType = K;

    #[inline]
    fn get_key(&self) -> &Self::KeyType {
        &self.0
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
