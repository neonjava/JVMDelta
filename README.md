# JVMDelta

JVMDelta is a Rust project for comparing JVM performance measurements before and after a software change. GC logs and JFR recordings can contain evidence that is tedious to inspect by hand. JVMDelta aims to make those measurements accessible locally and in CI without needing a JVM for analysis.

## Current status

The current CLI inspects a narrow subset of G1 Unified GC pause summaries. It counts supported young, full, remark, and cleanup pauses, unrelated entries, and unknown pause categories. The core library exposes parsed events with exact units and source-line provenance. Invalid supported summaries fail with a line-numbered error. See [supported syntax and limitations](docs/GC_LOG_SUPPORT.md).

Comparison, regression detection, JFR parsing, JSON/HTML reports, and a GitHub Action are **planned**, not implemented.

## Architecture

- `crates/jvmdelta-core`: bounded streaming GC parser and small domain modules for future metrics, comparison, regression, and reports.
- `crates/jvmdelta-cli`: Clap executable, file access, counts, and exit status.
- `tests/fixtures`: synthetic JDK 17/21-style examples and selected real JDK 25 G1 output.
- `docs`: [architecture](docs/ARCHITECTURE.md), [format support](docs/GC_LOG_SUPPORT.md), [development](docs/DEVELOPMENT.md), and [roadmap](docs/ROADMAP.md).

## Build and use

Requires stable Rust 1.85 or newer. Java is not required to build, test, or inspect a log.

```sh
cargo build --workspace
cargo run -p jvmdelta-cli -- --help
cargo run -p jvmdelta-cli -- --version
cargo run -p jvmdelta-cli -- gc inspect tests/fixtures/jdk25-g1-real.log
```

The installed executable is named `jvmdelta`. `gc inspect` uses strict parsing: unreadable files and malformed supported summaries exit nonzero with an error on stderr.

## Development and contributing

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

Focused issues and pull requests are welcome. Discuss new GC variants or comparison semantics before extending support. Add representative, redistributable fixtures and tests, keep errors explicit, and document limitations. Contributions are licensed under [Apache-2.0](LICENSE).
