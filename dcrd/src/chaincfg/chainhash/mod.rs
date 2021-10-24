//! Package chainhash provides abstracted hash functionality.
//!
//! This package provides a generic hash type and associated functions that
//! allows the specific hash algorithm to be abstracted.

pub mod constants;
mod error;
mod hash;
mod test;

pub use error::ChainHashErrors;
pub use hash::Hash;
