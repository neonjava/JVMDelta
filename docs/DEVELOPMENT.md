# Development

Use stable Rust 1.85 or newer. The workspace uses Rust 2024 edition and Cargo resolver 3.

Run the same commands as CI before submitting a change:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

Keep core logic independent of CLI formatting. Add unit tests for domain behavior and CLI integration tests for observable exit codes and output. Store small, redistributable sample recordings in `tests/fixtures` once parsers exist; document where each fixture came from. Do not commit generated reports, recordings containing sensitive application data, credentials, or `target` artifacts.
