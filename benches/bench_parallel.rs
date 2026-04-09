#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use hylic::cata::exec::funnel;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec};
use support::scenario::{self, Scale, PreparedScenario};
use support::runners::{self, FunnelSpecs};
use support::bench_cell;

fn bench_parallel(c: &mut Criterion) {
    let nw = support::config::bench_workers();
    let funnel_specs = FunnelSpecs::new(nw);

    WorkPool::with(WorkPoolSpec::threads(nw), |wpool| {
        funnel::Pool::with(nw, |fpool| {
            let mut group = c.benchmark_group("parallel");

            for def in scenario::all_scenarios(Scale::from_env()) {
                let s = PreparedScenario::from_def(&def, "sm");
                let mut all = runners::parallel_runners(&s, wpool, nw);
                all.push(runners::funnel_variant("funnel", &s, fpool, &funnel_specs.pw_final));
                all.extend(runners::baseline_runners(&s, wpool));
                for r in &all {
                    bench_cell(&mut group, r.name, &s.name,
                        |b, _| b.iter(|| black_box((r.run)())),
                    );
                }
            }

            group.finish();
        });
    });
}

criterion_group!(benches, bench_parallel);
criterion_main!(benches);
