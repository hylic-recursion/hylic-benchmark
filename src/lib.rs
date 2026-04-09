//! Benchmark suite and comparison executors for hylic.
//!
//! Provides executors used only for benchmarking:
//! - Rayon: parallel via rayon::par_iter
//! - Sequential: Vec-collect baseline
//! - HyloSheque: CPS zipper with reactive accumulation (original benchmark baseline)

pub mod executor;
