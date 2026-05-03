mod comparator_common;

use comparator_common::*;

#[test]
fn singleton_union_reflexive() {
    for atom in [t_int(), t_string(), t_bool(), t_float(), null(), mixed(), t_object_any()] {
        let cb = empty_codebase();
        let union = u(atom);
        assert!(is_contained(&union, &union, &cb));
    }
}

#[test]
fn int_in_int_or_string() {
    assert_subtype(&u(t_int()), &u_many(vec![t_int(), t_string()]));
}

#[test]
fn string_in_int_or_string() {
    assert_subtype(&u(t_string()), &u_many(vec![t_int(), t_string()]));
}

#[test]
fn float_not_in_int_or_string() {
    assert_not_subtype(&u(t_float()), &u_many(vec![t_int(), t_string()]));
}

#[test]
fn int_or_string_in_int_or_string() {
    assert_subtype(&u_many(vec![t_int(), t_string()]), &u_many(vec![t_int(), t_string()]));
}

#[test]
fn int_or_string_not_in_int() {
    assert_not_subtype(&u_many(vec![t_int(), t_string()]), &u(t_int()));
}

#[test]
fn int_or_string_in_int_or_string_or_float() {
    assert_subtype(&u_many(vec![t_int(), t_string()]), &u_many(vec![t_int(), t_string(), t_float()]));
}

#[test]
fn lit_int_in_int_or_string() {
    for v in [-100i64, 0, 1, 100] {
        assert_subtype(&u(t_lit_int(v)), &u_many(vec![t_int(), t_string()]));
    }
}

#[test]
fn lit_string_in_int_or_string() {
    for s in ["a", "hi", ""] {
        assert_subtype(&u(t_lit_string(s)), &u_many(vec![t_int(), t_string()]));
    }
}

#[test]
fn nullable_int_contains_int_and_null() {
    let nullable_int = u_many(vec![t_int(), null()]);
    assert_subtype(&u(t_int()), &nullable_int);
    assert_subtype(&u(null()), &nullable_int);
    assert_subtype(&u(t_lit_int(5)), &nullable_int);
}

#[test]
fn nullable_int_does_not_contain_string() {
    let nullable_int = u_many(vec![t_int(), null()]);
    assert_not_subtype(&u(t_string()), &nullable_int);
    assert_not_subtype(&u(t_bool()), &nullable_int);
}

#[test]
fn never_in_any_union() {
    let unions =
        [u(t_int()), u_many(vec![t_int(), t_string()]), u_many(vec![t_int(), null()]), u(mixed()), u(t_object_any())];
    for c in unions {
        assert_subtype(&u(never()), &c);
    }
}

#[test]
fn anything_in_mixed_union() {
    let mixed_u = u(mixed());
    for atom in [t_int(), t_string(), t_float(), t_bool(), null(), t_object_any(), t_resource()] {
        assert_subtype(&u(atom), &mixed_u);
    }
}

#[test]
fn three_way_union_membership() {
    let container = u_many(vec![t_int(), t_string(), null()]);
    assert_subtype(&u(t_int()), &container);
    assert_subtype(&u(t_string()), &container);
    assert_subtype(&u(null()), &container);
    assert_not_subtype(&u(t_float()), &container);
    assert_not_subtype(&u(t_bool()), &container);
    assert_not_subtype(&u(t_object_any()), &container);
}

#[test]
fn order_independent_unions() {
    let int_string = u_many(vec![t_int(), t_string()]);
    let string_int = u_many(vec![t_string(), t_int()]);
    assert_subtype(&int_string, &string_int);
    assert_subtype(&string_int, &int_string);
}

#[test]
fn union_with_three_atoms_subtypes() {
    let small = u_many(vec![t_int(), t_string()]);
    let big = u_many(vec![t_int(), t_string(), t_float()]);
    assert_subtype(&small, &big);
    assert_not_subtype(&big, &small);
}

#[test]
fn union_with_lit_subtypes_general() {
    let lits = u_many(vec![t_lit_int(1), t_lit_int(2), t_lit_int(3)]);
    assert_subtype(&lits, &u(t_int()));
}

#[test]
fn union_string_lits_subtypes_string() {
    let lits = u_many(vec![t_lit_string("a"), t_lit_string("b")]);
    assert_subtype(&lits, &u(t_string()));
}

#[test]
fn ignore_null_flag_skips_null_in_input() {
    let cb = empty_codebase();
    let nullable_int = u_many(vec![t_int(), null()]);
    let int_only = u(t_int());
    // Without ignore_null, nullable_int is NOT a subtype of int.
    assert!(!is_contained_with(&nullable_int, &int_only, &cb, false, false, false));
    // With ignore_null, the null in input is skipped, so nullable_int IS a subtype.
    assert!(is_contained_with(&nullable_int, &int_only, &cb, true, false, false));
}

#[test]
fn ignore_false_flag_skips_false_in_input() {
    let cb = empty_codebase();
    let int_or_false = u_many(vec![t_int(), t_false()]);
    let int_only = u(t_int());
    assert!(!is_contained_with(&int_or_false, &int_only, &cb, false, false, false));
    assert!(is_contained_with(&int_or_false, &int_only, &cb, false, true, false));
}

#[test]
fn many_lits_in_int() {
    let lits: Vec<_> = (0..20i64).map(t_lit_int).collect();
    let union = u_many(lits);
    assert_subtype(&union, &u(t_int()));
}

#[test]
fn many_lits_in_array_key() {
    let mut lits = vec![];
    for i in 0..15i64 {
        lits.push(t_lit_int(i));
    }
    for s in ["a", "b", "c", "d", "e"] {
        lits.push(t_lit_string(s));
    }
    let union = u_many(lits);
    assert_subtype(&union, &u(t_array_key()));
}

#[test]
fn nullable_string_in_nullable_array_key() {
    let nullable_string = u_many(vec![t_string(), null()]);
    let nullable_arraykey = u_many(vec![t_array_key(), null()]);
    assert_subtype(&nullable_string, &nullable_arraykey);
}
