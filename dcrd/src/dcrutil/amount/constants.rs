//! Amount constants.

/// Number of atomic units in one coin cent.
pub const ATOMS_PER_CENT: f64 = 1e6;
/// Number of atomic units in one coin.
pub const ATOMS_PER_COIN: f64 = 1e8;
/// Maximum transaction amount allowed in atoms.
pub const MAX_AMOUNT: f64 = 21e6 * ATOMS_PER_COIN;
