//! Module resolution simulation — hylic vs vanilla on realistic dependency graphs.

#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use hylic::cata::exec::funnel;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec};
use support::executor_set::{ExecutorSet, FunnelSpecs};
use support::{module_sim, runners, bench_cell};

fn bench_module_sim(c: &mut Criterion) {
    let nw = support::config::bench_workers();

    WorkPool::with(WorkPoolSpec::threads(nw), |wpool| {
        funnel::Pool::with(nw, |fpool| {
            let es = ExecutorSet {
                wpool, fpool, nw,
                sheque: hylic_benchmark::executor::hylo_sheque::Spec::default(nw),
                funnel: FunnelSpecs::new(nw),
            };
            let mut group = c.benchmark_group("module-sim");

            for spec in module_sim::all_module_scenarios(false) {
                let sim = module_sim::prepare(&spec);
                let p = module_sim::as_problem(&sim);

                let mut all = runners::all_hylic_runners(&p, &es);
                all.extend(module_sim::vanilla_baselines(&sim));

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

criterion_group!(benches, bench_module_sim);
criterion_main!(benches);
