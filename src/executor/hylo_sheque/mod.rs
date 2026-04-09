//! HyloSheque: CPS zipper with reactive accumulation.
//! Original hylomorphic executor, retained as benchmark baseline.

pub(crate) mod fold_chain;
mod walk;

use std::sync::Arc;
use hylic::domain::Domain;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec, PoolExecView};
use hylic::cata::exec::{Executor, ExecutorSpec};

pub struct Spec {
    pub n_workers: usize,
}

impl Spec {
    pub fn default(n_workers: usize) -> Self { Spec { n_workers } }
}

impl ExecutorSpec for Spec {
    type Session<'s> = Session<'s>;
    fn with_session<R>(&self, f: impl for<'s> FnOnce(&Session<'s>) -> R) -> R {
        WorkPool::with(WorkPoolSpec::threads(self.n_workers), |pool| {
            f(&Session { pool })
        })
    }
}

pub struct Session<'s> {
    pool: &'s Arc<WorkPool>,
}

impl Session<'_> {
    pub fn from_pool<'s>(pool: &'s Arc<WorkPool>) -> Session<'s> {
        Session { pool }
    }
    pub fn pool(&self) -> &Arc<WorkPool> { self.pool }
}

impl<N, R, D: Domain<N>> Executor<N, R, D> for Session<'_>
where N: Clone + Send + 'static, R: Clone + Send + 'static,
{
    fn run<H: 'static>(&self, fold: &D::Fold<H, R>, graph: &D::Treeish, root: &N) -> R {
        let view = PoolExecView::new(self.pool);
        walk::run_fold(fold, graph, root, &view)
    }
}
