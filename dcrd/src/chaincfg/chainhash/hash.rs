use {
    super::{constants::HASH_SIZE, ChainHashError},
    std::convert::TryInto,
};

pub struct Hash([u8; HASH_SIZE]);

impl Hash {
    // Returns the Hash as the hexadecimal string of the byte-reversed
    // hash.
    pub fn string(&self) -> Result<String, ChainHashError> {
        let mut hash = self.0;

        let mut i = 0;
        while i < HASH_SIZE / 2 {
            hash.swap(i, HASH_SIZE - 1 - i);
            i += 1;
        }

        let s = hex::encode(hash);

        Ok(s)
    }

    pub fn clone_hash(&self) -> Hash {
        Self(*self.bytes())
    }

    pub fn bytes(&self) -> &[u8; HASH_SIZE] {
        &self.0
    }

    /// Sets the bytes which represent the hash.  An error is returned if
    /// the number of bytes passed in is not HASH_SIZE.
    pub fn set_bytes(&mut self, hash: Vec<u8>) -> Result<(), ChainHashError> {
        if hash.len() != HASH_SIZE {
            return Err(ChainHashError::HashSize);
        }

        let boxed_slice = hash.into_boxed_slice();

        let boxed_array: Box<[u8; HASH_SIZE]> = match boxed_slice.try_into() {
            Ok(ba) => ba,
            Err(_) => return Err(ChainHashError::HashSize),
        };

        self.0 = *boxed_array;
        Ok(())
    }

    /// Get length of hash.
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if `Hash` is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if target is the same as hash.
    pub fn is_equal(&self, hash: &Self) -> bool {
        self.0 == hash.0
    }

    /// Returns a new Hash from a byte slice.  An error is returned if
    /// the number of bytes passed in is not HASH_SIZE.
    pub fn new(hash: Vec<u8>) -> Result<Self, ChainHashError> {
        let mut h = Self { 0: [0; 32] };

        match h.set_bytes(hash) {
            Ok(_) => {}

            Err(e) => return Err(e),
        };

        Ok(h)
    }

    /// Creates a Hash from a hash string.  The string should be the hexadecimal
    /// string of a byte-reversed hash, but any missing characters result in zero padding at
    /// at the end of the Hash.
    pub fn new_from_str(value: &str) -> Result<Hash, ChainHashError> {
        let mut h = Self { 0: [0; HASH_SIZE] };
        match h.decode(value) {
            Ok(_) => {}

            Err(e) => return Err(e),
        };

        Ok(h)
    }

    /// Decodes the byte-reversed hexadecimal string encoding of a Hash to a
    /// destination.
    pub fn decode(&mut self, src: &str) -> Result<(), ChainHashError> {
        // Return error if hash string is too long.
        if src.len() > super::constants::MAX_HASH_STRING_SIZE {
            return Err(ChainHashError::HashStringSize);
        }

        // Hex decoder expects the hash to be a multiple of two.  When not, pad
        // with a leading zero.
        let src_bytes = if src.len() % 2 == 0 {
            src.as_bytes().to_vec()
        } else {
            let mut v = Vec::with_capacity(1 + src.len()); //src.as_bytes().to_vec();
            v.push(48); // Add '0'.

            for a in src.as_bytes().iter() {
                v.push(*a);
            }
            v
        };

        let mut reversed_hash = [0; HASH_SIZE];
        let src_len = src_bytes.len() / 2;

        match hex::decode_to_slice(src_bytes, &mut reversed_hash[HASH_SIZE - src_len..]) {
            Ok(_) => {}
            Err(e) => return Err(ChainHashError::HexDecode(e)),
        };

        let mut i = HASH_SIZE / 2;

        while i < HASH_SIZE {
            self.0[i] = reversed_hash[HASH_SIZE - 1 - i];
            self.0[HASH_SIZE - 1 - i] = reversed_hash[i];
            i += 1;
        }

        Ok(())
    }
}

impl std::fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hash({:?})", self.bytes())
    }
}
