mod comparator_common;

use comparator_common::*;

use std::collections::BTreeMap;

#[test]
fn list_of_lists_reflexive() {
    let inner = u(t_list(u(t_int()), false));
    let outer = t_list(inner.clone(), false);
    assert_atomic_subtype(&outer.clone(), &outer);
}

#[test]
fn deeply_nested_list_lit_in_general() {
    let lit3 = t_list(u(t_list(u(t_list(u(t_lit_int(1)), false)), false)), false);
    let int3 = t_list(u(t_list(u(t_list(u(t_int()), false)), false)), false);
    assert_atomic_subtype(&lit3, &int3);
    assert_atomic_not_subtype(&int3, &lit3);
}

#[test]
fn list_of_keyed_arrays() {
    let inner = u(t_keyed_unsealed(u(t_string()), u(t_int()), false));
    let outer = t_list(inner.clone(), false);
    assert_atomic_subtype(&outer.clone(), &outer);
}

#[test]
fn keyed_of_lists() {
    let inner = u(t_list(u(t_int()), false));
    let outer = t_keyed_unsealed(u(t_string()), inner, false);
    assert_atomic_subtype(&outer.clone(), &outer);
}

#[test]
fn shaped_array_simple() {
    let s = t_keyed_sealed(
        BTreeMap::from([(ak_str("name"), (false, u(t_string()))), (ak_str("age"), (false, u(t_int())))]),
        false,
    );
    assert_atomic_subtype(&s.clone(), &s);
}

#[test]
fn shaped_array_with_lit_values_in_general_shape() {
    let lit = t_keyed_sealed(
        BTreeMap::from([(ak_str("name"), (false, us("Alice"))), (ak_str("age"), (false, ui(30)))]),
        false,
    );
    let general = t_keyed_sealed(
        BTreeMap::from([(ak_str("name"), (false, u(t_string()))), (ak_str("age"), (false, u(t_int())))]),
        false,
    );
    assert_atomic_subtype(&lit, &general);
    assert_atomic_not_subtype(&general, &lit);
}

#[test]
fn shaped_array_required_in_optional() {
    let req = t_keyed_sealed(
        BTreeMap::from([(ak_str("name"), (false, u(t_string()))), (ak_str("age"), (false, u(t_int())))]),
        false,
    );
    let opt_age = t_keyed_sealed(
        BTreeMap::from([(ak_str("name"), (false, u(t_string()))), (ak_str("age"), (true, u(t_int())))]),
        false,
    );
    assert_atomic_subtype(&req, &opt_age);
    assert_atomic_not_subtype(&opt_age, &req);
}

#[test]
fn shaped_array_subset_with_optional_extra() {
    let small = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, u(t_int())))]), false);
    let big_opt = t_keyed_sealed(
        BTreeMap::from([(ak_str("a"), (false, u(t_int()))), (ak_str("b"), (true, u(t_string())))]),
        false,
    );
    assert_atomic_subtype(&small, &big_opt);
}

#[test]
fn shaped_array_extra_required_not_subtype() {
    let small = t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, u(t_int())))]), false);
    let big = t_keyed_sealed(
        BTreeMap::from([(ak_str("a"), (false, u(t_int()))), (ak_str("b"), (false, u(t_string())))]),
        false,
    );
    assert_atomic_not_subtype(&small, &big);
}

#[test]
fn nested_shape_with_list_value() {
    let lit = t_keyed_sealed(BTreeMap::from([(ak_str("items"), (false, u(t_list(u(t_lit_int(1)), false))))]), false);
    let general = t_keyed_sealed(BTreeMap::from([(ak_str("items"), (false, u(t_list(u(t_int()), false))))]), false);
    assert_atomic_subtype(&lit, &general);
}

#[test]
fn nested_shape_with_keyed_value() {
    let lit = t_keyed_sealed(
        BTreeMap::from([(
            ak_str("user"),
            (false, u(t_keyed_sealed(BTreeMap::from([(ak_str("name"), (false, us("Alice")))]), false))),
        )]),
        false,
    );
    let general = t_keyed_sealed(
        BTreeMap::from([(
            ak_str("user"),
            (false, u(t_keyed_sealed(BTreeMap::from([(ak_str("name"), (false, u(t_string())))]), false))),
        )]),
        false,
    );
    assert_atomic_subtype(&lit, &general);
    assert_atomic_not_subtype(&general, &lit);
}

#[test]
fn deeply_nested_keyed_shape_three_levels() {
    let make = |inner: mago_codex::ttype::union::TUnion| {
        t_keyed_sealed(
            BTreeMap::from([(
                ak_str("level1"),
                (
                    false,
                    u(t_keyed_sealed(
                        BTreeMap::from([(
                            ak_str("level2"),
                            (false, u(t_keyed_sealed(BTreeMap::from([(ak_str("level3"), (false, inner))]), false))),
                        )]),
                        false,
                    )),
                ),
            )]),
            false,
        )
    };
    let lit = make(ui(42));
    let general = make(u(t_int()));
    assert_atomic_subtype(&lit, &general);
    assert_atomic_not_subtype(&general, &lit);
}

#[test]
fn list_with_known_elements_lit_in_general() {
    let lit = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1))), (1, (false, ui(2))), (2, (false, ui(3)))]));
    let general = t_sealed_list(BTreeMap::from([
        (0_usize, (false, u(t_int()))),
        (1, (false, u(t_int()))),
        (2, (false, u(t_int()))),
    ]));
    assert_atomic_subtype(&lit, &general);
}

#[test]
fn list_with_known_elements_in_unsealed_list() {
    let known = t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1))), (1, (false, ui(2)))]));
    assert_atomic_subtype(&known, &t_list(u(t_int()), false));
}

#[test]
fn shape_in_unsealed_keyed_array() {
    let s = t_keyed_sealed(
        BTreeMap::from([(ak_str("a"), (false, u(t_int()))), (ak_str("b"), (false, u(t_string())))]),
        false,
    );
    assert_atomic_subtype(&s, &t_keyed_unsealed(u(t_string()), u(t_array_key()), false));
}

#[test]
fn deep_object_with_generic_param() {
    let cb = codebase_from_php(r"<?php /** @template-covariant T */ class Box {}");
    let lit = t_generic_named("Box", vec![ui(42)]);
    let general = t_generic_named("Box", vec![u(t_int())]);
    assert!(atomic_is_contained(&lit, &general, &cb));
}

#[test]
fn deep_nested_object_in_box() {
    let cb = codebase_from_php(r"<?php /** @template-covariant T */ class Box {}");
    let inner = t_generic_named("Box", vec![ui(1)]);
    let outer = t_generic_named("Box", vec![u(inner)]);
    let inner_general = t_generic_named("Box", vec![u(t_int())]);
    let outer_general = t_generic_named("Box", vec![u(inner_general)]);
    assert!(atomic_is_contained(&outer, &outer_general, &cb));
}

#[test]
fn list_in_iterable_with_lit_values() {
    let list_lit = t_list(u(t_lit_int(1)), false);
    let iter_int = t_iterable(u(t_int()), u(t_int()));
    assert_atomic_subtype(&list_lit, &iter_int);
}

#[test]
fn array_of_arrays_chain() {
    let inner = u(t_keyed_unsealed(u(t_string()), u(t_int()), false));
    let outer = t_keyed_unsealed(u(t_string()), inner.clone(), false);
    assert_atomic_subtype(&outer.clone(), &outer);
}

#[test]
fn many_shape_widths() {
    for n_keys in 1..=5_usize {
        let mut lit_map = BTreeMap::new();
        let mut general_map = BTreeMap::new();
        for i in 0..n_keys {
            let key = ak_str(&format!("k{i}"));
            lit_map.insert(key, (false, ui(i as i64)));
            general_map.insert(key, (false, u(t_int())));
        }
        let lit = t_keyed_sealed(lit_map, false);
        let general = t_keyed_sealed(general_map, false);
        assert_atomic_subtype(&lit, &general);
    }
}

#[test]
fn shape_with_string_in_string_lit() {
    for s in ["hello", "world", "foo", "bar"] {
        let lit = t_keyed_sealed(BTreeMap::from([(ak_str("k"), (false, us(s)))]), false);
        let general = t_keyed_sealed(BTreeMap::from([(ak_str("k"), (false, u(t_string())))]), false);
        assert_atomic_subtype(&lit, &general);
    }
}

#[test]
fn list_of_generic_objects() {
    let cb = codebase_from_php(r"<?php /** @template-covariant T */ class Box {}");
    let inner_lit = t_generic_named("Box", vec![ui(1)]);
    let inner_int = t_generic_named("Box", vec![u(t_int())]);
    let outer_lit = t_list(u(inner_lit), false);
    let outer_int = t_list(u(inner_int), false);
    assert!(atomic_is_contained(&outer_lit, &outer_int, &cb));
}

#[test]
fn keyed_with_object_values() {
    let cb = codebase_from_php(r"<?php class User {} class Admin extends User {}");
    let admin_keyed = t_keyed_unsealed(u(t_string()), u(t_named("Admin")), false);
    let user_keyed = t_keyed_unsealed(u(t_string()), u(t_named("User")), false);
    assert!(atomic_is_contained(&admin_keyed, &user_keyed, &cb));
    assert!(!atomic_is_contained(&user_keyed, &admin_keyed, &cb));
}

#[test]
fn list_with_class_hierarchy() {
    let cb = codebase_from_php(r"<?php class A {} class B extends A {}");
    let list_b = t_list(u(t_named("B")), false);
    let list_a = t_list(u(t_named("A")), false);
    assert!(atomic_is_contained(&list_b, &list_a, &cb));
    assert!(!atomic_is_contained(&list_a, &list_b, &cb));
}
