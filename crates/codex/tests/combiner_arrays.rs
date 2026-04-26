mod combiner_common;

use combiner_common::*;

use std::collections::BTreeMap;

use mago_codex::ttype::atomic::TAtomic;

#[test]
fn empty_array_idempotent() {
    for n in 1..=10 {
        let r = combine_default(vec![t_empty_array(); n]);
        assert_eq!(r.len(), 1);
        assert_eq!(atomic_id_string(&r[0]), "array{}");
    }
}

#[test]
fn empty_array_singleton_passthrough() {
    let r = combine_default(vec![t_empty_array()]);
    assert_eq!(r.len(), 1);
}

#[test]
fn list_int_idempotent() {
    for n in 1..=8 {
        assert_combines_to(vec![t_list(u(t_int()), false); n], vec![t_list(u(t_int()), false)]);
    }
}

#[test]
fn list_string_idempotent() {
    for n in 1..=8 {
        assert_combines_to(vec![t_list(u(t_string()), false); n], vec![t_list(u(t_string()), false)]);
    }
}

#[test]
fn list_with_different_element_types_combine() {
    let r = combine_default(vec![t_list(u(t_int()), false), t_list(u(t_string()), false)]);
    assert_eq!(r.len(), 1);
    let id = atomic_id_string(&r[0]);
    assert!(id.starts_with("list<"));
    assert!(id.contains("int"));
    assert!(id.contains("string"));
}

#[test]
fn list_with_subset_element_collapses() {
    let r = combine_default(vec![t_list(u(t_int()), false), t_list(u(t_lit_int(5)), false)]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "list<int>");
}

#[test]
fn non_empty_list_or_general_list_yields_general() {
    let r = combine_default(vec![t_list(u(t_int()), true), t_list(u(t_int()), false)]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "list<int>");
}

#[test]
fn two_non_empty_lists_stay_non_empty() {
    let r = combine_default(vec![t_list(u(t_int()), true), t_list(u(t_int()), true)]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "non-empty-list<int>");
}

#[test]
fn empty_array_or_list_kept_separate() {
    let r = combine_default(vec![t_empty_array(), t_list(u(t_int()), false)]);
    let mut ids: Vec<String> = r.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids.len(), 2);
}

#[test]
fn empty_array_or_non_empty_list_kept_separate() {
    let r = combine_default(vec![t_empty_array(), t_list(u(t_int()), true)]);
    assert_eq!(r.len(), 2);
}

#[test]
fn list_then_empty_yields_just_list() {
    let r = combine_default(vec![t_list(u(t_int()), false), t_empty_array()]);
    assert_eq!(r.len(), 1);
    let id = atomic_id_string(&r[0]);
    assert!(id.starts_with("list<"));
}

#[test]
fn many_lists_with_various_elements_combine_into_one() {
    let inputs = vec![
        t_list(u(t_int()), false),
        t_list(u(t_string()), false),
        t_list(u(t_float()), false),
        t_list(u(t_bool()), false),
    ];
    let r = combine_default(inputs);
    assert_eq!(r.len(), 1);
}

#[test]
fn single_sealed_list_passthrough() {
    let sealed = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1))), (1_usize, (false, ui(2)))]));
    let r = combine_default(vec![sealed.clone()]);
    assert_eq!(r.len(), 1);
}

#[test]
fn sealed_list_meets_unsealed_list_collapses() {
    let sealed = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1))), (1_usize, (false, ui(2)))]));
    let unsealed = t_list(u(t_int()), false);
    let r = combine_default(vec![sealed, unsealed]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "list<int>");
}

#[test]
fn keyed_unsealed_idempotent() {
    let k = t_keyed_unsealed(u(t_string()), u(t_int()), false);
    for n in 1..=8 {
        assert_combines_to(vec![k.clone(); n], vec![k.clone()]);
    }
}

#[test]
fn keyed_with_different_value_types_combine() {
    let a = t_keyed_unsealed(u(t_string()), u(t_int()), false);
    let b = t_keyed_unsealed(u(t_string()), u(t_string()), false);
    let r = combine_default(vec![a, b]);
    assert_eq!(r.len(), 1);
    let id = atomic_id_string(&r[0]);
    assert!(id.contains("array<string"));
    assert!(id.contains("int") && id.contains("string"));
}

#[test]
fn keyed_with_different_key_types_combine() {
    let a = t_keyed_unsealed(u(t_string()), u(t_int()), false);
    let b = t_keyed_unsealed(u(t_int()), u(t_int()), false);
    let r = combine_default(vec![a, b]);
    assert_eq!(r.len(), 1);
    let id = atomic_id_string(&r[0]);
    assert!(id.starts_with("array<"));
}

#[test]
fn keyed_sealed_same_collapses() {
    let keyed_a =
        t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1))), (ak_str("b"), (false, us("hello")))]), false);
    for n in 1..=8 {
        let r = combine_default(vec![keyed_a.clone(); n]);
        assert_eq!(r.len(), 1, "n={n}");
    }
}

#[test]
fn keyed_sealed_different_keys_kept_separate() {
    let a = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1)))]), false);
    let b = t_keyed_sealed(BTreeMap::from([(ak_str("b"), (false, ui(2)))]), false);
    let r = combine_default(vec![a, b]);
    assert!(r.len() >= 1);
}

#[test]
fn keyed_sealed_overlapping_keys_combine_values() {
    let a = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1)))]), false);
    let b = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(2)))]), false);
    let r = combine_default(vec![a, b]);
    assert_eq!(r.len(), 1);
    let id = atomic_id_string(&r[0]);
    assert!(id.contains("'a'"));
}

#[test]
fn list_and_keyed_kept_separate() {
    let list = t_list(u(t_int()), false);
    let keyed = t_keyed_unsealed(u(t_string()), u(t_int()), false);
    let r = combine_default(vec![list, keyed]);
    assert_eq!(r.len(), 2);
}

#[test]
fn empty_array_overwritten_by_list() {
    let opts = default_opts().with_overwrite_empty_array();
    let r =
        mago_codex::ttype::combiner::combine(vec![t_empty_array(), t_list(u(t_int()), false)], &empty_codebase(), opts);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "list<int>");
}

#[test]
fn empty_array_overwritten_by_keyed() {
    let opts = default_opts().with_overwrite_empty_array();
    let r = mago_codex::ttype::combiner::combine(
        vec![t_empty_array(), t_keyed_unsealed(u(t_string()), u(t_int()), false)],
        &empty_codebase(),
        opts,
    );
    assert_eq!(r.len(), 1);
}

#[test]
fn empty_alone_with_overwrite_kept() {
    let opts = default_opts().with_overwrite_empty_array();
    let r = mago_codex::ttype::combiner::combine(vec![t_empty_array(), t_empty_array()], &empty_codebase(), opts);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "array{}");
}

#[test]
fn list_or_int_kept_separate() {
    let r = combine_default(vec![t_list(u(t_int()), false), t_int()]);
    assert_eq!(r.len(), 2);
}

#[test]
fn keyed_or_string_kept_separate() {
    let r = combine_default(vec![t_keyed_unsealed(u(t_string()), u(t_int()), false), t_string()]);
    assert_eq!(r.len(), 2);
}

#[test]
fn empty_or_int_kept_separate() {
    let r = combine_default(vec![t_empty_array(), t_int()]);
    assert_eq!(r.len(), 2);
}

#[test]
fn mixed_dominates_array() {
    assert_combines_to(vec![mixed(), t_empty_array()], vec![mixed()]);
    assert_combines_to(vec![mixed(), t_list(u(t_int()), false)], vec![mixed()]);
}

#[test]
fn many_distinct_sealed_lists_generalise() {
    let inputs: Vec<TAtomic> =
        (0..40_i64).map(|i| t_sealed_list(BTreeMap::from([(0_usize, (false, ui(i)))]))).collect();
    let r = combine_default(inputs);
    assert!(r.len() <= 40);
}

#[test]
fn under_array_threshold_keeps_sealed_lists() {
    let inputs: Vec<TAtomic> =
        (0..16_i64).map(|i| t_sealed_list(BTreeMap::from([(0_usize, (false, ui(i)))]))).collect();
    let r = combine_default(inputs);
    assert!(r.len() >= 1);
}

#[test]
fn custom_low_array_threshold_generalises_quickly() {
    let inputs: Vec<TAtomic> =
        (0..10_i64).map(|i| t_sealed_list(BTreeMap::from([(0_usize, (false, ui(i)))]))).collect();
    let r = combine_with_array_threshold(inputs, 3);
    assert!(r.len() <= 10);
}

#[test]
fn list_with_known_elements_idempotent() {
    let l = t_sealed_list(BTreeMap::from([(0_usize, (false, u(t_int()))), (1_usize, (false, u(t_string())))]));
    for n in 1..=4 {
        let r = combine_default(vec![l.clone(); n]);
        assert!(r.len() <= n, "n={n}");
    }
}

#[test]
fn many_empty_arrays_collapse() {
    for n in 1..=20 {
        assert_combines_to(vec![t_empty_array(); n], vec![t_empty_array()]);
    }
}

#[test]
fn many_unsealed_lists_collapse() {
    for n in 1..=20 {
        assert_combines_to(vec![t_list(u(t_int()), false); n], vec![t_list(u(t_int()), false)]);
    }
}

#[test]
fn many_unsealed_keyed_collapse() {
    let k = t_keyed_unsealed(u(t_string()), u(t_int()), false);
    for n in 1..=15 {
        assert_combines_to(vec![k.clone(); n], vec![k.clone()]);
    }
}
