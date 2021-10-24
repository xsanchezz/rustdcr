pub mod constants;
mod error;
pub use error::AmountError;


use std::cmp::Ordering;
use std::fmt::{self};

/// Various denominations when describing a coin monetary amount.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Denomination {
    /// Coin * 10^6
    AmountMegaCoin,
    /// Coin * 10^3
    AmountKiloCoin,
    /// Coin
    AmountCoin,
    /// Coin * 10^-3
    AmountMilliCoin,
    /// Coin * 10^-6
    AmountMicroCoin,
    /// Coin * 10^-8
    AmountAtom,
}

impl Denomination {
    /// The number of decimal places.
    pub fn precision(self) -> i32 {
        match self {
            Denomination::AmountMegaCoin => 6,
            Denomination::AmountKiloCoin => 3,
            Denomination::AmountCoin => 0,
            Denomination::AmountMilliCoin => -3,
            Denomination::AmountMicroCoin => -6,
            Denomination::AmountAtom => -8,
        }
    }
}

impl fmt::Display for Denomination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Denomination::AmountMegaCoin => "MDCR",
            Denomination::AmountKiloCoin => "kDCR",
            Denomination::AmountCoin => "DCR",
            Denomination::AmountMilliCoin => "mDCR",
            Denomination::AmountMicroCoin => "μDCR",
            Denomination::AmountAtom => "Atom",
        })
    }
}



/// Converts a floating point number, which may or may not be representable
/// as an integer, to the Amount integer type by rounding to the nearest integer.
/// This is performed by adding or subtracting 0.5 depending on the sign, and
/// relying on integer truncation to round the value to the nearest Amount.
fn round(f: f64) -> Amount {
    if f < 0.0 {
        return Amount((f - 0.5) as i64);
    }

    Amount((f + 0.5) as i64)
}

/// Creates an Amount from a floating point value representing
/// some value in the currency.  Errors if f is NaN or +-Infinity,
/// but does not check that the amount is within the total amount of coins
/// producible as f may not refer to an amount at a single moment in time.
///
/// It is for specifically for converting DCR to Atoms (atomic units).
/// For creating a new Amount with an i64 value which denotes a quantity of
/// Atoms, do a simple type conversion from type i64 to Amount.
pub fn new(amount: f64) -> Result<Amount, AmountError> {
    if amount.is_nan() || amount.is_infinite() {
        return Err(AmountError::InvalidCoinAmount);
    }

    Ok(round(amount * constants::ATOMS_PER_COIN))
}

/// Amount represents the base coin monetary unit (colloquially referred
/// to as an `Atom').  A single Amount is equal to 1e-8 of a coin.
pub struct Amount(i64);

impl Amount {
    /// Converts a monetary amount counted in coin base units to a
    /// floating point value representing an amount of coins.
    pub fn to_unit(&self, denom: Denomination) -> f64 {
         self.0 as f64 / 10.0f64.powi(denom.precision() + 8)
    }

    // Equivalent of calling ToUnit with AmountCoin.
    pub fn to_coin(&self) -> f64 {
        self.to_unit(Denomination::AmountCoin)
    }

    /// Formats a monetary amount counted in coin base units as a
    /// string for a given unit.  The conversion will succeed for any unit,
    /// however, known units will be formatted with an appended label describing
    /// the units with SI notation, or "atom" for the base unit.
    pub fn format(&self, denomination: Denomination) -> String {
        format!("{} {}", self.to_unit(denomination), denomination)
    }

    /// Multiplies an Amount by a floating point value.  While this is not
    /// an operation that must typically be done by a full node or wallet, it is
    /// useful for services that build on top of Decred (for example, calculating
    /// a fee by multiplying by a percentage).
    pub fn mul_f64(&self, f: f64) -> Amount {
        round(self.0 as f64 * f)
    }
}

impl ToString for Amount {
    fn to_string(&self) -> String {
        self.format(Denomination::AmountCoin)
    }
}

impl std::cmp::PartialOrd for Amount {
    fn partial_cmp(&self, other: &Amount) -> Option<Ordering> {
        let amount = self.to_coin();
        let other_amount = other.to_coin();

        if amount > other_amount {
            Some(Ordering::Greater)
        } else if amount < other_amount {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl std::cmp::PartialEq for Amount {
    fn eq(&self, other: &Self) -> bool {
        self.to_coin() == other.to_coin()
    }
}

impl std::cmp::Eq for Amount {}

impl std::cmp::Ord for Amount {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

// Implements sorting interface to allow a slice of Amounts to
// be sorted.
pub struct AmountSorter(Vec<Amount>);

impl AmountSorter {
    /// Returns the number of Amounts in the slice.  It is part of the
    /// sorter implementation.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool{
        self.0.is_empty()
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.0.swap(i, j)
    }

    pub fn less(&self, i: usize, j: usize) -> bool {
        // self.0[i]<self.0.[j]
        self.0[i] < self.0[j]
    }
}
