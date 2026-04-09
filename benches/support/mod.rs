#![allow(dead_code)]

pub mod tree;
pub mod work;
pub mod scenario;
pub mod config;
pub mod runners;
pub mod module_sim;

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
