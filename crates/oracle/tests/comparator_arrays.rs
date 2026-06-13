mod common;

use std::collections::BTreeMap;

use common::*;

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
fn empty_in_empty() {
    fixture(|f| {
        let empty = f.t_empty_array();
        assert_atomic_subtype(f, empty, empty);
    });
}

#[test]
fn empty_in_list() {
    fixture(|f| {
        let empty = f.t_empty_array();
        for element in [f.t_int(), f.t_string(), f.mixed()] {
            let element_type = f.u(element);
            let list = f.t_list(element_type, false);
            assert_atomic_subtype(f, empty, list);
        }
    });
}

#[test]
fn empty_not_in_non_empty_list() {
    fixture(|f| {
        let empty = f.t_empty_array();
        for element in [f.t_int(), f.t_string()] {
            let element_type = f.u(element);
            let list = f.t_list(element_type, true);
            assert_atomic_not_subtype(f, empty, list);
        }
    });
}

#[test]
fn list_not_in_empty() {
    fixture(|f| {
        let empty = f.t_empty_array();
        let int = f.t_int();
        let int_type = f.u(int);
        let list = f.t_list(int_type, false);
        let non_empty_list = f.t_list(int_type, true);
        assert_atomic_not_subtype(f, list, empty);
        assert_atomic_not_subtype(f, non_empty_list, empty);
    });
}

#[test]
fn list_reflexive() {
    fixture(|f| {
        for element in [f.t_int(), f.t_string(), f.t_float(), f.t_bool(), f.mixed()] {
            let element_type = f.u(element);
            let list = f.t_list(element_type, false);
            let non_empty_list = f.t_list(element_type, true);
            assert_atomic_subtype(f, list, list);
            assert_atomic_subtype(f, non_empty_list, non_empty_list);
        }
    });
}

#[test]
fn ne_list_in_list() {
    fixture(|f| {
        for element in [f.t_int(), f.t_string()] {
            let element_type = f.u(element);
            let non_empty_list = f.t_list(element_type, true);
            let list = f.t_list(element_type, false);
            assert_atomic_subtype(f, non_empty_list, list);
        }
    });
}

#[test]
fn list_not_in_ne_list() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let list = f.t_list(int_type, false);
        let non_empty_list = f.t_list(int_type, true);
        assert_atomic_not_subtype(f, list, non_empty_list);
    });
}

#[test]
fn list_covariance_in_element() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let scalar = f.t_scalar();
        let mixed = f.mixed();
        let lit_int = f.t_lit_int(5);
        let lit_string = f.t_lit_string("a");
        let int_type = f.u(int);
        let string_type = f.u(string);
        let scalar_type = f.u(scalar);
        let mixed_type = f.u(mixed);
        let lit_int_type = f.u(lit_int);
        let lit_string_type = f.u(lit_string);
        let int_list = f.t_list(int_type, false);
        let string_list = f.t_list(string_type, false);
        let scalar_list = f.t_list(scalar_type, false);
        let mixed_list = f.t_list(mixed_type, false);
        let lit_int_list = f.t_list(lit_int_type, false);
        let lit_string_list = f.t_list(lit_string_type, false);
        assert_atomic_subtype(f, int_list, scalar_list);
        assert_atomic_subtype(f, int_list, mixed_list);
        assert_atomic_subtype(f, string_list, scalar_list);
        assert_atomic_subtype(f, lit_int_list, int_list);
        assert_atomic_subtype(f, lit_string_list, string_list);
    });
}

#[test]
fn list_not_covariance_when_disjoint_elements() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let boolean = f.t_bool();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let boolean_type = f.u(boolean);
        let int_list = f.t_list(int_type, false);
        let string_list = f.t_list(string_type, false);
        let boolean_list = f.t_list(boolean_type, false);
        assert_atomic_not_subtype(f, int_list, string_list);
        assert_atomic_not_subtype(f, string_list, int_list);
        assert_atomic_not_subtype(f, boolean_list, string_list);
    });
}

#[test]
fn list_not_in_narrower_element() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let scalar = f.t_scalar();
        let lit_int = f.t_lit_int(5);
        let lit_string = f.t_lit_string("a");
        let int_type = f.u(int);
        let string_type = f.u(string);
        let scalar_type = f.u(scalar);
        let lit_int_type = f.u(lit_int);
        let lit_string_type = f.u(lit_string);
        let int_list = f.t_list(int_type, false);
        let string_list = f.t_list(string_type, false);
        let scalar_list = f.t_list(scalar_type, false);
        let lit_int_list = f.t_list(lit_int_type, false);
        let lit_string_list = f.t_list(lit_string_type, false);
        assert_atomic_not_subtype(f, scalar_list, int_list);
        assert_atomic_not_subtype(f, int_list, lit_int_list);
        assert_atomic_not_subtype(f, string_list, lit_string_list);
    });
}

#[test]
fn keyed_reflexive() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        assert_atomic_subtype(f, keyed, keyed);
    });
}

#[test]
fn keyed_value_covariance() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let scalar = f.t_scalar();
        let lit_int = f.t_lit_int(5);
        let string_type = f.u(string);
        let int_type = f.u(int);
        let scalar_type = f.u(scalar);
        let lit_int_type = f.u(lit_int);
        let int_keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let scalar_keyed = f.t_keyed_unsealed(string_type, scalar_type, false);
        let lit_keyed = f.t_keyed_unsealed(string_type, lit_int_type, false);
        assert_atomic_subtype(f, int_keyed, scalar_keyed);
        assert_atomic_subtype(f, lit_keyed, int_keyed);
    });
}

#[test]
fn keyed_key_covariance_to_array_key() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let array_key = f.t_array_key();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let array_key_type = f.u(array_key);
        let string_keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let int_keyed = f.t_keyed_unsealed(int_type, int_type, false);
        let array_key_keyed = f.t_keyed_unsealed(array_key_type, int_type, false);
        assert_atomic_subtype(f, string_keyed, array_key_keyed);
        assert_atomic_subtype(f, int_keyed, array_key_keyed);
    });
}

#[test]
fn keyed_disjoint_keys() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let int_keyed = f.t_keyed_unsealed(int_type, string_type, false);
        let string_keyed = f.t_keyed_unsealed(string_type, string_type, false);
        assert_atomic_not_subtype(f, int_keyed, string_keyed);
    });
}

#[test]
fn keyed_disjoint_values() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let int_valued = f.t_keyed_unsealed(string_type, int_type, false);
        let string_valued = f.t_keyed_unsealed(string_type, string_type, false);
        assert_atomic_not_subtype(f, int_valued, string_valued);
    });
}

#[test]
fn list_in_array_with_int_keys() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let list = f.t_list(int_type, false);
        let keyed = f.t_keyed_unsealed(int_type, int_type, false);
        assert_atomic_subtype(f, list, keyed);
    });
}

#[test]
fn list_in_array_with_array_key() {
    fixture(|f| {
        let int = f.t_int();
        let array_key = f.t_array_key();
        let int_type = f.u(int);
        let array_key_type = f.u(array_key);
        let list = f.t_list(int_type, false);
        let keyed = f.t_keyed_unsealed(array_key_type, int_type, false);
        assert_atomic_subtype(f, list, keyed);
    });
}

#[test]
fn array_with_int_keys_not_in_list() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let keyed = f.t_keyed_unsealed(int_type, int_type, false);
        let list = f.t_list(int_type, false);
        assert_atomic_not_subtype(f, keyed, list);
    });
}

#[test]
fn sealed_list_reflexive() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let single = t_sealed_list(f, &[int_type], true);
        let pair = t_sealed_list(f, &[int_type, string_type], true);
        assert_atomic_subtype(f, single, single);
        assert_atomic_subtype(f, pair, pair);
    });
}

#[test]
fn sealed_list_distinct_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let int_at_zero = t_sealed_list(f, &[int_type], false);
        let string_at_zero = t_sealed_list(f, &[string_type], false);
        assert_atomic_not_subtype(f, int_at_zero, string_at_zero);
        assert_atomic_not_subtype(f, string_at_zero, int_at_zero);
    });
}

#[test]
fn sealed_list_in_widened_sealed() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        let int = f.t_int();
        let one_type = f.u(one);
        let two_type = f.u(two);
        let int_type = f.u(int);
        let literal = t_sealed_list(f, &[one_type, two_type], false);
        let general = t_sealed_list(f, &[int_type, int_type], false);
        assert_atomic_subtype(f, literal, general);
    });
}

#[test]
fn sealed_list_in_unsealed_list() {
    fixture(|f| {
        let int = f.t_int();
        let scalar = f.t_scalar();
        let int_type = f.u(int);
        let scalar_type = f.u(scalar);
        let sealed = t_sealed_list(f, &[int_type, int_type], false);
        let unsealed_int = f.t_list(int_type, false);
        let unsealed_scalar = f.t_list(scalar_type, false);
        assert_atomic_subtype(f, sealed, unsealed_int);
        assert_atomic_subtype(f, sealed, unsealed_scalar);
    });
}

#[test]
fn unsealed_list_not_in_sealed_list() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let unsealed = f.t_list(int_type, false);
        let unsealed_non_empty = f.t_list(int_type, true);
        let sealed = t_sealed_list(f, &[int_type], true);
        assert_atomic_not_subtype(f, unsealed, sealed);
        assert_atomic_not_subtype(f, unsealed_non_empty, sealed);
    });
}

#[test]
fn keyed_sealed_reflexive() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let b_key = f.ak_str("b");
        let one = f.ui(1);
        let hi = f.us("hi");
        let shape = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, one)), (b_key, (false, hi))]), false);
        assert_atomic_subtype(f, shape, shape);
    });
}

#[test]
fn keyed_sealed_distinct_keys_disjoint() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let b_key = f.ak_str("b");
        let one = f.ui(1);
        let two = f.ui(2);
        let a = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, one))]), false);
        let b = f.t_keyed_sealed(BTreeMap::from([(b_key, (false, two))]), false);
        assert_atomic_not_subtype(f, a, b);
        assert_atomic_not_subtype(f, b, a);
    });
}

#[test]
fn keyed_sealed_value_covariance() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let one = f.ui(1);
        let int = f.t_int();
        let int_type = f.u(int);
        let literal = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, one))]), false);
        let general = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, int_type))]), false);
        assert_atomic_subtype(f, literal, general);
        assert_atomic_not_subtype(f, general, literal);
    });
}

#[test]
fn keyed_sealed_required_in_optional() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let one = f.ui(1);
        let required = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, one))]), false);
        let optional = f.t_keyed_sealed(BTreeMap::from([(a_key, (true, one))]), false);
        assert_atomic_subtype(f, required, optional);
    });
}

#[test]
fn keyed_sealed_optional_not_in_required() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let one = f.ui(1);
        let required = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, one))]), false);
        let optional = f.t_keyed_sealed(BTreeMap::from([(a_key, (true, one))]), false);
        assert_atomic_not_subtype(f, optional, required);
    });
}

#[test]
fn keyed_sealed_in_unsealed_keyed() {
    fixture(|f| {
        let a_key = f.ak_str("a");
        let one = f.ui(1);
        let sealed = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, one))]), false);
        let string = f.t_string();
        let int = f.t_int();
        let array_key = f.t_array_key();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let array_key_type = f.u(array_key);
        let string_keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let array_key_keyed = f.t_keyed_unsealed(array_key_type, int_type, false);
        assert_atomic_subtype(f, sealed, string_keyed);
        assert_atomic_subtype(f, sealed, array_key_keyed);
    });
}

#[test]
fn iterable_reflexive() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let iterable = f.t_iterable(int_type, int_type);
        assert_atomic_subtype(f, iterable, iterable);
    });
}

#[test]
fn list_in_iterable() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let list = f.t_list(int_type, false);
        let iterable = f.t_iterable(int_type, int_type);
        assert_atomic_subtype(f, list, iterable);
    });
}

#[test]
fn keyed_in_iterable() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let string_keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let string_iterable = f.t_iterable(string_type, int_type);
        let int_keyed = f.t_keyed_unsealed(int_type, string_type, false);
        let int_iterable = f.t_iterable(int_type, string_type);
        assert_atomic_subtype(f, string_keyed, string_iterable);
        assert_atomic_subtype(f, int_keyed, int_iterable);
    });
}

#[test]
fn iterable_not_in_list() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let iterable = f.t_iterable(int_type, int_type);
        let list = f.t_list(int_type, false);
        assert_atomic_not_subtype(f, iterable, list);
    });
}

#[test]
fn iterable_not_in_keyed() {
    fixture(|f| {
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let iterable = f.t_iterable(string_type, int_type);
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        assert_atomic_not_subtype(f, iterable, keyed);
    });
}

#[test]
fn deep_list_of_lists() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let int_list = f.t_list(int_type, false);
        let inner = f.u(int_list);
        let outer = f.t_list(inner, false);
        assert_atomic_subtype(f, outer, outer);

        let lit = f.t_lit_int(5);
        let lit_type = f.u(lit);
        let lit_list = f.t_list(lit_type, false);
        let lit_inner = f.u(lit_list);
        let lit_outer = f.t_list(lit_inner, false);
        let int_outer = f.t_list(inner, false);
        assert_atomic_subtype(f, lit_outer, int_outer);
        assert_atomic_not_subtype(f, int_outer, lit_outer);
    });
}

#[test]
fn deep_keyed_of_lists() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let int_list = f.t_list(int_type, false);
        let string_list = f.t_list(string_type, false);
        let inner_int_list = f.u(int_list);
        let inner_string_list = f.u(string_list);
        let a = f.t_keyed_unsealed(string_type, inner_int_list, false);
        let b = f.t_keyed_unsealed(string_type, inner_string_list, false);
        assert_atomic_subtype(f, a, a);
        assert_atomic_not_subtype(f, a, b);
    });
}

#[test]
fn deep_list_of_keyed() {
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
fn array_not_in_int() {
    fixture(|f| {
        let empty = f.t_empty_array();
        let int = f.t_int();
        let int_type = f.u(int);
        let list = f.t_list(int_type, false);
        assert_atomic_not_subtype(f, empty, int);
        assert_atomic_not_subtype(f, list, int);
    });
}

#[test]
fn array_not_in_object() {
    fixture(|f| {
        let empty = f.t_empty_array();
        let object = f.t_object_any();
        let int = f.t_int();
        let int_type = f.u(int);
        let list = f.t_list(int_type, false);
        assert_atomic_not_subtype(f, empty, object);
        assert_atomic_not_subtype(f, list, object);
    });
}

#[test]
fn array_in_mixed() {
    fixture(|f| {
        let empty = f.t_empty_array();
        let mixed = f.mixed();
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let list = f.t_list(int_type, false);
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        assert_atomic_subtype(f, empty, mixed);
        assert_atomic_subtype(f, list, mixed);
        assert_atomic_subtype(f, keyed, mixed);
    });
}

#[test]
fn list_string_in_list_array_key() {
    fixture(|f| {
        let string = f.t_string();
        let array_key = f.t_array_key();
        let string_type = f.u(string);
        let array_key_type = f.u(array_key);
        let string_list = f.t_list(string_type, false);
        let array_key_list = f.t_list(array_key_type, false);
        assert_atomic_subtype(f, string_list, array_key_list);
    });
}

#[test]
fn deep_list_three_levels() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let one_type = f.u(one);
        let lit_level1 = f.t_list(one_type, false);
        let lit_level1_type = f.u(lit_level1);
        let lit_level2 = f.t_list(lit_level1_type, false);
        let lit_level2_type = f.u(lit_level2);
        let lit = f.t_list(lit_level2_type, false);
        let int = f.t_int();
        let int_type = f.u(int);
        let int_level1 = f.t_list(int_type, false);
        let int_level1_type = f.u(int_level1);
        let int_level2 = f.t_list(int_level1_type, false);
        let int_level2_type = f.u(int_level2);
        let general = f.t_list(int_level2_type, false);
        assert_atomic_subtype(f, lit, general);
        assert_atomic_not_subtype(f, general, lit);
    });
}

#[test]
fn deep_keyed_with_optional_property() {
    fixture(|f| {
        let name_key = f.ak_str("name");
        let age_key = f.ak_str("age");
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let a =
            f.t_keyed_sealed(BTreeMap::from([(name_key, (false, string_type)), (age_key, (false, int_type))]), false);
        let b =
            f.t_keyed_sealed(BTreeMap::from([(name_key, (false, string_type)), (age_key, (true, int_type))]), false);
        assert_atomic_subtype(f, a, b);
        assert_atomic_not_subtype(f, b, a);
    });
}

#[test]
fn deep_keyed_extra_keys_not_subtype() {
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
fn deep_keyed_subset_with_optional() {
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
fn many_distinct_lists_disjoint() {
    fixture(|f| {
        let pairs = [
            (f.t_int(), f.t_string()),
            (f.t_string(), f.t_int()),
            (f.t_int(), f.t_bool()),
            (f.t_bool(), f.t_int()),
            (f.t_string(), f.t_bool()),
            (f.t_bool(), f.t_string()),
            (f.t_string(), f.t_float()),
            (f.t_float(), f.t_string()),
            (f.t_float(), f.t_bool()),
            (f.t_bool(), f.t_float()),
        ];
        for (a, b) in pairs {
            let a_type = f.u(a);
            let b_type = f.u(b);
            let a_list = f.t_list(a_type, false);
            let b_list = f.t_list(b_type, false);
            assert_atomic_not_subtype(f, a_list, b_list);
        }
    });
}

#[test]
fn list_int_not_in_list_float() {
    fixture(|f| {
        let int = f.t_int();
        let float = f.t_float();
        let lit = f.t_lit_int(5);
        let int_type = f.u(int);
        let float_type = f.u(float);
        let lit_type = f.u(lit);
        let int_list = f.t_list(int_type, false);
        let float_list = f.t_list(float_type, false);
        let lit_list = f.t_list(lit_type, false);
        assert_atomic_not_subtype(f, int_list, float_list);
        assert_atomic_not_subtype(f, lit_list, float_list);
    });
}

#[test]
fn list_int_lits_in_list_int() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let int_list = f.t_list(int_type, false);
        for value in [-100i64, 0, 1, 100] {
            let literal = f.t_lit_int(value);
            let literal_type = f.u(literal);
            let literal_list = f.t_list(literal_type, false);
            assert_atomic_subtype(f, literal_list, int_list);
        }
    });
}

#[test]
fn list_string_lits_in_list_string() {
    fixture(|f| {
        let string = f.t_string();
        let string_type = f.u(string);
        let string_list = f.t_list(string_type, false);
        for value in ["", "hi", "abc"] {
            let literal = f.t_lit_string(value);
            let literal_type = f.u(literal);
            let literal_list = f.t_list(literal_type, false);
            assert_atomic_subtype(f, literal_list, string_list);
        }
    });
}

#[test]
fn keyed_with_value_subtypes_for_many_combos() {
    fixture(|f| {
        let world = empty_world();
        for outer_value in [f.t_int(), f.t_string(), f.t_bool()] {
            for inner_value in [f.t_lit_int(5), f.t_lit_string("hi")] {
                let string = f.t_string();
                let string_type = f.u(string);
                let inner_in_outer = f.u(inner_value);
                let outer_type = f.u(outer_value);
                let outer_uniform = f.t_keyed_unsealed(string_type, outer_type, false);
                let inner_keyed = f.t_keyed_unsealed(string_type, inner_in_outer, false);
                let result = atomic_is_contained(f, inner_keyed, outer_uniform, &world);
                let expected = atomic_is_contained(f, inner_value, outer_value, &world);
                assert_eq!(result, expected, "keyed<string,{inner_value:?}> <: keyed<string,{outer_value:?}>");
            }
        }
    });
}
