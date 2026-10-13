//! Exact, incremental measurements over validated GC pause events.

use crate::gc_log::{EventKind, GcEvent};
use std::fmt;

/// Units used by core measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Nanoseconds,
    Bytes,
    Count,
}

/// Counts and duration statistics for pause summary records.
///
/// `count` counts records, not distinct GC IDs or collection cycles. `total_ns`
/// uses `u128` so it can represent sums larger than a single `u64` duration.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PauseStats {
    count: u64,
    total_ns: u128,
    min_ns: Option<u64>,
    max_ns: Option<u64>,
}

impl PauseStats {
    /// Number of pause summary records observed.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Exact sum of pause durations in nanoseconds; zero when empty.
    pub fn total_ns(&self) -> u128 {
        self.total_ns
    }

    /// Shortest pause in nanoseconds, or `None` when empty.
    pub fn min_ns(&self) -> Option<u64> {
        self.min_ns
    }

    /// Longest pause in nanoseconds, or `None` when empty.
    pub fn max_ns(&self) -> Option<u64> {
        self.max_ns
    }

    /// Arithmetic mean, rounded down to a whole nanosecond; `None` when empty.
    pub fn mean_ns_floor(&self) -> Option<u64> {
        if self.count == 0 {
            return None;
        }
        // A mean of u64 inputs cannot exceed u64::MAX.
        u64::try_from(self.total_ns / u128::from(self.count)).ok()
    }

    fn with_duration(self, duration_ns: u64) -> Result<Self, SummaryError> {
        let count = self
            .count
            .checked_add(1)
            .ok_or(SummaryError::CountOverflow)?;
        let total_ns = self
            .total_ns
            .checked_add(u128::from(duration_ns))
            .ok_or(SummaryError::DurationOverflow)?;
        Ok(Self {
            count,
            total_ns,
            min_ns: Some(
                self.min_ns
                    .map_or(duration_ns, |value| value.min(duration_ns)),
            ),
            max_ns: Some(
                self.max_ns
                    .map_or(duration_ns, |value| value.max(duration_ns)),
            ),
        })
    }
}

/// Exact pause summaries for all events and each supported category.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GcSummary {
    all: PauseStats,
    young: PauseStats,
    full: PauseStats,
    remark: PauseStats,
    cleanup: PauseStats,
}

impl GcSummary {
    /// Add one validated GC pause event. If an overflow occurs, the summary is unchanged.
    pub fn record(&mut self, event: &GcEvent) -> Result<(), SummaryError> {
        let next_all = self.all.with_duration(event.duration_ns)?;
        let category = match event.kind {
            EventKind::Young => &mut self.young,
            EventKind::Full => &mut self.full,
            EventKind::Remark => &mut self.remark,
            EventKind::Cleanup => &mut self.cleanup,
        };
        let next_category = category.with_duration(event.duration_ns)?;
        self.all = next_all;
        *category = next_category;
        Ok(())
    }

    /// Statistics over every supported pause summary record.
    pub fn all(&self) -> &PauseStats {
        &self.all
    }

    /// Statistics for one supported pause category.
    pub fn category(&self, kind: EventKind) -> &PauseStats {
        match kind {
            EventKind::Young => &self.young,
            EventKind::Full => &self.full,
            EventKind::Remark => &self.remark,
            EventKind::Cleanup => &self.cleanup,
        }
    }
}

/// An arithmetic limit was reached while accumulating events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryError {
    CountOverflow,
    DurationOverflow,
}

impl fmt::Display for SummaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOverflow => write!(f, "GC pause count exceeds u64"),
            Self::DurationOverflow => write!(f, "total GC pause duration exceeds u128 nanoseconds"),
        }
    }
}

impl std::error::Error for SummaryError {}

#[cfg(test)]
mod tests {
    use super::{PauseStats, SummaryError};

    #[test]
    fn count_overflow_is_explicit() {
        let stats = PauseStats {
            count: u64::MAX,
            ..PauseStats::default()
        };
        assert_eq!(stats.with_duration(1), Err(SummaryError::CountOverflow));
    }

    #[test]
    fn total_overflow_is_explicit() {
        let stats = PauseStats {
            count: 1,
            total_ns: u128::MAX,
            ..PauseStats::default()
        };
        assert_eq!(stats.with_duration(1), Err(SummaryError::DurationOverflow));
    }
}
