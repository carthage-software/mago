mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;

#[test]
fn int_absorbs_every_literal_in_minus_500_to_500() {
    fixture(|f| {
        for v in -500..=500i64 {
            assert_combines_to(f, vec![f.t_int(), f.t_lit_int(v)], vec![f.t_int()]);
            assert_combines_to(f, vec![f.t_lit_int(v), f.t_int()], vec![f.t_int()]);
        }
    });
}

#[test]
fn unspec_int_absorbs_lit_unspec_lit() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_int(), f.t_int_unspec_lit()], vec![f.t_int()]);
        assert_combines_to(f, vec![f.t_int_unspec_lit(), f.t_int()], vec![f.t_int()]);
    });
}

#[test]
fn lit_int_self_dedup_for_many_values() {
    fixture(|f| {
        for v in -50..=50i64 {
            for n in 2..=5 {
                let result = combine_default(f, vec![f.t_lit_int(v); n]);
                assert_eq!(result.len(), 1);
            }
        }
    });
}

#[test]
fn non_adjacent_lit_pairs_kept_apart() {
    fixture(|f| {
        for a in -10..=10i64 {
            for b in -10..=10i64 {
                if a == b || (a - b).abs() == 1 {
                    continue;
                }
                let result = combine_default(f, vec![f.t_lit_int(a), f.t_lit_int(b)]);
                assert_eq!(result.len(), 2, "{a} | {b}");
            }
        }
    });
}

#[test]
fn adjacent_lit_pairs_merge_to_range() {
    fixture(|f| {
        for a in -5..=5i64 {
            let b = a + 1;
            let range = f.t_int_range(a, b);
            let result = combine_default(f, vec![f.t_lit_int(a), f.t_lit_int(b)]);
            assert_eq!(result, vec![range], "{a} | {b}");
        }
    });
}

#[test]
fn ranges_with_overlaps_collapse() {
    fixture(|f| {
        for low in -5..=5i64 {
            let first = f.t_int_range(low, low + 3);
            let second = f.t_int_range(low + 1, low + 5);
            let merged = f.t_int_range(low, low + 5);
            assert_combines_to(f, vec![first, second], vec![merged]);
        }
    });
}

#[test]
fn from_n_with_lit_n_minus_1_extends_for_many_n() {
    fixture(|f| {
        for n in -5..=5i64 {
            let from_n = f.t_int_from(n);
            let predecessor = f.t_lit_int(n - 1);
            let extended = f.t_int_from(n - 1);
            assert_combines_to(f, vec![from_n, predecessor], vec![extended]);
        }
    });
}

#[test]
fn to_n_with_lit_n_plus_1_extends_for_many_n() {
    fixture(|f| {
        for n in -5..=5i64 {
            let to_n = f.t_int_to(n);
            let successor = f.t_lit_int(n + 1);
            let extended = f.t_int_to(n + 1);
            assert_combines_to(f, vec![to_n, successor], vec![extended]);
        }
    });
}

#[test]
fn string_absorbs_many_literals() {
    fixture(|f| {
        for i in 0..200 {
            let s = format!("test_{i}");
            let literal = f.t_lit_string(&s);
            assert_combines_to(f, vec![f.t_string(), literal], vec![f.t_string()]);
            assert_combines_to(f, vec![literal, f.t_string()], vec![f.t_string()]);
        }
    });
}

#[test]
fn lit_string_self_dedup_many_values() {
    fixture(|f| {
        for i in 0..50 {
            let s = format!("v_{i}");
            let literal = f.t_lit_string(&s);
            for n in 2..=5 {
                let result = combine_default(f, vec![literal; n]);
                assert_eq!(result.len(), 1);
            }
        }
    });
}

#[test]
fn distinct_lit_string_pairs_kept_apart() {
    fixture(|f| {
        for i in 0..30 {
            for j in (i + 1)..30 {
                let a = format!("a_{i}");
                let b = format!("b_{j}");
                let first = f.t_lit_string(&a);
                let second = f.t_lit_string(&b);
                let result = combine_default(f, vec![first, second]);
                assert_eq!(result.len(), 2);
            }
        }
    });
}

#[test]
fn float_absorbs_many_literals() {
    fixture(|f| {
        for i in 0..200 {
            let v = f64::from(i).mul_add(0.5, -50.0);
            assert_combines_to(f, vec![f.t_float(), f.t_lit_float(v)], vec![f.t_float()]);
            assert_combines_to(f, vec![f.t_lit_float(v), f.t_float()], vec![f.t_float()]);
        }
    });
}

#[test]
fn lit_float_self_dedup_many_values() {
    fixture(|f| {
        for i in 0..50 {
            let v = f64::from(i) * 0.25;
            for n in 2..=5 {
                assert_combines_to(f, vec![f.t_lit_float(v); n], vec![f.t_lit_float(v)]);
            }
        }
    });
}

#[test]
fn all_bool_triples_collapse() {
    fixture(|f| {
        let bools = [f.t_true(), f.t_false(), f.t_bool()];
        let valid = [f.t_true(), f.t_false(), f.t_bool()];
        for a in &bools {
            for b in &bools {
                for c in &bools {
                    let result = combine_default(f, vec![*a, *b, *c]);
                    assert_eq!(result.len(), 1, "{a:?} {b:?} {c:?}");
                    assert!(valid.contains(&result[0]), "got {:?}", result[0]);
                }
            }
        }
    });
}

#[test]
fn many_distinct_named_objects_kept_apart() {
    fixture(|f| {
        for n in [3usize, 5, 10, 20, 50] {
            let inputs: Vec<Atom<'_>> = (0..n).map(|i| f.t_named(&format!("Class{i}"))).collect();
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), n);
        }
    });
}

#[test]
fn same_named_with_many_copies_collapses() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        for n in 1..=20 {
            assert_combines_to(f, vec![foo; n], vec![foo]);
        }
    });
}

#[test]
fn object_any_absorbs_many_named() {
    fixture(|f| {
        for n in 1..=20 {
            let mut inputs = vec![f.t_object_any()];
            for i in 0..n {
                inputs.push(f.t_named(&format!("C{i}")));
            }
            assert_combines_to(f, inputs, vec![f.t_object_any()]);
        }
    });
}

#[test]
fn generic_with_n_distinct_int_params_kept_separate() {
    fixture(|f| {
        for n in [3usize, 5, 10] {
            let containers: Vec<_> = (0..n)
                .map(|i| {
                    let argument = f.ui(i as i64);
                    f.t_generic_named("Box", vec![argument])
                })
                .collect();
            let result = combine_default(f, containers);
            assert_eq!(result.len(), n, "n={n}");
        }
    });
}

#[test]
fn generic_with_int_and_string_param_kept_separate() {
    fixture(|f| {
        let int_argument = f.u(f.t_int());
        let string_argument = f.u(f.t_string());
        let box_int = f.t_generic_named("Box", vec![int_argument]);
        let box_string = f.t_generic_named("Box", vec![string_argument]);
        let result = combine_default(f, vec![box_int, box_string]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&box_int));
        assert!(result.contains(&box_string));
    });
}

#[test]
fn many_distinct_enums_kept_apart() {
    fixture(|f| {
        for n in [3usize, 5, 10, 20] {
            let inputs: Vec<Atom<'_>> = (0..n).map(|i| f.t_enum(&format!("E{i}"))).collect();
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), n);
        }
    });
}

#[test]
fn many_distinct_enum_cases_kept_apart() {
    fixture(|f| {
        for n in [3usize, 5, 10] {
            let inputs: Vec<Atom<'_>> = (0..n).map(|i| f.t_enum_case("E", &format!("Case{i}"))).collect();
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), n);
        }
    });
}

#[test]
fn many_copies_of_simple_atoms_collapse() {
    fixture(|f| {
        let atoms = [
            f.t_int(),
            f.t_string(),
            f.t_float(),
            f.t_bool(),
            f.t_true(),
            f.t_false(),
            f.null(),
            f.never(),
            f.t_object_any(),
            f.t_named("Foo"),
            f.t_resource(),
            f.t_open_resource(),
            f.t_closed_resource(),
            f.t_empty_array(),
        ];
        for atom in &atoms {
            for n in [2usize, 5, 10, 20, 50, 100] {
                let result = combine_default(f, vec![*atom; n]);
                assert_eq!(result.len(), 1, "{n} copies of {atom:?}");
            }
        }
    });
}

#[test]
fn three_way_stable_primitives_consistent() {
    fixture(|f| {
        let stable = [
            f.t_int(),
            f.t_string(),
            f.t_float(),
            f.t_bool(),
            f.null(),
            f.t_object_any(),
            f.t_named("X"),
            f.t_resource(),
        ];
        for a in &stable {
            for b in &stable {
                for c in &stable {
                    let result = combine_default(f, vec![*a, *b, *c]);
                    let result_reversed = combine_default(f, vec![*c, *b, *a]);
                    assert_multiset_eq(&result, &result_reversed);
                }
            }
        }
    });
}

#[test]
fn mixed_dominates_every_simple_atom() {
    fixture(|f| {
        let atoms = [
            f.t_int(),
            f.t_string(),
            f.t_float(),
            f.t_bool(),
            f.t_true(),
            f.t_false(),
            f.null(),
            f.void(),
            f.never(),
            f.t_object_any(),
            f.t_named("Foo"),
            f.t_enum("E"),
            f.t_resource(),
            f.t_empty_array(),
            f.t_array_key(),
            f.t_numeric(),
            f.t_scalar(),
            f.t_class_string(),
        ];
        for atom in &atoms {
            let result = combine_default(f, vec![*atom, f.mixed()]);
            let result_reversed = combine_default(f, vec![f.mixed(), *atom]);
            assert_eq!(result.len(), 1);
            assert_eq!(result_reversed.len(), 1);
            assert_eq!(result[0].kind(), AtomKind::Mixed);
            assert_eq!(result_reversed[0].kind(), AtomKind::Mixed);
        }
    });
}

#[test]
fn never_absorbed_by_every_non_void_atom() {
    fixture(|f| {
        let never = f.never();
        let atoms = [
            f.t_int(),
            f.t_string(),
            f.t_float(),
            f.t_bool(),
            f.t_true(),
            f.t_false(),
            f.null(),
            f.t_object_any(),
            f.t_named("Foo"),
            f.t_enum("E"),
            f.t_resource(),
            f.t_open_resource(),
            f.t_closed_resource(),
            f.t_empty_array(),
            f.t_array_key(),
            f.t_numeric(),
            f.t_scalar(),
            f.t_class_string(),
            f.t_lit_int(0),
            f.t_lit_string("x"),
            f.t_lit_float(1.0),
        ];
        for atom in &atoms {
            let result = combine_default(f, vec![*atom, never]);
            let result_reversed = combine_default(f, vec![never, *atom]);
            assert!(result.iter().all(|candidate| *candidate != never), "never leaked: {atom:?}");
            assert!(result_reversed.iter().all(|candidate| *candidate != never), "never leaked rev: {atom:?}");
        }
    });
}

#[test]
fn many_named_with_lits() {
    fixture(|f| {
        for n in 1..=10 {
            let mut inputs = vec![];
            for i in 0..n {
                inputs.push(f.t_named(&format!("C{i}")));
                inputs.push(f.t_lit_int(i as i64));
            }
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), n + 1, "adjacent literals merge into one range; {n} named objects stay distinct");
        }
    });
}

#[test]
fn alternating_int_string_collapses_to_two() {
    fixture(|f| {
        for n in 1..=20 {
            let mut inputs = Vec::new();
            for _ in 0..n {
                inputs.push(f.t_int());
                inputs.push(f.t_string());
            }
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), 2);
        }
    });
}

#[test]
fn alternating_named_collapses() {
    fixture(|f| {
        for n in 1..=15 {
            let mut inputs = Vec::new();
            for _ in 0..n {
                inputs.push(f.t_named("A"));
                inputs.push(f.t_named("B"));
            }
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), 2);
        }
    });
}

#[test]
fn n_copies_plus_adjacent_int_merges_to_range() {
    fixture(|f| {
        for n in [1usize, 5, 10, 50, 100] {
            let mut inputs: Vec<Atom<'_>> = std::iter::repeat_with(|| f.t_lit_int(0)).take(n).collect();
            inputs.push(f.t_lit_int(1));
            let range = f.t_int_range(0, 1);
            let result = combine_default(f, inputs);
            assert_eq!(result, vec![range], "n={n}");
        }
    });
}
