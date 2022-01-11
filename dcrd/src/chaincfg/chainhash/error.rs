/// Contains all chain hash errors.
pub enum ChainHashError {
    /// Describes an error where the caller specified a hash string that has too many characters.
    HashStringSize,

    /// Describes an error where the hash size is not same as specified.
    HashSize,

    /// Invalid hash to string conversion.
    HashToString(std::fmt::Error),

    /// Invalid hash decoding.
    HexDecode(hex::FromHexError),
}

impl std::fmt::Display for ChainHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChainHashError::HashStringSize => write!(
                f,
                "Max hash string length is {} bytes",
                super::constants::MAX_HASH_STRING_SIZE
            ),
            ChainHashError::HashSize => write!(
                f,
                "Max hash length is {} bytes",
                super::constants::HASH_SIZE
            ),
            ChainHashError::HashToString(e) => {
                write!(f, "Invalid hex to string conversion, error: {}", e)
            }
            ChainHashError::HexDecode(e) => write!(f, "Error decoding hex, error: {}", e),
        }
    }
}

impl std::fmt::Debug for ChainHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChainHashError::HashStringSize => write!(
                f,
                "ChainHashError(max hash string length is {} bytes)",
                super::constants::HASH_SIZE
            ),
            ChainHashError::HashSize => write!(
                f,
                "ChainHashError(Max hash length is {} bytes)",
                super::constants::HASH_SIZE
            ),
            ChainHashError::HashToString(e) => write!(
                f,
                "ChainHashError(Invalid hex to string conversion, error: {})",
                e
            ),
            ChainHashError::HexDecode(e) => {
                write!(f, "ChainHashError(Error decoding hex, error: {})", e)
            }
        }
    }
}
