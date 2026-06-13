mod common;

use std::collections::BTreeMap;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::KnownElement;
use mago_oracle::ty::well_known;

fn sealed_list<'arena>(f: &mut Fixture<'_, 'arena>, elements: &[Type<'arena>], non_empty: bool) -> Atom<'arena> {
    let known: Vec<KnownElement<'arena>> = elements
        .iter()
        .enumerate()
        .map(|(index, value)| KnownElement { index: index as u32, value: *value, optional: false })
        .collect();

    f.builder.sealed_list(&known, non_empty)
}

#[test]
fn empty_array_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            let r = combine_default(f, vec![f.t_empty_array(); n]);
            assert_eq!(r, vec![f.t_empty_array()]);
        }
    });
}

#[test]
fn empty_array_singleton_passthrough() {
    fixture(|f| {
        let r = combine_default(f, vec![f.t_empty_array()]);
        assert_eq!(r, vec![f.t_empty_array()]);
    });
}

#[test]
fn list_int_idempotent() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, false);
        assert_combines_to(f, vec![list], vec![list]);
    });
}

#[test]
fn list_string_idempotent() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let list = f.t_list(string_type, false);
        assert_combines_to(f, vec![list], vec![list]);
    });
}

#[test]
fn list_with_different_element_types_combine() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let int_list = f.t_list(int_type, false);
        let string_list = f.t_list(string_type, false);
        let int_or_string = f.u_many(vec![f.t_int(), f.t_string()]);
        let merged = f.t_list(int_or_string, false);
        assert_combines_to(f, vec![int_list, string_list], vec![merged]);
    });
}

#[test]
fn list_with_subset_element_collapses() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let lit_int_type = f.ui(7);
        let int_list = f.t_list(int_type, false);
        let lit_int_list = f.t_list(lit_int_type, false);
        assert_combines_to(f, vec![int_list, lit_int_list], vec![int_list]);
    });
}

#[test]
fn non_empty_list_or_general_list_yields_general() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, false);
        let non_empty_list = f.t_list(int_type, true);
        assert_combines_to(f, vec![list, non_empty_list], vec![list]);
    });
}

#[test]
fn two_non_empty_lists_stay_non_empty() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let int_list = f.t_list(int_type, true);
        let string_list = f.t_list(string_type, true);
        assert_single(f, vec![int_list, string_list], |atom| atom.to_string().starts_with("non-empty-list"));
    });
}

#[test]
fn empty_array_absorbed_into_possibly_empty_list() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, false);
        assert_combines_to(f, vec![f.t_empty_array(), list], vec![list]);
    });
}

#[test]
fn empty_array_or_non_empty_list_kept_separate() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let non_empty_list = f.t_list(int_type, true);
        assert_combines_to(f, vec![f.t_empty_array(), non_empty_list], vec![f.t_empty_array(), non_empty_list]);
    });
}

#[test]
fn list_then_empty_yields_just_list() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, false);
        let r = combine_overwrite(f, vec![list, f.t_empty_array()]);
        assert_eq!(r, vec![list]);
    });
}

#[test]
fn many_lists_with_various_elements_combine_into_one() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let bool_type = f.u(f.t_bool());
        let int_list = f.t_list(int_type, false);
        let string_list = f.t_list(string_type, false);
        let bool_list = f.t_list(bool_type, false);
        let merged_element = f.u_many(vec![f.t_int(), f.t_string(), f.t_bool()]);
        let merged = f.t_list(merged_element, false);
        assert_combines_to(f, vec![int_list, string_list, bool_list], vec![merged]);
    });
}

#[test]
fn single_sealed_list_passthrough() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let sealed = sealed_list(f, &[int_type, string_type], false);
        assert_combines_to(f, vec![sealed], vec![sealed]);
    });
}

#[test]
fn sealed_list_meets_unsealed_list_collapses() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let sealed = sealed_list(f, &[int_type, int_type], true);
        let unsealed = f.t_list(int_type, false);
        let r = combine_default(f, vec![sealed, unsealed]);
        assert_eq!(r, vec![unsealed]);
    });
}

#[test]
fn keyed_unsealed_idempotent() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        assert_combines_to(f, vec![keyed], vec![keyed]);
    });
}

#[test]
fn keyed_with_different_value_types_combine() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let bool_type = f.u(f.t_bool());
        let int_valued = f.t_keyed_unsealed(string_type, int_type, false);
        let bool_valued = f.t_keyed_unsealed(string_type, bool_type, false);
        let merged_value = f.u_many(vec![f.t_int(), f.t_bool()]);
        let merged = f.t_keyed_unsealed(string_type, merged_value, false);
        assert_combines_to(f, vec![int_valued, bool_valued], vec![merged]);
    });
}

#[test]
fn keyed_with_different_key_types_combine() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let value_type = f.u(f.t_bool());
        let string_keyed = f.t_keyed_unsealed(string_type, value_type, false);
        let int_keyed = f.t_keyed_unsealed(int_type, value_type, false);
        let merged_key = f.u_many(vec![f.t_int(), f.t_string()]);
        let merged = f.t_keyed_unsealed(merged_key, value_type, false);
        assert_combines_to(f, vec![string_keyed, int_keyed], vec![merged]);
    });
}

#[test]
fn keyed_sealed_same_collapses() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let a_key = f.ak_str("a");
        let sealed = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, int_type))]), false);
        assert_combines_to(f, vec![sealed, sealed], vec![sealed]);
    });
}

#[test]
fn keyed_sealed_different_keys_kept_separate() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let a_key = f.ak_str("a");
        let b_key = f.ak_str("b");
        let a = f.t_keyed_sealed(BTreeMap::from([(a_key, (false, int_type))]), false);
        let b = f.t_keyed_sealed(BTreeMap::from([(b_key, (false, string_type))]), false);
        assert_combines_to(f, vec![a, b], vec![a, b]);
    });
}

#[test]
fn keyed_sealed_overlapping_keys_combine_values() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let shared_key = f.ak_str("k");
        let a = f.t_keyed_sealed(BTreeMap::from([(shared_key, (false, int_type))]), false);
        let b = f.t_keyed_sealed(BTreeMap::from([(shared_key, (false, string_type))]), false);
        let merged_value = f.u_many(vec![f.t_int(), f.t_string()]);
        let merged = f.t_keyed_sealed(BTreeMap::from([(shared_key, (false, merged_value))]), false);
        assert_combines_to(f, vec![a, b], vec![merged]);
    });
}

#[test]
fn list_and_keyed_kept_separate() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let list = f.t_list(int_type, false);
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        assert_combines_to(f, vec![list, keyed], vec![list, keyed]);
    });
}

#[test]
fn empty_array_overwritten_by_list() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, false);
        let r = combine_overwrite(f, vec![f.t_empty_array(), list]);
        assert_eq!(r, vec![list]);
    });
}

#[test]
fn empty_array_overwritten_by_keyed() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let r = combine_overwrite(f, vec![f.t_empty_array(), keyed]);
        assert_eq!(r, vec![keyed]);
    });
}

#[test]
fn empty_alone_with_overwrite_kept() {
    fixture(|f| {
        let r = combine_overwrite(f, vec![f.t_empty_array(), f.t_empty_array()]);
        assert_eq!(r, vec![f.t_empty_array()]);
    });
}

#[test]
fn list_or_int_kept_separate() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, false);
        let r = combine_default(f, vec![list, f.t_int()]);
        assert_eq!(r.len(), 2);
        assert_combines_to(f, vec![list, f.t_int()], vec![f.t_int(), list]);
    });
}

#[test]
fn keyed_or_string_kept_separate() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let r = combine_default(f, vec![keyed, f.t_string()]);
        assert_eq!(r.len(), 2);
        assert_combines_to(f, vec![keyed, f.t_string()], vec![f.t_string(), keyed]);
    });
}

#[test]
fn empty_or_int_kept_separate() {
    fixture(|f| {
        let r = combine_default(f, vec![f.t_empty_array(), f.t_int()]);
        assert_eq!(r.len(), 2);
    });
}

#[test]
fn mixed_dominates_array() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.t_empty_array()], vec![f.mixed()]);
    });
}

#[test]
fn many_distinct_sealed_lists_generalise() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let mut inputs = Vec::new();
        for length in 1..=40usize {
            let elements = vec![int_type; length];
            let sealed = sealed_list(f, &elements, true);
            inputs.push(sealed);
        }
        let r = combine_default(f, inputs);
        assert_eq!(r, vec![well_known::ARRAY_KEY_MIXED]);
    });
}

#[test]
fn under_array_threshold_keeps_sealed_lists() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let mut inputs = Vec::new();
        for length in 1..=5usize {
            let elements = vec![int_type; length];
            let sealed = sealed_list(f, &elements, true);
            inputs.push(sealed);
        }
        let r = combine_with_array_threshold(f, inputs, 32);
        assert_eq!(r.len(), 5);
        assert!(r.iter().all(|atom| atom.kind() == AtomKind::List));
    });
}

#[test]
fn custom_low_array_threshold_generalises_quickly() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let a = sealed_list(f, &[int_type], true);
        let b = sealed_list(f, &[int_type, int_type], true);
        let c = sealed_list(f, &[int_type, int_type, int_type], true);
        let r = combine_with_array_threshold(f, vec![a, b, c], 2);
        assert_eq!(r, vec![well_known::ARRAY_KEY_MIXED]);
    });
}

#[test]
fn list_with_known_elements_idempotent() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let sealed = sealed_list(f, &[int_type, string_type], true);
        assert_combines_to(f, vec![sealed], vec![sealed]);
    });
}

#[test]
fn many_empty_arrays_collapse() {
    fixture(|f| {
        for n in 1..=20 {
            assert_combines_to(f, vec![f.t_empty_array(); n], vec![f.t_empty_array()]);
        }
    });
}

#[test]
fn many_unsealed_lists_collapse() {
    fixture(|f| {
        let mut inputs = Vec::new();
        let mut element_atoms = Vec::new();
        for n in 0..30usize {
            let literal = f.t_lit_string(&format!("s{n}"));
            let element = f.u(literal);
            let list = f.t_list(element, false);
            inputs.push(list);
            element_atoms.push(literal);
        }
        let merged_element = f.u_many(element_atoms);
        let merged = f.t_list(merged_element, false);
        assert_combines_to(f, inputs, vec![merged]);
    });
}

#[test]
fn many_unsealed_keyed_collapse() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let mut inputs = Vec::new();
        let mut value_atoms = Vec::new();
        for n in 0..30usize {
            let literal = f.t_lit_string(&format!("v{n}"));
            let value = f.u(literal);
            let keyed = f.t_keyed_unsealed(string_type, value, false);
            inputs.push(keyed);
            value_atoms.push(literal);
        }
        let merged_value = f.u_many(value_atoms);
        let merged = f.t_keyed_unsealed(string_type, merged_value, false);
        assert_combines_to(f, inputs, vec![merged]);
    });
}
