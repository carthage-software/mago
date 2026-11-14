#![allow(clippy::expect_used)]

use std::borrow::Cow;
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_twig_formatter::Formatter;
use mago_twig_formatter::settings::FormatSettings;

const TAGS: &[u8] = include_bytes!("../tests/cases/tags/before.twig");
const EXPRESSIONS: &[u8] = include_bytes!("../tests/cases/expressions/before.twig");
const COMMENTS: &[u8] = include_bytes!("../tests/cases/commented_expressions/before.twig");

fn bench_formatter(criterion: &mut Criterion) {
    let inline = b"{{ name }}".repeat(5_000);
    let comments = b"{# a comment #}".repeat(5_000);
    let html = b"<section>  text  </section>\n".repeat(20_000);
    let cases = [
        ("tags", TAGS),
        ("expressions", EXPRESSIONS),
        ("comments", COMMENTS),
        ("dense-inline", inline.as_slice()),
        ("dense-comments", comments.as_slice()),
        ("html-only", html.as_slice()),
    ];
    let mut group = criterion.benchmark_group("twig-formatter");
    for (name, source) in cases {
        let file = File::ephemeral(Cow::Borrowed(b"bench.twig"), Cow::Owned(source.to_vec()));
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(name, |bencher| {
            bencher.iter(|| {
                let arena = LocalArena::new();
                let formatter = Formatter::new(&arena, FormatSettings::default());
                black_box(formatter.format_file(black_box(&file)).expect("valid fixture").len())
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_formatter);
criterion_main!(benches);
