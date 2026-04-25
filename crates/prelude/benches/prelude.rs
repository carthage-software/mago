use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_prelude::Prelude;

fn bench_prelude_build(c: &mut Criterion) {
    c.bench_function("prelude_build", |b| {
        b.iter(Prelude::build);
    });
}

criterion_group!(benches, bench_prelude_build);
criterion_main!(benches);
