# Architecture

The workspace separates analysis from command-line presentation.

`jvmdelta-core` owns input interpretation, measurements, comparison rules, regression decisions, and report data. `jvmdelta-cli` will parse user arguments, call core APIs, select an output format, and choose an exit status. A caller using the library will not need Clap or a JVM.

Intended data flow:

```text
GC log / JFR recording
        ↓
format-specific parser and validated events
        ↓
measurements with explicit units and provenance
        ↓
baseline/candidate comparison
        ↓
regression decisions and structured report
        ↓
terminal / JSON / HTML renderer
```

Only module boundaries and a few domain types exist on Day 1. A source line type validates one-based line numbering. Units, comparison sides, regression direction, and report formats are enums for upcoming APIs. There is no parser, analysis, or renderer yet.

Future parsers should preserve source location in errors and distinguish unsupported input from malformed input. Metric definitions should state units, aggregation, and whether higher or lower values are worse. Comparison should avoid inferring statistical significance from a single sample. Structured report data should remain independent of output rendering.

