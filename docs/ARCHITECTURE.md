# Architecture

`jvmdelta-core` owns format parsing and analysis types. `jvmdelta-cli` owns argument parsing, file opening, presentation, and process exit status. The core parser takes any `BufRead`; it does not depend on Clap, paths, or a JVM.

The implemented GC path is:

```text
UTF-8 G1 Unified GC log
        ↓ bounded line reader (64 KiB per line)
validated pause event / explicit skip reason / structured error
        ↓
constant-size pause statistics or SDK caller processing
```

`GcLogParser` yields records one at a time, so SDK users can aggregate or store only what they need. An event retains its source line and original line, GC ID, pause category, description, optional heap values in bytes, duration in nanoseconds, and optional uptime in nanoseconds. `GcSummary` consumes borrowed events, retaining only category and overall counts, exact `u128` nanosecond totals, and optional `u64` extrema. Its mean is integer division of total by count, rounded down to one nanosecond. Recording is atomic on overflow. The CLI stops at the first parser error. SDK users may continue after malformed candidate lines, but terminal errors end iteration.

The comparison, regression, and report modules remain small domain definitions for later work. They do not calculate results. The intended later flow is measurements → baseline/candidate comparison → regression decisions → report model → output renderer. Comparison must avoid implying statistical significance from one sample.

See [GC log support](GC_LOG_SUPPORT.md) for the exact accepted syntax and limitations.
