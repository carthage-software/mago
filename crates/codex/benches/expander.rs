use std::hint::black_box;
use std::sync::Arc;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;

fn make_self_object() -> TUnion {
    TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))))
}

fn make_named_object(name: &str) -> TUnion {
    TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom(name)))))
}

fn options_with_self(self_class: &str) -> TypeExpansionOptions {
    TypeExpansionOptions { self_class: Some(ascii_lowercase_atom(self_class)), ..Default::default() }
}

fn bench_non_expandable(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();
    let options = TypeExpansionOptions::default();

    c.bench_function("non_expandable_int", |b| {
        b.iter(|| {
            let mut t = get_int();
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });

    c.bench_function("non_expandable_string", |b| {
        b.iter(|| {
            let mut t = get_string();
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });
}

fn bench_simple_self_expansion(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();
    let options = options_with_self("Foo");

    c.bench_function("single_self_to_class", |b| {
        b.iter(|| {
            let mut t = make_self_object();
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });
}

fn bench_union_expansion(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();
    let options = options_with_self("Foo");

    c.bench_function("union_3_types_with_self", |b| {
        b.iter(|| {
            let mut t = TUnion::from_vec(vec![
                TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))),
                get_int().types[0].clone(),
                get_string().types[0].clone(),
            ]);
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });

    c.bench_function("union_5_types_with_self", |b| {
        b.iter(|| {
            let mut t = TUnion::from_vec(vec![
                TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))),
                get_int().types[0].clone(),
                get_string().types[0].clone(),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("bar")))),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("baz")))),
            ]);
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });
}

fn bench_nested_array(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();
    let options = options_with_self("Foo");

    c.bench_function("list_of_self", |b| {
        b.iter(|| {
            let list = TList::new(Arc::new(make_self_object()));
            let mut t = TUnion::from_atomic(TAtomic::Array(TArray::List(list)));
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });

    c.bench_function("nested_list_3_deep", |b| {
        b.iter(|| {
            let inner = TList::new(Arc::new(make_self_object()));
            let middle = TList::new(Arc::new(TUnion::from_atomic(TAtomic::Array(TArray::List(inner)))));
            let outer = TList::new(Arc::new(TUnion::from_atomic(TAtomic::Array(TArray::List(middle)))));
            let mut t = TUnion::from_atomic(TAtomic::Array(TArray::List(outer)));
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });
}

fn bench_generic_object(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();
    let options = options_with_self("Foo");

    c.bench_function("generic_object_1_param", |b| {
        b.iter(|| {
            let named = TNamedObject::new_with_type_parameters(
                ascii_lowercase_atom("container"),
                Some(vec![make_self_object()]),
            );
            let mut t = TUnion::from_atomic(TAtomic::Object(TObject::Named(named)));
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });

    c.bench_function("generic_object_3_params", |b| {
        b.iter(|| {
            let named = TNamedObject::new_with_type_parameters(
                ascii_lowercase_atom("container"),
                Some(vec![make_self_object(), get_int(), get_string()]),
            );
            let mut t = TUnion::from_atomic(TAtomic::Object(TObject::Named(named)));
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });
}

fn bench_already_expanded(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();
    let options = TypeExpansionOptions::default();

    c.bench_function("already_expanded_object", |b| {
        b.iter(|| {
            let mut t = make_named_object("Foo");
            expand_union(black_box(&codebase), black_box(&mut t), black_box(&options));
            t
        });
    });
}

criterion_group!(
    benches,
    bench_non_expandable,
    bench_simple_self_expansion,
    bench_union_expansion,
    bench_nested_array,
    bench_generic_object,
    bench_already_expanded,
);

criterion_main!(benches);
