# JVMDelta

JVMDelta is a Rust project for comparing JVM performance measurements before and after a software change. The goal is to make regressions visible in local development and CI without requiring a JVM to analyze recorded data.

Performance evidence is often spread across GC logs and JFR recordings. Comparing runs by hand is slow and makes it easy to overlook a change in pause time or allocation. JVMDelta aims to ingest those recordings, compare a baseline with a candidate, and produce clear reports.

## Current status

Day 1 is a foundation release. The `jvmdelta` executable provides `--help` and `--version`. It does **not** parse recordings, compare runs, detect regressions, or produce reports yet. Unsupported commands fail with a nonzero exit status.

## Architecture

- `crates/jvmdelta-core`: reusable library with small domain modules for GC logs, metrics, comparisons, regression rules, and reports. Its current types define vocabulary only.
- `crates/jvmdelta-cli`: Clap based executable. CLI presentation stays outside the core library.
- `tests/fixtures`: reserved for representative, licensed input files as parsers are implemented.
- `docs`: architecture, development guidance, and the implementation roadmap.

See [architecture](docs/ARCHITECTURE.md) for the intended data flow and boundaries.

## Planned features

- Parse JVM Unified GC logs and JFR recordings.
- Compute supported pause, memory, and allocation metrics.
- Compare baseline and candidate measurements and detect configured regressions.
- Render terminal, JSON, and HTML reports.
- Supply a GitHub Action for CI use.

These features are planned; no analysis command is available today.

## Build and use

Requires stable Rust 1.85 or newer. No JVM is needed for the current CLI.

```sh
cargo build --workspace
cargo run -p jvmdelta-cli -- --help
cargo run -p jvmdelta-cli -- --version
```

The installed binary is named `jvmdelta`; for example, `jvmdelta --help`.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

See [development guidance](docs/DEVELOPMENT.md) for test and contribution practices.

## Contributing

Issues and focused pull requests are welcome. Discuss input format choices and regression semantics before implementing a parser or detector. Add representative fixtures and tests for behavior, keep parser errors explicit, and run the checks above. Contributions are licensed under [Apache-2.0](LICENSE).
