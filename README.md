# hylic-benchmark

Criterion harness for [`hylic`](https://github.com/hylic-recursion/hylic). Not published to crates.io; lives next to the core crate so the benchmark workloads can call into internal APIs (`Funnel::Pool::with`, `StealQueue`, the per-policy `Spec` constructors) directly.

Three benches:

- **Overhead** — pits `Fused` against handrolled single-threaded recursions, then lists `Funnel`, `Rayon`, and a scoped pool on the same scenarios. Establishes how much the closure-based Fold/Treeish indirection costs against a hand-written `fn rec(node) -> R`. Across the rows, `Fused` is within ±20% of `hand.seq` (faster on most), and `hylic-rayon` is within ±15% of `real.rayon` on most rows with a worst-case +33%.
- **Matrix** — sixteen `Funnel` policy variants × fourteen workload scenarios, each row marking the winner and showing every other entry as a percentage from it. The shape of which preset wins which row is the substance: shallow-wide rows want `Shared` queues with `OnArrival`, deep-narrow rows want `PerWorker` with `OnFinalize`, and the wake axis can move a row by 10–30% on its own. Rendered output includes an interactive viewer that marginalises on any axis.
- **Module-sim** — eight workloads on a synthetic dependency-graph resolver, parameterised on (sparse vs dense graph) × (fast vs slow per-node work). On the four `_fast` rows `Funnel` variants take three of the four winners; on the `_slow` rows the runners cluster because per-node work dominates scheduling.
- **Quick** — five runners over nine scenarios, fast enough for development iteration. Has a multi-revision A/B mode (`make bench-quick-light-ab label=gitref`) that rebuilds older `hylic` revisions and lays them next to the current run.

Rendered output, including the interactive Matrix viewer and the rendered tables, is published with the docs site:
**[Benchmark results](https://hylic-recursion.github.io/hylic-docs/cookbook/benchmarks.html)**.

## Running locally

```bash
make bench-overhead          # ~1 minute
make bench-quick-light       # ~3 minutes — subset for dev iteration
make bench-matrix            # ~20 minutes — full 16-funnel × 14-scenario matrix
make bench-modsim            # ~10 minutes — module-resolution simulator
```

The make targets dispatch through scripts at the workspace root: criterion JSON is post-processed into HTML, CSV, and txt tables, written to `target/bench-latest/`, and copied into the hylic-docs source tree for the next book build.

Workload definitions are in `benches/support/`: scenarios in `scenario.rs`, work shapes in `work.rs`, the runner matrix in `runners.rs`, and the policy presets used in the Matrix bench in `executor_set.rs`. Every bench harness asserts that the computed `R` matches a reference Fused run before timing begins; a runner producing a faster-but-incorrect answer cannot reach the tables.

## Related crates

[`hylic`](https://github.com/hylic-recursion/hylic) is what's being benchmarked. [`hylic-docs`](https://github.com/hylic-recursion/hylic-docs) is where the rendered results are published.

## License

Licensed under the [MIT License](./LICENSE).
