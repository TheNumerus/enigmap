#[macro_use]
extern crate criterion;

use criterion::Criterion;

use enigmap::{
    HexMap,
    generators::{MapGen, Inland, Islands, Circle}
};

macro_rules! map_bench {
    ($c:ident, $t:ty) => {
        let name = format!("map_{}_small", stringify!($t));
        $c.bench_function(&name, |b| {
            let mut map = HexMap::new(60, 40);
            let gen = <$t>::default();
            b.iter(|| gen.generate(&mut map))
        });
        let name = format!("map_{}_med", stringify!($t));
        $c.bench_function(&name, |b| {
            let mut map = HexMap::new(100, 75);
            let gen = <$t>::default();
            b.iter(|| gen.generate(&mut map))
        });
        let name = format!("map_{}_big", stringify!($t));
        $c.bench_function(&name, |b| {
            let mut map = HexMap::new(200, 150);
            let gen = <$t>::default();
            b.iter(|| gen.generate(&mut map))
        });
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    map_bench!(c, Circle);
    map_bench!(c, Islands);
    map_bench!(c, Inland);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);