/// Contains all chain hash errors.
pub enum ChainHashErrors {
    /// Describes an error that indicates the caller specified a hash string that has too many characters.
    HashStringSize,
}

impl std::fmt::Display for ChainHashErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChainHashErrors::HashStringSize => write!(f, "Invalid string size."),
        }
    }
}

impl std::fmt::Debug for ChainHashErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChainHashErrors::HashStringSize => write!(f, "ChainHashErrors(Invalid string size)"),
        }
    }
}
