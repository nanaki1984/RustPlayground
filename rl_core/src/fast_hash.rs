pub trait FashHash {
    fn fast_hash(&self) -> usize;
}

pub trait SetKey : Copy + Eq + Unpin + FashHash { }
