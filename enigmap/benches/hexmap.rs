#[macro_use]
extern crate criterion;

use criterion::Criterion;

use enigmap::HexMap;

fn gen(x: u32, y: u32) {
    let hexmap = HexMap::new(x, y);
    hexmap.get_closest_hex_index(98.4, 62.6);
}

fn hexmap(c: &mut Criterion) {
    c.bench_function("hex_distance", |b| b.iter(|| gen(100, 75)));
}

criterion_group!(benches, hexmap);
criterion_main!(benches);