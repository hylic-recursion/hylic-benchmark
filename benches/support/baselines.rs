//! Handrolled baselines — problem-specific, not generic over N.
//!
//! These bypass the hylic fold/treeish abstraction entirely,
//! operating on raw adjacency lists or domain-specific data.
//! They exist to measure framework overhead.

use std::sync::Arc;
use hylic_parallel_lifts::{WorkPool, fork_join_map, SyncRef};

use super::runners::Runner;
use super::tree::NodeId;
use super::work::WorkSpec;
use super::scenario::PreparedScenario;

// ── NodeId baselines (for scenario-based benchmarks) ──

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

/// All handrolled baselines for NodeId scenarios.
pub fn hand_baselines<'a>(s: &'a PreparedScenario, wpool: &'a Arc<WorkPool>) -> Vec<Runner<'a>> {
    vec![hand_seq(s), hand_rayon(s), hand_pool(s, wpool), real_seq(s), real_rayon(s)]
}

// ── Implementations ─────────────────────────────────

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
    fn recurse(children: &[Vec<NodeId>], work: &WorkSpec, node: NodeId) -> u64 {
        work.do_graph();
        let mut heap = work.do_init();
        let ch = &children[node];
        if ch.len() <= 1 {
            for &child in ch {
                work.do_accumulate(&mut heap, &recurse(children, work, child));
            }
        } else {
            let results: Vec<u64> = ch.par_iter()
                .map(|&c| recurse(children, work, c))
                .collect();
            for r in &results { work.do_accumulate(&mut heap, r); }
        }
        work.do_finalize(&heap)
    }
    recurse(&s.children, &s.work, s.root)
}

fn handrolled_pool(
    children: &Arc<Vec<Vec<NodeId>>>,
    work: &Arc<WorkSpec>,
    wpool: &Arc<WorkPool>,
    root: NodeId,
) -> u64 {
    use hylic_parallel_lifts::PoolExecView;
    fn recurse(children: &SyncRef<'_, Vec<Vec<NodeId>>>, work: &SyncRef<'_, WorkSpec>, view: &PoolExecView, node: NodeId) -> u64 {
        work.do_graph();
        let mut heap = work.do_init();
        let ch = &children[node];
        if ch.len() >= 2 {
            let results = fork_join_map(view, ch, &|&c| recurse(children, work, view, c), 0, 8);
            for r in &results { work.do_accumulate(&mut heap, r); }
        } else {
            for &child in ch {
                work.do_accumulate(&mut heap, &recurse(children, work, view, child));
            }
        }
        work.do_finalize(&heap)
    }
    let view = PoolExecView::new(wpool);
    recurse(&SyncRef(children.as_ref()), &SyncRef(work.as_ref()), &view, root)
}

fn realworld_seq(s: &PreparedScenario) -> u64 {
    use std::collections::HashMap;
    fn resolve(children: &[Vec<NodeId>], work: &WorkSpec, node: NodeId, cache: &mut HashMap<NodeId, u64>) -> u64 {
        if let Some(&v) = cache.get(&node) { return v; }
        work.do_graph();
        let mut heap = work.do_init();
        for &child in &children[node] {
            work.do_accumulate(&mut heap, &resolve(children, work, child, cache));
        }
        let result = work.do_finalize(&heap);
        cache.insert(node, result);
        result
    }
    let mut cache = HashMap::new();
    resolve(&s.children, &s.work, s.root, &mut cache)
}

fn realworld_rayon(s: &PreparedScenario) -> u64 {
    use rayon::prelude::*;
    fn resolve(children: &[Vec<NodeId>], work: &WorkSpec, node: NodeId) -> u64 {
        work.do_graph();
        let mut heap = work.do_init();
        let ch = &children[node];
        if ch.len() <= 1 {
            for &child in ch {
                work.do_accumulate(&mut heap, &resolve(children, work, child));
            }
        } else {
            let results: Vec<u64> = ch.par_iter()
                .map(|&c| resolve(children, work, c))
                .collect();
            for r in &results { work.do_accumulate(&mut heap, r); }
        }
        work.do_finalize(&heap)
    }
    resolve(&s.children, &s.work, s.root)
}
