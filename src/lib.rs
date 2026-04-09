//! Benchmark suite and comparison executors for hylic.
//!
//! Provides executors used only for benchmarking:
//! - Rayon: parallel via rayon::par_iter
//! - Sequential: Vec-collect baseline
//! - HyloSheque: CPS zipper with reactive accumulation (original benchmark baseline)

pub mod executor;

/// Domain-bound executor constants for benchmarking.
/// Reconstructed here since hylic::domain::shared only exports FUSED.
pub mod statics {
    use hylic::cata::exec::Exec;
    use hylic::domain;

    pub const SEQUENTIAL: Exec<domain::Shared, crate::executor::sequential::Spec> =
        Exec::new(crate::executor::sequential::Spec);
    pub const RAYON: Exec<domain::Shared, crate::executor::rayon::Spec> =
        Exec::new(crate::executor::rayon::Spec);
}
