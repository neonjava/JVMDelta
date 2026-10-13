//! Report output domain.

/// Intended report formats. Rendering is not implemented yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Terminal,
    Json,
    Html,
}
