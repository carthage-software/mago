mod common;

use common::*;

use std::collections::BTreeMap;

use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::join;
use mago_oracle::ty::join::JoinOptions;
use mago_oracle::ty::well_known;

fn t_array_with_items<'arena>(f: &mut Fixture<'_, 'arena>, items: &[(ArrayKey<'arena>, Type<'arena>)]) -> Atom<'arena> {
    let mut entries: BTreeMap<ArrayKey<'arena>, (bool, Type<'arena>)> = BTreeMap::new();
    for (key, value) in items {
        entries.insert(*key, (false, *value));
    }

    f.t_keyed_sealed(entries, false)
}

fn t_array_with_params<'arena>(f: &mut Fixture<'_, 'arena>, key: Type<'arena>, value: Type<'arena>) -> Atom<'arena> {
    f.t_keyed_unsealed(key, value, false)
}

#[test]
fn overwrite_empty_array_drops_when_other_array_present() {
    fixture(|f| {
        let string = f.u(f.t_string());
        let int = f.u(f.t_int());
        let other = t_array_with_params(f, string, int);
        let options = JoinOptions::structural().with_overwrite_empty_array(true);
        let out = join::compute_with(&[f.t_empty_array(), other], &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], other);
    });
}

#[test]
fn overwrite_empty_array_keeps_when_alone() {
    fixture(|f| {
        let options = JoinOptions::structural().with_overwrite_empty_array(true);
        let out = join::compute_with(&[f.t_empty_array()], &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out, vec![f.t_empty_array()]);
    });
}

#[test]
fn overwrite_empty_array_off_keeps_both() {
    fixture(|f| {
        let string = f.u(f.t_string());
        let int = f.u(f.t_int());
        let other = t_array_with_params(f, string, int);
        let out = join::compute_with(&[f.t_empty_array(), other], &JoinOptions::structural(), &mut f.builder)
            .as_slice()
            .to_vec();
        let mut sorted = out;
        sorted.sort_unstable();
        let mut expected = vec![f.t_empty_array(), other];
        expected.sort_unstable();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn string_literal_collapse_fires_above_threshold() {
    fixture(|f| {
        let literals = (0..5).map(|n| f.t_lit_string(&format!("s{n}"))).collect::<Vec<_>>();
        let options = JoinOptions::structural().with_string_literal_collapse_threshold(3);
        let out = join::compute_with(&literals, &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out, vec![well_known::STRING]);
    });
}

#[test]
fn string_literal_collapse_at_or_below_threshold_keeps_literals() {
    fixture(|f| {
        let literals = (0..3).map(|n| f.t_lit_string(&format!("s{n}"))).collect::<Vec<_>>();
        let options = JoinOptions::structural().with_string_literal_collapse_threshold(3);
        let out = join::compute_with(&literals, &options, &mut f.builder).as_slice().to_vec();
        let mut sorted = out;
        sorted.sort_unstable();
        let mut expected = literals;
        expected.sort_unstable();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn merge_int_ranges_collapses_consecutive_literals() {
    fixture(|f| {
        let options = JoinOptions::structural().with_merge_int_ranges(true);
        let out = join::compute_with(
            &[f.t_lit_int(0), f.t_lit_int(1), f.t_lit_int(2), f.t_lit_int(3)],
            &options,
            &mut f.builder,
        )
        .as_slice()
        .to_vec();
        assert_eq!(out, vec![f.t_int_range(0, 3)]);
    });
}

#[test]
fn merge_int_ranges_with_gap_keeps_separate() {
    fixture(|f| {
        let options = JoinOptions::structural().with_merge_int_ranges(true);
        let out = join::compute_with(&[f.t_lit_int(0), f.t_lit_int(1), f.t_lit_int(5)], &options, &mut f.builder)
            .as_slice()
            .to_vec();
        let mut sorted = out;
        sorted.sort_unstable();
        let mut expected = vec![f.t_int_range(0, 1), f.t_lit_int(5)];
        expected.sort_unstable();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn merge_int_ranges_combines_overlapping_ranges() {
    fixture(|f| {
        let options = JoinOptions::structural().with_merge_int_ranges(true);
        let zero_to_ten = f.t_int_range(0, 10);
        let five_to_fifteen = f.t_int_range(5, 15);
        let out = join::compute_with(&[zero_to_ten, five_to_fifteen], &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out, vec![f.t_int_range(0, 15)]);
    });
}

#[test]
fn merge_int_ranges_off_keeps_separate() {
    fixture(|f| {
        let out = join::compute_with(&[f.t_lit_int(0), f.t_lit_int(1)], &JoinOptions::structural(), &mut f.builder)
            .as_slice()
            .to_vec();
        let mut sorted = out;
        sorted.sort_unstable();
        let mut expected = vec![f.t_lit_int(0), f.t_lit_int(1)];
        expected.sort_unstable();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn rewrite_int_keyed_to_list_converts_contiguous_indices() {
    fixture(|f| {
        let int = f.u(f.t_int());
        let string = f.u(f.t_string());
        let array = t_array_with_items(f, &[(f.ak_int(0), int), (f.ak_int(1), string)]);
        let options = JoinOptions::structural().with_rewrite_int_keyed_to_list(true);
        let out = join::compute_with(&[array], &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind(), AtomKind::List);
    });
}

#[test]
fn rewrite_int_keyed_to_list_skips_non_contiguous() {
    fixture(|f| {
        let int = f.u(f.t_int());
        let string = f.u(f.t_string());
        let array = t_array_with_items(f, &[(f.ak_int(0), int), (f.ak_int(5), string)]);
        let options = JoinOptions::structural().with_rewrite_int_keyed_to_list(true);
        let out = join::compute_with(&[array], &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out, vec![array]);
    });
}

#[test]
fn rewrite_int_keyed_to_list_skips_string_keys() {
    fixture(|f| {
        let int = f.u(f.t_int());
        let name_key = f.ak_str("name");
        let array = t_array_with_items(f, &[(name_key, int)]);
        let options = JoinOptions::structural().with_rewrite_int_keyed_to_list(true);
        let out = join::compute_with(&[array], &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out, vec![array]);
    });
}

#[test]
fn merge_array_shapes_combines_overlapping_keys() {
    fixture(|f| {
        let int = f.u(f.t_int());
        let string = f.u(f.t_string());
        let key = f.ak_str("k");
        let a = t_array_with_items(f, &[(key, int)]);
        let b = t_array_with_items(f, &[(key, string)]);
        let options = JoinOptions::structural().with_merge_array_shapes(true);
        let out = join::compute_with(&[a, b], &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out.len(), 1);
        let Atom::Array(merged) = out[0] else {
            panic!("expected a keyed array, got {:?}", out[0]);
        };
        let Some(entries) = merged.known_items else {
            panic!("merged array must have known items");
        };
        assert_eq!(entries.len(), 1);
        let value_atoms = entries[0].value.atoms;
        assert!(value_atoms.contains(&f.t_int()));
        assert!(value_atoms.contains(&f.t_string()));
    });
}

#[test]
fn merge_array_shapes_skips_disjoint_keys() {
    fixture(|f| {
        let int = f.u(f.t_int());
        let string = f.u(f.t_string());
        let first_key = f.ak_str("k1");
        let second_key = f.ak_str("k2");
        let a = t_array_with_items(f, &[(first_key, int)]);
        let b = t_array_with_items(f, &[(second_key, string)]);
        let options = JoinOptions::structural().with_merge_array_shapes(true);
        let out = join::compute_with(&[a, b], &options, &mut f.builder).as_slice().to_vec();
        assert_eq!(out.len(), 2);
    });
}

#[test]
fn merge_array_shapes_off_keeps_separate() {
    fixture(|f| {
        let int = f.u(f.t_int());
        let string = f.u(f.t_string());
        let key = f.ak_str("k");
        let a = t_array_with_items(f, &[(key, int)]);
        let b = t_array_with_items(f, &[(key, string)]);
        let out = join::compute_with(&[a, b], &JoinOptions::structural(), &mut f.builder).as_slice().to_vec();
        assert_eq!(out.len(), 2);
    });
}

#[test]
fn default_options_match_compute() {
    fixture(|f| {
        let elements = vec![f.t_int(), f.t_string(), f.t_lit_int(42)];
        let a = join::compute(&elements, &mut f.builder).as_slice().to_vec();
        let b = join::compute_with(&elements, &JoinOptions::default(), &mut f.builder).as_slice().to_vec();
        assert_eq!(a, b);
    });
}
