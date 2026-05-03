mod comparator_common;

use comparator_common::*;

#[test]
fn mixed_to_int_sets_coerced_and_from_nested() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&mixed(), &t_int(), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
    assert_eq!(r.type_coerced_from_nested_mixed, Some(true));
}

#[test]
fn mixed_to_string_sets_coerced_and_from_nested() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&mixed(), &t_string(), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
    assert_eq!(r.type_coerced_from_nested_mixed, Some(true));
}

#[test]
fn array_key_to_int_sets_coerced() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_array_key(), &t_int(), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
    assert_eq!(r.type_coerced_from_nested_mixed, None);
}

#[test]
fn array_key_to_string_sets_coerced() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_array_key(), &t_string(), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
}

#[test]
fn object_to_named_sets_coerced() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_object_any(), &t_named("Foo"), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
}

#[test]
fn bool_to_true_sets_coerced() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_bool(), &t_true(), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
}

#[test]
fn bool_to_false_sets_coerced() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_bool(), &t_false(), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
}

#[test]
fn lit_int_in_int_no_flags_set() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_lit_int(5), &t_int(), &cb);
    assert!(v);
    assert_eq!(r.type_coerced, None);
    assert_eq!(r.type_coerced_from_nested_mixed, None);
}

#[test]
fn int_to_lit_int_no_coerced_flag() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_int(), &t_lit_int(5), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, None);
}

#[test]
fn int_to_positive_int_no_coerced_flag() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_int(), &t_positive_int(), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, None);
}

#[test]
fn distinct_classes_no_coerced_flag() {
    let cb = codebase_from_php("<?php class Foo {} class Bar {}");
    let (v, r) = atomic_is_contained_capturing(&t_named("Foo"), &t_named("Bar"), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, None);
}

#[test]
fn parent_to_child_class_sets_coerced() {
    let cb = codebase_from_php("<?php class Animal {} class Dog extends Animal {}");
    let (v, r) = atomic_is_contained_capturing(&t_named("Animal"), &t_named("Dog"), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
}

#[test]
fn child_to_parent_class_no_coerced_flag() {
    let cb = codebase_from_php("<?php class Animal {} class Dog extends Animal {}");
    let (v, r) = atomic_is_contained_capturing(&t_named("Dog"), &t_named("Animal"), &cb);
    assert!(v);
    assert_eq!(r.type_coerced, None);
}

#[test]
fn equal_atoms_no_flags() {
    let cb = empty_codebase();
    for atom in [t_int(), t_string(), t_bool(), t_float(), null(), t_object_any(), mixed()] {
        let (v, r) = atomic_is_contained_capturing(&atom, &atom, &cb);
        assert!(v, "{atom:?} should equal itself");
        assert_eq!(r.type_coerced, None);
        assert_eq!(r.type_coerced_from_nested_mixed, None);
    }
}

#[test]
fn never_to_anything_no_flags() {
    let cb = empty_codebase();
    for atom in [t_int(), t_string(), t_object_any(), mixed()] {
        let (v, r) = atomic_is_contained_capturing(&never(), &atom, &cb);
        assert!(v, "never <: {atom:?}");
        assert_eq!(r.type_coerced, None);
    }
}

#[test]
fn mixed_to_never_sets_coerced() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&mixed(), &never(), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, Some(true));
}

#[test]
fn concrete_to_never_does_not_set_coerced() {
    let cb = empty_codebase();
    for atom in [t_int(), t_string()] {
        let (v, r) = atomic_is_contained_capturing(&atom, &never(), &cb);
        assert!(!v);
        assert_eq!(r.type_coerced, None);
    }
}

#[test]
fn literal_int_to_other_literal_no_coerced_flag() {
    let cb = empty_codebase();
    let (v, r) = atomic_is_contained_capturing(&t_lit_int(1), &t_lit_int(2), &cb);
    assert!(!v);
    assert_eq!(r.type_coerced, None);
}

#[test]
fn nullable_int_to_int_with_ignore_null_passes() {
    let cb = empty_codebase();
    let nullable = u_many(vec![t_int(), null()]);
    let int_only = u(t_int());
    assert!(is_contained_with(&nullable, &int_only, &cb, true, false, false));
}

#[test]
fn int_or_false_to_int_with_ignore_false_passes() {
    let cb = empty_codebase();
    let int_or_false = u_many(vec![t_int(), t_false()]);
    let int_only = u(t_int());
    assert!(is_contained_with(&int_or_false, &int_only, &cb, false, true, false));
}

#[test]
fn nullable_int_to_int_without_ignore_null_fails() {
    let cb = empty_codebase();
    let nullable = u_many(vec![t_int(), null()]);
    let int_only = u(t_int());
    assert!(!is_contained_with(&nullable, &int_only, &cb, false, false, false));
}
