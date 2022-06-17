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

impl SetKey for i32 { }
impl SetKey for u32 { }
impl SetKey for i64 { }
impl SetKey for u64 { }

pub trait SetItem : Sized + Unpin {
    type KeyType : SetKey;

    fn get_key(&self) -> Self::KeyType;
}

pub struct KeyValuePair<K: SetKey, V: Unpin>(K, V);

impl<K: SetKey, V: Unpin> SetItem for KeyValuePair<K, V> {
    type KeyType = K;

    fn get_key(&self) -> Self::KeyType {
        self.0
    }
}

impl<K: SetKey, V: Unpin> KeyValuePair<K, V> {
    fn get_value(&self) -> &V {
        &self.1
    }

    fn get_value_mut(&mut self) -> &mut V {
        &mut self.1
    }
}
