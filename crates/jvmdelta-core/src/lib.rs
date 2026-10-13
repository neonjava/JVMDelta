//! Core types for JVM performance regression analysis.
//!
//! Parsing and analysis APIs will be added as their formats and behavior are
//! defined. No analysis is performed by this crate yet.

pub mod comparison;
pub mod gc_log;
pub mod metrics;
pub mod regression;
pub mod report;
