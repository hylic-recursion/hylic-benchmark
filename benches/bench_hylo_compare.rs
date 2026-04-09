//! Funnel policy matrix vs hylomorphic and rayon baselines.
//!
//! 8 funnel policy variants × 6 tuned scenarios. Each variant uses
//! Pool::with for realistic pool-reuse measurement. Per-fold memory
//! (arenas, stores) is fresh each iteration — only threads reused.
//!
//! This is the benchmark used by the tier1 isolation experiment
//! (run-all.sh) for cross-variant comparison.

#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use hylic::cata::exec::funnel;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec};

use support::scenario::{Scale, PreparedScenario, ScenarioDef};
use support::tree::TreeSpec;
use support::work::WorkSpec;
use support::config;
use support::runners::{self, FunnelSpecs};
use support::bench_cell;

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
    let nw = config::bench_workers();
    let funnel_specs = FunnelSpecs::new(nw);
    eprintln!("[hylo-compare] using {nw} worker threads");

    WorkPool::with(WorkPoolSpec::threads(nw), |wpool| {
        funnel::Pool::with(nw, |fpool| {
            let mut group = c.benchmark_group("hylo-compare");

            for def in hylo_scenarios(Scale::from_env()) {
                let s = PreparedScenario::from_def(&def, "sm");

                let mut all = vec![
                    runners::rayon(&s),
                    runners::hand_rayon(&s),
                    runners::sheque(&s, wpool),
                ];
                all.extend(runners::funnel_runners(&s, fpool, &funnel_specs));

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

criterion_group!(benches, bench_hylo_compare);
criterion_main!(benches);
