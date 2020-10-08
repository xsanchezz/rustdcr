/// Amount related errors.
pub enum AmountError {
    InvalidCoinAmount,
}

impl std::fmt::Display for AmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AmountError::InvalidCoinAmount => write!(f, "Invalid coin amount."),
        }
    }
}

impl std::fmt::Debug for AmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AmountError::InvalidCoinAmount => write!(f, "AmountError(Invalid coin amount)"),
        }
    }
}
