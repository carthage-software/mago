mod combiner_common;

use combiner_common::*;

use mago_codex::ttype::union::TUnion;

#[test]
fn object_any_idempotent() {
    for n in 1..=10 {
        assert_self_idempotent(t_object_any(), n);
    }
}

#[test]
fn named_object_idempotent() {
    for name in ["Foo", "Bar\\Baz", "Vendor\\Pkg\\Class"] {
        for n in 1..=8 {
            assert_self_idempotent(t_named(name), n);
        }
    }
}

#[test]
fn enum_idempotent() {
    for name in ["E", "MyEnum", "App\\Status"] {
        for n in 1..=8 {
            assert_self_idempotent(t_enum(name), n);
        }
    }
}

#[test]
fn enum_case_idempotent() {
    for (name, case) in [("E", "A"), ("Status", "Active"), ("Color", "Red")] {
        for n in 1..=8 {
            assert_self_idempotent(t_enum_case(name, case), n);
        }
    }
}

#[test]
fn object_any_absorbs_named() {
    for name in ["Foo", "Bar", "X"] {
        assert_combines_to(vec![t_object_any(), t_named(name)], vec![t_object_any()]);
        assert_combines_to(vec![t_named(name), t_object_any()], vec![t_object_any()]);
    }
}

#[test]
fn object_any_absorbs_many_nameds() {
    let names = ["A", "B", "C", "D", "E"];
    let mut inputs = vec![t_object_any()];
    for n in names {
        inputs.push(t_named(n));
    }
    assert_combines_to(inputs, vec![t_object_any()]);
}

#[test]
fn many_object_any_collapse() {
    for n in 1..=10 {
        assert_combines_to(vec![t_object_any(); n], vec![t_object_any()]);
    }
}

// Named objects: same name collapses, different names kept apart (without
// codebase hierarchy info)

#[test]
fn same_named_collapses() {
    for name in ["Foo", "App\\Bar"] {
        for n in 1..=8 {
            assert_combines_to(vec![t_named(name); n], vec![t_named(name)]);
        }
    }
}

#[test]
fn distinct_named_kept_apart() {
    let r = combine_default(vec![t_named("Foo"), t_named("Bar")]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["Bar", "Foo"]);
}

#[test]
fn many_distinct_named_kept_apart() {
    let names = ["A", "B", "C", "D", "E", "F", "G", "H"];
    let inputs: Vec<_> = names.iter().map(|n| t_named(n)).collect();
    let r = combine_default(inputs);
    assert_eq!(r.len(), names.len());
}

#[test]
fn same_generic_same_params_collapses() {
    let a = t_generic_named("Container", vec![TUnion::from_atomic(t_int())]);
    let b = t_generic_named("Container", vec![TUnion::from_atomic(t_int())]);
    let r = combine_default(vec![a, b]);
    assert_eq!(r.len(), 1);
}

#[test]
fn same_generic_different_params_combine_params() {
    let a = t_generic_named("Container", vec![TUnion::from_atomic(t_int())]);
    let b = t_generic_named("Container", vec![TUnion::from_atomic(t_string())]);
    let r = combine_default(vec![a, b]);
    assert_eq!(r.len(), 1);
    let id = atomic_id_string(&r[0]);
    assert!(id.contains("Container"), "expected Container, got {id}");
}

#[test]
fn different_generic_kept_apart() {
    let a = t_generic_named("Container", vec![TUnion::from_atomic(t_int())]);
    let b = t_generic_named("Bag", vec![TUnion::from_atomic(t_int())]);
    let r = combine_default(vec![a, b]);
    assert_eq!(r.len(), 2);
}

#[test]
fn generic_int_or_lit_int_absorbs() {
    let a = t_generic_named("Container", vec![TUnion::from_atomic(t_int())]);
    let b = t_generic_named("Container", vec![TUnion::from_atomic(t_lit_int(5))]);
    let r = combine_default(vec![a, b]);
    assert_eq!(r.len(), 1);
}

#[test]
fn generic_with_many_distinct_params_combine() {
    let inputs: Vec<_> =
        (0..10i64).map(|i| t_generic_named("Container", vec![TUnion::from_atomic(t_lit_int(i))])).collect();
    let r = combine_default(inputs);
    assert_eq!(r.len(), 1);
}

#[test]
fn distinct_enums_kept_apart() {
    let r = combine_default(vec![t_enum("E"), t_enum("F")]);
    assert_eq!(r.len(), 2);
}

#[test]
fn same_enum_collapses() {
    assert_combines_to(vec![t_enum("E"); 5], vec![t_enum("E")]);
}

#[test]
fn distinct_enum_cases_same_enum_kept_apart() {
    let r = combine_default(vec![t_enum_case("E", "A"), t_enum_case("E", "B")]);
    assert_eq!(r.len(), 2);
}

#[test]
fn same_enum_case_collapses() {
    assert_combines_to(vec![t_enum_case("E", "A"); 5], vec![t_enum_case("E", "A")]);
}

#[test]
fn enum_and_case_kept_separate_but_collapse_via_combiner() {
    let r = combine_default(vec![t_enum("E"), t_enum_case("E", "A")]);
    assert_eq!(r.len(), 2);
}

#[test]
fn enum_or_named_kept_apart() {
    let r = combine_default(vec![t_enum("E"), t_named("Foo")]);
    assert_eq!(r.len(), 2);
}

#[test]
fn object_or_int_kept_separate() {
    let r = combine_default(vec![t_object_any(), t_int()]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["int", "object"]);
}

#[test]
fn named_or_string_kept_separate() {
    let r = combine_default(vec![t_named("Foo"), t_string()]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["Foo", "string"]);
}

#[test]
fn many_objects_with_int_kept_separate() {
    let r = combine_default(vec![t_named("A"), t_named("B"), t_named("C"), t_int()]);
    assert_eq!(r.len(), 4);
}

#[test]
fn object_dominated_by_mixed() {
    assert_combines_to(vec![t_object_any(), mixed()], vec![mixed()]);
    assert_combines_to(vec![mixed(), t_object_any()], vec![mixed()]);
    assert_combines_to(vec![t_named("Foo"), mixed()], vec![mixed()]);
}
