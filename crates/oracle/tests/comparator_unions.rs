mod common;

use common::*;

#[test]
fn singleton_union_reflexive() {
    fixture(|f| {
        for atom in [f.t_int(), f.t_string(), f.t_bool(), f.t_float(), f.null(), f.mixed(), f.t_object_any()] {
            let symbols = empty_symbol_table(f.arena);
            let union = f.u(atom);
            assert!(is_contained(f, union, union, &symbols));
        }
    });
}

#[test]
fn int_in_int_or_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let input = f.u(int);
        let container = f.u_many(vec![int, string]);
        assert_subtype(f, input, container);
    });
}

#[test]
fn string_in_int_or_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let input = f.u(string);
        let container = f.u_many(vec![int, string]);
        assert_subtype(f, input, container);
    });
}

#[test]
fn float_not_in_int_or_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let float = f.t_float();
        let input = f.u(float);
        let container = f.u_many(vec![int, string]);
        assert_not_subtype(f, input, container);
    });
}

#[test]
fn int_or_string_in_int_or_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let input = f.u_many(vec![int, string]);
        let container = f.u_many(vec![int, string]);
        assert_subtype(f, input, container);
    });
}

#[test]
fn int_or_string_not_in_int() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let input = f.u_many(vec![int, string]);
        let container = f.u(int);
        assert_not_subtype(f, input, container);
    });
}

#[test]
fn int_or_string_in_int_or_string_or_float() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let float = f.t_float();
        let input = f.u_many(vec![int, string]);
        let container = f.u_many(vec![int, string, float]);
        assert_subtype(f, input, container);
    });
}

#[test]
fn lit_int_in_int_or_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let container = f.u_many(vec![int, string]);
        for value in [-100i64, 0, 1, 100] {
            let input = f.ui(value);
            assert_subtype(f, input, container);
        }
    });
}

#[test]
fn lit_string_in_int_or_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let container = f.u_many(vec![int, string]);
        for text in ["a", "hi", ""] {
            let input = f.us(text);
            assert_subtype(f, input, container);
        }
    });
}

#[test]
fn nullable_int_contains_int_and_null() {
    fixture(|f| {
        let int = f.t_int();
        let null = f.null();
        let nullable_int = f.u_many(vec![int, null]);
        let int_type = f.u(int);
        assert_subtype(f, int_type, nullable_int);
        let null_type = f.u(null);
        assert_subtype(f, null_type, nullable_int);
        let five = f.ui(5);
        assert_subtype(f, five, nullable_int);
    });
}

#[test]
fn nullable_int_does_not_contain_string() {
    fixture(|f| {
        let int = f.t_int();
        let null = f.null();
        let nullable_int = f.u_many(vec![int, null]);
        let string = f.t_string();
        let string_type = f.u(string);
        assert_not_subtype(f, string_type, nullable_int);
        let bool_atom = f.t_bool();
        let bool_type = f.u(bool_atom);
        assert_not_subtype(f, bool_type, nullable_int);
    });
}

#[test]
fn never_in_any_union() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        let mixed = f.mixed();
        let object = f.t_object_any();
        let unions = [f.u(int), f.u_many(vec![int, string]), f.u_many(vec![int, null]), f.u(mixed), f.u(object)];
        let never = f.never();
        let never_type = f.u(never);
        for container in unions {
            assert_subtype(f, never_type, container);
        }
    });
}

#[test]
fn anything_in_mixed_union() {
    fixture(|f| {
        let mixed = f.mixed();
        let mixed_union = f.u(mixed);
        for atom in [f.t_int(), f.t_string(), f.t_float(), f.t_bool(), f.null(), f.t_object_any(), f.t_resource()] {
            let input = f.u(atom);
            assert_subtype(f, input, mixed_union);
        }
    });
}

#[test]
fn three_way_union_membership() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        let container = f.u_many(vec![int, string, null]);
        let int_type = f.u(int);
        assert_subtype(f, int_type, container);
        let string_type = f.u(string);
        assert_subtype(f, string_type, container);
        let null_type = f.u(null);
        assert_subtype(f, null_type, container);
        let float = f.t_float();
        let float_type = f.u(float);
        assert_not_subtype(f, float_type, container);
        let bool_atom = f.t_bool();
        let bool_type = f.u(bool_atom);
        assert_not_subtype(f, bool_type, container);
        let object = f.t_object_any();
        let object_type = f.u(object);
        assert_not_subtype(f, object_type, container);
    });
}

#[test]
fn order_independent_unions() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_string = f.u_many(vec![int, string]);
        let string_int = f.u_many(vec![string, int]);
        assert_subtype(f, int_string, string_int);
        assert_subtype(f, string_int, int_string);
    });
}

#[test]
fn union_with_three_atoms_subtypes() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let float = f.t_float();
        let small = f.u_many(vec![int, string]);
        let big = f.u_many(vec![int, string, float]);
        assert_subtype(f, small, big);
        assert_not_subtype(f, big, small);
    });
}

#[test]
fn union_with_lit_subtypes_general() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        let three = f.t_lit_int(3);
        let literals = f.u_many(vec![one, two, three]);
        let int = f.t_int();
        let int_type = f.u(int);
        assert_subtype(f, literals, int_type);
    });
}

#[test]
fn union_string_lits_subtypes_string() {
    fixture(|f| {
        let left = f.t_lit_string("a");
        let right = f.t_lit_string("b");
        let literals = f.u_many(vec![left, right]);
        let string = f.t_string();
        let string_type = f.u(string);
        assert_subtype(f, literals, string_type);
    });
}

#[test]
fn ignore_null_flag_skips_null_in_input() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let null = f.null();
        let nullable_int = f.u_many(vec![int, null]);
        let int_only = f.u(int);
        assert!(!is_contained_with(f, nullable_int, int_only, &symbols, false, false, false));
        assert!(is_contained_with(f, nullable_int, int_only, &symbols, true, false, false));
    });
}

#[test]
fn ignore_false_flag_skips_false_in_input() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let false_atom = f.t_false();
        let int_or_false = f.u_many(vec![int, false_atom]);
        let int_only = f.u(int);
        assert!(!is_contained_with(f, int_or_false, int_only, &symbols, false, false, false));
        assert!(is_contained_with(f, int_or_false, int_only, &symbols, false, true, false));
    });
}

#[test]
fn many_lits_in_int() {
    fixture(|f| {
        let literals: Vec<_> = (0..20i64).map(|value| f.t_lit_int(value)).collect();
        let union = f.u_many(literals);
        let int = f.t_int();
        let int_type = f.u(int);
        assert_subtype(f, union, int_type);
    });
}

#[test]
fn many_lits_in_array_key() {
    fixture(|f| {
        let mut literals = vec![];
        for value in 0..15i64 {
            literals.push(f.t_lit_int(value));
        }
        for text in ["a", "b", "c", "d", "e"] {
            literals.push(f.t_lit_string(text));
        }
        let union = f.u_many(literals);
        let array_key = f.t_array_key();
        let array_key_type = f.u(array_key);
        assert_subtype(f, union, array_key_type);
    });
}

#[test]
fn nullable_string_in_nullable_array_key() {
    fixture(|f| {
        let string = f.t_string();
        let null = f.null();
        let array_key = f.t_array_key();
        let nullable_string = f.u_many(vec![string, null]);
        let nullable_array_key = f.u_many(vec![array_key, null]);
        assert_subtype(f, nullable_string, nullable_array_key);
    });
}
