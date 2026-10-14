use jvmdelta_core::gc_log::{EventKind, GcLogParser, ParseErrorKind, Record, SkipReason};
use std::io::Cursor;

fn events(input: &str) -> Vec<jvmdelta_core::gc_log::GcEvent> {
    GcLogParser::new(Cursor::new(input))
        .filter_map(|record| match record.expect("fixture must parse") {
            Record::Event(event) => Some(event),
            Record::Skipped(_) => None,
        })
        .collect()
}

#[test]
fn jdk17_synthetic_categories_and_exact_values() {
    let input = include_str!("../../../tests/fixtures/jdk17-g1-synthetic.log");
    let parsed = events(input);
    assert_eq!(parsed.len(), 4);
    let young = &parsed[0];
    assert_eq!(young.source_line, 1);
    assert_eq!(young.gc_id, 0);
    assert_eq!(young.kind, EventKind::Young);
    assert_eq!(young.description, "(Normal) (G1 Evacuation Pause)");
    assert_eq!(young.uptime_ns, Some(123_000_000));
    assert_eq!(young.duration_ns, 3_452_000);
    let heap = young.heap.expect("young pause has heap");
    assert_eq!(heap.before_bytes, 20 * 1024 * 1024);
    assert_eq!(heap.after_bytes, 8 * 1024 * 1024);
    assert_eq!(heap.capacity_bytes, 128 * 1024 * 1024);
    assert_eq!(parsed[1].kind, EventKind::Full);
    assert_eq!(parsed[1].source_line, 3);
    assert_eq!(parsed[2].kind, EventKind::Remark);
    assert_eq!(parsed[2].heap, None);
    assert_eq!(parsed[3].kind, EventKind::Cleanup);
}

#[test]
fn jdk21_synthetic_and_real_jdk25_details() {
    let input = include_str!("../../../tests/fixtures/jdk21-g1-synthetic.log");
    let parsed = events(input);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].gc_id, 12);
    assert_eq!(parsed[0].duration_ns, 125_000);
    assert_eq!(parsed[0].heap.expect("heap").before_bytes, 1024 * 1024);
    assert_eq!(
        parsed[1].heap.expect("heap").capacity_bytes,
        2 * 1024 * 1024 * 1024
    );
    let real = include_str!("../../../tests/fixtures/jdk25-g1-real.log");
    let records: Vec<_> = GcLogParser::new(Cursor::new(real))
        .collect::<Result<_, _>>()
        .expect("real log parses");
    assert_eq!(
        records
            .iter()
            .filter(|r| matches!(r, Record::Event(_)))
            .count(),
        2
    );
    assert_eq!(
        records
            .iter()
            .filter(|r| matches!(r, Record::Skipped(SkipReason::Unrelated)))
            .count(),
        4
    );
}

#[test]
fn empty_and_bare_lines() {
    assert_eq!(GcLogParser::new(Cursor::new("")).count(), 0);
    let parsed = events("GC(4) Pause Young (Normal) (G1 Evacuation Pause) 1B->0B(2B) 0.000001ms\n");
    assert_eq!(parsed[0].duration_ns, 1);
    assert_eq!(parsed[0].uptime_ns, None);
}

#[test]
fn malformed_supported_lines_are_errors_with_provenance() {
    let cases = [
        "[0.1s][info][gc] GC() Pause Young (Normal) 1M->0M(2M) 1ms",
        "[0.1s][info][gc] GC(abc) Pause Young (Normal) 1M->0M(2M) 1ms",
        "[0.1s][info][gc] GC(1 Pause Young (Normal) 1M->0M(2M) 1ms",
        "[0.1s][info][gc] GC(1) Pause Young (Normal) 1T->0M(2M) 1ms",
        "[0.1s][info][gc] GC(1) Pause Young (Normal) 18446744073709551615G->0M(2M) 1ms",
        "[0.1s][info][gc] GC(1) Pause Young (Normal) 1M->0M(2M) -1ms",
        "[0.1s][info][gc] GC(1) Pause Young (Normal) 1M->0M(2M) NaNms",
        "[0.1s][info][gc] GC(1) Pause Young (Normal) 1M->0M(2M)",
        "[0.1s][info][gc] GC(1) Pause Young (Normal) 1M->0M(2M) 18446744073709551615ms",
        "[0.1s][info][gc] GC(1) Pause Young (Normal) 1M->0M(2M) 1.1234567ms",
        "[0.1s][info][gc] GC(1) Pause Young (Normal) 1M->0M(2M) 1ms [truncated",
        "[nonsense][info][gc] GC(1) Pause Young (Normal) 1M->0M(2M) 1ms",
    ];
    for line in cases {
        let input = format!("unrelated\n{line}\n");
        let error = GcLogParser::new(Cursor::new(input))
            .nth(1)
            .expect("second line")
            .expect_err(line);
        assert_eq!(error.line, 2, "{line}");
    }
}

#[test]
fn unsupported_and_detail_records_are_distinct() {
    let input = "[0.1s][info][gc,phases] GC(1) Pause Young phase 1ms\n[0.1s][info][gc] GC(1) Pause Future 1ms\n[0.1s][info][gc] GC(1) Concurrent Mark 1ms\nplain message\n";
    let records: Vec<_> = GcLogParser::new(Cursor::new(input))
        .collect::<Result<_, _>>()
        .expect("classified");
    assert_eq!(
        records,
        vec![
            Record::Skipped(SkipReason::Unrelated),
            Record::Skipped(SkipReason::UnsupportedEvent),
            Record::Skipped(SkipReason::Unrelated),
            Record::Skipped(SkipReason::Unrelated)
        ]
    );
}

#[test]
fn sdk_can_continue_after_malformed_candidate() {
    let input = "[info][gc] GC(1) Pause Young (Normal) 1T->0M(2M) 1ms\n[info][gc] GC(2) Pause Young (Normal) 1M->0M(2M) 1ms\n";
    let mut parser = GcLogParser::new(Cursor::new(input));
    assert_eq!(
        parser
            .next()
            .expect("first line")
            .expect_err("malformed")
            .line,
        1
    );
    let second = parser
        .next()
        .expect("second line")
        .expect("valid second line");
    assert!(matches!(second, Record::Event(event) if event.gc_id == 2 && event.source_line == 2));
}

#[test]
fn invalid_utf8_and_long_lines_fail_explicitly() {
    let error = GcLogParser::new(Cursor::new(vec![0xff, b'\n']))
        .next()
        .expect("line")
        .expect_err("invalid UTF-8");
    assert!(matches!(error.kind, ParseErrorKind::InvalidUtf8));
    let error = GcLogParser::new(Cursor::new(vec![b'a'; 65_537]))
        .next()
        .expect("line")
        .expect_err("long line");
    assert!(matches!(error.kind, ParseErrorKind::LineTooLong));
}

#[test]
fn streams_ten_thousand_events() {
    let line =
        "[0.1s][info][gc] GC(1) Pause Young (Normal) (G1 Evacuation Pause) 2M->1M(4M) 0.5ms\n";
    let input = line.repeat(10_000);
    let mut count = 0;
    for record in GcLogParser::new(Cursor::new(input)) {
        assert!(matches!(record.expect("valid event"), Record::Event(_)));
        count += 1;
    }
    assert_eq!(count, 10_000);
}
