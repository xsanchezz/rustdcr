use super::{constants::HASH_SIZE, ChainHashErrors};

pub struct Hash([u8; HASH_SIZE]);

impl Hash {
    pub fn string(&self) -> String {
        todo!()
    }

    pub fn clone_hash(&self) -> Hash {
        todo!()
    }

    pub fn bytes(&self) -> &[u8; HASH_SIZE] {
        &self.0
    }

    pub fn set_bytes(&self, hash: Vec<u8>) -> Result<(), ChainHashErrors> {
        todo!()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_equal(&self, hash: &Self) -> bool {
        todo!()
    }

    pub fn new(hash: Vec<u8>) -> Result<Self, ChainHashErrors> {
        todo!()
    }

    pub fn new_from_str(value: &str) -> Result<Hash, ChainHashErrors> {
        todo!()
    }

    pub fn decode(&self) -> Result<(), ChainHashErrors> {
        todo!()
    }
}

impl std::fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hash({:?})", self.bytes())
    }
}
