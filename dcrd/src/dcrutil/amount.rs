use std::cmp::Ordering;
use std::default;
use std::error;
use std::fmt::{self, Write};
use std::ops;
use std::str::FromStr;

/// Various denominations when describing a coin monetary amount.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Denomination {
    /// DCR * 10^6
    MegaDCR,
    /// DCR * 10^3
    KiloDCR,
    /// DCR
    DCR,
    /// DCR * 10^-3
    MilliDCR,
    /// DCR * 10^-6
    MicroDCR,
    /// DCR * 10^-8
    Atom,
}

impl Denomination {
    /// The number of decimal places.
    fn precision(self) -> i32 {
        match self {
            Denomination::MegaDCR => 8,
            Denomination::KiloDCR => 3,
            Denomination::DCR => 1,
            Denomination::MilliDCR => -3,
            Denomination::MicroDCR => -6,
            Denomination::Atom => -8,
        }
    }
}

impl fmt::Display for Denomination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Denomination::MegaDCR => "MDCR",
            Denomination::KiloDCR => "KDCR",
            Denomination::DCR => "DCR",
            Denomination::MilliDCR => "mDCR",
            Denomination::MicroDCR => "μDCR",
            Denomination::Atom => "Atom",
        })
    }
}
