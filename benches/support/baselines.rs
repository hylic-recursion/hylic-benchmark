//! Handrolled baselines — problem-specific, not generic over N.
//!
//! These bypass the hylic fold/treeish abstraction entirely,
//! operating on raw adjacency lists or domain-specific data.
//! They exist to measure framework overhead.

use super::runners::Runner;
use super::problem::BenchProblem;
use super::tree::NodeId;
use super::work::WorkSpec;
use super::scenario::PreparedScenario;

// ── Fused executor (hylic sequential baseline) ──────

pub fn fused<'a, N: Clone + 'static>(p: &'a BenchProblem<N>) -> Runner<'a> {
    use hylic::domain::shared as dom;
    Runner { name: "fused", run: Box::new(|| dom::FUSED.run(&p.fold, &p.treeish, &p.root)) }
}

// ── Handrolled baselines (no hylic, raw recursion) ──

pub fn hand_seq<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "hand.seq", run: Box::new(|| handrolled_seq(s)) }
}

pub fn hand_rayon<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "hand.rayon", run: Box::new(|| handrolled_rayon(s)) }
}

pub fn real_seq<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "real.seq", run: Box::new(|| realworld_seq(s)) }
}

pub fn real_rayon<'a>(s: &'a PreparedScenario) -> Runner<'a> {
    Runner { name: "real.rayon", run: Box::new(|| realworld_rayon(s)) }
}

/// Parallel handrolled baselines for the matrix benchmark.
/// Sequential baselines (hand.seq, real.seq) are in the overhead suite only.
pub fn hand_baselines<'a>(s: &'a PreparedScenario) -> Vec<Runner<'a>> {
    vec![hand_rayon(s), real_rayon(s)]
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
