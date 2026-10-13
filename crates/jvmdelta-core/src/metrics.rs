//! Performance measurement definitions.

/// Units supported by future measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Nanoseconds,
    Bytes,
    Count,
}
