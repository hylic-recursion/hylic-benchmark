//! Benchmark runners — uniform constructors generic over node type.
//!
//! Every runner is `Runner<'a> = { name, Box<dyn Fn() -> u64> }`.
//! Hylic runners and handrolled baselines share the same type.
//! All hylic runners are generic over N — the same code serves
//! NodeId scenarios and String module-sim scenarios.

use hylic::domain::shared as dom;
use hylic::cata::exec::funnel;
use hylic::cata::exec::funnel::policy::FunnelPolicy;
use hylic_parallel_lifts::{ParLazy, ParEager};

use super::problem::BenchProblem;
use super::executor_set::ExecutorSet;

// ── Runner type ─────────────────────────────────────

pub struct Runner<'a> {
    pub name: &'static str,
    pub run: Box<dyn Fn() -> u64 + 'a>,
}

// ── Direct executors (generic over N) ───────────────

pub fn fused<'a, N: Clone + 'static>(p: &'a BenchProblem<N>) -> Runner<'a> {
    Runner { name: "fused", run: Box::new(|| dom::FUSED.run(&p.fold, &p.treeish, &p.root)) }
}

pub fn sequential<'a, N: Clone + 'static>(p: &'a BenchProblem<N>) -> Runner<'a> {
    let exec = dom::exec(hylic_benchmark::executor::sequential::Spec);
    Runner { name: "sequential", run: Box::new(move || exec.run(&p.fold, &p.treeish, &p.root)) }
}

pub fn rayon<'a, N: Clone + Send + Sync + 'static>(p: &'a BenchProblem<N>) -> Runner<'a> {
    let exec = dom::exec(hylic_benchmark::executor::rayon::Spec);
    Runner { name: "rayon", run: Box::new(move || exec.run(&p.fold, &p.treeish, &p.root)) }
}

pub fn sheque<'a, N: Clone + Send + 'static>(p: &'a BenchProblem<N>, es: &'a ExecutorSet) -> Runner<'a> {
    let exec = dom::exec(es.sheque).attach(es.wpool);
    Runner { name: "sheque", run: Box::new(move || exec.run(&p.fold, &p.treeish, &p.root)) }
}

pub fn funnel_variant<'a, N: Clone + Send + 'static, P: FunnelPolicy>(
    name: &'static str, p: &'a BenchProblem<N>, es: &'a ExecutorSet, spec: funnel::Spec<P>,
) -> Runner<'a> {
    let exec = dom::exec(spec).attach(es.fpool);
    Runner { name, run: Box::new(move || exec.run(&p.fold, &p.treeish, &p.root)) }
}

// ── Lifted executors (generic over N) ───────────────

pub fn fused_par_lazy<'a, N: Clone + Send + Sync + 'static>(p: &'a BenchProblem<N>, es: &'a ExecutorSet) -> Runner<'a> {
    let lift = ParLazy::lift::<hylic::domain::Shared, N, u64, u64>(es.wpool);
    Runner { name: "fused.par-lazy", run: Box::new(move || dom::FUSED.run_lifted(&lift, &p.fold, &p.treeish, &p.root)) }
}

pub fn rayon_par_lazy<'a, N: Clone + Send + Sync + 'static>(p: &'a BenchProblem<N>, es: &'a ExecutorSet) -> Runner<'a> {
    let lift = ParLazy::lift::<hylic::domain::Shared, N, u64, u64>(es.wpool);
    let exec = dom::exec(hylic_benchmark::executor::rayon::Spec);
    Runner { name: "rayon.par-lazy", run: Box::new(move || exec.run_lifted(&lift, &p.fold, &p.treeish, &p.root)) }
}

pub fn fused_par_eager<'a, N: Clone + Send + Sync + 'static>(p: &'a BenchProblem<N>, es: &'a ExecutorSet) -> Runner<'a> {
    let lift = ParEager::lift::<hylic::domain::Shared, N, u64, u64>(es.wpool, hylic_parallel_lifts::EagerSpec::default_for(es.nw));
    Runner { name: "fused.par-eager", run: Box::new(move || dom::FUSED.run_lifted(&lift, &p.fold, &p.treeish, &p.root)) }
}

pub fn rayon_par_eager<'a, N: Clone + Send + Sync + 'static>(p: &'a BenchProblem<N>, es: &'a ExecutorSet) -> Runner<'a> {
    let lift = ParEager::lift::<hylic::domain::Shared, N, u64, u64>(es.wpool, hylic_parallel_lifts::EagerSpec::default_for(es.nw));
    let exec = dom::exec(hylic_benchmark::executor::rayon::Spec);
    Runner { name: "rayon.par-eager", run: Box::new(move || exec.run_lifted(&lift, &p.fold, &p.treeish, &p.root)) }
}

pub fn funnel_lo_par_lazy<'a, N: Clone + Send + 'static>(p: &'a BenchProblem<N>, es: &'a ExecutorSet) -> Runner<'a> {
    let exec = dom::exec(es.funnel.lo).attach(es.fpool);
    let lift = ParLazy::lift(es.wpool);
    Runner { name: "funnel.lo.par-lazy", run: Box::new(move || exec.run_lifted(&lift, &p.fold, &p.treeish, &p.root)) }
}

pub fn funnel_lo_par_eager<'a, N: Clone + Send + 'static>(p: &'a BenchProblem<N>, es: &'a ExecutorSet) -> Runner<'a> {
    let exec = dom::exec(es.funnel.lo).attach(es.fpool);
    let lift = ParEager::lift(es.wpool, hylic_parallel_lifts::EagerSpec::default_for(es.nw));
    Runner { name: "funnel.lo.par-eager", run: Box::new(move || exec.run_lifted(&lift, &p.fold, &p.treeish, &p.root)) }
}

// ── Grouped constructors ────────────────────────────

/// All 8 funnel policy variants.
pub fn funnel_runners<'a, N: Clone + Send + 'static>(
    p: &'a BenchProblem<N>, es: &'a ExecutorSet,
) -> Vec<Runner<'a>> {
    let s = &es.funnel;
    vec![
        funnel_variant("funnel.pw.fin",         p, es, s.pw_final),
        funnel_variant("funnel.pw.arrv",        p, es, s.pw_arrive),
        funnel_variant("funnel.sh.fin",         p, es, s.sh_final),
        funnel_variant("funnel.sh.arrv",        p, es, s.sh_arrive),
        funnel_variant("funnel.pw.fin.batch",   p, es, s.pw_final_batch),
        funnel_variant("funnel.pw.fin.k4",      p, es, s.pw_final_k4),
        funnel_variant("funnel.pw.fin.k2",      p, es, s.pw_final_k2),
        funnel_variant("funnel.sh.arrv.batch",  p, es, s.sh_arrive_batch),
    ]
}

/// All lifted variants.
pub fn lift_runners<'a, N: Clone + Send + Sync + 'static>(
    p: &'a BenchProblem<N>, es: &'a ExecutorSet,
) -> Vec<Runner<'a>> {
    vec![
        fused_par_lazy(p, es),
        rayon_par_lazy(p, es),
        fused_par_eager(p, es),
        rayon_par_eager(p, es),
        funnel_lo_par_lazy(p, es),
        funnel_lo_par_eager(p, es),
    ]
}

/// Sequential-only runners (no parallelism, no ExecutorSet needed).
pub fn sequential_runners<'a, N: Clone + 'static>(p: &'a BenchProblem<N>) -> Vec<Runner<'a>> {
    vec![fused(p), sequential(p)]
}

/// All hylic runners: direct + funnel variants + lifts.
pub fn all_hylic_runners<'a, N: Clone + Send + Sync + 'static>(
    p: &'a BenchProblem<N>, es: &'a ExecutorSet,
) -> Vec<Runner<'a>> {
    let mut v = vec![
        fused(p),
        sequential(p),
        rayon(p),
        sheque(p, es),
    ];
    v.extend(funnel_runners(p, es));
    v.extend(lift_runners(p, es));
    v
}
