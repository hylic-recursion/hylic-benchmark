//! Funnel policy matrix vs sheque and rayon baselines.
//!
//! 8 funnel policy variants x 6 tuned scenarios. Each variant uses
//! Pool::with for realistic pool-reuse measurement. Per-fold memory
//! is fresh each iteration — only threads reused.

#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use hylic::cata::exec::funnel;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec};
use support::scenario::{Scale, PreparedScenario, ScenarioDef};
use support::tree::TreeSpec;
use support::work::WorkSpec;
use support::executor_set::{ExecutorSet, FunnelSpecs};
use support::{runners, baselines, bench_cell};

fn hylo_scenarios(_scale: Scale) -> Vec<ScenarioDef> {
    let w = |init, acc, fin, graph, io| WorkSpec {
        init_work: init, accumulate_work: acc, finalize_work: fin,
        graph_work: graph, graph_io_us: io,
    };
    vec![
        ScenarioDef { name: "noop",        moniker: "noop",     tree: TreeSpec { node_count: 200, branch_factor: 8 },  work: w(0, 0, 0, 0, 0) },
        ScenarioDef { name: "wide-light",  moniker: "wide-lt",  tree: TreeSpec { node_count: 80, branch_factor: 20 },  work: w(50_000, 10_000, 10_000, 10_000, 0) },
        ScenarioDef { name: "fold-light",  moniker: "fold-lt",  tree: TreeSpec { node_count: 80, branch_factor: 8 },   work: w(50_000, 50_000, 50_000, 5_000, 0) },
        ScenarioDef { name: "graph-heavy", moniker: "graph-hv", tree: TreeSpec { node_count: 500, branch_factor: 8 },  work: w(5_000, 5_000, 5_000, 500_000, 0) },
        ScenarioDef { name: "fold-heavy",  moniker: "fold-hv",  tree: TreeSpec { node_count: 500, branch_factor: 8 },  work: w(200_000, 200_000, 200_000, 5_000, 0) },
        ScenarioDef { name: "bal-heavy",   moniker: "bal-hv",   tree: TreeSpec { node_count: 500, branch_factor: 8 },  work: w(100_000, 100_000, 100_000, 100_000, 0) },
    ]
}

fn bench_hylo_compare(c: &mut Criterion) {
    let nw = support::config::bench_workers();
    eprintln!("[hylo-compare] using {nw} worker threads");

    WorkPool::with(WorkPoolSpec::threads(nw), |wpool| {
        funnel::Pool::with(nw, |fpool| {
            let es = ExecutorSet {
                wpool, fpool, nw,
                sheque: hylic_benchmark::executor::hylo_sheque::Spec::default(nw),
                funnel: FunnelSpecs::new(nw),
            };
            let mut group = c.benchmark_group("hylo-compare");

            for def in hylo_scenarios(Scale::from_env()) {
                let s = PreparedScenario::from_def(&def, "sm");
                let p = s.as_problem();

                let mut all = vec![
                    runners::rayon(&p),
                    baselines::hand_rayon(&s),
                    runners::sheque(&p, &es),
                ];
                all.extend(runners::funnel_runners(&p, &es));

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

criterion_group!(benches, bench_hylo_compare);
criterion_main!(benches);
