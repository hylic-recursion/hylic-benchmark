//! Head-to-head: all executor types on the general scenario set.
//! rayon, hylo, funnel(robust) + handrolled baselines.

#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use hylic::cata::exec::funnel;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec};

use support::config;
use support::scenario::{self, Scale, PreparedScenario};
use support::runners::{self, FunnelSpecs};
use support::bench_cell;

fn bench_executor_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor-compare");
    let nw = config::bench_workers();
    let funnel_specs = FunnelSpecs::new(nw);

    WorkPool::with(WorkPoolSpec::threads(nw), |wpool| {
        funnel::Pool::with(nw, |fpool| {
            for def in scenario::all_scenarios(Scale::from_env()) {
                let s = PreparedScenario::from_def(&def, "sm");

                let all = vec![
                    runners::rayon(&s),
                    runners::sheque(&s, wpool),
                    runners::funnel_variant("funnel", &s, fpool, &funnel_specs.pw_final),
                    runners::hand_rayon(&s),
                    runners::hand_pool(&s, wpool),
                ];

                for r in &all {
                    bench_cell(&mut group, r.name, &s.name,
                        |b, _| b.iter(|| black_box((r.run)())),
                    );
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_executor_compare);
criterion_main!(benches);
