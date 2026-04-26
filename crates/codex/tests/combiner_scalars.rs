mod combiner_common;

use combiner_common::*;

use mago_codex::ttype::atomic::TAtomic;

#[test]
fn idempotent_zoo() {
    for atom in scalar_atom_zoo() {
        for n in [1_usize, 2, 3, 5, 10, 25] {
            assert_self_idempotent(atom.clone(), n);
        }
    }
}

#[test]
fn single_input_passthrough_zoo() {
    for atom in scalar_atom_zoo() {
        let result = combine_default(vec![atom.clone()]);
        assert_eq!(result.len(), 1, "single-input passthrough for {atom:?}");
        assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&atom));
    }
}

#[test]
fn true_or_false_yields_bool() {
    assert_combines_to(vec![t_true(), t_false()], vec![t_bool()]);
    assert_combines_to(vec![t_false(), t_true()], vec![t_bool()]);
}

#[test]
fn bool_absorbs_true_either_order() {
    assert_combines_to(vec![t_bool(), t_true()], vec![t_bool()]);
    assert_combines_to(vec![t_true(), t_bool()], vec![t_bool()]);
}

#[test]
fn bool_absorbs_false_either_order() {
    assert_combines_to(vec![t_bool(), t_false()], vec![t_bool()]);
    assert_combines_to(vec![t_false(), t_bool()], vec![t_bool()]);
}

#[test]
fn bool_absorbs_true_and_false() {
    for inputs in [
        vec![t_bool(), t_true(), t_false()],
        vec![t_true(), t_bool(), t_false()],
        vec![t_true(), t_false(), t_bool()],
        vec![t_false(), t_true(), t_bool()],
        vec![t_false(), t_bool(), t_true()],
        vec![t_bool(), t_false(), t_true()],
    ] {
        assert_combines_to(inputs, vec![t_bool()]);
    }
}

#[test]
fn duplicated_true_collapses() {
    for n in 1..=10 {
        assert_combines_to(vec![t_true(); n], vec![t_true()]);
        assert_combines_to(vec![t_false(); n], vec![t_false()]);
        assert_combines_to(vec![t_bool(); n], vec![t_bool()]);
    }
}

#[test]
fn many_bool_variants_collapse_to_bool() {
    for inputs in [
        vec![t_bool(), t_true(), t_false(), t_bool(), t_false()],
        vec![t_true(), t_false(), t_true(), t_false()],
        vec![t_true(), t_true(), t_false()],
        vec![t_false(), t_true(), t_true()],
    ] {
        let result = combine_default(inputs);
        assert_eq!(result.len(), 1);
        assert_eq!(atomic_id_string(&result[0]), "bool");
    }
}

#[test]
fn float_absorbs_literal_float_either_order() {
    for v in [-3.14, 0.0, 1.5, 1e10] {
        assert_combines_to(vec![t_float(), t_lit_float(v)], vec![t_float()]);
        assert_combines_to(vec![t_lit_float(v), t_float()], vec![t_float()]);
    }
}

#[test]
fn distinct_literal_floats_kept_apart() {
    for vs in [vec![1.0_f64, 2.0], vec![-1.0, 0.0, 1.0], vec![1.0, 2.0, 3.0, 4.0]] {
        let inputs: Vec<TAtomic> = vs.iter().map(|&v| t_lit_float(v)).collect();
        let result = combine_default(inputs);
        assert_eq!(result.len(), vs.len());
    }
}

#[test]
fn equal_literal_floats_collapse() {
    for v in [-1.0, 0.0, 1.0, 1.5, 1e10] {
        assert_combines_to(vec![t_lit_float(v); 5], vec![t_lit_float(v)]);
    }
}

#[test]
fn int_absorbs_literal_int_either_order() {
    for v in [-1_000_000_i64, -100, -1, 0, 1, 42, 1_000_000] {
        assert_combines_to(vec![t_int(), t_lit_int(v)], vec![t_int()]);
        assert_combines_to(vec![t_lit_int(v), t_int()], vec![t_int()]);
    }
}

#[test]
fn equal_literal_ints_collapse() {
    for v in [-100, -1, 0, 1, 100] {
        assert_combines_to(vec![t_lit_int(v); 5], vec![t_lit_int(v)]);
    }
}

#[test]
fn distinct_literal_ints_kept_apart_under_threshold() {
    let inputs: Vec<TAtomic> = (1..=10).map(t_lit_int).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 10);
}

#[test]
fn string_absorbs_literal_string_either_order() {
    for s in ["", "hello", "0", "123", "Hello World"] {
        assert_combines_to(vec![t_string(), t_lit_string(s)], vec![t_string()]);
        assert_combines_to(vec![t_lit_string(s), t_string()], vec![t_string()]);
    }
}

#[test]
fn distinct_literal_strings_kept_apart() {
    let strs = ["a", "b", "c", "d", "e"];
    let inputs: Vec<TAtomic> = strs.iter().map(|s| t_lit_string(s)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 5);
}

#[test]
fn equal_literal_strings_collapse() {
    for s in ["", "hello", "world"] {
        assert_combines_to(vec![t_lit_string(s); 5], vec![t_lit_string(s)]);
    }
}

#[test]
fn numeric_absorbs_int_only_when_first() {
    assert_combines_to(vec![t_numeric(), t_int()], vec![t_numeric()]);
    assert_combines_to(vec![t_int(), t_numeric()], vec![t_int(), t_numeric()]);
}

#[test]
fn numeric_absorbs_float_only_when_first() {
    assert_combines_to(vec![t_numeric(), t_float()], vec![t_numeric()]);
    assert_combines_to(vec![t_float(), t_numeric()], vec![t_float(), t_numeric()]);
}

#[test]
fn numeric_absorbs_literal_int_only_when_first() {
    for v in [-5_i64, 0, 5, 100] {
        assert_combines_to(vec![t_numeric(), t_lit_int(v)], vec![t_numeric()]);
        assert_combines_to(vec![t_lit_int(v), t_numeric()], vec![t_lit_int(v), t_numeric()]);
    }
}

#[test]
fn numeric_absorbs_literal_float_only_when_first() {
    for v in [-1.0_f64, 0.0, 1.5, 100.0] {
        assert_combines_to(vec![t_numeric(), t_lit_float(v)], vec![t_numeric()]);
        assert_combines_to(vec![t_lit_float(v), t_numeric()], vec![t_lit_float(v), t_numeric()]);
    }
}

#[test]
fn numeric_does_not_absorb_string_either_order() {
    let result = combine_default(vec![t_numeric(), t_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["numeric", "string"]);

    let result = combine_default(vec![t_string(), t_numeric()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["numeric", "string"]);
}

#[test]
fn array_key_absorbs_int_either_order() {
    assert_combines_to(vec![t_array_key(), t_int()], vec![t_array_key()]);
    assert_combines_to(vec![t_int(), t_array_key()], vec![t_array_key()]);
}

#[test]
fn array_key_absorbs_string_either_order() {
    assert_combines_to(vec![t_array_key(), t_string()], vec![t_array_key()]);
    assert_combines_to(vec![t_string(), t_array_key()], vec![t_array_key()]);
}

#[test]
fn array_key_absorbs_literal_int_either_order() {
    for v in [-5_i64, 0, 5, 42] {
        assert_combines_to(vec![t_array_key(), t_lit_int(v)], vec![t_array_key()]);
        assert_combines_to(vec![t_lit_int(v), t_array_key()], vec![t_array_key()]);
    }
}

#[test]
fn array_key_absorbs_literal_string_either_order() {
    for s in ["a", "b", "hello", ""] {
        assert_combines_to(vec![t_array_key(), t_lit_string(s)], vec![t_array_key()]);
        assert_combines_to(vec![t_lit_string(s), t_array_key()], vec![t_array_key()]);
    }
}

#[test]
fn array_key_does_not_absorb_float() {
    let result = combine_default(vec![t_array_key(), t_float()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["array-key", "float"]);

    let result = combine_default(vec![t_float(), t_array_key()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["array-key", "float"]);
}

#[test]
fn array_key_does_not_absorb_bool() {
    let result = combine_default(vec![t_array_key(), t_bool()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["array-key", "bool"]);
}

#[test]
fn scalar_absorbs_int_either_order() {
    assert_combines_to(vec![t_scalar(), t_int()], vec![t_scalar()]);
    assert_combines_to(vec![t_int(), t_scalar()], vec![t_scalar()]);
}

#[test]
fn scalar_absorbs_string_either_order() {
    assert_combines_to(vec![t_scalar(), t_string()], vec![t_scalar()]);
    assert_combines_to(vec![t_string(), t_scalar()], vec![t_scalar()]);
}

#[test]
fn scalar_absorbs_float_either_order() {
    assert_combines_to(vec![t_scalar(), t_float()], vec![t_scalar()]);
    assert_combines_to(vec![t_float(), t_scalar()], vec![t_scalar()]);
}

#[test]
fn scalar_absorbs_numeric_only_when_numeric_first() {
    assert_combines_to(vec![t_numeric(), t_scalar()], vec![t_scalar()]);
    let result = combine_default(vec![t_scalar(), t_numeric()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["numeric", "scalar"]);
}

#[test]
fn scalar_absorbs_array_key_either_order() {
    assert_combines_to(vec![t_scalar(), t_array_key()], vec![t_scalar()]);
    assert_combines_to(vec![t_array_key(), t_scalar()], vec![t_scalar()]);
}

#[test]
fn scalar_absorbs_literals_either_order() {
    assert_combines_to(vec![t_scalar(), t_lit_int(5)], vec![t_scalar()]);
    assert_combines_to(vec![t_lit_int(5), t_scalar()], vec![t_scalar()]);
    assert_combines_to(vec![t_scalar(), t_lit_string("hi")], vec![t_scalar()]);
    assert_combines_to(vec![t_lit_string("hi"), t_scalar()], vec![t_scalar()]);
    assert_combines_to(vec![t_scalar(), t_lit_float(1.5)], vec![t_scalar()]);
    assert_combines_to(vec![t_lit_float(1.5), t_scalar()], vec![t_scalar()]);
}

#[test]
fn scalar_does_not_absorb_bool_when_scalar_first() {
    let result = combine_default(vec![t_scalar(), t_bool()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["bool", "scalar"]);
}

#[test]
fn scalar_absorbs_bool_when_bool_first() {
    assert_combines_to(vec![t_bool(), t_scalar()], vec![t_scalar()]);
    assert_combines_to(vec![t_true(), t_scalar()], vec![t_scalar()]);
    assert_combines_to(vec![t_false(), t_scalar()], vec![t_scalar()]);
}

#[test]
fn scalar_synthesised_from_string_float_bool_int() {
    let result = combine_default(vec![t_string(), t_float(), t_bool(), t_int()]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "scalar");
}

#[test]
fn scalar_not_synthesised_when_no_unspecified_int() {
    let result = combine_default(vec![t_string(), t_float(), t_bool(), t_lit_int(5)]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["bool", "float", "int(5)", "string"]);
}

#[test]
fn class_string_kept_separate_from_string() {
    let result = combine_default(vec![t_class_string(), t_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["class-string", "string"]);

    let result = combine_default(vec![t_string(), t_class_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["class-string", "string"]);
}

#[test]
fn class_string_kept_separate_from_array_key() {
    let result = combine_default(vec![t_class_string(), t_array_key()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["array-key", "class-string"]);
}

#[test]
fn class_string_kept_separate_from_scalar() {
    let result = combine_default(vec![t_class_string(), t_scalar()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["class-string", "scalar"]);
}

#[test]
fn distinct_class_like_kinds_kept_apart() {
    let result = combine_default(vec![t_class_string(), t_interface_string(), t_enum_string(), t_trait_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids.len(), 4);
}

#[test]
fn duplicated_class_like_collapses() {
    for atom in [t_class_string(), t_interface_string(), t_enum_string(), t_trait_string()] {
        assert_combines_to(vec![atom.clone(); 5], vec![atom.clone()]);
    }
}

#[test]
fn int_string_kept_separate() {
    let result = combine_default(vec![t_int(), t_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["int", "string"]);
}

#[test]
fn float_string_kept_separate() {
    let result = combine_default(vec![t_float(), t_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["float", "string"]);
}

#[test]
fn int_float_kept_separate() {
    let result = combine_default(vec![t_int(), t_float()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["float", "int"]);
}

#[test]
fn int_bool_kept_separate() {
    let result = combine_default(vec![t_int(), t_bool()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["bool", "int"]);
}

#[test]
fn float_bool_kept_separate() {
    let result = combine_default(vec![t_float(), t_bool()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["bool", "float"]);
}

#[test]
fn string_bool_kept_separate() {
    let result = combine_default(vec![t_string(), t_bool()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["bool", "string"]);
}

#[test]
fn numeric_bool_kept_separate() {
    let result = combine_default(vec![t_numeric(), t_bool()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["bool", "numeric"]);
}

#[test]
fn lit_int_lit_string_kept_separate() {
    for (i, s) in [(0_i64, "a"), (-1, "b"), (42, "hello")] {
        let result = combine_default(vec![t_lit_int(i), t_lit_string(s)]);
        assert_eq!(result.len(), 2);
    }
}

#[test]
fn lit_int_lit_float_kept_separate() {
    let result = combine_default(vec![t_lit_int(1), t_lit_float(1.5)]);
    assert_eq!(result.len(), 2);
}

#[test]
fn lit_string_lit_float_kept_separate() {
    let result = combine_default(vec![t_lit_string("a"), t_lit_float(1.5)]);
    assert_eq!(result.len(), 2);
}

#[test]
fn many_lit_int_with_int_collapses() {
    let mut inputs = vec![t_int()];
    for i in 0..50 {
        inputs.push(t_lit_int(i));
    }
    assert_combines_to(inputs, vec![t_int()]);
}

#[test]
fn many_lit_string_with_string_collapses() {
    let mut inputs = vec![t_string()];
    for i in 0..30 {
        inputs.push(t_lit_string(&format!("s{i}")));
    }
    assert_combines_to(inputs, vec![t_string()]);
}

#[test]
fn many_lit_float_with_float_collapses() {
    let mut inputs = vec![t_float()];
    for i in 0..20 {
        inputs.push(t_lit_float(f64::from(i)));
    }
    assert_combines_to(inputs, vec![t_float()]);
}

#[test]
fn big_zoo_singleton_passthrough() {
    for atom in scalar_atom_zoo() {
        let r = combine_default(vec![atom.clone()]);
        assert_eq!(r.len(), 1);
        assert_eq!(atomic_id_string(&r[0]), atomic_id_string(&atom));
    }
}

#[test]
fn big_zoo_self_dedup() {
    for atom in scalar_atom_zoo() {
        for n in 2..=8 {
            let r = combine_default(vec![atom.clone(); n]);
            assert_eq!(r.len(), 1, "self-dedup failed for {atom:?} (n={n})");
            assert_eq!(atomic_id_string(&r[0]), atomic_id_string(&atom));
        }
    }
}

fn scalar_atom_zoo() -> Vec<TAtomic> {
    vec![
        t_bool(),
        t_true(),
        t_false(),
        t_int(),
        t_lit_int(0),
        t_lit_int(42),
        t_lit_int(-1),
        t_positive_int(),
        t_negative_int(),
        t_non_negative_int(),
        t_non_positive_int(),
        t_int_range(0, 10),
        t_int_range(-5, 5),
        t_int_from(100),
        t_int_to(-100),
        t_int_unspec_lit(),
        t_string(),
        t_lit_string(""),
        t_lit_string("hi"),
        t_lit_string("0"),
        t_non_empty_string(),
        t_numeric_string(),
        t_lower_string(),
        t_upper_string(),
        t_truthy_string(),
        t_callable_string(),
        t_unspec_lit_string(false),
        t_unspec_lit_string(true),
        t_class_string(),
        t_interface_string(),
        t_enum_string(),
        t_trait_string(),
        t_lit_class_string("Foo"),
        t_float(),
        t_lit_float(0.0),
        t_lit_float(1.5),
        t_lit_float(-3.14),
        t_unspec_lit_float(),
        t_numeric(),
        t_array_key(),
        t_scalar(),
    ]
}
