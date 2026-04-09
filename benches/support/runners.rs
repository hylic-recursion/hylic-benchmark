//! Benchmark runner constructors -- DRY infrastructure for all bench suites.
//!
//! Each runner is `(name, Box<dyn Fn() -> u64>)`. Hylic executors go through
//! `Exec<D, S>::run()`. Handrolled baselines are standalone recursive functions.
//! Both produce the same type -- the benchmark framework treats them uniformly.

use std::sync::Arc;
use std::hint::black_box;
use hylic::domain::shared as dom;
use hylic::cata::exec::funnel;
use hylic::cata::exec::funnel::policy;
use hylic_benchmark::executor::hylo_sheque;
use hylic_parallel_lifts::{WorkPool, ParLazy, ParEager, PoolExecView};

use super::scenario::PreparedScenario;
use super::tree::NodeId;
use super::work::{WorkSpec, busy_work, spin_wait_us};

// -- Runner type --

pub struct Runner<'a> {
    pub name: &'static str,
    pub run: Box<dyn Fn() -> u64 + 'a>,
}

// ANCHOR: funnel_specs
pub struct FunnelSpecs {
    pub pw_final:         funnel::Spec<policy::Default>,
    pub pw_arrive:        funnel::Spec<policy::PerWorkerArrival>,
    pub sh_final:         funnel::Spec<policy::SharedDefault>,
    pub sh_arrive:        funnel::Spec<policy::WideLight>,
    pub pw_final_batch:   funnel::Spec<policy::LowOverhead>,
    pub pw_final_k4:      funnel::Spec<policy::HighThroughput>,
    pub pw_final_k2:      funnel::Spec<policy::DeepNarrow>,
    pub sh_arrive_batch:  funnel::Spec<policy::StreamingWide>,
}

impl FunnelSpecs {
    pub fn new(nw: usize) -> Self {
        use funnel::queue::per_worker::PerWorkerSpec;
        use funnel::queue::shared::SharedSpec;
        use funnel::accumulate::on_arrival::OnArrivalSpec;
        use funnel::accumulate::on_finalize::OnFinalizeSpec;
        use funnel::wake::every_push::EveryPushSpec;

        FunnelSpecs {
            pw_final:        funnel::Spec::default(nw),
            pw_arrive:       funnel::Spec::<policy::PerWorkerArrival>::new(nw, PerWorkerSpec { deque_capacity: 4096 }, OnArrivalSpec, EveryPushSpec),
            sh_final:        funnel::Spec::<policy::SharedDefault>::new(nw, SharedSpec, OnFinalizeSpec, EveryPushSpec),
            sh_arrive:       funnel::Spec::for_wide_light(nw),
            pw_final_batch:  funnel::Spec::for_low_overhead(nw),
            pw_final_k4:     funnel::Spec::for_high_throughput(nw),
            pw_final_k2:     funnel::Spec::for_deep_narrow(nw),
            sh_arrive_batch: funnel::Spec::for_streaming_wide(nw),
        }
    }
}
// ANCHOR_END: funnel_specs

// -- Individual runner constructors --

pub fn fused<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "fused", run: Box::new(move || dom::FUSED.run(&s.fold, &s.treeish, &s.root)) }
}

pub fn sequential<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "sequential", run: Box::new(move || hylic_benchmark::statics::SEQUENTIAL.run(&s.fold, &s.treeish, &s.root)) }
}

pub fn rayon<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "rayon", run: Box::new(move || hylic_benchmark::statics::RAYON.run(&s.fold, &s.treeish, &s.root)) }
}

pub fn sheque<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>) -> Runner<'a> {
    let exec = dom::exec(hylo_sheque::Session::from_pool(wpool));
    Runner { name: "sheque", run: Box::new(move || exec.run(&s.fold, &s.treeish, &s.root)) }
}

pub fn funnel_variant<'a, P: funnel::policy::FunnelPolicy>(
    name: &'static str, s: &'a PreparedScenario,
    fpool: &'a funnel::Pool<'_>, spec: &'a funnel::Spec<P>,
) -> Runner<'a> {
    let exec = dom::exec(funnel::Session::from_pool(fpool, spec));
    Runner { name, run: Box::new(move || exec.run(&s.fold, &s.treeish, &s.root)) }
}

// -- Lift runners --

pub fn fused_par_lazy<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>) -> Runner<'a> {
    let lift = ParLazy::lift::<hylic::domain::Shared, NodeId, u64, u64>(wpool);
    Runner { name: "fused.par-lazy", run: Box::new(move || dom::FUSED.run_lifted(&lift, &s.fold, &s.treeish, &s.root)) }
}

pub fn rayon_par_lazy<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>) -> Runner<'a> {
    let lift = ParLazy::lift::<hylic::domain::Shared, NodeId, u64, u64>(wpool);
    Runner { name: "rayon.par-lazy", run: Box::new(move || hylic_benchmark::statics::RAYON.run_lifted(&lift, &s.fold, &s.treeish, &s.root)) }
}

pub fn fused_par_eager<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>, nw: usize) -> Runner<'a> {
    let lift = ParEager::lift::<hylic::domain::Shared, NodeId, u64, u64>(wpool, hylic_parallel_lifts::EagerSpec::default_for(nw));
    Runner { name: "fused.par-eager", run: Box::new(move || dom::FUSED.run_lifted(&lift, &s.fold, &s.treeish, &s.root)) }
}

pub fn rayon_par_eager<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>, nw: usize) -> Runner<'a> {
    let lift = ParEager::lift::<hylic::domain::Shared, NodeId, u64, u64>(wpool, hylic_parallel_lifts::EagerSpec::default_for(nw));
    Runner { name: "rayon.par-eager", run: Box::new(move || hylic_benchmark::statics::RAYON.run_lifted(&lift, &s.fold, &s.treeish, &s.root)) }
}

pub fn funnel_lo_par_lazy<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>, fpool: &'a funnel::Pool<'_>, lo_spec: &'a funnel::Spec<policy::LowOverhead>) -> Runner<'a> {
    let exec = dom::exec(funnel::Session::from_pool(fpool, lo_spec));
    let lift = ParLazy::lift(wpool);
    Runner { name: "funnel.lo.par-lazy", run: Box::new(move || exec.run_lifted(&lift, &s.fold, &s.treeish, &s.root)) }
}

pub fn funnel_lo_par_eager<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>, fpool: &'a funnel::Pool<'_>, lo_spec: &'a funnel::Spec<policy::LowOverhead>, nw: usize) -> Runner<'a> {
    let exec = dom::exec(funnel::Session::from_pool(fpool, lo_spec));
    let lift = ParEager::lift(wpool, hylic_parallel_lifts::EagerSpec::default_for(nw));
    Runner { name: "funnel.lo.par-eager", run: Box::new(move || exec.run_lifted(&lift, &s.fold, &s.treeish, &s.root)) }
}

// -- Grouped constructors --

pub fn sequential_runners<'a>(s: &'a PreparedScenario) -> Vec<Runner<'a>> {
    vec![fused(s), sequential(s), hand_seq(s), real_seq(s)]
}

pub fn parallel_runners<'a>(
    s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>, nw: usize,
) -> Vec<Runner<'a>> {
    vec![
        rayon(s),
        sheque(s, wpool),
        fused_par_lazy(s, wpool),
        rayon_par_lazy(s, wpool),
        fused_par_eager(s, wpool, nw),
        rayon_par_eager(s, wpool, nw),
    ]
}

pub fn funnel_runners<'a>(
    s: &'a PreparedScenario, fpool: &'a funnel::Pool<'_>, specs: &'a FunnelSpecs,
) -> Vec<Runner<'a>> {
    vec![
        funnel_variant("funnel.pw.fin",             s, fpool, &specs.pw_final),
        funnel_variant("funnel.pw.arrv",           s, fpool, &specs.pw_arrive),
        funnel_variant("funnel.sh.fin",             s, fpool, &specs.sh_final),
        funnel_variant("funnel.sh.arrv",            s, fpool, &specs.sh_arrive),
        funnel_variant("funnel.pw.fin.batch",      s, fpool, &specs.pw_final_batch),
        funnel_variant("funnel.pw.fin.k4",         s, fpool, &specs.pw_final_k4),
        funnel_variant("funnel.pw.fin.k2",         s, fpool, &specs.pw_final_k2),
        funnel_variant("funnel.sh.arrv.batch",     s, fpool, &specs.sh_arrive_batch),
    ]
}

pub fn baseline_runners<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>) -> Vec<Runner<'a>> {
    vec![hand_rayon(s), hand_pool(s, wpool), real_rayon(s)]
}

// -- Handrolled baselines --

pub fn hand_seq<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "hand.seq", run: Box::new(|| handrolled_seq(s)) }
}

pub fn hand_rayon<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "hand.rayon", run: Box::new(|| handrolled_rayon(s)) }
}

pub fn hand_pool<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>) -> Runner<'a> {
    let work = Arc::new(s.work.clone());
    let children = s.children.clone();
    let root = s.root;
    Runner { name: "hand.pool", run: Box::new(move || {
        handrolled_pool(&children, &work, wpool, root)
    })}
}

pub fn real_seq<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "real.seq", run: Box::new(|| realworld_seq(s)) }
}

pub fn real_rayon<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "real.rayon", run: Box::new(|| realworld_rayon(s)) }
}

// -- Handrolled implementations --

fn handrolled_seq(s: &PreparedScenario) -> u64 {
    fn recurse(children: &[Vec<NodeId>], work: &WorkSpec, node: NodeId) -> u64 {
        work.do_graph();
        let mut heap = work.do_init();
        for &child in &children[node] {
            work.do_accumulate(&mut heap, &recurse(children, work, child));
        }
        work.do_finalize(&heap)
    }
    recurse(&s.children, &s.work, s.root)
}

fn handrolled_rayon(s: &PreparedScenario) -> u64 {
    use rayon::prelude::*;
    fn recurse(children: &Arc<Vec<Vec<NodeId>>>, work: &WorkSpec, node: NodeId) -> u64 {
        work.do_graph();
        let mut heap = work.do_init();
        let ch = &children[node];
        if ch.len() <= 1 {
            for &child in ch { work.do_accumulate(&mut heap, &recurse(children, work, child)); }
        } else {
            let results: Vec<u64> = ch.par_iter().map(|&c| recurse(children, work, c)).collect();
            for r in &results { work.do_accumulate(&mut heap, r); }
        }
        work.do_finalize(&heap)
    }
    recurse(&s.children, &s.work, s.root)
}

fn handrolled_pool(children: &Arc<Vec<Vec<NodeId>>>, work: &Arc<WorkSpec>, pool: &Arc<WorkPool>, root: NodeId) -> u64 {
    let view = PoolExecView::new(pool);
    fn recurse(children: &[Vec<NodeId>], work: &WorkSpec, view: &PoolExecView, node: NodeId) -> u64 {
        work.do_graph();
        let mut heap = work.do_init();
        let ch = &children[node];
        if ch.len() <= 1 {
            for &child in ch { work.do_accumulate(&mut heap, &recurse(children, work, view, child)); }
        } else {
            let mid = ch.len() / 2;
            let (left, right) = view.join(
                || ch[..mid].iter().map(|&c| recurse(children, work, view, c)).collect::<Vec<_>>(),
                || ch[mid..].iter().map(|&c| recurse(children, work, view, c)).collect::<Vec<_>>(),
            );
            for r in left.iter().chain(right.iter()) { work.do_accumulate(&mut heap, r); }
        }
        work.do_finalize(&heap)
    }
    recurse(&children, &work, &view, root)
}

fn realworld_seq(s: &PreparedScenario) -> u64 {
    let (iw, aw, fw, gw, gio) = (s.work.init_work, s.work.accumulate_work, s.work.finalize_work, s.work.graph_work, s.work.graph_io_us);
    fn recurse(children: &[Vec<NodeId>], node: NodeId, iw: u64, aw: u64, fw: u64, gw: u64, gio: u64) -> u64 {
        spin_wait_us(gio);
        if gw > 0 { black_box(busy_work(gw)); }
        let mut result = if iw > 0 { busy_work(iw) } else { 0 };
        for &child in &children[node] {
            let cr = recurse(children, child, iw, aw, fw, gw, gio);
            if aw > 0 { result = result.wrapping_add(busy_work(aw)); }
            result = result.wrapping_add(cr);
        }
        if fw > 0 { result = result.wrapping_add(busy_work(fw)); }
        result
    }
    recurse(&s.children, s.root, iw, aw, fw, gw, gio)
}

fn realworld_rayon(s: &PreparedScenario) -> u64 {
    use rayon::prelude::*;
    let (iw, aw, fw, gw, gio) = (s.work.init_work, s.work.accumulate_work, s.work.finalize_work, s.work.graph_work, s.work.graph_io_us);
    fn recurse(children: &Arc<Vec<Vec<NodeId>>>, node: NodeId, iw: u64, aw: u64, fw: u64, gw: u64, gio: u64) -> u64 {
        spin_wait_us(gio);
        if gw > 0 { black_box(busy_work(gw)); }
        let mut result = if iw > 0 { busy_work(iw) } else { 0 };
        let ch = &children[node];
        if ch.len() <= 1 {
            for &child in ch {
                let cr = recurse(children, child, iw, aw, fw, gw, gio);
                if aw > 0 { result = result.wrapping_add(busy_work(aw)); }
                result = result.wrapping_add(cr);
            }
        } else {
            let results: Vec<u64> = ch.par_iter().map(|&c| recurse(children, c, iw, aw, fw, gw, gio)).collect();
            for r in &results {
                if aw > 0 { result = result.wrapping_add(busy_work(aw)); }
                result = result.wrapping_add(*r);
            }
        }
        if fw > 0 { result = result.wrapping_add(busy_work(fw)); }
        result
    }
    recurse(&s.children, s.root, iw, aw, fw, gw, gio)
}
