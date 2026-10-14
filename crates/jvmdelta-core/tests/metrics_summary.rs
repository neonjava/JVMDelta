use jvmdelta_core::gc_log::{EventKind, GcEvent, GcLogParser, Record};
use jvmdelta_core::metrics::GcSummary;
use std::io::Cursor;

fn summarize(input: &str) -> GcSummary {
    let mut summary = GcSummary::default();
    for record in GcLogParser::new(Cursor::new(input)) {
        if let Record::Event(event) = record.expect("fixture parses") {
            summary.record(&event).expect("sample fits in summary");
        }
    }
    summary
}

#[test]
fn empty_summary_has_no_extrema_or_mean() {
    let summary = summarize("");
    assert_eq!(summary.all().count(), 0);
    assert_eq!(summary.all().total_ns(), 0);
    assert_eq!(summary.all().min_ns(), None);
    assert_eq!(summary.all().max_ns(), None);
    assert_eq!(summary.all().mean_ns_floor(), None);
    assert_eq!(summary.category(EventKind::Full).mean_ns_floor(), None);
}

#[test]
fn category_and_overall_durations_are_exact() {
    let input = include_str!("../../../tests/fixtures/jdk17-g1-synthetic.log");
    let summary = summarize(input);
    assert_eq!(summary.all().count(), 4);
    assert_eq!(summary.all().total_ns(), 10_877_000);
    assert_eq!(summary.all().min_ns(), Some(50_000));
    assert_eq!(summary.all().max_ns(), Some(6_250_000));
    assert_eq!(summary.all().mean_ns_floor(), Some(2_719_250));
    assert_eq!(summary.category(EventKind::Young).count(), 1);
    assert_eq!(summary.category(EventKind::Young).total_ns(), 3_452_000);
    assert_eq!(summary.category(EventKind::Full).total_ns(), 6_250_000);
    assert_eq!(summary.category(EventKind::Remark).total_ns(), 1_125_000);
    assert_eq!(summary.category(EventKind::Cleanup).total_ns(), 50_000);
}

#[test]
fn mean_rounds_down_to_nanosecond_and_ids_are_not_deduplicated() {
    let mut summary = GcSummary::default();
    for duration in [1, 2] {
        summary
            .record(&GcEvent {
                source_line: 1,
                gc_id: 7,
                kind: EventKind::Remark,
                description: String::new(),
                heap: None,
                duration_ns: duration,
                uptime_ns: None,
                source: String::new(),
            })
            .expect("fits");
    }
    assert_eq!(summary.category(EventKind::Remark).count(), 2);
    assert_eq!(summary.category(EventKind::Remark).total_ns(), 3);
    assert_eq!(summary.category(EventKind::Remark).mean_ns_floor(), Some(1));
    assert_eq!(summary.category(EventKind::Full).min_ns(), None);
}

#[test]
fn real_jdk17_and_21_samples_parse_and_aggregate() {
    let jdk17 = summarize(include_str!("../../../tests/fixtures/jdk17-g1-real.log"));
    assert_eq!(jdk17.all().count(), 2);
    assert_eq!(jdk17.category(EventKind::Young).total_ns(), 1_420_000);
    assert_eq!(jdk17.category(EventKind::Full).total_ns(), 2_327_000);
    assert_eq!(jdk17.all().total_ns(), 3_747_000);

    let jdk21 = summarize(include_str!("../../../tests/fixtures/jdk21-g1-real.log"));
    assert_eq!(jdk21.all().count(), 2);
    assert_eq!(jdk21.category(EventKind::Young).total_ns(), 1_456_000);
    assert_eq!(jdk21.category(EventKind::Full).total_ns(), 3_257_000);
    assert_eq!(jdk21.all().total_ns(), 4_713_000);
}
