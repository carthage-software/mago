mod combiner_common;

use combiner_common::*;

use std::collections::BTreeMap;

use mago_codex::ttype::atomic::TAtomic;

type Case = (&'static str, Vec<TAtomic>, Vec<&'static str>);

fn run_cases(cases: Vec<Case>) {
    for (label, input, expected) in cases {
        let result = combine_default(input);
        let mut actual: Vec<String> = result.iter().map(atomic_id_string).collect();
        actual.sort();
        let mut expected_sorted: Vec<&str> = expected;
        expected_sorted.sort_unstable();
        let expected_strings: Vec<String> = expected_sorted.iter().map(|s| s.to_string()).collect();
        assert_eq!(actual, expected_strings, "case `{label}` failed");
    }
}

#[test]
fn bool_cases() {
    run_cases(vec![
        ("true", vec![t_true()], vec!["true"]),
        ("false", vec![t_false()], vec!["false"]),
        ("bool", vec![t_bool()], vec!["bool"]),
        ("true ∨ true", vec![t_true(), t_true()], vec!["true"]),
        ("false ∨ false", vec![t_false(), t_false()], vec!["false"]),
        ("bool ∨ bool", vec![t_bool(), t_bool()], vec!["bool"]),
        ("true ∨ false", vec![t_true(), t_false()], vec!["bool"]),
        ("false ∨ true", vec![t_false(), t_true()], vec!["bool"]),
        ("bool ∨ true", vec![t_bool(), t_true()], vec!["bool"]),
        ("true ∨ bool", vec![t_true(), t_bool()], vec!["bool"]),
        ("bool ∨ false", vec![t_bool(), t_false()], vec!["bool"]),
        ("false ∨ bool", vec![t_false(), t_bool()], vec!["bool"]),
        ("true,false,bool", vec![t_true(), t_false(), t_bool()], vec!["bool"]),
        ("bool,true,false", vec![t_bool(), t_true(), t_false()], vec!["bool"]),
    ]);
}

#[test]
fn integer_cases() {
    run_cases(vec![
        ("int", vec![t_int()], vec!["int"]),
        ("int(0)", vec![t_lit_int(0)], vec!["int(0)"]),
        ("int(1)", vec![t_lit_int(1)], vec!["int(1)"]),
        ("int(-1)", vec![t_lit_int(-1)], vec!["int(-1)"]),
        ("int ∨ int(0)", vec![t_int(), t_lit_int(0)], vec!["int"]),
        ("int(0) ∨ int", vec![t_lit_int(0), t_int()], vec!["int"]),
        ("int(0) ∨ int(0)", vec![t_lit_int(0), t_lit_int(0)], vec!["int(0)"]),
        ("int(0) ∨ int(1)", vec![t_lit_int(0), t_lit_int(1)], vec!["int(0)", "int(1)"]),
        (
            "int(0) ∨ int(1) ∨ int(2)",
            vec![t_lit_int(0), t_lit_int(1), t_lit_int(2)],
            vec!["int(0)", "int(1)", "int(2)"],
        ),
        ("positive-int ∨ int(5)", vec![t_positive_int(), t_lit_int(5)], vec!["positive-int"]),
        ("non-negative ∨ int(0)", vec![t_non_negative_int(), t_lit_int(0)], vec!["non-negative-int"]),
        ("Range(0,10) ∨ Range(5,15)", vec![t_int_range(0, 10), t_int_range(5, 15)], vec!["int<0, 15>"]),
        ("Range(0,10) ∨ Range(11,20)", vec![t_int_range(0, 10), t_int_range(11, 20)], vec!["int<0, 20>"]),
        (
            "Range(0,10) ∨ Range(20,30)",
            vec![t_int_range(0, 10), t_int_range(20, 30)],
            vec!["int<0, 10>", "int<20, 30>"],
        ),
        ("From(5) ∨ int(4)", vec![t_int_from(5), t_lit_int(4)], vec!["int<4, max>"]),
        ("To(0) ∨ int(1)", vec![t_int_to(0), t_lit_int(1)], vec!["int<min, 1>"]),
        ("From(10) ∨ To(0)", vec![t_int_from(10), t_int_to(0)], vec!["int<10, max>", "non-positive-int"]),
        ("From(1) ∨ To(0)", vec![t_int_from(1), t_int_to(0)], vec!["int"]),
        (
            "int(0) ∨ int(1) ∨ int(-1)",
            vec![t_lit_int(0), t_lit_int(1), t_lit_int(-1)],
            vec!["int(0)", "int(1)", "int(-1)"],
        ),
    ]);
}

#[test]
fn string_cases() {
    run_cases(vec![
        ("string", vec![t_string()], vec!["string"]),
        ("''", vec![t_lit_string("")], vec!["string('')"]),
        ("'hi'", vec![t_lit_string("hi")], vec!["string('hi')"]),
        ("non-empty", vec![t_non_empty_string()], vec!["non-empty-string"]),
        ("numeric-string", vec![t_numeric_string()], vec!["numeric-string"]),
        ("lowercase-string", vec![t_lower_string()], vec!["lowercase-string"]),
        ("uppercase-string", vec![t_upper_string()], vec!["uppercase-string"]),
        ("truthy-string", vec![t_truthy_string()], vec!["truthy-string"]),
        ("string ∨ ''", vec![t_string(), t_lit_string("")], vec!["string"]),
        ("string ∨ 'hi'", vec![t_string(), t_lit_string("hi")], vec!["string"]),
        ("'hi' ∨ string", vec![t_lit_string("hi"), t_string()], vec!["string"]),
        ("'a' ∨ 'b'", vec![t_lit_string("a"), t_lit_string("b")], vec!["string('a')", "string('b')"]),
        ("'a' ∨ 'a'", vec![t_lit_string("a"), t_lit_string("a")], vec!["string('a')"]),
        ("non-empty ∨ 'hi'", vec![t_non_empty_string(), t_lit_string("hi")], vec!["non-empty-string"]),
        ("'hi' ∨ non-empty", vec![t_lit_string("hi"), t_non_empty_string()], vec!["non-empty-string"]),
        ("non-empty ∨ ''", vec![t_non_empty_string(), t_lit_string("")], vec!["non-empty-string", "string('')"]),
        ("'' ∨ non-empty", vec![t_lit_string(""), t_non_empty_string()], vec!["string"]),
        ("numeric ∨ '123'", vec![t_numeric_string(), t_lit_string("123")], vec!["numeric-string"]),
        ("numeric ∨ 'abc'", vec![t_numeric_string(), t_lit_string("abc")], vec!["numeric-string", "string('abc')"]),
        ("lower ∨ 'hi'", vec![t_lower_string(), t_lit_string("hi")], vec!["lowercase-string"]),
        ("lower ∨ 'HI'", vec![t_lower_string(), t_lit_string("HI")], vec!["lowercase-string", "string('HI')"]),
        ("upper ∨ 'HI'", vec![t_upper_string(), t_lit_string("HI")], vec!["uppercase-string"]),
        ("upper ∨ 'hi'", vec![t_upper_string(), t_lit_string("hi")], vec!["string('hi')", "uppercase-string"]),
        ("truthy ∨ 'hi'", vec![t_truthy_string(), t_lit_string("hi")], vec!["truthy-string"]),
        ("truthy ∨ '0'", vec![t_truthy_string(), t_lit_string("0")], vec!["string('0')", "truthy-string"]),
        ("truthy ∨ ''", vec![t_truthy_string(), t_lit_string("")], vec!["string('')", "truthy-string"]),
        ("lower ∨ upper", vec![t_lower_string(), t_upper_string()], vec!["string"]),
        ("non-empty ∨ truthy", vec![t_non_empty_string(), t_truthy_string()], vec!["non-empty-string"]),
        ("truthy ∨ non-empty", vec![t_truthy_string(), t_non_empty_string()], vec!["non-empty-string"]),
        ("non-empty ∨ lower", vec![t_non_empty_string(), t_lower_string()], vec!["string"]),
    ]);
}

#[test]
fn float_cases() {
    run_cases(vec![
        ("float", vec![t_float()], vec!["float"]),
        ("float(1.5)", vec![t_lit_float(1.5)], vec!["float(1.5)"]),
        ("float ∨ float(1.5)", vec![t_float(), t_lit_float(1.5)], vec!["float"]),
        ("float(1.5) ∨ float", vec![t_lit_float(1.5), t_float()], vec!["float"]),
        ("float(1.5) ∨ float(1.5)", vec![t_lit_float(1.5), t_lit_float(1.5)], vec!["float(1.5)"]),
        ("float(1.5) ∨ float(2.5)", vec![t_lit_float(1.5), t_lit_float(2.5)], vec!["float(1.5)", "float(2.5)"]),
    ]);
}

#[test]
fn cross_family_cases() {
    run_cases(vec![
        ("int ∨ string", vec![t_int(), t_string()], vec!["int", "string"]),
        ("int ∨ float", vec![t_int(), t_float()], vec!["float", "int"]),
        ("int ∨ bool", vec![t_int(), t_bool()], vec!["bool", "int"]),
        ("int ∨ string ∨ float", vec![t_int(), t_string(), t_float()], vec!["float", "int", "string"]),
        (
            "int ∨ string ∨ float ∨ bool",
            vec![t_int(), t_string(), t_float(), t_bool()],
            vec!["scalar"], // synthesised
        ),
        (
            "int ∨ string ∨ float ∨ bool ∨ null",
            vec![t_int(), t_string(), t_float(), t_bool(), null()],
            vec!["null", "scalar"],
        ),
        ("array-key ∨ int", vec![t_array_key(), t_int()], vec!["array-key"]),
        ("array-key ∨ string", vec![t_array_key(), t_string()], vec!["array-key"]),
        ("scalar ∨ int", vec![t_scalar(), t_int()], vec!["scalar"]),
        ("int ∨ scalar", vec![t_int(), t_scalar()], vec!["scalar"]),
        ("scalar ∨ bool", vec![t_scalar(), t_bool()], vec!["bool", "scalar"]),
        ("bool ∨ scalar", vec![t_bool(), t_scalar()], vec!["scalar"]),
    ]);
}

#[test]
fn special_type_cases() {
    run_cases(vec![
        ("null", vec![null()], vec!["null"]),
        ("void", vec![void()], vec!["void"]),
        ("never", vec![never()], vec!["never"]),
        ("null ∨ null", vec![null(), null()], vec!["null"]),
        ("void ∨ void", vec![void(), void()], vec!["void"]),
        ("never ∨ never", vec![never(), never()], vec!["never"]),
        ("null ∨ void", vec![null(), void()], vec!["null"]),
        ("void ∨ null", vec![void(), null()], vec!["null"]),
        ("null ∨ never", vec![null(), never()], vec!["null"]),
        ("never ∨ null", vec![never(), null()], vec!["null"]),
        ("void ∨ never", vec![void(), never()], vec!["null"]),
        ("never ∨ void", vec![never(), void()], vec!["null"]),
        ("never ∨ int", vec![never(), t_int()], vec!["int"]),
        ("int ∨ never", vec![t_int(), never()], vec!["int"]),
        ("void ∨ int", vec![void(), t_int()], vec!["int", "null"]),
        ("int ∨ void", vec![t_int(), void()], vec!["int", "null"]),
        ("null ∨ int", vec![null(), t_int()], vec!["int", "null"]),
        ("int ∨ null", vec![t_int(), null()], vec!["int", "null"]),
        ("null ∨ string", vec![null(), t_string()], vec!["null", "string"]),
        ("null ∨ object", vec![null(), t_object_any()], vec!["null", "object"]),
        ("null ∨ named", vec![null(), t_named("Foo")], vec!["Foo", "null"]),
    ]);
}

#[test]
fn resource_cases() {
    run_cases(vec![
        ("resource", vec![t_resource()], vec!["resource"]),
        ("open-resource", vec![t_open_resource()], vec!["open-resource"]),
        ("closed-resource", vec![t_closed_resource()], vec!["closed-resource"]),
        ("resource ∨ open", vec![t_resource(), t_open_resource()], vec!["resource"]),
        ("open ∨ resource", vec![t_open_resource(), t_resource()], vec!["resource"]),
        ("resource ∨ closed", vec![t_resource(), t_closed_resource()], vec!["resource"]),
        ("closed ∨ resource", vec![t_closed_resource(), t_resource()], vec!["resource"]),
        ("open ∨ closed", vec![t_open_resource(), t_closed_resource()], vec!["resource"]),
        ("closed ∨ open", vec![t_closed_resource(), t_open_resource()], vec!["resource"]),
        ("open ∨ open", vec![t_open_resource(), t_open_resource()], vec!["open-resource"]),
        ("closed ∨ closed", vec![t_closed_resource(), t_closed_resource()], vec!["closed-resource"]),
        ("resource ∨ int", vec![t_resource(), t_int()], vec!["int", "resource"]),
        ("open ∨ int", vec![t_open_resource(), t_int()], vec!["int", "open-resource"]),
        ("closed ∨ string", vec![t_closed_resource(), t_string()], vec!["closed-resource", "string"]),
    ]);
}

#[test]
fn object_cases() {
    run_cases(vec![
        ("object", vec![t_object_any()], vec!["object"]),
        ("Foo", vec![t_named("Foo")], vec!["Foo"]),
        ("E (enum)", vec![t_enum("E")], vec!["enum(E)"]),
        ("E::A (case)", vec![t_enum_case("E", "A")], vec!["enum(E::A)"]),
        ("object ∨ Foo", vec![t_object_any(), t_named("Foo")], vec!["object"]),
        ("Foo ∨ object", vec![t_named("Foo"), t_object_any()], vec!["object"]),
        ("Foo ∨ Foo", vec![t_named("Foo"), t_named("Foo")], vec!["Foo"]),
        ("Foo ∨ Bar", vec![t_named("Foo"), t_named("Bar")], vec!["Bar", "Foo"]),
        ("E ∨ E", vec![t_enum("E"), t_enum("E")], vec!["enum(E)"]),
        ("E ∨ F", vec![t_enum("E"), t_enum("F")], vec!["enum(E)", "enum(F)"]),
        ("E::A ∨ E::A", vec![t_enum_case("E", "A"), t_enum_case("E", "A")], vec!["enum(E::A)"]),
        ("E::A ∨ E::B", vec![t_enum_case("E", "A"), t_enum_case("E", "B")], vec!["enum(E::A)", "enum(E::B)"]),
        ("E ∨ E::A", vec![t_enum("E"), t_enum_case("E", "A")], vec!["enum(E)", "enum(E::A)"]),
        ("Foo ∨ int", vec![t_named("Foo"), t_int()], vec!["Foo", "int"]),
        ("object ∨ int", vec![t_object_any(), t_int()], vec!["int", "object"]),
        ("Foo ∨ string", vec![t_named("Foo"), t_string()], vec!["Foo", "string"]),
    ]);
}

#[test]
fn array_cases() {
    run_cases(vec![
        ("array{}", vec![t_empty_array()], vec!["array{}"]),
        ("array{} ∨ array{}", vec![t_empty_array(), t_empty_array()], vec!["array{}"]),
        ("list<int>", vec![t_list(u(t_int()), false)], vec!["list<int>"]),
        ("non-empty-list<int>", vec![t_list(u(t_int()), true)], vec!["non-empty-list<int>"]),
        ("list<int> ∨ list<int>", vec![t_list(u(t_int()), false), t_list(u(t_int()), false)], vec!["list<int>"]),
        (
            "list<int> ∨ list<string>",
            vec![t_list(u(t_int()), false), t_list(u(t_string()), false)],
            vec!["list<int|string>"],
        ),
        ("non-empty-list ∨ list", vec![t_list(u(t_int()), true), t_list(u(t_int()), false)], vec!["list<int>"]),
        (
            "non-empty ∨ non-empty",
            vec![t_list(u(t_int()), true), t_list(u(t_int()), true)],
            vec!["non-empty-list<int>"],
        ),
        ("array{} ∨ list<int>", vec![t_empty_array(), t_list(u(t_int()), false)], vec!["array{}", "list<int>"]),
        ("list<int> ∨ array{}", vec![t_list(u(t_int()), false), t_empty_array()], vec!["list<int>"]),
        ("list ∨ int", vec![t_list(u(t_int()), false), t_int()], vec!["int", "list<int>"]),
        (
            "keyed ∨ int",
            vec![t_keyed_unsealed(u(t_string()), u(t_int()), false), t_int()],
            vec!["array<string, int>", "int"],
        ),
        (
            "sealed_list ∨ unsealed_list<int>",
            vec![t_sealed_list(BTreeMap::from([(0usize, (false, u(t_int())))])), t_list(u(t_int()), false)],
            vec!["list<int>"],
        ),
    ]);
}

#[test]
fn mixed_dominance_cases() {
    run_cases(vec![
        ("mixed", vec![mixed()], vec!["mixed"]),
        ("mixed ∨ int", vec![mixed(), t_int()], vec!["mixed"]),
        ("int ∨ mixed", vec![t_int(), mixed()], vec!["mixed"]),
        ("mixed ∨ string", vec![mixed(), t_string()], vec!["mixed"]),
        ("mixed ∨ object", vec![mixed(), t_object_any()], vec!["mixed"]),
        ("mixed ∨ array{}", vec![mixed(), t_empty_array()], vec!["mixed"]),
        ("mixed ∨ null", vec![mixed(), null()], vec!["mixed"]),
        ("mixed ∨ never", vec![mixed(), never()], vec!["mixed"]),
        ("mixed ∨ resource", vec![mixed(), t_resource()], vec!["mixed"]),
        ("truthy-mixed", vec![mixed_truthy()], vec!["truthy-mixed"]),
        ("falsy-mixed", vec![mixed_falsy()], vec!["falsy-mixed"]),
        ("nonnull-mixed", vec![mixed_nonnull()], vec!["nonnull"]),
        ("truthy ∨ falsy", vec![mixed_truthy(), mixed_falsy()], vec!["nonnull"]),
        ("falsy_mixed ∨ null", vec![mixed_falsy(), null()], vec!["falsy-mixed"]),
        ("null ∨ falsy_mixed", vec![null(), mixed_falsy()], vec!["falsy-mixed"]),
        ("nonnull_mixed ∨ null", vec![mixed_nonnull(), null()], vec!["mixed"]),
    ]);
}

#[test]
fn multi_atom_cases() {
    run_cases(vec![
        ("3 ints", vec![t_lit_int(1), t_lit_int(2), t_lit_int(3)], vec!["int(1)", "int(2)", "int(3)"]),
        ("2 ints + int", vec![t_lit_int(1), t_lit_int(2), t_int()], vec!["int"]),
        ("lit ∨ string + int", vec![t_lit_string("hi"), t_string(), t_int()], vec!["int", "string"]),
        ("int ∨ string ∨ Foo", vec![t_int(), t_string(), t_named("Foo")], vec!["Foo", "int", "string"]),
        ("null ∨ int ∨ string", vec![null(), t_int(), t_string()], vec!["int", "null", "string"]),
        (
            "5 distinct named objects",
            vec![t_named("A"), t_named("B"), t_named("C"), t_named("D"), t_named("E")],
            vec!["A", "B", "C", "D", "E"],
        ),
        (
            "4 distinct enums",
            vec![t_enum("E1"), t_enum("E2"), t_enum("E3"), t_enum("E4")],
            vec!["enum(E1)", "enum(E2)", "enum(E3)", "enum(E4)"],
        ),
        (
            "list<int> + list<string>",
            vec![t_list(u(t_int()), false), t_list(u(t_string()), false)],
            vec!["list<int|string>"],
        ),
    ]);
}

#[test]
fn class_like_string_cases() {
    run_cases(vec![
        ("class-string", vec![t_class_string()], vec!["class-string"]),
        ("interface-string", vec![t_interface_string()], vec!["interface-string"]),
        ("enum-string", vec![t_enum_string()], vec!["enum-string"]),
        ("trait-string", vec![t_trait_string()], vec!["trait-string"]),
        ("class-string ∨ string", vec![t_class_string(), t_string()], vec!["class-string", "string"]),
        ("string ∨ class-string", vec![t_string(), t_class_string()], vec!["class-string", "string"]),
        ("class-string ∨ array-key", vec![t_class_string(), t_array_key()], vec!["array-key", "class-string"]),
        ("class-string ∨ scalar", vec![t_class_string(), t_scalar()], vec!["class-string", "scalar"]),
        (
            "all 4 class-like kinds",
            vec![t_class_string(), t_interface_string(), t_enum_string(), t_trait_string()],
            vec!["class-string", "enum-string", "interface-string", "trait-string"],
        ),
    ]);
}
