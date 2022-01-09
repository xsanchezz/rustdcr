//! Houses all JSON command types

use std::fmt;

use serde::{Deserialize, Serialize};

/// EstimateSmartFeeMode defines estimation mode to be used with
/// the estimatesmartfee command.
#[derive(Debug, Deserialize)]
pub enum EstimateSmartFeeMode {
    Economical,
    Conservative,
}

impl Serialize for EstimateSmartFeeMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            EstimateSmartFeeMode::Conservative => {
                serializer.serialize_str(&format!("{}", EstimateSmartFeeMode::Conservative))
            }
            EstimateSmartFeeMode::Economical => {
                serializer.serialize_str(&format!("{}", EstimateSmartFeeMode::Economical))
            }
        }
    }
}

impl fmt::Display for EstimateSmartFeeMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EstimateSmartFeeMode::Conservative => write!(f, "conservative"),
            EstimateSmartFeeMode::Economical => write!(f, "economical"),
        }
    }
}
