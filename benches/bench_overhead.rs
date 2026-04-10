//! Overhead — framework cost.
//! Fused executor vs handrolled recursive baselines. No parallelism.
//! Reproduce: make -C hylic-benchmark _bench-overhead

#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use support::scenario::{self, Scale, PreparedScenario};
use support::{baselines, bench_cell};

fn bench_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("overhead");

    for def in scenario::all_scenarios(Scale::from_env()) {
        let s = PreparedScenario::from_def(&def, "sm");
        let p = s.as_problem();
        let all = vec![
            baselines::fused(&p),
            baselines::hand_seq(&s),
            baselines::real_seq(&s),
        ];
        for r in &all {
            bench_cell(&mut group, r.name, &p.name,
                |b, _| b.iter(|| black_box((r.run)())),
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_overhead);
criterion_main!(benches);
