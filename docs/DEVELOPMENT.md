# Development

Use stable Rust 1.85 or newer. The workspace uses Rust 2024 edition and Cargo resolver 3. Rust tests use checked-in fixtures and do not require Java.

Run the CI checks before submitting a change:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

Run the CLI against real captured fixtures:

```sh
cargo run -p jvmdelta-cli -- gc inspect tests/fixtures/jdk25-g1-real.log
cargo run -p jvmdelta-cli -- gc inspect tests/fixtures/jdk17-g1-real.log
cargo run -p jvmdelta-cli -- gc inspect tests/fixtures/jdk21-g1-real.log
```

Keep core logic independent of CLI formatting. Add unit tests for parser and aggregate behavior and CLI integration tests for exit codes and output. Record each fixture's origin, JDK version, collector, and reproduction procedure in `tests/fixtures/README.md`. Rust tests must not require Java. Do not commit sensitive recordings, credentials, or build artifacts. See [format support](GC_LOG_SUPPORT.md) before extending the parser.
