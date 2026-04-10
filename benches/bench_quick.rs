//! Quick — WIP benchmark for tracking improvements.
//!
//! Subset: real.rayon baseline + 4 funnel variants (pw/sh × arrv/fin, all k4)
//! across 9 workload scenarios. Two Makefile variants:
//!   bench-quick-light  — 20 samples, 5s measure (fast iteration)
//!   bench-quick-heavy  — 150 samples, 40s measure (overnight)
//!
//! Reproduce: make bench-quick-light  (or bench-quick-heavy)

#[path = "support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use hylic::cata::exec::funnel;
use hylic::domain::shared as dom;
use support::scenario::{self, Scale, PreparedScenario};
use support::{baselines, bench_cell};
use support::runners::Runner;

fn bench_quick(c: &mut Criterion) {
    let nw = support::config::bench_workers();

    funnel::Pool::with(nw, |fpool| {
        // OnArrival × k4
        let pw_arrv_k4 = funnel::Spec::for_perworker_arrival(nw)
            .with_wake::<funnel::wake::EveryK<4>>(funnel::wake::every_k::EveryKSpec);
        let sh_arrv_k4 = funnel::Spec::for_wide_light(nw)
            .with_wake::<funnel::wake::EveryK<4>>(funnel::wake::every_k::EveryKSpec);
        // OnFinalize × k4
        let pw_fin_k4 = funnel::Spec::for_high_throughput(nw);
        let sh_fin_k4 = funnel::Spec::for_shared_default(nw)
            .with_wake::<funnel::wake::EveryK<4>>(funnel::wake::every_k::EveryKSpec);

        let mut group = c.benchmark_group("quick");

        for def in scenario::quick_scenarios(Scale::from_env()) {
            let s = PreparedScenario::from_def(&def, "sm");
            let p = s.as_problem();

            let baseline = baselines::real_rayon(&s);

            let ses_pw_arrv = dom::exec(pw_arrv_k4).attach(fpool);
            let ses_sh_arrv = dom::exec(sh_arrv_k4).attach(fpool);
            let ses_pw_fin = dom::exec(pw_fin_k4).attach(fpool);
            let ses_sh_fin = dom::exec(sh_fin_k4).attach(fpool);

            let runners: Vec<Runner> = vec![
                baseline,
                Runner { name: "funnel.pw.arrv.k4", run: Box::new(|| ses_pw_arrv.run(&p.fold, &p.treeish, &p.root)) },
                Runner { name: "funnel.sh.arrv.k4", run: Box::new(|| ses_sh_arrv.run(&p.fold, &p.treeish, &p.root)) },
                Runner { name: "funnel.pw.fin.k4",  run: Box::new(|| ses_pw_fin.run(&p.fold, &p.treeish, &p.root)) },
                Runner { name: "funnel.sh.fin.k4",  run: Box::new(|| ses_sh_fin.run(&p.fold, &p.treeish, &p.root)) },
            ];

            for r in &runners {
                bench_cell(&mut group, r.name, &p.name,
                    |b, _| b.iter(|| black_box((r.run)())),
                );
            }
        }

        group.finish();
    });
}

criterion_group!(benches, bench_quick);
criterion_main!(benches);
