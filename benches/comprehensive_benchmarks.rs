//! Comprehensive benchmark placeholder (disabled aggregator).
use criterion::{criterion_group, criterion_main, Criterion};

fn noop(_c: &mut Criterion) {}

criterion_group!(benches, noop);
criterion_main!(benches);
#![allow(unused_imports, unused_must_use, unused_variables, unused_mut)]
