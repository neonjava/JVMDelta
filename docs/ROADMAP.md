# Day 2–7 roadmap

These are implementation targets, not shipped features. Adjust scope when format research or representative fixtures reveal constraints.

## Day 2 — GC log contract

Choose supported Java Unified Logging syntax and versions. Add small, redistributable fixtures covering timestamps, tags, units, malformed lines, and unknown events. Define event types and useful parser errors.

## Day 3 — GC parsing and measurements

Implement a streaming parser for the documented subset. Calculate pause counts and duration summaries with explicit units. Test event extraction against fixtures and malformed input.

## Day 4 — Comparison semantics

Define baseline and candidate data sets, missing-metric behavior, and percentage-change rules. Implement deterministic comparisons with tests for zero baselines and unequal sample sizes.

## Day 5 — Regression policy and CLI

Add explicit thresholds and exit codes. Introduce a compare command only after it can reject unsupported or incomplete input clearly. Test CLI error paths and real fixture comparisons.

## Day 6 — Reports and CI usage

Create a structured report model and terminal/JSON output. Document reproducible CI invocation and add a minimal GitHub Action workflow example. Keep HTML as a later milestone if report data is still changing.

## Day 7 — JFR research and hardening

Document JFR format and licensing considerations, decide on a JVM-free parsing approach, and prototype a narrow reader if feasible. Review performance on bounded fixtures and improve error messages. Schedule full JFR, HTML, and distribution work based on results.
