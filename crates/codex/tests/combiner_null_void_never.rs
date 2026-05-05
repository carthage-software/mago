mod combiner_common;

use combiner_common::*;

#[test]
fn null_idempotent() {
    for n in 1..=10 {
        assert_self_idempotent(null(), n);
    }
}

#[test]
fn void_idempotent() {
    for n in 1..=10 {
        let r = combine_default(vec![void(); n]);
        assert_eq!(r.len(), 1);
        assert_eq!(atomic_id_string(&r[0]), "void");
    }
}

#[test]
fn never_idempotent() {
    for n in 1..=10 {
        assert_self_idempotent(never(), n);
    }
}

#[test]
fn void_or_null_yields_null() {
    assert_combines_to(vec![void(), null()], vec![null()]);
    assert_combines_to(vec![null(), void()], vec![null()]);
}

#[test]
fn void_with_int_yields_int_or_null() {
    assert_combines_to(vec![void(), t_int()], vec![t_int(), null()]);
    assert_combines_to(vec![t_int(), void()], vec![t_int(), null()]);
}

#[test]
fn void_with_string_yields_string_or_null() {
    assert_combines_to(vec![void(), t_string()], vec![t_string(), null()]);
    assert_combines_to(vec![t_string(), void()], vec![t_string(), null()]);
}

#[test]
fn void_with_bool_yields_bool_or_null() {
    assert_combines_to(vec![void(), t_bool()], vec![t_bool(), null()]);
    assert_combines_to(vec![t_bool(), void()], vec![t_bool(), null()]);
}

#[test]
fn void_with_object_yields_object_or_null() {
    assert_combines_to(vec![void(), t_object_any()], vec![t_object_any(), null()]);
    assert_combines_to(vec![void(), t_named("Foo")], vec![t_named("Foo"), null()]);
}

#[test]
fn void_with_resource_yields_resource_or_null() {
    assert_combines_to(vec![void(), t_resource()], vec![t_resource(), null()]);
    assert_combines_to(vec![void(), t_open_resource()], vec![t_open_resource(), null()]);
}

#[test]
fn void_or_never_yields_null() {
    assert_combines_to(vec![void(), never()], vec![null()]);
    assert_combines_to(vec![never(), void()], vec![null()]);
}

#[test]
fn void_with_two_other_types_becomes_null() {
    let r = combine_default(vec![void(), t_int(), t_string()]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["int", "null", "string"]);
}

#[test]
fn null_or_int_kept_separate() {
    let r = combine_default(vec![null(), t_int()]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["int", "null"]);
}

#[test]
fn null_or_string_kept_separate() {
    let r = combine_default(vec![null(), t_string()]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["null", "string"]);
}

#[test]
fn null_or_bool_kept_separate() {
    let r = combine_default(vec![null(), t_bool()]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["bool", "null"]);
}

#[test]
fn null_or_object_kept_separate() {
    let r = combine_default(vec![null(), t_object_any()]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["null", "object"]);
}

#[test]
fn null_absorbs_never() {
    assert_combines_to(vec![null(), never()], vec![null()]);
    assert_combines_to(vec![never(), null()], vec![null()]);
}

#[test]
fn never_dropped_with_int() {
    assert_combines_to(vec![never(), t_int()], vec![t_int()]);
    assert_combines_to(vec![t_int(), never()], vec![t_int()]);
}

#[test]
fn never_dropped_with_string() {
    assert_combines_to(vec![never(), t_string()], vec![t_string()]);
}

#[test]
fn never_dropped_with_array() {
    assert_combines_to(vec![never(), t_empty_array()], vec![t_empty_array()]);
}

#[test]
fn never_dropped_with_object() {
    assert_combines_to(vec![never(), t_named("X")], vec![t_named("X")]);
}

#[test]
fn never_dropped_with_three_atoms() {
    let r = combine_default(vec![never(), t_int(), t_string()]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["int", "string"]);
}

#[test]
fn many_nevers_collapse() {
    for n in 1..=10 {
        assert_combines_to(vec![never(); n], vec![never()]);
    }
}

#[test]
fn never_with_many_others_disappears() {
    let mut inputs = vec![never()];
    for i in 0..5 {
        inputs.push(t_lit_int(i));
    }
    let r = combine_default(inputs);
    assert_eq!(r.len(), 5);
    assert!(r.iter().all(|a| !matches!(a, mago_codex::ttype::atomic::TAtomic::Never)));
}
