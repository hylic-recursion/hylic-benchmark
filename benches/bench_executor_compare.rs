//! Head-to-head: key executor types on the general scenario set.

#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use hylic::cata::exec::funnel;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec};
use support::scenario::{self, Scale, PreparedScenario};
use support::executor_set::{ExecutorSet, FunnelSpecs};
use support::{runners, baselines, bench_cell};

fn bench_executor_compare(c: &mut Criterion) {
    let nw = support::config::bench_workers();

    WorkPool::with(WorkPoolSpec::threads(nw), |wpool| {
        funnel::Pool::with(nw, |fpool| {
            let es = ExecutorSet {
                wpool, fpool, nw,
                sheque: hylic_benchmark::executor::hylo_sheque::Spec::default(nw),
                funnel: FunnelSpecs::new(nw),
            };
            let mut group = c.benchmark_group("executor-compare");

            for def in scenario::all_scenarios(Scale::from_env()) {
                let s = PreparedScenario::from_def(&def, "sm");
                let p = s.as_problem();

                let all = vec![
                    runners::rayon(&p),
                    runners::sheque(&p, &es),
                    runners::funnel_variant("funnel.pw.fin", &p, &es, es.funnel.pw_final),
                    baselines::hand_rayon(&s),
                    baselines::hand_pool(&s, wpool),
                ];

                for r in &all {
                    bench_cell(&mut group, r.name, &p.name,
                        |b, _| b.iter(|| black_box((r.run)())),
                    );
                }
            }

            group.finish();
        });
    });
}

criterion_group!(benches, bench_executor_compare);
criterion_main!(benches);
