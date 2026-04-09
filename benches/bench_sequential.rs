#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use support::scenario::{self, Scale, PreparedScenario};
use support::runners;
use support::bench_cell;

fn bench_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential");

    for def in scenario::all_scenarios(Scale::from_env()) {
        let s = PreparedScenario::from_def(&def, "sm");
        for r in runners::sequential_runners(&s) {
            bench_cell(&mut group, r.name, &s.name,
                |b, _| b.iter(|| black_box((r.run)())),
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_sequential);
criterion_main!(benches);
