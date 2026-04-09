#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use support::scenario::{self, Scale, PreparedScenario};
use support::{runners, baselines, bench_cell};

fn bench_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential");

    for def in scenario::all_scenarios(Scale::from_env()) {
        let s = PreparedScenario::from_def(&def, "sm");
        let p = s.as_problem();
        let mut all = runners::sequential_runners(&p);
        all.push(baselines::hand_seq(&s));
        all.push(baselines::real_seq(&s));
        for r in &all {
            bench_cell(&mut group, r.name, &p.name,
                |b, _| b.iter(|| black_box((r.run)())),
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_sequential);
criterion_main!(benches);
