//! Package chainhash provides abstracted hash functionality.
//!
//! This package provides a generic hash type and associated functions that
//! allows the specific hash algorithm to be abstracted.

pub mod constants;
mod errors;
mod hash;
mod tests;

pub use errors::ChainHashErrors;
pub use hash::Hash;
