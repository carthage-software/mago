mod common;

use common::*;

use mago_oracle::ty::Atom;

type Case<'arena> = (&'static str, Vec<Atom<'arena>>, Vec<Atom<'arena>>);

#[track_caller]
fn run_cases<'arena>(f: &mut Fixture<'_, 'arena>, cases: Vec<Case<'arena>>) {
    for (label, input, expected) in cases {
        let result = combine_default(f, input);
        let mut actual = result.clone();
        actual.sort_unstable();
        let mut expected_sorted = expected.clone();
        expected_sorted.sort_unstable();
        assert_eq!(actual, expected_sorted, "case `{label}` failed: got {result:?}, expected {expected:?}");
    }
}

#[test]
fn bool_cases() {
    fixture(|f| {
        let cases = vec![
            ("true", vec![f.t_true()], vec![f.t_true()]),
            ("false", vec![f.t_false()], vec![f.t_false()]),
            ("bool", vec![f.t_bool()], vec![f.t_bool()]),
            ("true | true", vec![f.t_true(), f.t_true()], vec![f.t_true()]),
            ("false | false", vec![f.t_false(), f.t_false()], vec![f.t_false()]),
            ("bool | bool", vec![f.t_bool(), f.t_bool()], vec![f.t_bool()]),
            ("true | false", vec![f.t_true(), f.t_false()], vec![f.t_bool()]),
            ("false | true", vec![f.t_false(), f.t_true()], vec![f.t_bool()]),
            ("bool | true", vec![f.t_bool(), f.t_true()], vec![f.t_bool()]),
            ("true | bool", vec![f.t_true(), f.t_bool()], vec![f.t_bool()]),
            ("bool | false", vec![f.t_bool(), f.t_false()], vec![f.t_bool()]),
            ("false | bool", vec![f.t_false(), f.t_bool()], vec![f.t_bool()]),
            ("true,false,bool", vec![f.t_true(), f.t_false(), f.t_bool()], vec![f.t_bool()]),
            ("bool,true,false", vec![f.t_bool(), f.t_true(), f.t_false()], vec![f.t_bool()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn integer_cases_structural() {
    fixture(|f| {
        let cases = vec![
            ("int", vec![f.t_int()], vec![f.t_int()]),
            ("int(0)", vec![f.t_lit_int(0)], vec![f.t_lit_int(0)]),
            ("int(1)", vec![f.t_lit_int(1)], vec![f.t_lit_int(1)]),
            ("int(-1)", vec![f.t_lit_int(-1)], vec![f.t_lit_int(-1)]),
            ("int | int(0)", vec![f.t_int(), f.t_lit_int(0)], vec![f.t_int()]),
            ("int(0) | int", vec![f.t_lit_int(0), f.t_int()], vec![f.t_int()]),
            ("int(0) | int(0)", vec![f.t_lit_int(0), f.t_lit_int(0)], vec![f.t_lit_int(0)]),
            ("int(0) | int(1)", vec![f.t_lit_int(0), f.t_lit_int(1)], vec![f.t_int_range(0, 1)]),
            (
                "int(0) | int(1) | int(2)",
                vec![f.t_lit_int(0), f.t_lit_int(1), f.t_lit_int(2)],
                vec![f.t_int_range(0, 2)],
            ),
            (
                "int(0) | int(1) | int(-1)",
                vec![f.t_lit_int(0), f.t_lit_int(1), f.t_lit_int(-1)],
                vec![f.t_int_range(-1, 1)],
            ),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn integer_cases_subtype() {
    fixture(|f| {
        let range_0_2 = f.t_int_range(0, 2);
        let range_0_10 = f.t_int_range(0, 10);
        let cases = vec![
            ("int | int(5)", vec![f.t_int(), f.t_lit_int(5)], vec![f.t_int()]),
            ("int(5) | int", vec![f.t_lit_int(5), f.t_int()], vec![f.t_int()]),
            ("positive-int | int(5)", vec![f.t_positive_int(), f.t_lit_int(5)], vec![f.t_positive_int()]),
            ("non-negative-int | int(0)", vec![f.t_non_negative_int(), f.t_lit_int(0)], vec![f.t_non_negative_int()]),
            (
                "positive-int | non-negative-int",
                vec![f.t_positive_int(), f.t_non_negative_int()],
                vec![f.t_non_negative_int()],
            ),
            ("int<0,10> | int(5)", vec![range_0_10, f.t_lit_int(5)], vec![range_0_10]),
            ("int(1) | int<0,2>", vec![f.t_lit_int(1), range_0_2], vec![range_0_2]),
            ("literal-int | int(5)", vec![f.t_int_unspec_lit(), f.t_lit_int(5)], vec![f.t_int_unspec_lit()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn string_cases_structural() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let hi = f.t_lit_string("hi");
        let letter_a = f.t_lit_string("a");
        let letter_b = f.t_lit_string("b");
        let cases = vec![
            ("string", vec![f.t_string()], vec![f.t_string()]),
            ("''", vec![empty], vec![empty]),
            ("'hi'", vec![hi], vec![hi]),
            ("non-empty", vec![f.t_non_empty_string()], vec![f.t_non_empty_string()]),
            ("numeric-string", vec![f.t_numeric_string()], vec![f.t_numeric_string()]),
            ("lowercase-string", vec![f.t_lower_string()], vec![f.t_lower_string()]),
            ("uppercase-string", vec![f.t_upper_string()], vec![f.t_upper_string()]),
            ("truthy-string", vec![f.t_truthy_string()], vec![f.t_truthy_string()]),
            ("string | ''", vec![f.t_string(), empty], vec![f.t_string()]),
            ("string | 'hi'", vec![f.t_string(), hi], vec![f.t_string()]),
            ("'hi' | string", vec![hi, f.t_string()], vec![f.t_string()]),
            ("'a' | 'b'", vec![letter_a, letter_b], vec![letter_a, letter_b]),
            ("'a' | 'a'", vec![letter_a, letter_a], vec![letter_a]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn string_cases_subtype() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        let lower_lit = f.t_lit_string("abc");
        let upper_lit = f.t_lit_string("ABC");
        let truthy_lit = f.t_lit_string("true");
        let numeric_lit = f.t_lit_string("123");
        let cases = vec![
            ("string | 'hi'", vec![f.t_string(), hi], vec![f.t_string()]),
            ("non-empty-string | 'hi'", vec![f.t_non_empty_string(), hi], vec![f.t_non_empty_string()]),
            ("lowercase-string | 'abc'", vec![f.t_lower_string(), lower_lit], vec![f.t_lower_string()]),
            ("uppercase-string | 'ABC'", vec![f.t_upper_string(), upper_lit], vec![f.t_upper_string()]),
            ("truthy-string | 'true'", vec![f.t_truthy_string(), truthy_lit], vec![f.t_truthy_string()]),
            ("numeric-string | '123'", vec![f.t_numeric_string(), numeric_lit], vec![f.t_numeric_string()]),
            ("string | non-empty-string", vec![f.t_string(), f.t_non_empty_string()], vec![f.t_string()]),
            ("string | numeric-string", vec![f.t_string(), f.t_numeric_string()], vec![f.t_string()]),
            ("class-string | string", vec![f.t_class_string(), f.t_string()], vec![f.t_string()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn float_cases() {
    fixture(|f| {
        let cases = vec![
            ("float", vec![f.t_float()], vec![f.t_float()]),
            ("float(1.5)", vec![f.t_lit_float(1.5)], vec![f.t_lit_float(1.5)]),
            ("float | float(1.5)", vec![f.t_float(), f.t_lit_float(1.5)], vec![f.t_float()]),
            ("float(1.5) | float", vec![f.t_lit_float(1.5), f.t_float()], vec![f.t_float()]),
            ("float(1.5) | float(1.5)", vec![f.t_lit_float(1.5), f.t_lit_float(1.5)], vec![f.t_lit_float(1.5)]),
            (
                "float(1.5) | float(2.5)",
                vec![f.t_lit_float(1.5), f.t_lit_float(2.5)],
                vec![f.t_lit_float(1.5), f.t_lit_float(2.5)],
            ),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn cross_family_cases_structural() {
    fixture(|f| {
        let cases = vec![
            ("int | string", vec![f.t_int(), f.t_string()], vec![f.t_int(), f.t_string()]),
            ("int | float", vec![f.t_int(), f.t_float()], vec![f.t_float(), f.t_int()]),
            ("int | bool", vec![f.t_int(), f.t_bool()], vec![f.t_bool(), f.t_int()]),
            (
                "int | string | float",
                vec![f.t_int(), f.t_string(), f.t_float()],
                vec![f.t_float(), f.t_int(), f.t_string()],
            ),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn cross_family_cases_subtype() {
    fixture(|f| {
        let cases = vec![
            ("int | numeric", vec![f.t_int(), f.t_numeric()], vec![f.t_numeric()]),
            ("float | numeric", vec![f.t_float(), f.t_numeric()], vec![f.t_numeric()]),
            ("int | array-key", vec![f.t_int(), f.t_array_key()], vec![f.t_array_key()]),
            ("string | array-key", vec![f.t_string(), f.t_array_key()], vec![f.t_array_key()]),
            ("int | scalar", vec![f.t_int(), f.t_scalar()], vec![f.t_scalar()]),
            ("string | scalar", vec![f.t_string(), f.t_scalar()], vec![f.t_scalar()]),
            ("float | scalar", vec![f.t_float(), f.t_scalar()], vec![f.t_scalar()]),
            ("bool | scalar", vec![f.t_bool(), f.t_scalar()], vec![f.t_scalar()]),
            ("numeric | scalar", vec![f.t_numeric(), f.t_scalar()], vec![f.t_scalar()]),
            ("array-key | scalar", vec![f.t_array_key(), f.t_scalar()], vec![f.t_scalar()]),
            ("int | string | scalar", vec![f.t_int(), f.t_string(), f.t_scalar()], vec![f.t_scalar()]),
            ("numeric | int | float", vec![f.t_numeric(), f.t_int(), f.t_float()], vec![f.t_numeric()]),
            ("array-key | int | string", vec![f.t_array_key(), f.t_int(), f.t_string()], vec![f.t_array_key()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn special_type_cases() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let cases = vec![
            ("null", vec![f.null()], vec![f.null()]),
            ("void", vec![f.void()], vec![f.void()]),
            ("never", vec![f.never()], vec![f.never()]),
            ("null | null", vec![f.null(), f.null()], vec![f.null()]),
            ("void | void", vec![f.void(), f.void()], vec![f.void()]),
            ("never | never", vec![f.never(), f.never()], vec![f.never()]),
            ("null | void", vec![f.null(), f.void()], vec![f.null()]),
            ("void | null", vec![f.void(), f.null()], vec![f.null()]),
            ("null | never", vec![f.null(), f.never()], vec![f.null()]),
            ("never | null", vec![f.never(), f.null()], vec![f.null()]),
            ("void | never", vec![f.void(), f.never()], vec![f.void()]),
            ("never | void", vec![f.never(), f.void()], vec![f.void()]),
            ("never | int", vec![f.never(), f.t_int()], vec![f.t_int()]),
            ("int | never", vec![f.t_int(), f.never()], vec![f.t_int()]),
            ("void | int", vec![f.void(), f.t_int()], vec![f.t_int(), f.null()]),
            ("int | void", vec![f.t_int(), f.void()], vec![f.t_int(), f.null()]),
            ("null | int", vec![f.null(), f.t_int()], vec![f.t_int(), f.null()]),
            ("int | null", vec![f.t_int(), f.null()], vec![f.t_int(), f.null()]),
            ("null | string", vec![f.null(), f.t_string()], vec![f.null(), f.t_string()]),
            ("null | object", vec![f.null(), f.t_object_any()], vec![f.null(), f.t_object_any()]),
            ("null | named", vec![f.null(), foo], vec![foo, f.null()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn resource_cases() {
    fixture(|f| {
        let cases = vec![
            ("resource", vec![f.t_resource()], vec![f.t_resource()]),
            ("open-resource", vec![f.t_open_resource()], vec![f.t_open_resource()]),
            ("closed-resource", vec![f.t_closed_resource()], vec![f.t_closed_resource()]),
            ("resource | open", vec![f.t_resource(), f.t_open_resource()], vec![f.t_resource()]),
            ("open | resource", vec![f.t_open_resource(), f.t_resource()], vec![f.t_resource()]),
            ("resource | closed", vec![f.t_resource(), f.t_closed_resource()], vec![f.t_resource()]),
            ("closed | resource", vec![f.t_closed_resource(), f.t_resource()], vec![f.t_resource()]),
            ("open | closed", vec![f.t_open_resource(), f.t_closed_resource()], vec![f.t_resource()]),
            ("closed | open", vec![f.t_closed_resource(), f.t_open_resource()], vec![f.t_resource()]),
            ("open | open", vec![f.t_open_resource(), f.t_open_resource()], vec![f.t_open_resource()]),
            ("closed | closed", vec![f.t_closed_resource(), f.t_closed_resource()], vec![f.t_closed_resource()]),
            ("resource | int", vec![f.t_resource(), f.t_int()], vec![f.t_int(), f.t_resource()]),
            ("open | int", vec![f.t_open_resource(), f.t_int()], vec![f.t_int(), f.t_open_resource()]),
            ("closed | string", vec![f.t_closed_resource(), f.t_string()], vec![f.t_closed_resource(), f.t_string()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn object_cases() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let enum_e = f.t_enum("E");
        let enum_f = f.t_enum("F");
        let case_a = f.t_enum_case("E", "A");
        let case_b = f.t_enum_case("E", "B");
        let cases = vec![
            ("object", vec![f.t_object_any()], vec![f.t_object_any()]),
            ("Foo", vec![foo], vec![foo]),
            ("E (enum)", vec![enum_e], vec![enum_e]),
            ("E::A (case)", vec![case_a], vec![case_a]),
            ("object | Foo", vec![f.t_object_any(), foo], vec![f.t_object_any()]),
            ("Foo | object", vec![foo, f.t_object_any()], vec![f.t_object_any()]),
            ("Foo | Foo", vec![foo, foo], vec![foo]),
            ("Foo | Bar", vec![foo, bar], vec![bar, foo]),
            ("E | E", vec![enum_e, enum_e], vec![enum_e]),
            ("E | F", vec![enum_e, enum_f], vec![enum_e, enum_f]),
            ("E::A | E::A", vec![case_a, case_a], vec![case_a]),
            ("E::A | E::B", vec![case_a, case_b], vec![case_a, case_b]),
            ("E | E::A", vec![enum_e, case_a], vec![enum_e]),
            ("Foo | int", vec![foo, f.t_int()], vec![foo, f.t_int()]),
            ("object | int", vec![f.t_object_any(), f.t_int()], vec![f.t_int(), f.t_object_any()]),
            ("Foo | string", vec![foo, f.t_string()], vec![foo, f.t_string()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn array_cases_empty() {
    fixture(|f| {
        let cases = vec![
            ("array{}", vec![f.t_empty_array()], vec![f.t_empty_array()]),
            ("array{} | array{}", vec![f.t_empty_array(), f.t_empty_array()], vec![f.t_empty_array()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn array_cases_shapes() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let array_key_type = f.u(f.t_array_key());
        let mixed_type = f.u(f.mixed());

        let list_int = f.t_list(int_type, false);
        let list_string = f.t_list(string_type, false);
        let list_mixed = f.t_list(mixed_type, false);
        let list_int_string = f.u_many(vec![f.t_int(), f.t_string()]);
        let list_int_or_string = f.t_list(list_int_string, false);
        let non_empty_list_int = f.t_list(int_type, true);
        let keyed_string_int = f.t_keyed_unsealed(string_type, int_type, false);
        let keyed_array_key_mixed = f.t_keyed_unsealed(array_key_type, mixed_type, false);

        let cases = vec![
            ("list<int> | list<int>", vec![list_int, list_int], vec![list_int]),
            ("list<int> | list<string>", vec![list_int, list_string], vec![list_int_or_string]),
            ("list<int> | list<mixed>", vec![list_int, list_mixed], vec![list_mixed]),
            ("list<int> | non-empty-list<int>", vec![list_int, non_empty_list_int], vec![list_int]),
            ("array{} | list<int>", vec![f.t_empty_array(), list_int], vec![list_int]),
            (
                "array{} | non-empty-list<int>",
                vec![f.t_empty_array(), non_empty_list_int],
                vec![f.t_empty_array(), non_empty_list_int],
            ),
            ("keyed | keyed (same)", vec![keyed_string_int, keyed_string_int], vec![keyed_string_int]),
            ("list<int> | keyed<string,int>", vec![list_int, keyed_string_int], vec![list_int, keyed_string_int]),
            ("list<int> | int", vec![list_int, f.t_int()], vec![f.t_int(), list_int]),
            ("keyed<string,int> | string", vec![keyed_string_int, f.t_string()], vec![keyed_string_int, f.t_string()]),
            ("mixed | list<int>", vec![f.mixed(), list_int], vec![f.mixed()]),
            (
                "keyed | array<array-key,mixed>",
                vec![keyed_string_int, keyed_array_key_mixed],
                vec![keyed_array_key_mixed],
            ),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn mixed_dominance_cases() {
    fixture(|f| {
        let cases = vec![
            ("mixed", vec![f.mixed()], vec![f.mixed()]),
            ("mixed | int", vec![f.mixed(), f.t_int()], vec![f.mixed()]),
            ("int | mixed", vec![f.t_int(), f.mixed()], vec![f.mixed()]),
            ("mixed | string", vec![f.mixed(), f.t_string()], vec![f.mixed()]),
            ("mixed | object", vec![f.mixed(), f.t_object_any()], vec![f.mixed()]),
            ("mixed | array{}", vec![f.mixed(), f.t_empty_array()], vec![f.mixed()]),
            ("mixed | null", vec![f.mixed(), f.null()], vec![f.mixed()]),
            ("mixed | never", vec![f.mixed(), f.never()], vec![f.mixed()]),
            ("mixed | resource", vec![f.mixed(), f.t_resource()], vec![f.mixed()]),
            ("truthy-mixed", vec![f.mixed_truthy()], vec![f.mixed_truthy()]),
            ("falsy-mixed", vec![f.mixed_falsy()], vec![f.mixed_falsy()]),
            ("nonnull-mixed", vec![f.mixed_nonnull()], vec![f.mixed_nonnull()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn mixed_axis_cases() {
    fixture(|f| {
        let cases = vec![
            ("vanilla | truthy-mixed", vec![f.mixed(), f.mixed_truthy()], vec![f.mixed()]),
            ("truthy-mixed | vanilla", vec![f.mixed_truthy(), f.mixed()], vec![f.mixed_nonnull()]),
            ("vanilla | falsy-mixed", vec![f.mixed(), f.mixed_falsy()], vec![f.mixed()]),
            ("falsy-mixed | vanilla", vec![f.mixed_falsy(), f.mixed()], vec![f.mixed()]),
            ("vanilla | nonnull-mixed", vec![f.mixed(), f.mixed_nonnull()], vec![f.mixed()]),
            ("nonnull-mixed | vanilla", vec![f.mixed_nonnull(), f.mixed()], vec![f.mixed_nonnull()]),
            ("truthy-mixed | falsy-mixed", vec![f.mixed_truthy(), f.mixed_falsy()], vec![f.mixed_nonnull()]),
            ("truthy-mixed | nonnull-mixed", vec![f.mixed_truthy(), f.mixed_nonnull()], vec![f.mixed_nonnull()]),
            ("nonnull-mixed | null", vec![f.mixed_nonnull(), f.null()], vec![f.mixed()]),
            ("falsy-mixed | null", vec![f.mixed_falsy(), f.null()], vec![f.mixed_falsy()]),
            ("truthy-mixed | null", vec![f.mixed_truthy(), f.null()], vec![f.mixed_nonnull()]),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn multi_atom_cases_structural() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let named_a = f.t_named("A");
        let named_b = f.t_named("B");
        let named_c = f.t_named("C");
        let named_d = f.t_named("D");
        let named_e = f.t_named("E");
        let enum_one = f.t_enum("E1");
        let enum_two = f.t_enum("E2");
        let enum_three = f.t_enum("E3");
        let enum_four = f.t_enum("E4");
        let cases = vec![
            ("3 ints", vec![f.t_lit_int(1), f.t_lit_int(2), f.t_lit_int(3)], vec![f.t_int_range(1, 3)]),
            ("2 ints + int", vec![f.t_lit_int(1), f.t_lit_int(2), f.t_int()], vec![f.t_int()]),
            ("int | string | Foo", vec![f.t_int(), f.t_string(), foo], vec![foo, f.t_int(), f.t_string()]),
            ("null | int | string", vec![f.null(), f.t_int(), f.t_string()], vec![f.t_int(), f.null(), f.t_string()]),
            (
                "5 distinct named",
                vec![named_a, named_b, named_c, named_d, named_e],
                vec![named_a, named_b, named_c, named_d, named_e],
            ),
            (
                "4 distinct enums",
                vec![enum_one, enum_two, enum_three, enum_four],
                vec![enum_one, enum_two, enum_three, enum_four],
            ),
        ];
        run_cases(f, cases);
    });
}

#[test]
fn class_like_string_cases() {
    fixture(|f| {
        let cases = vec![
            ("class-string", vec![f.t_class_string()], vec![f.t_class_string()]),
            ("interface-string", vec![f.t_interface_string()], vec![f.t_interface_string()]),
            ("enum-string", vec![f.t_enum_string()], vec![f.t_enum_string()]),
            ("trait-string", vec![f.t_trait_string()], vec![f.t_trait_string()]),
            ("class-string | string", vec![f.t_class_string(), f.t_string()], vec![f.t_string()]),
            ("string | class-string", vec![f.t_string(), f.t_class_string()], vec![f.t_string()]),
            ("class-string | array-key", vec![f.t_class_string(), f.t_array_key()], vec![f.t_array_key()]),
            ("class-string | scalar", vec![f.t_class_string(), f.t_scalar()], vec![f.t_scalar()]),
            (
                "all 4 class-like kinds",
                vec![f.t_class_string(), f.t_interface_string(), f.t_enum_string(), f.t_trait_string()],
                vec![f.t_class_string(), f.t_interface_string(), f.t_enum_string(), f.t_trait_string()],
            ),
        ];
        run_cases(f, cases);
    });
}
