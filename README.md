# hylic-benchmark

Criterion-based benchmark suite for the [`hylic`](../hylic/) core
and [`hylic-parallel-lifts`](../hylic-parallel-lifts/) executors.

## What's here

- `benches/` — criterion bench files for fold/lift comparisons,
  module-resolution simulation, executor matrices.
- `tests/` — integration tests that call the bench helpers without
  the criterion harness (run as part of `cargo test`).
- `src/` — runner / scenario generator helpers shared by the bench
  files.

## Running

From the workspace root:

```bash
make bench-quick-light       # 5 runners × 9 scenarios, ~3 min
make bench-quick-heavy       # 80 samples / 20s measure, ~18 min
make bench-matrix            # full 16-funnel × 14-scenario matrix
make bench-modsim            # module-resolution simulator
```

These targets dispatch to the workspace-root
[`Makefile-scripting/`](../Makefile-scripting/) shell + Python pipeline
(criterion JSON → HTML/CSV/txt reports). Output lands under
`target/bench-latest/` and (for the docs build) under
[`hylic-docs/book/src/bench-results/`](../hylic-docs/book/src/bench-results/).

## Status

Auxiliary crate. Not published to crates.io.
