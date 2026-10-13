//! Baseline and candidate terminology for future comparisons.

/// Identifies the role of a measurement set in a comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Baseline,
    Candidate,
}
