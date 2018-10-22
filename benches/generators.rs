#[macro_use]
extern crate criterion;

extern crate enigmap;

use criterion::Criterion;

use enigmap::{
    HexMap,
    generators::{MapGen, Islands}
};

fn gen(x: u32, y: u32) {
    let mut hexmap = HexMap::new(x, y);
    let gen = Islands::default();
    gen.generate(&mut hexmap);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("small_map", |b| b.iter(|| gen(60, 40)));
    c.bench_function("middle_map", |b| b.iter(|| gen(100, 75)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);