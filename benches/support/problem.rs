//! BenchProblem<N>: the generic fold problem that all hylic runners operate on.

use hylic::domain::shared as dom;

/// A prepared fold problem, generic over node type.
/// Hylic runners consume this uniformly — no node-type-specific code.
pub struct BenchProblem<N: 'static> {
    pub name: String,
    pub fold: dom::Fold<N, u64, u64>,
    pub treeish: hylic::graph::Treeish<N>,
    pub root: N,
    pub expected: u64,
}
