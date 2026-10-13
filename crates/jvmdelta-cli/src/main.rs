use clap::Parser;

/// JVM performance regression analysis (analysis commands are planned).
#[derive(Debug, Parser)]
#[command(
    name = "jvmdelta",
    version,
    about = "JVM performance regression analysis; analysis commands are planned",
    long_about = None
)]
struct Cli {}

fn main() {
    Cli::parse();
}
