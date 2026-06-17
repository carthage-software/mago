mod common;

use std::collections::BTreeMap;

use common::*;

use mago_oracle::symbol::part::generic::Variance;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::KnownElement;

fn t_sealed_list<'arena>(f: &mut Fixture<'_, 'arena>, elements: &[Type<'arena>], non_empty: bool) -> Atom<'arena> {
    let entries: Vec<KnownElement<'arena>> = elements
        .iter()
        .enumerate()
        .map(|(index, value)| KnownElement { index: index as u32, value: *value, optional: false })
        .collect();

    f.builder.sealed_list(&entries, non_empty)
}

#[test]
fn list_of_lists_reflexive() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let int_list = f.t_list(int_type, false);
        let inner = f.u(int_list);
        let outer = f.t_list(inner, false);
        assert_atomic_subtype(f, outer, outer);
    });
}

#[test]
fn deeply_nested_list_lit_in_general() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let one_type = f.u(one);
        let lit_level1 = f.t_list(one_type, false);
        let lit_level1_type = f.u(lit_level1);
        let lit_level2 = f.t_list(lit_level1_type, false);
        let lit_level2_type = f.u(lit_level2);
        let lit3 = f.t_list(lit_level2_type, false);
        let int = f.t_int();
        let int_type = f.u(int);
        let int_level1 = f.t_list(int_type, false);
        let int_level1_type = f.u(int_level1);
        let int_level2 = f.t_list(int_level1_type, false);
        let int_level2_type = f.u(int_level2);
        let int3 = f.t_list(int_level2_type, false);
        assert_atomic_subtype(f, lit3, int3);
        assert_atomic_not_subtype(f, int3, lit3);
    });
}

#[test]
fn list_of_keyed_arrays() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let inner = f.u(keyed);
        let outer = f.t_list(inner, false);
        assert_atomic_subtype(f, outer, outer);
    });
}

#[test]
fn keyed_of_lists() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let list = f.t_list(int_type, false);
        let inner = f.u(list);
        let outer = f.t_keyed_unsealed(string_type, inner, false);
        assert_atomic_subtype(f, outer, outer);
    });
}

#[test]
fn shaped_array_simple() {
    fixture(|f| {
        let name_key = f.ak_str("name");
        let age_key = f.ak_str("age");
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let shape =
            f.t_keyed_sealed(BTreeMap::from([(name_key, (false, string_type)), (age_key, (false, int_type))]), false);
        assert_atomic_subtype(f, shape, shape);
    });
}

#[test]
fn shaped_array_with_lit_values_in_general_shape() {
    fixture(|f| {
        let name_key = f.ak_str("name");
        let age_key = f.ak_str("age");
        let alice = f.us("Alice");
        let thirty = f.ui(30);
        let literal = f.t_keyed_sealed(BTreeMap::from([(name_key, (false, alice)), (age_key, (false, thirty))]), false);
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let general =
            f.t_keyed_sealed(BTreeMap::from([(name_key, (false, string_type)), (age_key, (false, int_type))]), false);
        assert_atomic_subtype(f, literal, general);
        assert_atomic_not_subtype(f, general, literal);
    });
}

#[test]
fn shaped_array_required_in_optional() {
    fixture(|f| {
        let name_key = f.ak_str("name");
        let age_key = f.ak_str("age");
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let required =
            f.t_keyed_sealed(BTreeMap::from([(name_key, (false, string_type)), (age_key, (false, int_type))]), false);
        let optional_age =
            f.t_keyed_sealed(BTreeMap::from([(name_key, (false, string_type)), (age_key, (true, int_type))]), false);
        assert_atomic_subtype(f, required, optional_age);
        assert_atomic_not_subtype(f, optional_age, required);
    });
}

#[test]
fn shaped_array_subset_with_optional_extra() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let b_key = f.ak_str("b");
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let small = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, int_type))]), false);
        let big_optional =
            f.t_keyed_sealed(BTreeMap::from([(a_key, (false, int_type)), (b_key, (true, string_type))]), false);
        assert_atomic_subtype(f, small, big_optional);
    });
}

#[test]
fn shaped_array_extra_required_not_subtype() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let b_key = f.ak_str("b");
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let small = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, int_type))]), false);
        let big = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, int_type)), (b_key, (false, string_type))]), false);
        assert_atomic_not_subtype(f, small, big);
    });
}

#[test]
fn nested_shape_with_list_value() {
    fixture(|f| {
        let items_key = f.ak_str("items");
        let one = f.t_lit_int(1);
        let one_type = f.u(one);
        let literal_list = f.t_list(one_type, false);
        let literal_list_type = f.u(literal_list);
        let literal = f.t_keyed_sealed(BTreeMap::from([(items_key, (false, literal_list_type))]), false);
        let int = f.t_int();
        let int_type = f.u(int);
        let int_list = f.t_list(int_type, false);
        let int_list_type = f.u(int_list);
        let general = f.t_keyed_sealed(BTreeMap::from([(items_key, (false, int_list_type))]), false);
        assert_atomic_subtype(f, literal, general);
    });
}

#[test]
fn nested_shape_with_keyed_value() {
    fixture(|f| {
        let name_key = f.ak_str("name");
        let user_key = f.ak_str("user");
        let alice = f.us("Alice");
        let inner_literal = f.t_keyed_sealed(BTreeMap::from([(name_key, (false, alice))]), false);
        let inner_literal_type = f.u(inner_literal);
        let literal = f.t_keyed_sealed(BTreeMap::from([(user_key, (false, inner_literal_type))]), false);
        let string = f.t_string();
        let string_type = f.u(string);
        let inner_general = f.t_keyed_sealed(BTreeMap::from([(name_key, (false, string_type))]), false);
        let inner_general_type = f.u(inner_general);
        let general = f.t_keyed_sealed(BTreeMap::from([(user_key, (false, inner_general_type))]), false);
        assert_atomic_subtype(f, literal, general);
        assert_atomic_not_subtype(f, general, literal);
    });
}

#[test]
fn deeply_nested_keyed_shape_three_levels() {
    fixture(|f| {
        let level1_key = f.ak_str("level1");
        let level2_key = f.ak_str("level2");
        let level3_key = f.ak_str("level3");
        let forty_two = f.ui(42);
        let int = f.t_int();
        let int_type = f.u(int);
        let mut make = |inner| {
            let level3 = f.t_keyed_sealed(BTreeMap::from([(level3_key, (false, inner))]), false);
            let level3_type = f.u(level3);
            let level2 = f.t_keyed_sealed(BTreeMap::from([(level2_key, (false, level3_type))]), false);
            let level2_type = f.u(level2);
            f.t_keyed_sealed(BTreeMap::from([(level1_key, (false, level2_type))]), false)
        };
        let literal = make(forty_two);
        let general = make(int_type);
        assert_atomic_subtype(f, literal, general);
        assert_atomic_not_subtype(f, general, literal);
    });
}

#[test]
fn list_with_known_elements_lit_in_general() {
    fixture(|f| {
        let alice = f.us("Alice");
        let thirty = f.ui(30);
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let literal = t_sealed_list(f, &[alice, thirty], false);
        let general = t_sealed_list(f, &[string_type, int_type], false);
        assert_atomic_subtype(f, literal, general);
    });
}

#[test]
fn list_with_known_elements_in_unsealed_list() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        let one_type = f.u(one);
        let two_type = f.u(two);
        let int = f.t_int();
        let int_type = f.u(int);
        let scalar = f.t_scalar();
        let scalar_type = f.u(scalar);
        let known = t_sealed_list(f, &[one_type, two_type], false);
        let unsealed_int = f.t_list(int_type, false);
        let unsealed_scalar = f.t_list(scalar_type, false);
        assert_atomic_subtype(f, known, unsealed_int);
        assert_atomic_subtype(f, known, unsealed_scalar);
    });
}

#[test]
fn shape_in_unsealed_keyed_array() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let b_key = f.ak_str("b");
        let int = f.t_int();
        let string = f.t_string();
        let array_key = f.t_array_key();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let array_key_type = f.u(array_key);
        let shape =
            f.t_keyed_sealed(BTreeMap::from([(a_key, (false, int_type)), (b_key, (false, string_type))]), false);
        let container = f.t_keyed_unsealed(string_type, array_key_type, false);
        assert_atomic_subtype(f, shape, container);
    });
}

#[test]
fn deep_object_with_generic_param() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Covariant)]);
        let forty_two = f.ui(42);
        let literal = f.t_generic_named("Box", vec![forty_two]);
        let int = f.t_int();
        let int_type = f.u(int);
        let general = f.t_generic_named("Box", vec![int_type]);
        assert!(atomic_is_contained(f, literal, general, &world));
    });
}

#[test]
fn deep_nested_object_in_box() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Covariant)]);
        let one = f.ui(1);
        let inner = f.t_generic_named("Box", vec![one]);
        let inner_type = f.u(inner);
        let outer = f.t_generic_named("Box", vec![inner_type]);
        let int = f.t_int();
        let int_type = f.u(int);
        let inner_general = f.t_generic_named("Box", vec![int_type]);
        let inner_general_type = f.u(inner_general);
        let outer_general = f.t_generic_named("Box", vec![inner_general_type]);
        assert!(atomic_is_contained(f, outer, outer_general, &world));
    });
}

#[test]
fn list_in_iterable_with_lit_values() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let one_type = f.u(one);
        let list_lit = f.t_list(one_type, false);
        let int = f.t_int();
        let int_type = f.u(int);
        let iter_int = f.t_iterable(int_type, int_type);
        assert_atomic_subtype(f, list_lit, iter_int);
    });
}

#[test]
fn array_of_arrays_chain() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let inner = f.u(keyed);
        let outer = f.t_keyed_unsealed(string_type, inner, false);
        assert_atomic_subtype(f, outer, outer);
    });
}

#[test]
fn many_shape_widths() {
    fixture(|f| {
        for width in 1..=5usize {
            let int = f.t_int();
            let int_type = f.u(int);
            let mut literal_map = BTreeMap::new();
            let mut general_map = BTreeMap::new();
            for index in 0..width {
                let key = f.ak_str(&format!("k{index}"));
                let value = f.ui(index as i64);
                literal_map.insert(key, (false, value));
                general_map.insert(key, (false, int_type));
            }
            let literal = f.t_keyed_sealed(literal_map, false);
            let general = f.t_keyed_sealed(general_map, false);
            assert_atomic_subtype(f, literal, general);
        }
    });
}

#[test]
fn shape_with_string_in_string_lit() {
    fixture(|f| {
        for value in ["hello", "world", "foo", "bar"] {
            let key = f.ak_str("k");
            let literal_value = f.us(value);
            let literal = f.t_keyed_sealed(BTreeMap::from([(key, (false, literal_value))]), false);
            let string = f.t_string();
            let string_type = f.u(string);
            let general = f.t_keyed_sealed(BTreeMap::from([(key, (false, string_type))]), false);
            assert_atomic_subtype(f, literal, general);
        }
    });
}

#[test]
fn list_of_generic_objects() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Covariant)]);
        let one = f.ui(1);
        let inner_lit = f.t_generic_named("Box", vec![one]);
        let int = f.t_int();
        let int_type = f.u(int);
        let inner_int = f.t_generic_named("Box", vec![int_type]);
        let inner_lit_type = f.u(inner_lit);
        let inner_int_type = f.u(inner_int);
        let outer_lit = f.t_list(inner_lit_type, false);
        let outer_int = f.t_list(inner_int_type, false);
        assert!(atomic_is_contained(f, outer_lit, outer_int, &world));
    });
}

#[test]
fn keyed_with_object_values() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[("Admin", "User")]);
        let string = f.t_string();
        let string_type = f.u(string);
        let admin = f.t_named("Admin");
        let admin_type = f.u(admin);
        let user = f.t_named("User");
        let user_type = f.u(user);
        let admin_keyed = f.t_keyed_unsealed(string_type, admin_type, false);
        let user_keyed = f.t_keyed_unsealed(string_type, user_type, false);
        assert!(atomic_is_contained(f, admin_keyed, user_keyed, &world));
        assert!(!atomic_is_contained(f, user_keyed, admin_keyed, &world));
    });
}

#[test]
fn list_with_class_hierarchy() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[("B", "A")]);
        let b = f.t_named("B");
        let b_type = f.u(b);
        let a = f.t_named("A");
        let a_type = f.u(a);
        let list_b = f.t_list(b_type, false);
        let list_a = f.t_list(a_type, false);
        assert!(atomic_is_contained(f, list_b, list_a, &world));
        assert!(!atomic_is_contained(f, list_a, list_b, &world));
    });
}
