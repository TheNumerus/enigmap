#[macro_use]
extern crate criterion;

use criterion::Criterion;

use enigmap::{
    HexMap,
    renderers::{Renderer, Basic},
    generators::{MapGen, Islands}
};

fn criterion_benchmark(c: &mut Criterion) {
    let mut hexmap = HexMap::new(100, 75);
    let gen = Islands::default();
    gen.generate(&mut hexmap);
    let mut ren = Basic::default();
    ren.set_scale(15.0);
    c.bench_function("render_basic", move |b| {
        b.iter(|| ren.render(&hexmap))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);