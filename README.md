# hylic-benchmark

Criterion benchmark harness for [`hylic`](https://github.com/hylic-recursion/hylic).
Internal: not published to crates.io.

Three benches:

- **Matrix** compares the Funnel executor's 16 policy variants
  against Rayon and scoped-pool baselines across 14 workload
  scenarios.
- **Module-sim** runs a synthetic dependency-graph resolver —
  the workload that originally motivated the library.
- **Quick** is a fast-returning subset for tracking changes
  during development, including a multi-revision A/B mode.

The rendered output, including an interactive viewer over the
Matrix axes, is published with the docs site:
**[Benchmark results](https://hylic-recursion.github.io/hylic-docs/cookbook/benchmarks.html)**.

## Running locally

```bash
make bench-quick-light       # subset for dev iteration
make bench-matrix            # full 16-funnel × 14-scenario matrix
make bench-modsim            # module-resolution simulator
```

The make targets dispatch through scripts at the workspace
root (criterion JSON → HTML/CSV/txt). Output ends up under
`target/bench-latest/` and is also written into the hylic-docs
source tree for the next book build.

## Related crates

- [`hylic`](https://github.com/hylic-recursion/hylic) — core
  (what's being benchmarked).
- [`hylic-docs`](https://github.com/hylic-recursion/hylic-docs)
  — where the rendered results are published.

## License

Licensed under the [MIT License](./LICENSE).
