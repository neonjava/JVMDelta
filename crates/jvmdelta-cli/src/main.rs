use clap::{Parser, Subcommand};
use jvmdelta_core::gc_log::{EventKind, GcLogParser, Record, SkipReason};
use jvmdelta_core::metrics::{GcSummary, PauseStats};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Inspect JVM GC logs. Comparison and regression analysis are planned.
#[derive(Debug, Parser)]
#[command(name = "jvmdelta", version, about = "Inspect JVM GC logs; comparison and regression analysis are planned", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect supported G1 Unified GC pause summaries.
    Gc {
        #[command(subcommand)]
        command: GcCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GcCommand {
    /// Count and classify pause summaries in a GC log.
    Inspect {
        /// Path to a UTF-8 JVM Unified GC log.
        path: PathBuf,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("jvmdelta: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Gc {
            command: GcCommand::Inspect { path },
        }) => inspect(path),
        None => Ok(()),
    }
}

fn inspect(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let file =
        File::open(&path).map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut summary = GcSummary::default();
    let mut skipped = 0_u64;
    let mut unsupported = 0_u64;
    for record in GcLogParser::new(BufReader::new(file)) {
        match record? {
            Record::Event(event) => summary.record(&event)?,
            Record::Skipped(SkipReason::Unrelated) => skipped += 1,
            Record::Skipped(SkipReason::UnsupportedEvent) => unsupported += 1,
        }
    }
    println!("File: {}", path.display());
    println!("Supported events: {}", summary.all().count());
    for (name, kind) in categories() {
        println!("{name}: {}", summary.category(kind).count());
    }
    println!("Unrelated entries skipped: {skipped}");
    println!("Unsupported pause types skipped: {unsupported}");
    println!("Pause durations (ms; mean rounded down to 1 ns):");
    print_duration_stats("All", summary.all());
    for (name, kind) in categories() {
        print_duration_stats(name, summary.category(kind));
    }
    Ok(())
}

fn categories() -> [(&'static str, EventKind); 4] {
    [
        ("Young", EventKind::Young),
        ("Full", EventKind::Full),
        ("Remark", EventKind::Remark),
        ("Cleanup", EventKind::Cleanup),
    ]
}

fn print_duration_stats(name: &str, stats: &PauseStats) {
    let min = stats.min_ns().map(|value| format_ms(u128::from(value)));
    let mean = stats
        .mean_ns_floor()
        .map(|value| format_ms(u128::from(value)));
    let max = stats.max_ns().map(|value| format_ms(u128::from(value)));
    println!(
        "  {name}: total={} min={} mean={} max={}",
        format_ms(stats.total_ns()),
        min.as_deref().unwrap_or("n/a"),
        mean.as_deref().unwrap_or("n/a"),
        max.as_deref().unwrap_or("n/a")
    );
}

fn format_ms(ns: u128) -> String {
    format!("{}.{:06}", ns / 1_000_000, ns % 1_000_000)
}
