//! Regression detection domain.

/// Direction in which a measured value would be considered worse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorseWhen {
    Higher,
    Lower,
}
