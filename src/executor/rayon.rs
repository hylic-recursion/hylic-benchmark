//! Rayon executor: parallel child recursion via rayon's par_iter.
//! Shared domain only — requires Sync on graph references.

use hylic::ops::{FoldOps, TreeOps};
use hylic::domain;
use hylic::cata::exec::{Executor, ExecutorSpec};

pub struct Spec;

impl Clone for Spec { fn clone(&self) -> Self { *self } }
impl Copy for Spec {}
impl std::fmt::Debug for Spec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "Rayon") }
}

impl ExecutorSpec for Spec {
    type Resource<'r> = ();
    type Session<'s> = Self;
    fn attach(self, _: Self::Resource<'_>) -> Self::Session<'_> { self }
    fn with_session<R>(&self, f: impl for<'s> FnOnce(&Self) -> R) -> R { f(self) }
}

impl<N, R, G> Executor<N, R, domain::Shared, G> for Spec
where
    N: Clone + Send + Sync + 'static,
    R: Send + Sync + 'static,
    G: TreeOps<N> + Sync + 'static,
{
    fn run<H: 'static>(
        &self,
        fold: &<domain::Shared as domain::Domain<N>>::Fold<H, R>,
        graph: &G,
        root: &N,
    ) -> R {
        recurse(fold, graph, root)
    }
}

fn recurse<N, H, R>(
    fold: &(impl FoldOps<N, H, R> + Sync),
    graph: &(impl TreeOps<N> + Sync),
    node: &N,
) -> R
where N: Clone + Send + Sync, R: Send + Sync,
{
    use rayon::prelude::*;

    let mut heap = fold.init(node);
    let children = graph.apply(node);

    if children.len() <= 1 {
        for child in &children {
            fold.accumulate(&mut heap, &recurse(fold, graph, child));
        }
    } else {
        let results: Vec<R> = children.par_iter()
            .map(|c| recurse(fold, graph, c))
            .collect();
        for r in &results { fold.accumulate(&mut heap, r); }
    }

    fold.finalize(&heap)
}
