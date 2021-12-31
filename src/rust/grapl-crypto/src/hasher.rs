use blake3::Hash;

#[derive(Clone)]
pub(crate) struct Hasher {
    hasher: blake3::Hasher,
}

impl Default for Hasher {
    fn default() -> Self {
        Hasher { hasher: blake3::Hasher::new() }
    }
}

impl Hasher {
    pub(crate) fn hash<F>(&mut self, f: F) -> Hash
        where F: FnOnce(&mut blake3::Hasher)
    {
        self.hasher.reset();
        f(&mut self.hasher);
        self.hasher.finalize()
    }
}
