mod comparator_common;

use comparator_common::*;

use std::collections::BTreeMap;

#[test]
fn empty_in_empty() {
    assert_atomic_subtype(&t_empty_array(), &t_empty_array());
}

#[test]
fn empty_in_list() {
    assert_atomic_subtype(&t_empty_array(), &t_list(u(t_int()), false));
    assert_atomic_subtype(&t_empty_array(), &t_list(u(t_string()), false));
    assert_atomic_subtype(&t_empty_array(), &t_list(u(mixed()), false));
}

#[test]
fn empty_not_in_non_empty_list() {
    assert_atomic_not_subtype(&t_empty_array(), &t_list(u(t_int()), true));
    assert_atomic_not_subtype(&t_empty_array(), &t_list(u(t_string()), true));
}

#[test]
fn list_not_in_empty() {
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &t_empty_array());
    assert_atomic_not_subtype(&t_list(u(t_int()), true), &t_empty_array());
}

#[test]
fn list_reflexive() {
    for elem in [t_int(), t_string(), t_float(), t_bool(), mixed()] {
        assert_atomic_subtype(&t_list(u(elem.clone()), false), &t_list(u(elem.clone()), false));
        assert_atomic_subtype(&t_list(u(elem.clone()), true), &t_list(u(elem.clone()), true));
    }
}

#[test]
fn ne_list_in_list() {
    assert_atomic_subtype(&t_list(u(t_int()), true), &t_list(u(t_int()), false));
    assert_atomic_subtype(&t_list(u(t_string()), true), &t_list(u(t_string()), false));
}

#[test]
fn list_not_in_ne_list() {
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &t_list(u(t_int()), true));
}

#[test]
fn list_covariance_in_element() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_list(u(t_scalar()), false));
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_list(u(mixed()), false));
    assert_atomic_subtype(&t_list(u(t_string()), false), &t_list(u(t_scalar()), false));
    assert_atomic_subtype(&t_list(u(t_lit_int(5)), false), &t_list(u(t_int()), false));
    assert_atomic_subtype(&t_list(u(t_lit_string("a")), false), &t_list(u(t_string()), false));
}

#[test]
fn list_not_covariance_when_disjoint_elements() {
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &t_list(u(t_string()), false));
    assert_atomic_not_subtype(&t_list(u(t_string()), false), &t_list(u(t_int()), false));
    assert_atomic_not_subtype(&t_list(u(t_bool()), false), &t_list(u(t_string()), false));
}

#[test]
fn list_not_in_narrower_element() {
    assert_atomic_not_subtype(&t_list(u(t_scalar()), false), &t_list(u(t_int()), false));
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &t_list(u(t_lit_int(5)), false));
    assert_atomic_not_subtype(&t_list(u(t_string()), false), &t_list(u(t_lit_string("a")), false));
}

#[test]
fn keyed_reflexive() {
    let a = t_keyed_unsealed(u(t_string()), u(t_int()), false);
    assert_atomic_subtype(&a.clone(), &a);
}

#[test]
fn keyed_value_covariance() {
    assert_atomic_subtype(
        &t_keyed_unsealed(u(t_string()), u(t_int()), false),
        &t_keyed_unsealed(u(t_string()), u(t_scalar()), false),
    );
    assert_atomic_subtype(
        &t_keyed_unsealed(u(t_string()), u(t_lit_int(5)), false),
        &t_keyed_unsealed(u(t_string()), u(t_int()), false),
    );
}

#[test]
fn keyed_key_covariance_to_array_key() {
    assert_atomic_subtype(
        &t_keyed_unsealed(u(t_string()), u(t_int()), false),
        &t_keyed_unsealed(u(t_array_key()), u(t_int()), false),
    );
    assert_atomic_subtype(
        &t_keyed_unsealed(u(t_int()), u(t_int()), false),
        &t_keyed_unsealed(u(t_array_key()), u(t_int()), false),
    );
}

#[test]
fn keyed_disjoint_keys() {
    assert_atomic_not_subtype(
        &t_keyed_unsealed(u(t_int()), u(t_string()), false),
        &t_keyed_unsealed(u(t_string()), u(t_string()), false),
    );
}

#[test]
fn keyed_disjoint_values() {
    assert_atomic_not_subtype(
        &t_keyed_unsealed(u(t_string()), u(t_int()), false),
        &t_keyed_unsealed(u(t_string()), u(t_string()), false),
    );
}

#[test]
fn list_in_array_with_int_keys() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_keyed_unsealed(u(t_int()), u(t_int()), false));
}

#[test]
fn list_in_array_with_array_key() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_keyed_unsealed(u(t_array_key()), u(t_int()), false));
}

#[test]
fn array_with_int_keys_not_in_list() {
    assert_atomic_not_subtype(&t_keyed_unsealed(u(t_int()), u(t_int()), false), &t_list(u(t_int()), false));
}

#[test]
fn sealed_list_reflexive() {
    let s = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1))), (1, (false, ui(2)))]));
    assert_atomic_subtype(&s.clone(), &s);
}

#[test]
fn sealed_list_distinct_disjoint() {
    let a = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1)))]));
    let b = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(2)))]));
    assert_atomic_not_subtype(&a, &b);
    assert_atomic_not_subtype(&b, &a);
}

#[test]
fn sealed_list_in_widened_sealed() {
    let lit = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1)))]));
    let int = t_sealed_list(BTreeMap::from([(0_usize, (false, u(t_int())))]));
    assert_atomic_subtype(&lit, &int);
    assert_atomic_not_subtype(&int, &lit);
}

#[test]
fn sealed_list_in_unsealed_list() {
    let s = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1)))]));
    assert_atomic_subtype(&s, &t_list(u(t_int()), false));
    assert_atomic_subtype(&s, &t_list(u(t_int()), true));
}

#[test]
fn unsealed_list_not_in_sealed_list() {
    let s = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1)))]));
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &s);
}

#[test]
fn keyed_sealed_reflexive() {
    let a = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1))), (ak_str("b"), (false, us("hi")))]), false);
    assert_atomic_subtype(&a.clone(), &a);
}

#[test]
fn keyed_sealed_distinct_keys_disjoint() {
    let a = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1)))]), false);
    let b = t_keyed_sealed(BTreeMap::from([(ak_str("b"), (false, ui(2)))]), false);
    assert_atomic_not_subtype(&a, &b);
    assert_atomic_not_subtype(&b, &a);
}

#[test]
fn keyed_sealed_value_covariance() {
    let lit = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1)))]), false);
    let int = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, u(t_int())))]), false);
    assert_atomic_subtype(&lit, &int);
    assert_atomic_not_subtype(&int, &lit);
}

#[test]
fn keyed_sealed_required_in_optional() {
    let req = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1)))]), false);
    let opt = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (true, ui(1)))]), false);
    assert_atomic_subtype(&req, &opt);
}

#[test]
fn keyed_sealed_optional_not_in_required() {
    let req = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1)))]), false);
    let opt = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (true, ui(1)))]), false);
    assert_atomic_not_subtype(&opt, &req);
}

#[test]
fn keyed_sealed_in_unsealed_keyed() {
    let s = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1)))]), false);
    assert_atomic_subtype(&s, &t_keyed_unsealed(u(t_string()), u(t_int()), false));
    assert_atomic_subtype(&s, &t_keyed_unsealed(u(t_array_key()), u(t_int()), false));
}

#[test]
fn iterable_reflexive() {
    let it = t_iterable(u(t_int()), u(t_int()));
    assert_atomic_subtype(&it.clone(), &it);
}

#[test]
fn list_in_iterable() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_iterable(u(t_int()), u(t_int())));
}

#[test]
fn keyed_in_iterable() {
    assert_atomic_subtype(&t_keyed_unsealed(u(t_string()), u(t_int()), false), &t_iterable(u(t_string()), u(t_int())));
    assert_atomic_subtype(&t_keyed_unsealed(u(t_int()), u(t_string()), false), &t_iterable(u(t_int()), u(t_string())));
}

#[test]
fn iterable_not_in_list() {
    assert_atomic_not_subtype(&t_iterable(u(t_int()), u(t_int())), &t_list(u(t_int()), false));
}

#[test]
fn iterable_not_in_keyed() {
    assert_atomic_not_subtype(
        &t_iterable(u(t_string()), u(t_int())),
        &t_keyed_unsealed(u(t_string()), u(t_int()), false),
    );
}

#[test]
fn deep_list_of_lists() {
    let inner = u(t_list(u(t_int()), false));
    let outer = t_list(inner.clone(), false);
    assert_atomic_subtype(&outer.clone(), &outer);

    let lit_inner = u(t_list(u(t_lit_int(5)), false));
    let lit_outer = t_list(lit_inner.clone(), false);
    let int_outer = t_list(u(t_list(u(t_int()), false)), false);
    assert_atomic_subtype(&lit_outer, &int_outer);
    assert_atomic_not_subtype(&int_outer, &lit_outer);
}

#[test]
fn deep_keyed_of_lists() {
    let inner_int_list = u(t_list(u(t_int()), false));
    let inner_string_list = u(t_list(u(t_string()), false));
    let a = t_keyed_unsealed(u(t_string()), inner_int_list.clone(), false);
    let b = t_keyed_unsealed(u(t_string()), inner_string_list.clone(), false);
    assert_atomic_subtype(&a.clone(), &a);
    assert_atomic_not_subtype(&a, &b);
}

#[test]
fn deep_list_of_keyed() {
    let inner = u(t_keyed_unsealed(u(t_string()), u(t_int()), false));
    let outer = t_list(inner.clone(), false);
    assert_atomic_subtype(&outer.clone(), &outer);
}

#[test]
fn array_not_in_int() {
    assert_atomic_not_subtype(&t_empty_array(), &t_int());
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &t_int());
}

#[test]
fn array_not_in_object() {
    assert_atomic_not_subtype(&t_empty_array(), &t_object_any());
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &t_object_any());
}

#[test]
fn array_in_mixed() {
    assert_atomic_subtype(&t_empty_array(), &mixed());
    assert_atomic_subtype(&t_list(u(t_int()), false), &mixed());
    assert_atomic_subtype(&t_keyed_unsealed(u(t_string()), u(t_int()), false), &mixed());
}

#[test]
fn list_string_in_list_array_key() {
    assert_atomic_subtype(&t_list(u(t_string()), false), &t_list(u(t_array_key()), false));
}

#[test]
fn deep_list_three_levels() {
    let lit = t_list(u(t_list(u(t_list(u(t_lit_int(1)), false)), false)), false);
    let int = t_list(u(t_list(u(t_list(u(t_int()), false)), false)), false);
    assert_atomic_subtype(&lit, &int);
    assert_atomic_not_subtype(&int, &lit);
}

#[test]
fn deep_keyed_with_optional_property() {
    let a = t_keyed_sealed(
        BTreeMap::from([(ak_str("name"), (false, u(t_string()))), (ak_str("age"), (false, u(t_int())))]),
        false,
    );
    let b = t_keyed_sealed(
        BTreeMap::from([(ak_str("name"), (false, u(t_string()))), (ak_str("age"), (true, u(t_int())))]),
        false,
    );
    assert_atomic_subtype(&a, &b);
    assert_atomic_not_subtype(&b, &a);
}

#[test]
fn deep_keyed_extra_keys_not_subtype() {
    let small = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, u(t_int())))]), false);
    let big = t_keyed_sealed(
        BTreeMap::from([(ak_str("a"), (false, u(t_int()))), (ak_str("b"), (false, u(t_string())))]),
        false,
    );
    assert_atomic_not_subtype(&small, &big);
}

#[test]
fn deep_keyed_subset_with_optional() {
    let small = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, u(t_int())))]), false);
    let big_opt = t_keyed_sealed(
        BTreeMap::from([(ak_str("a"), (false, u(t_int()))), (ak_str("b"), (true, u(t_string())))]),
        false,
    );
    assert_atomic_subtype(&small, &big_opt);
}

#[test]
fn many_distinct_lists_disjoint() {
    let pairs = [
        (t_int(), t_string()),
        (t_string(), t_int()),
        (t_int(), t_bool()),
        (t_bool(), t_int()),
        (t_string(), t_bool()),
        (t_bool(), t_string()),
        (t_string(), t_float()),
        (t_float(), t_string()),
        (t_float(), t_bool()),
        (t_bool(), t_float()),
    ];
    for (a, b) in pairs {
        assert_atomic_not_subtype(&t_list(u(a), false), &t_list(u(b), false));
    }
}

#[test]
fn list_int_in_list_float() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_list(u(t_float()), false));
    assert_atomic_subtype(&t_list(u(t_lit_int(5)), false), &t_list(u(t_float()), false));
}

#[test]
fn list_int_lits_in_list_int() {
    for v in [-100_i64, 0, 1, 100] {
        assert_atomic_subtype(&t_list(u(t_lit_int(v)), false), &t_list(u(t_int()), false));
    }
}

#[test]
fn list_string_lits_in_list_string() {
    for s in ["", "hi", "abc"] {
        assert_atomic_subtype(&t_list(u(t_lit_string(s)), false), &t_list(u(t_string()), false));
    }
}

#[test]
fn keyed_with_value_subtypes_for_many_combos() {
    for outer_v in [t_int(), t_string(), t_bool()] {
        for inner_v in [t_lit_int(5), t_lit_string("hi")] {
            let inner_in_outer = u(inner_v.clone());
            let outer_uniform = t_keyed_unsealed(u(t_string()), u(outer_v.clone()), false);
            let inner_keyed = t_keyed_unsealed(u(t_string()), inner_in_outer, false);
            // Match only when types are compatible
            let cb = empty_codebase();
            let r = atomic_is_contained(&inner_keyed, &outer_uniform, &cb);
            let expected = atomic_is_contained(&inner_v, &outer_v, &cb);
            assert_eq!(r, expected, "keyed<string,{:?}> <: keyed<string,{:?}>", inner_v, outer_v);
        }
    }
}
