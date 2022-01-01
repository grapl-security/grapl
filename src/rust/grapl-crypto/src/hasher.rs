use blake3::Hash;

#[derive(Clone)]
pub(crate) struct Hasher {
    hasher: blake3::Hasher,
    pepper: Option<[u8; 16]>
}

impl Hasher {
    pub(crate) fn new(pepper: impl Into<Option<[u8; 16]>>) -> Self {
        Self {
            hasher: blake3::Hasher::new(),
            pepper: pepper.into(),
        }
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Hasher { hasher: blake3::Hasher::new(), pepper: None }
    }
}

impl Hasher {
    pub(crate) fn hash<F>(&mut self, f: F) -> Hash
        where F: FnOnce(&mut blake3::Hasher)
    {
        self.hasher.reset();
        // If there's a pepper we seed it through every hash
        if let Some(pepper) = self.pepper {
            self.hasher.update(&pepper);
        }
        f(&mut self.hasher);
        self.hasher.finalize()
    }
}
