//! Streaming parser for a narrow subset of G1 Unified GC pause summaries.
//! See `docs/GC_LOG_SUPPORT.md` for the accepted syntax and error policy.

use std::fmt;
use std::io::{self, BufRead, Read};

/// Maximum accepted line length in bytes, including its line ending.
pub const MAX_LINE_BYTES: u64 = 64 * 1024;

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

/// Supported stop-the-world G1 pause categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Young,
    Full,
    Remark,
    Cleanup,
}

/// Heap usage in bytes. K, M, and G use powers of 1024.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeapUsage {
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub capacity_bytes: u64,
}

/// A validated pause summary. Durations and uptime are exact nanoseconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcEvent {
    pub source_line: usize,
    pub gc_id: u64,
    pub kind: EventKind,
    /// Text between the pause category and optional heap summary, without trimming inner text.
    pub description: String,
    pub heap: Option<HeapUsage>,
    pub duration_ns: u64,
    pub uptime_ns: Option<u64>,
    /// Original log line without the line ending.
    pub source: String,
}

/// Why a line did not produce a supported pause event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    Unrelated,
    UnsupportedEvent,
}

/// One classified input line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    Event(GcEvent),
    Skipped(SkipReason),
}

/// A structured parser failure.
#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub kind: ParseErrorKind,
}

/// Categories of parser failure. An I/O error terminates iteration.
#[derive(Debug)]
pub enum ParseErrorKind {
    Io(io::Error),
    LineTooLong,
    InvalidUtf8,
    Malformed(&'static str),
    InvalidNumber(&'static str),
    Overflow(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: ", self.line)?;
        match &self.kind {
            ParseErrorKind::Io(error) => write!(f, "I/O error: {error}"),
            ParseErrorKind::LineTooLong => write!(f, "line exceeds {MAX_LINE_BYTES} bytes"),
            ParseErrorKind::InvalidUtf8 => write!(f, "invalid UTF-8"),
            ParseErrorKind::Malformed(message) => write!(f, "malformed GC pause: {message}"),
            ParseErrorKind::InvalidNumber(field) => write!(f, "invalid {field}"),
            ParseErrorKind::Overflow(field) => {
                write!(f, "{field} overflows u64 nanoseconds or bytes")
            }
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ParseErrorKind::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Parses a `BufRead` stream one bounded line at a time.
///
/// Each line yields an event, a skip reason, or an error. Malformed lines do not
/// end iteration; I/O, invalid UTF-8, and overlong lines do. Callers choose strict
/// or permissive behavior by stopping or continuing after malformed lines.
/// No file path, CLI type, or entire-log buffer is required.
pub struct GcLogParser<R> {
    reader: R,
    line: usize,
    finished: bool,
    buffer: Vec<u8>,
}

impl<R: BufRead> GcLogParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            line: 0,
            finished: false,
            buffer: Vec::new(),
        }
    }
}

impl<R: BufRead> Iterator for GcLogParser<R> {
    type Item = Result<Record, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        self.buffer.clear();
        let read = self
            .reader
            .by_ref()
            .take(MAX_LINE_BYTES + 1)
            .read_until(b'\n', &mut self.buffer);
        let line = match self.line.checked_add(1) {
            Some(line) => line,
            None => {
                self.finished = true;
                return Some(Err(ParseError {
                    line: self.line,
                    kind: ParseErrorKind::Overflow("source line number"),
                }));
            }
        };
        self.line = line;
        match read {
            Ok(0) => {
                self.finished = true;
                None
            }
            Err(error) => {
                self.finished = true;
                Some(Err(ParseError {
                    line,
                    kind: ParseErrorKind::Io(error),
                }))
            }
            Ok(_) if self.buffer.len() as u64 > MAX_LINE_BYTES => {
                self.finished = true;
                Some(Err(ParseError {
                    line,
                    kind: ParseErrorKind::LineTooLong,
                }))
            }
            Ok(_) => {
                let bytes = self.buffer.strip_suffix(b"\n").unwrap_or(&self.buffer);
                let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);
                match std::str::from_utf8(bytes) {
                    Ok(text) => Some(parse_line(text, line)),
                    Err(_) => {
                        self.finished = true;
                        Some(Err(ParseError {
                            line,
                            kind: ParseErrorKind::InvalidUtf8,
                        }))
                    }
                }
            }
        }
    }
}

fn error(line: usize, kind: ParseErrorKind) -> ParseError {
    ParseError { line, kind }
}

fn parse_line(source: &str, line: usize) -> Result<Record, ParseError> {
    let mut rest = source;
    let mut uptime_ns = None;
    let mut level = None;
    let mut tags = None;
    let mut decorations = 0;
    while let Some(after_open) = rest.strip_prefix('[') {
        let Some((decoration, after_close)) = after_open.split_once(']') else {
            return if source.contains("GC(") || source.contains("Pause ") {
                Err(error(
                    line,
                    ParseErrorKind::Malformed("unterminated decoration"),
                ))
            } else {
                Ok(Record::Skipped(SkipReason::Unrelated))
            };
        };
        decorations += 1;
        if decorations > 8 {
            return Err(error(
                line,
                ParseErrorKind::Malformed("too many decorations"),
            ));
        }
        let decoration = decoration.trim();
        if decoration == "info"
            || decoration == "debug"
            || decoration == "trace"
            || decoration == "warning"
            || decoration == "error"
        {
            level = Some(decoration);
        } else if decoration == "gc" || decoration.starts_with("gc,") {
            tags = Some(decoration);
        } else if decoration.ends_with('s')
            && decoration[..decoration.len() - 1]
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b'.')
        {
            uptime_ns = Some(decimal_to_scaled(
                &decoration[..decoration.len() - 1],
                9,
                "uptime",
                line,
            )?);
        } else if decoration.contains('T') && decoration.contains(':') {
            // Wall-clock `time` decoration is accepted but not interpreted.
        } else if source.contains("GC(") || source.contains("Pause ") {
            return Err(error(
                line,
                ParseErrorKind::Malformed("unsupported decoration"),
            ));
        }
        rest = after_close;
    }
    if decorations > 0 && (!rest.starts_with(' ') || level.is_none() || tags.is_none()) {
        return if source.contains("GC(") || source.contains("Pause ") {
            Err(error(
                line,
                ParseErrorKind::Malformed("expected level, GC tags, and space after decorations"),
            ))
        } else {
            Ok(Record::Skipped(SkipReason::Unrelated))
        };
    }
    if decorations > 0 {
        rest = rest.trim_start();
    }
    if decorations > 0 && (tags != Some("gc") || level != Some("info")) {
        return Ok(Record::Skipped(SkipReason::Unrelated));
    }
    if !rest.starts_with("GC") {
        return if rest.starts_with("Pause ") {
            Err(error(line, ParseErrorKind::Malformed("missing GC ID")))
        } else {
            Ok(Record::Skipped(SkipReason::Unrelated))
        };
    }
    let Some(after_id_open) = rest.strip_prefix("GC(") else {
        return if rest.contains("Pause ") {
            Err(error(line, ParseErrorKind::Malformed("missing GC ID")))
        } else {
            Ok(Record::Skipped(SkipReason::Unrelated))
        };
    };
    let Some((id_text, message)) = after_id_open.split_once(')') else {
        return Err(error(line, ParseErrorKind::Malformed("unterminated GC ID")));
    };
    let gc_id = id_text
        .parse::<u64>()
        .map_err(|_| error(line, ParseErrorKind::InvalidNumber("GC ID")))?;
    let Some(message) = message.strip_prefix(' ') else {
        return Err(error(
            line,
            ParseErrorKind::Malformed("missing space after GC ID"),
        ));
    };
    let kind = if message.starts_with("Pause Young") {
        EventKind::Young
    } else if message.starts_with("Pause Full") {
        EventKind::Full
    } else if message.starts_with("Pause Remark") {
        EventKind::Remark
    } else if message.starts_with("Pause Cleanup") {
        EventKind::Cleanup
    } else {
        return Ok(Record::Skipped(if message.starts_with("Pause ") {
            SkipReason::UnsupportedEvent
        } else {
            SkipReason::Unrelated
        }));
    };
    let prefix = match kind {
        EventKind::Young => "Pause Young",
        EventKind::Full => "Pause Full",
        EventKind::Remark => "Pause Remark",
        EventKind::Cleanup => "Pause Cleanup",
    };
    let tail = message.strip_prefix(prefix).unwrap_or("");
    if !tail.is_empty() && !tail.starts_with(' ') {
        return Ok(Record::Skipped(SkipReason::UnsupportedEvent));
    }
    let tail = tail.trim();
    let (body, duration_text) = tail.rsplit_once(' ').unwrap_or(("", tail));
    let Some(duration_text) = duration_text.strip_suffix("ms") else {
        return Err(error(
            line,
            ParseErrorKind::Malformed("expected duration in ms"),
        ));
    };
    let duration_ns = decimal_to_scaled(duration_text, 6, "pause duration", line)?;
    let (description, heap) = match body.rsplit_once(' ') {
        Some((description, token)) if token.contains("->") => {
            (description.trim(), Some(parse_heap(token, line)?))
        }
        _ if body.contains("->") => ("", Some(parse_heap(body, line)?)),
        _ => (body.trim(), None),
    };
    if matches!(kind, EventKind::Young | EventKind::Full) && heap.is_none() {
        return Err(error(
            line,
            ParseErrorKind::Malformed("young/full pause requires heap summary"),
        ));
    }
    Ok(Record::Event(GcEvent {
        source_line: line,
        gc_id,
        kind,
        description: description.to_owned(),
        heap,
        duration_ns,
        uptime_ns,
        source: source.to_owned(),
    }))
}

fn parse_heap(text: &str, line: usize) -> Result<HeapUsage, ParseError> {
    let Some((before, rest)) = text.split_once("->") else {
        return Err(error(
            line,
            ParseErrorKind::Malformed("expected heap before->after(capacity)"),
        ));
    };
    let Some((after, capacity)) = rest.split_once('(') else {
        return Err(error(
            line,
            ParseErrorKind::Malformed("missing heap capacity"),
        ));
    };
    let Some(capacity) = capacity.strip_suffix(')') else {
        return Err(error(
            line,
            ParseErrorKind::Malformed("unterminated heap capacity"),
        ));
    };
    Ok(HeapUsage {
        before_bytes: parse_size(before, line)?,
        after_bytes: parse_size(after, line)?,
        capacity_bytes: parse_size(capacity, line)?,
    })
}

fn parse_size(text: &str, line: usize) -> Result<u64, ParseError> {
    let Some(unit) = text.as_bytes().last().copied() else {
        return Err(error(line, ParseErrorKind::InvalidNumber("heap size")));
    };
    let multiplier: u64 = match unit {
        b'B' => 1,
        b'K' => 1024,
        b'M' => 1024_u64.pow(2),
        b'G' => 1024_u64.pow(3),
        _ => {
            return Err(error(
                line,
                ParseErrorKind::InvalidNumber("heap unit (expected B, K, M, G)"),
            ));
        }
    };
    let number = text[..text.len() - 1]
        .parse::<u64>()
        .map_err(|_| error(line, ParseErrorKind::InvalidNumber("heap size")))?;
    number
        .checked_mul(multiplier)
        .ok_or_else(|| error(line, ParseErrorKind::Overflow("heap size")))
}

fn decimal_to_scaled(
    text: &str,
    fractional_digits: usize,
    field: &'static str,
    line: usize,
) -> Result<u64, ParseError> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > fractional_digits
        || text.ends_with('.')
    {
        return Err(error(line, ParseErrorKind::InvalidNumber(field)));
    }
    let scale = 10_u64.pow(fractional_digits as u32);
    let whole = whole
        .parse::<u64>()
        .map_err(|_| error(line, ParseErrorKind::Overflow(field)))?;
    let fraction_len = fraction.len();
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<u64>()
            .map_err(|_| error(line, ParseErrorKind::Overflow(field)))?
    };
    whole
        .checked_mul(scale)
        .and_then(|n| {
            n.checked_add(fraction * 10_u64.pow((fractional_digits - fraction_len) as u32))
        })
        .ok_or_else(|| error(line, ParseErrorKind::Overflow(field)))
}
