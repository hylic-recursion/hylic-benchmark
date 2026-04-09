#![allow(dead_code)]

pub mod tree;
pub mod work;
pub mod problem;
pub mod executor_set;
pub mod scenario;
pub mod runners;
pub mod baselines;
pub mod module_sim;
pub mod config;

use criterion::{BenchmarkGroup, BenchmarkId, measurement::WallTime};

pub fn bench_cell<F: FnMut(&mut criterion::Bencher, &())>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    mode: &str,
    scenario: &str,
    f: F,
) {
    eprintln!("[bench] {mode}/{scenario}");
    group.bench_with_input(BenchmarkId::new(mode, scenario), &(), f);
}
