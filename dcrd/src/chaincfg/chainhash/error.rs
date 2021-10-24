/// Contains all chain hash errors.
pub enum ChainHashErrors {
    /// Describes an error where the caller specified a hash string that has too many characters.
    HashStringSize,

    /// Describes an error where the hash size is not same as specified.
    HashSize,

    /// Invalid hash to string conversion.
    HashToString(std::fmt::Error),

    HexDecode(hex::FromHexError),
}

impl std::fmt::Display for ChainHashErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChainHashErrors::HashStringSize => write!(
                f,
                "Max hash string length is {} bytes",
                super::constants::MAX_HASH_STRING_SIZE
            ),
            ChainHashErrors::HashSize => write!(
                f,
                "Max hash length is {} bytes",
                super::constants::HASH_SIZE
            ),
            ChainHashErrors::HashToString(e) => {
                write!(f, "Invalid hex to string conversion, error: {}", e)
            }
            ChainHashErrors::HexDecode(e) => write!(f, "Error decoding hex, error: {}", e),
        }
    }
}

impl std::fmt::Debug for ChainHashErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChainHashErrors::HashStringSize => write!(
                f,
                "ChainHashErrors(max hash string length is {} bytes)",
                super::constants::HASH_SIZE
            ),
            ChainHashErrors::HashSize => write!(
                f,
                "ChainHashErrors(Max hash length is {} bytes)",
                super::constants::HASH_SIZE
            ),
            ChainHashErrors::HashToString(e) => write!(
                f,
                "ChainHashErrors(Invalid hex to string conversion, error: {})",
                e
            ),
            ChainHashErrors::HexDecode(e) => {
                write!(f, "ChainHashErrors(Error decoding hex, error: {})", e)
            }
        }
    }
}
