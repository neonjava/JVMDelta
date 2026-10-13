# G1 Unified GC log support

JVMDelta parses a deliberately narrow subset of HotSpot Unified Logging as emitted by G1 on JDK 17 and 21. The JDK 17 and 21 fixtures are **synthetic examples** modeled on documented syntax; they have not been validated against those local JVM versions. A separate small JDK 25 fixture was captured locally and checks the same syntax. The parser does not identify a JVM version or independently verify the collector from a log; callers should supply G1 logs.

Authoritative references:

- [JDK 17 `java` command, `-Xlog` syntax and decorations](https://docs.oracle.com/en/java/javase/17/docs/specs/man/java.html)
- [JDK 21 `java` command, `-Xlog` syntax and decorations](https://docs.oracle.com/en/java/javase/21/docs/specs/man/java.html)
- [JDK 21 G1 collector phases](https://docs.oracle.com/en/java/javase/21/gctuning/garbage-first-g1-garbage-collector1.html)
- [JDK 17 GC troubleshooting log examples](https://docs.oracle.com/en/java/javase/17/troubleshoot/troubleshooting-memory-leaks.html)

## Accepted records

An `info` record tagged exactly `gc` with `GC(<unsigned integer>)` followed by `Pause Young`, `Pause Full`, `Pause Remark`, or `Pause Cleanup` is a candidate. Young and Full summaries require `before->after(capacity)` and a duration. Remark and Cleanup require a duration and may omit heap sizes. Description text between the category and heap summary is retained. A summary may be undecorated or have `uptime` (`[0.123s]`), a wall-clock `time` decoration, `[info]`, and `[gc]`. The parser accepts those decorations in any order. Wall-clock time is not interpreted.

Example:

```text
[0.123s][info][gc] GC(0) Pause Young (Normal) (G1 Evacuation Pause) 20M->8M(128M) 3.452ms
```

The event has ID 0, uptime 123,000,000 ns, heap 20/8/128 MiB expressed as bytes, and pause duration 3,452,000 ns. Units `B`, `K`, `M`, and `G` use binary powers of 1024. Heap values must be unsigned integers. Millisecond durations accept up to six fractional digits; uptime seconds accept up to nine. Values must fit in `u64` after conversion.

`gc,start`, `gc,heap`, `gc,phases`, concurrent records, and non-GC lines are skipped, preventing phase lines from counting as pauses. Unknown `Pause` categories are counted separately as unsupported. `debug` and `trace` records are not interpreted as summary events. Legacy `-XX:+PrintGC` output, other collectors, alternate duration units, additional decorators, and arbitrary GC messages are outside this subset. A log line is limited to 64 KiB including its line ending.

## Errors and strictness

`GcLogParser<R: BufRead>` yields one classified result per line: event, skip reason, or `ParseError` with a one-based line number. Malformed candidate summaries fail explicitly, including invalid IDs, sizes, units, durations, or truncated summaries. Invalid UTF-8, I/O errors, and overlong lines fail explicitly and terminate iteration. Malformed candidate lines are recoverable for SDK callers that choose to continue iterating; this is the **permissive** policy. The `gc inspect` CLI uses the **strict** policy: it reports the first parser error on stderr and exits nonzero, with no partial count report. Unsupported categories are reported as skipped, not analyzed.

## CLI

```sh
cargo run -p jvmdelta-cli -- gc inspect tests/fixtures/jdk25-g1-real.log
```

The command prints supported event totals, counts for each category, unrelated entries skipped, and unsupported pause types skipped. It does not compute regression scores.

## Fixtures

See [`tests/fixtures/README.md`](../tests/fixtures/README.md). Rust tests use checked-in text only; Java is not required for test execution.
