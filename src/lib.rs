//! Benchmark suite and comparison executors for hylic.
//!
//! Provides executors used only for benchmarking:
//! - Rayon: parallel via rayon::par_iter
//! - Sequential: Vec-collect baseline

pub mod executor;
