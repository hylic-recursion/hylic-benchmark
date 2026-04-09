//! HyloSheque: CPS zipper with reactive accumulation.
//! Original hylomorphic executor, retained as benchmark baseline.

pub(crate) mod fold_chain;
mod walk;

use std::sync::Arc;
use hylic::domain::Domain;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec, PoolExecView};
use hylic::cata::exec::{Executor, ExecutorSpec};

#[derive(Clone, Copy)]
pub struct Spec {
    pub n_workers: usize,
}

impl Spec {
    pub fn default(n_workers: usize) -> Self { Spec { n_workers } }
}

impl ExecutorSpec for Spec {
    type Resource<'r> = &'r Arc<WorkPool>;
    type Session<'s> = Session<'s>;

    fn attach(self, pool: Self::Resource<'_>) -> Session<'_> {
        Session { pool, spec: self }
    }

    fn with_session<R>(&self, f: impl for<'s> FnOnce(&Session<'s>) -> R) -> R {
        WorkPool::with(WorkPoolSpec::threads(self.n_workers), |pool| {
            f(&(*self).attach(pool))
        })
    }
}

impl<N, R, D: Domain<N>> Executor<N, R, D> for Spec
where N: Clone + Send + 'static, R: Clone + Send + 'static,
{
    fn run<H: 'static>(&self, fold: &D::Fold<H, R>, graph: &D::Treeish, root: &N) -> R {
        self.with_session(|session| Executor::<N, R, D>::run(session, fold, graph, root))
    }
}

pub struct Session<'s> {
    pool: &'s Arc<WorkPool>,
    #[allow(dead_code)]
    spec: Spec,
}

impl<N, R, D: Domain<N>> Executor<N, R, D> for Session<'_>
where N: Clone + Send + 'static, R: Clone + Send + 'static,
{
    fn run<H: 'static>(&self, fold: &D::Fold<H, R>, graph: &D::Treeish, root: &N) -> R {
        let view = PoolExecView::new(self.pool);
        walk::run_fold(fold, graph, root, &view)
    }
}
