//! Helpers for Debug impls.

use std::fmt::{self, Debug};

pub(crate) struct DebugHexByte(pub u8);

impl Debug for DebugHexByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

/// Write a row of 16 hex-formatted bytes.
pub(crate) fn write_row(f: &mut fmt::Formatter<'_>, line: [u8; 16]) -> fmt::Result {
    write!(f, "{:02x}", line[0])?;
    for byte in &line[1..8] {
        write!(f, " {byte:02x}")?;
    }
    write!(f, " ")?;
    for byte in &line[8..] {
        write!(f, " {byte:02x}")?;
    }
    Ok(())
}
