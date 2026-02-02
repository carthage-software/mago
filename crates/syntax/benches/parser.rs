use std::hint::black_box;

use bumpalo::Bump;
use criterion::Criterion;
use criterion::Throughput;
use criterion::criterion_group;
use criterion::criterion_main;
use mago_database::file::FileId;
use mago_syntax::lexer::Lexer;
use mago_syntax::parser::parse_file_content;
use mago_syntax::settings::LexerSettings;
use mago_syntax_core::input::Input;

const SMALL_PHP: &str = r#"<?php

declare(strict_types=1);

function hello(): void {
    echo "Hello, World!";
}
"#;

const MEDIUM_PHP: &str = include_str!("fixtures/medium.php");
const LARGE_PHP: &str = include_str!("fixtures/large.php");

fn benchmark_mago_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("mago");

    group.throughput(Throughput::Bytes(SMALL_PHP.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| {
            let arena = Bump::new();
            let file_id = FileId::new("bench.php");
            let result = parse_file_content(&arena, file_id, black_box(SMALL_PHP));
            let _ = black_box(result.errors.len());
        })
    });

    group.throughput(Throughput::Bytes(MEDIUM_PHP.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| {
            let arena = Bump::new();
            let file_id = FileId::new("bench.php");
            let result = parse_file_content(&arena, file_id, black_box(MEDIUM_PHP));
            let _ = black_box(result.errors.len());
        })
    });

    group.throughput(Throughput::Bytes(LARGE_PHP.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| {
            let arena = Bump::new();
            let file_id = FileId::new("bench.php");
            let result = parse_file_content(&arena, file_id, black_box(LARGE_PHP));
            let _ = black_box(result.errors.len());
        })
    });

    group.finish();
}

fn benchmark_tree_sitter_php(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree-sitter-php");

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_php::LANGUAGE_PHP.into()).expect("Failed to load PHP grammar");

    group.throughput(Throughput::Bytes(SMALL_PHP.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| {
            let tree = parser.parse(black_box(SMALL_PHP), None).unwrap();
            let _ = black_box(tree.root_node().child_count());
        })
    });

    group.throughput(Throughput::Bytes(MEDIUM_PHP.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| {
            let tree = parser.parse(black_box(MEDIUM_PHP), None).unwrap();
            let _ = black_box(tree.root_node().child_count());
        })
    });

    group.throughput(Throughput::Bytes(LARGE_PHP.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| {
            let tree = parser.parse(black_box(LARGE_PHP), None).unwrap();
            let _ = black_box(tree.root_node().child_count());
        })
    });

    group.finish();
}

fn benchmark_mago_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("mago-lexer");

    group.throughput(Throughput::Bytes(LARGE_PHP.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| {
            let file_id = FileId::new("bench.php");
            let input = Input::new(file_id, black_box(LARGE_PHP.as_bytes()));
            let mut lexer = Lexer::new(input, LexerSettings::default());
            let mut count = 0usize;
            while let Some(result) = lexer.advance() {
                if result.is_ok() {
                    count += 1;
                }
            }
            let _ = black_box(count);
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_mago_parser, benchmark_mago_lexer, benchmark_tree_sitter_php);
criterion_main!(benches);
