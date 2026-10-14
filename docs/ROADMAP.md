# Development roadmap

## Completed — Day 1

Cargo workspace, CLI foundation, documentation, Apache-2.0 license, and CI.

## Completed — Day 2

A bounded streaming parser for selected G1 Unified GC pause summaries, typed events with exact units and source provenance, explicit errors and skip reasons, real and synthetic fixtures, and `gc inspect`. See [format support](GC_LOG_SUPPORT.md).

## Completed — Day 3

Validated young and full pause summaries against local Temurin 17 and 21 G1 output. Added constant-size category and overall pause counts and exact duration statistics, with explicit missing-data behavior and CLI output. Real fixture excerpts and reproduction instructions are checked in. Remark/cleanup examples remain synthetic.

## Day 4 — comparison semantics

Define baseline/candidate data sets, zero-baseline behavior, and missing-metric behavior. Implement deterministic comparisons without implying significance from a single run. Decide how to handle unequal run duration and event counts before showing deltas.

## Day 5 — regression policy and CLI

Add explicit thresholds and exit codes after comparison semantics are tested.

## Day 6 — reports and CI usage

Add structured report data, terminal and JSON output, and a reproducible CI example. Defer HTML if the report model is still changing.

## Day 7 — JFR research and hardening

Document a JVM-free JFR approach and format/licensing constraints. Improve diagnostics and review performance on bounded fixtures.
