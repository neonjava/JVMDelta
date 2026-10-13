//! JVM Unified GC log parsing domain.

/// A single raw log line, retained with its one-based source line number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLine<'a> {
    pub number: usize,
    pub text: &'a str,
}

impl<'a> SourceLine<'a> {
    /// Creates a source line. Line numbers are one-based.
    pub fn new(number: usize, text: &'a str) -> Result<Self, &'static str> {
        if number == 0 {
            return Err("line number must be one-based");
        }
        Ok(Self { number, text })
    }
}

#[cfg(test)]
mod tests {
    use super::SourceLine;

    #[test]
    fn source_lines_require_one_based_numbers() {
        assert!(SourceLine::new(0, "line").is_err());
        assert_eq!(SourceLine::new(1, "line").map(|line| line.number), Ok(1));
    }
}
