//! Benchmark runners — uniform constructors generic over node type.
//!
//! Every runner is `Runner<'a> = { name, Box<dyn Fn() -> u64> }`.
//! Hylic runners and handrolled baselines share the same type.
//! All hylic runners are generic over N — the same code serves
//! NodeId scenarios and String module-sim scenarios.

use hylic::domain::shared as dom;
use hylic::exec::funnel;
use hylic::exec::funnel::policy::FunnelPolicy;

use super::problem::BenchProblem;
use super::executor_set::ExecutorSet;

// ── Runner type ─────────────────────────────────────

pub struct Runner<'a> {
    pub name: &'static str,
    pub run: Box<dyn Fn() -> u64 + 'a>,
}

// ── Direct executors (generic over N) ───────────────

pub fn rayon<'a, N: Clone + Send + Sync + 'static>(p: &'a BenchProblem<N>) -> Runner<'a> {
    let exec = dom::exec(hylic_benchmark::executor::rayon::Spec);
    Runner { name: "rayon", run: Box::new(move || exec.run(&p.fold, &p.treeish, &p.root)) }
}

pub fn funnel_variant<'a, N: Clone + Send + 'static, P: FunnelPolicy>(
    name: &'static str, p: &'a BenchProblem<N>, es: &'a ExecutorSet, spec: funnel::Spec<P>,
) -> Runner<'a> {
    let exec = dom::exec(spec).attach(es.fpool);
    Runner { name, run: Box::new(move || exec.run(&p.fold, &p.treeish, &p.root)) }
}

// ── Grouped constructors ────────────────────────────

/// All 16 funnel policy variants: 4 queue×accumulate × 4 wake.
pub fn funnel_runners<'a, N: Clone + Send + 'static>(
    p: &'a BenchProblem<N>, es: &'a ExecutorSet,
) -> Vec<Runner<'a>> {
    let s = &es.funnel;
    vec![
        // PerWorker + OnArrival × wake
        funnel_variant("funnel.pw.arrv.push",     p, es, s.pw_arrv),
        funnel_variant("funnel.pw.arrv.batch",    p, es, s.pw_arrv_batch),
        funnel_variant("funnel.pw.arrv.k4",       p, es, s.pw_arrv_k4),
        funnel_variant("funnel.pw.arrv.k2",       p, es, s.pw_arrv_k2),
        // Shared + OnArrival × wake
        funnel_variant("funnel.sh.arrv.push",     p, es, s.sh_arrv),
        funnel_variant("funnel.sh.arrv.batch",    p, es, s.sh_arrv_batch),
        funnel_variant("funnel.sh.arrv.k4",       p, es, s.sh_arrv_k4),
        funnel_variant("funnel.sh.arrv.k2",       p, es, s.sh_arrv_k2),
        // PerWorker + OnFinalize × wake
        funnel_variant("funnel.pw.fin.push",      p, es, s.pw_fin),
        funnel_variant("funnel.pw.fin.batch",     p, es, s.pw_fin_batch),
        funnel_variant("funnel.pw.fin.k4",        p, es, s.pw_fin_k4),
        funnel_variant("funnel.pw.fin.k2",        p, es, s.pw_fin_k2),
        // Shared + OnFinalize × wake
        funnel_variant("funnel.sh.fin.push",      p, es, s.sh_fin),
        funnel_variant("funnel.sh.fin.batch",     p, es, s.sh_fin_batch),
        funnel_variant("funnel.sh.fin.k4",        p, es, s.sh_fin_k4),
        funnel_variant("funnel.sh.fin.k2",        p, es, s.sh_fin_k2),
    ]
}

/// All hylic runners: baselines + full funnel matrix.
pub fn all_hylic_runners<'a, N: Clone + Send + Sync + 'static>(
    p: &'a BenchProblem<N>, es: &'a ExecutorSet,
) -> Vec<Runner<'a>> {
    let mut v = vec![rayon(p)];
    v.extend(funnel_runners(p, es));
    v
}
