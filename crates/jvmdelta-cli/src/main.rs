use clap::{Parser, Subcommand};
use jvmdelta_core::gc_log::{EventKind, GcLogParser, Record, SkipReason};
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
    let mut counts = [0_u64; 4];
    let mut skipped = 0_u64;
    let mut unsupported = 0_u64;
    for record in GcLogParser::new(BufReader::new(file)) {
        match record? {
            Record::Event(event) => {
                let index = match event.kind {
                    EventKind::Young => 0,
                    EventKind::Full => 1,
                    EventKind::Remark => 2,
                    EventKind::Cleanup => 3,
                };
                counts[index] += 1;
            }
            Record::Skipped(SkipReason::Unrelated) => skipped += 1,
            Record::Skipped(SkipReason::UnsupportedEvent) => unsupported += 1,
        }
    }
    println!("File: {}", path.display());
    println!("Supported events: {}", counts.iter().sum::<u64>());
    println!("Young: {}", counts[0]);
    println!("Full: {}", counts[1]);
    println!("Remark: {}", counts[2]);
    println!("Cleanup: {}", counts[3]);
    println!("Unrelated entries skipped: {skipped}");
    println!("Unsupported pause types skipped: {unsupported}");
    Ok(())
}
