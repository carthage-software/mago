use std::hint::black_box;

use bumpalo::Bump;
use criterion::Criterion;
use criterion::Throughput;
use criterion::criterion_group;
use criterion::criterion_main;
use mago_database::file::FileId;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::input::Input;
use mago_type_syntax::lexer::TypeLexer;
use mago_type_syntax::parse_str;

const SMALL_TYPES: &str = include_str!("fixtures/small.txt");
const MEDIUM_TYPES: &str = include_str!("fixtures/medium.txt");
const LARGE_TYPES: &str = include_str!("fixtures/large.txt");

fn benchmark_type_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("type-lexer");
    let file_id = FileId::new("bench.php");

    group.throughput(Throughput::Bytes(LARGE_TYPES.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| {
            let input = Input::new(file_id, black_box(LARGE_TYPES.as_bytes()));
            let mut lexer = TypeLexer::new(input);
            let mut count = 0usize;
            while let Some(result) = lexer.advance() {
                if result.is_ok() {
                    count += 1;
                }
            }
            black_box(count)
        })
    });

    group.finish();
}

fn benchmark_type_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("type-parser");
    let file_id = FileId::new("bench.php");

    for (name, content) in [("small", SMALL_TYPES), ("medium", MEDIUM_TYPES), ("large", LARGE_TYPES)] {
        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let arena = Bump::new();
                let mut success_count = 0usize;
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with("//") {
                        continue;
                    }
                    let span = Span::new(file_id, Position::new(0), Position::new(line.len() as u32));
                    if parse_str(&arena, span, black_box(line)).is_ok() {
                        success_count += 1;
                    }
                }
                black_box(success_count)
            })
        });
    }

    group.finish();
}

fn benchmark_single_complex_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("type-complex");
    let file_id = FileId::new("bench.php");

    let complex_type = "array{users: list<object{id: positive-int, name: non-empty-string, email?: string, roles: list<string>}>, pagination: object{page: int, per_page: int, total: int}, filters?: array<string, mixed>}";

    group.throughput(Throughput::Bytes(complex_type.len() as u64));
    group.bench_function("nested_array_shape", |b| {
        b.iter(|| {
            let arena = Bump::new();
            let span = Span::new(file_id, Position::new(0), Position::new(complex_type.len() as u32));
            let ok = parse_str(&arena, span, black_box(complex_type)).is_ok();
            black_box(ok)
        })
    });

    let closure_type = "Closure(array{id: int, name: string}, list<string>, ?object{active: bool}): array{success: bool, errors?: list<string>}";

    group.throughput(Throughput::Bytes(closure_type.len() as u64));
    group.bench_function("complex_closure", |b| {
        b.iter(|| {
            let arena = Bump::new();
            let span = Span::new(file_id, Position::new(0), Position::new(closure_type.len() as u32));
            let ok = parse_str(&arena, span, black_box(closure_type)).is_ok();
            black_box(ok)
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_type_lexer, benchmark_type_parser, benchmark_single_complex_type);
criterion_main!(benches);
