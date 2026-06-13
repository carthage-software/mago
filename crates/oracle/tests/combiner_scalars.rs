mod common;

use common::*;

use mago_oracle::ty::Atom;

fn scalar_atom_zoo<'arena>(f: &mut Fixture<'_, 'arena>) -> Vec<Atom<'arena>> {
    vec![
        f.t_bool(),
        f.t_true(),
        f.t_false(),
        f.t_int(),
        f.t_lit_int(0),
        f.t_lit_int(42),
        f.t_lit_int(-1),
        f.t_positive_int(),
        f.t_negative_int(),
        f.t_non_negative_int(),
        f.t_non_positive_int(),
        f.t_int_range(0, 10),
        f.t_int_range(-5, 5),
        f.t_int_from(100),
        f.t_int_to(-100),
        f.t_int_unspec_lit(),
        f.t_string(),
        f.t_lit_string(""),
        f.t_lit_string("hi"),
        f.t_lit_string("0"),
        f.t_non_empty_string(),
        f.t_numeric_string(),
        f.t_lower_string(),
        f.t_upper_string(),
        f.t_truthy_string(),
        f.t_callable_string(),
        f.t_unspec_lit_string(false),
        f.t_unspec_lit_string(true),
        f.t_class_string(),
        f.t_interface_string(),
        f.t_enum_string(),
        f.t_trait_string(),
        f.t_lit_class_string("Foo"),
        f.t_float(),
        f.t_lit_float(0.0),
        f.t_lit_float(1.5),
        f.t_lit_float(-3.25),
        f.t_unspec_lit_float(),
        f.t_numeric(),
        f.t_array_key(),
        f.t_scalar(),
    ]
}

#[test]
fn idempotent_zoo() {
    fixture(|f| {
        for atom in scalar_atom_zoo(f) {
            for n in [1usize, 2, 3, 5, 10, 25] {
                assert_self_idempotent(f, atom, n);
            }
        }
    });
}

#[test]
fn single_input_passthrough_zoo() {
    fixture(|f| {
        for atom in scalar_atom_zoo(f) {
            let result = combine_default(f, vec![atom]);
            assert_eq!(result.len(), 1, "single-input passthrough for {atom:?}");
            assert_eq!(result[0], atom);
        }
    });
}

#[test]
fn true_or_false_yields_bool() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_true(), f.t_false()], vec![f.t_bool()]);
        assert_combines_to(f, vec![f.t_false(), f.t_true()], vec![f.t_bool()]);
    });
}

#[test]
fn bool_absorbs_true_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_bool(), f.t_true()], vec![f.t_bool()]);
        assert_combines_to(f, vec![f.t_true(), f.t_bool()], vec![f.t_bool()]);
    });
}

#[test]
fn bool_absorbs_false_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_bool(), f.t_false()], vec![f.t_bool()]);
        assert_combines_to(f, vec![f.t_false(), f.t_bool()], vec![f.t_bool()]);
    });
}

#[test]
fn bool_absorbs_true_and_false() {
    fixture(|f| {
        for inputs in [
            vec![f.t_bool(), f.t_true(), f.t_false()],
            vec![f.t_true(), f.t_bool(), f.t_false()],
            vec![f.t_true(), f.t_false(), f.t_bool()],
            vec![f.t_false(), f.t_true(), f.t_bool()],
            vec![f.t_false(), f.t_bool(), f.t_true()],
            vec![f.t_bool(), f.t_false(), f.t_true()],
        ] {
            assert_combines_to(f, inputs, vec![f.t_bool()]);
        }
    });
}

#[test]
fn duplicated_true_collapses() {
    fixture(|f| {
        for n in 1..=10 {
            assert_combines_to(f, vec![f.t_true(); n], vec![f.t_true()]);
            assert_combines_to(f, vec![f.t_false(); n], vec![f.t_false()]);
            assert_combines_to(f, vec![f.t_bool(); n], vec![f.t_bool()]);
        }
    });
}

#[test]
fn many_bool_variants_collapse_to_bool() {
    fixture(|f| {
        for inputs in [
            vec![f.t_bool(), f.t_true(), f.t_false(), f.t_bool(), f.t_false()],
            vec![f.t_true(), f.t_false(), f.t_true(), f.t_false()],
            vec![f.t_true(), f.t_true(), f.t_false()],
            vec![f.t_false(), f.t_true(), f.t_true()],
        ] {
            let result = combine_default(f, inputs);
            assert_eq!(result, vec![f.t_bool()]);
        }
    });
}

#[test]
fn float_absorbs_literal_float_either_order() {
    fixture(|f| {
        for v in [-3.25, 0.0, 1.5, 1e10] {
            assert_combines_to(f, vec![f.t_float(), f.t_lit_float(v)], vec![f.t_float()]);
            assert_combines_to(f, vec![f.t_lit_float(v), f.t_float()], vec![f.t_float()]);
        }
    });
}

#[test]
fn distinct_literal_floats_kept_apart() {
    fixture(|f| {
        for vs in [vec![1.0f64, 2.0], vec![-1.0, 0.0, 1.0], vec![1.0, 2.0, 3.0, 4.0]] {
            let inputs: Vec<_> = vs.iter().map(|&v| f.t_lit_float(v)).collect();
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), vs.len());
        }
    });
}

#[test]
fn equal_literal_floats_collapse() {
    fixture(|f| {
        for v in [-1.0, 0.0, 1.0, 1.5, 1e10] {
            assert_combines_to(f, vec![f.t_lit_float(v); 5], vec![f.t_lit_float(v)]);
        }
    });
}

#[test]
fn int_absorbs_literal_int_either_order() {
    fixture(|f| {
        for v in [-1_000_000i64, -100, -1, 0, 1, 42, 1_000_000] {
            assert_combines_to(f, vec![f.t_int(), f.t_lit_int(v)], vec![f.t_int()]);
            assert_combines_to(f, vec![f.t_lit_int(v), f.t_int()], vec![f.t_int()]);
        }
    });
}

#[test]
fn equal_literal_ints_collapse() {
    fixture(|f| {
        for v in [-100, -1, 0, 1, 100] {
            assert_combines_to(f, vec![f.t_lit_int(v); 5], vec![f.t_lit_int(v)]);
        }
    });
}

#[test]
fn non_adjacent_literal_ints_kept_apart_under_threshold() {
    fixture(|f| {
        let inputs: Vec<_> = (1..=10i64).map(|i| f.t_lit_int(i * 10)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 10);
    });
}

#[test]
fn string_absorbs_literal_string_either_order() {
    fixture(|f| {
        for s in ["", "hello", "0", "123", "Hello World"] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_string(), lit], vec![f.t_string()]);
            assert_combines_to(f, vec![lit, f.t_string()], vec![f.t_string()]);
        }
    });
}

#[test]
fn distinct_literal_strings_kept_apart() {
    fixture(|f| {
        let strs = ["a", "b", "c", "d", "e"];
        let inputs: Vec<_> = strs.iter().map(|s| f.t_lit_string(s)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 5);
    });
}

#[test]
fn equal_literal_strings_collapse() {
    fixture(|f| {
        for s in ["", "hello", "world"] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![lit; 5], vec![lit]);
        }
    });
}

#[test]
fn numeric_absorbs_int_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_numeric(), f.t_int()], vec![f.t_numeric()]);
        assert_combines_to(f, vec![f.t_int(), f.t_numeric()], vec![f.t_numeric()]);
    });
}

#[test]
fn numeric_absorbs_float_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_numeric(), f.t_float()], vec![f.t_numeric()]);
        assert_combines_to(f, vec![f.t_float(), f.t_numeric()], vec![f.t_numeric()]);
    });
}

#[test]
fn numeric_absorbs_literal_int_either_order() {
    fixture(|f| {
        for v in [-5i64, 0, 5, 100] {
            assert_combines_to(f, vec![f.t_numeric(), f.t_lit_int(v)], vec![f.t_numeric()]);
            assert_combines_to(f, vec![f.t_lit_int(v), f.t_numeric()], vec![f.t_numeric()]);
        }
    });
}

#[test]
fn numeric_absorbs_literal_float_either_order() {
    fixture(|f| {
        for v in [-1.0f64, 0.0, 1.5, 100.0] {
            assert_combines_to(f, vec![f.t_numeric(), f.t_lit_float(v)], vec![f.t_numeric()]);
            assert_combines_to(f, vec![f.t_lit_float(v), f.t_numeric()], vec![f.t_numeric()]);
        }
    });
}

#[test]
fn numeric_does_not_absorb_string_either_order() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_numeric(), f.t_string()]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&f.t_numeric()));
        assert!(result.contains(&f.t_string()));
    });
}

#[test]
fn array_key_absorbs_int_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_array_key(), f.t_int()], vec![f.t_array_key()]);
        assert_combines_to(f, vec![f.t_int(), f.t_array_key()], vec![f.t_array_key()]);
    });
}

#[test]
fn array_key_absorbs_string_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_array_key(), f.t_string()], vec![f.t_array_key()]);
        assert_combines_to(f, vec![f.t_string(), f.t_array_key()], vec![f.t_array_key()]);
    });
}

#[test]
fn array_key_absorbs_literal_int_either_order() {
    fixture(|f| {
        for v in [-5i64, 0, 5, 42] {
            assert_combines_to(f, vec![f.t_array_key(), f.t_lit_int(v)], vec![f.t_array_key()]);
            assert_combines_to(f, vec![f.t_lit_int(v), f.t_array_key()], vec![f.t_array_key()]);
        }
    });
}

#[test]
fn array_key_absorbs_literal_string_either_order() {
    fixture(|f| {
        for s in ["a", "b", "hello", ""] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_array_key(), lit], vec![f.t_array_key()]);
            assert_combines_to(f, vec![lit, f.t_array_key()], vec![f.t_array_key()]);
        }
    });
}

#[test]
fn array_key_does_not_absorb_float() {
    fixture(|f| {
        let result_ak_first = combine_default(f, vec![f.t_array_key(), f.t_float()]);
        assert_eq!(result_ak_first.len(), 2);
        let result_float_first = combine_default(f, vec![f.t_float(), f.t_array_key()]);
        assert_eq!(result_float_first.len(), 2);
    });
}

#[test]
fn array_key_does_not_absorb_bool() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_array_key(), f.t_bool()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn scalar_absorbs_int_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_scalar(), f.t_int()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_int(), f.t_scalar()], vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_absorbs_string_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_scalar(), f.t_string()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_string(), f.t_scalar()], vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_absorbs_float_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_scalar(), f.t_float()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_float(), f.t_scalar()], vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_absorbs_numeric_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_numeric(), f.t_scalar()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_scalar(), f.t_numeric()], vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_absorbs_array_key_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_scalar(), f.t_array_key()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_array_key(), f.t_scalar()], vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_absorbs_literals_either_order() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        assert_combines_to(f, vec![f.t_scalar(), f.t_lit_int(5)], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_lit_int(5), f.t_scalar()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_scalar(), hi], vec![f.t_scalar()]);
        assert_combines_to(f, vec![hi, f.t_scalar()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_scalar(), f.t_lit_float(1.5)], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_lit_float(1.5), f.t_scalar()], vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_absorbs_bool_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_scalar(), f.t_bool()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_bool(), f.t_scalar()], vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_absorbs_true_false_either_order() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_bool(), f.t_scalar()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_true(), f.t_scalar()], vec![f.t_scalar()]);
        assert_combines_to(f, vec![f.t_false(), f.t_scalar()], vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_synthesised_from_string_float_bool_int() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_string(), f.t_float(), f.t_bool(), f.t_int()]);
        assert_eq!(result, vec![f.t_scalar()]);
    });
}

#[test]
fn scalar_not_synthesised_when_no_unspecified_int() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_string(), f.t_float(), f.t_bool(), f.t_lit_int(5)]);
        assert_eq!(result.len(), 4);
    });
}

#[test]
fn class_string_absorbed_by_string() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_class_string(), f.t_string()], vec![f.t_string()]);
        assert_combines_to(f, vec![f.t_string(), f.t_class_string()], vec![f.t_string()]);
    });
}

#[test]
fn class_string_absorbed_by_array_key() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_class_string(), f.t_array_key()], vec![f.t_array_key()]);
    });
}

#[test]
fn class_string_absorbed_by_scalar() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_class_string(), f.t_scalar()], vec![f.t_scalar()]);
    });
}

#[test]
fn distinct_class_like_kinds_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_class_string(), f.t_interface_string(), f.t_enum_string(), f.t_trait_string()];
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 4);
    });
}

#[test]
fn duplicated_class_like_collapses() {
    fixture(|f| {
        for atom in [f.t_class_string(), f.t_interface_string(), f.t_enum_string(), f.t_trait_string()] {
            assert_combines_to(f, vec![atom; 5], vec![atom]);
        }
    });
}

#[test]
fn int_string_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_int(), f.t_string()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn float_string_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_float(), f.t_string()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn int_float_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_int(), f.t_float()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn int_bool_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_int(), f.t_bool()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn float_bool_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_float(), f.t_bool()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn string_bool_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_string(), f.t_bool()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn numeric_bool_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_numeric(), f.t_bool()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn lit_int_lit_string_kept_separate() {
    fixture(|f| {
        for (i, s) in [(0i64, "a"), (-1, "b"), (42, "hello")] {
            let lit = f.t_lit_string(s);
            let result = combine_default(f, vec![f.t_lit_int(i), lit]);
            assert_eq!(result.len(), 2);
        }
    });
}

#[test]
fn lit_int_lit_float_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_lit_int(1), f.t_lit_float(1.5)]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn lit_string_lit_float_kept_separate() {
    fixture(|f| {
        let lit = f.t_lit_string("a");
        let result = combine_default(f, vec![lit, f.t_lit_float(1.5)]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn many_lit_int_with_int_collapses() {
    fixture(|f| {
        let mut inputs = vec![f.t_int()];
        for i in 0..50 {
            inputs.push(f.t_lit_int(i));
        }
        assert_combines_to(f, inputs, vec![f.t_int()]);
    });
}

#[test]
fn many_lit_string_with_string_collapses() {
    fixture(|f| {
        let mut inputs = vec![f.t_string()];
        for i in 0..30 {
            inputs.push(f.t_lit_string(&format!("s{i}")));
        }
        assert_combines_to(f, inputs, vec![f.t_string()]);
    });
}

#[test]
fn many_lit_float_with_float_collapses() {
    fixture(|f| {
        let mut inputs = vec![f.t_float()];
        for i in 0..20 {
            inputs.push(f.t_lit_float(f64::from(i)));
        }
        assert_combines_to(f, inputs, vec![f.t_float()]);
    });
}

#[test]
fn big_zoo_singleton_passthrough() {
    fixture(|f| {
        for atom in scalar_atom_zoo(f) {
            let r = combine_default(f, vec![atom]);
            assert_eq!(r.len(), 1);
            assert_eq!(r[0], atom);
        }
    });
}

#[test]
fn big_zoo_self_dedup() {
    fixture(|f| {
        for atom in scalar_atom_zoo(f) {
            for n in 2..=8 {
                let r = combine_default(f, vec![atom; n]);
                assert_eq!(r.len(), 1, "self-dedup failed for {atom:?} (n={n})");
                assert_eq!(r[0], atom);
            }
        }
    });
}
