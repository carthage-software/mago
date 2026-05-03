#![allow(clippy::expect_used, clippy::default_constructed_unit_structs, clippy::missing_assert_message)]

use std::hint::black_box;

use bumpalo::Bump;
use criterion::Criterion;
use criterion::Throughput;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_database::file::FileId;
use mago_syntax_core::input::Input;
use mago_twig_syntax::lexer::TwigLexer;
use mago_twig_syntax::parser::parse_file_content;
use mago_twig_syntax::settings::LexerSettings;

const SMALL: &str = include_str!("fixtures/small.twig");
const MEDIUM: &str = include_str!("fixtures/medium.twig");
const LARGE: &str = include_str!("fixtures/large.twig");
const HUGE: &str = include_str!("fixtures/huge.twig");

const FIXTURES: &[(&str, &str)] = &[("small", SMALL), ("medium", MEDIUM), ("large", LARGE), ("huge", HUGE)];

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("twig-lexer");
    for &(name, src) in FIXTURES {
        group.throughput(Throughput::Bytes(src.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let input = Input::new(FileId::zero(), black_box(src).as_bytes());
                let mut lexer = TwigLexer::new(input, LexerSettings::default());
                let mut count = 0usize;
                while let Some(result) = lexer.advance() {
                    let _ = result.expect("lex ok");
                    count += 1;
                }
                black_box(count)
            });
        });
    }

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("twig-roundtrip");
    for &(name, src) in FIXTURES {
        group.throughput(Throughput::Bytes(src.len() as u64));
        group.bench_function(name, |b| {
            let mut out = String::with_capacity(src.len());
            b.iter(|| {
                out.clear();
                let input = Input::new(FileId::zero(), black_box(src).as_bytes());
                let mut lexer = TwigLexer::new(input, LexerSettings::default());
                while let Some(result) = lexer.advance() {
                    out.push_str(result.expect("lex ok").value);
                }
                black_box(out.len())
            });
        });
    }

    group.finish();
}

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("twig-parser");
    for &(name, src) in FIXTURES {
        group.throughput(Throughput::Bytes(src.len() as u64));
        group.bench_function(name, |b| {
            let mut arena = Bump::new();
            b.iter(|| {
                arena.reset();
                let tpl = parse_file_content(&arena, FileId::zero(), black_box(src));
                assert!(!tpl.has_errors());
                black_box(tpl.statements.len())
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_lexer, bench_roundtrip, bench_parser);
criterion_main!(benches);
