//! Sequential executor: collect children to Vec, iterate.
//! Supports ALL domains — it borrows fold/graph, never clones them.

use hylic::ops::{FoldOps, TreeOps};
use hylic::domain::Domain;
use hylic::cata::exec::{Executor, ExecutorSpec};

pub struct Spec;

impl Clone for Spec { fn clone(&self) -> Self { *self } }
impl Copy for Spec {}
impl std::fmt::Debug for Spec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "Sequential") }
}

impl ExecutorSpec for Spec {
    type Session<'s> = Self;
    fn with_session<R>(&self, f: impl for<'s> FnOnce(&Self) -> R) -> R { f(self) }
}

impl<N: Clone + 'static, R: 'static, D: Domain<N>> Executor<N, R, D> for Spec {
    fn run<H: 'static>(&self, fold: &D::Fold<H, R>, graph: &D::Treeish, root: &N) -> R {
        recurse(fold, graph, root)
    }
}

fn recurse<N: Clone, H, R>(
    fold: &impl FoldOps<N, H, R>,
    graph: &impl TreeOps<N>,
    node: &N,
) -> R {
    let mut heap = fold.init(node);
    for child in graph.apply(node) {
        fold.accumulate(&mut heap, &recurse(fold, graph, &child));
    }
    fold.finalize(&heap)
}
