mod common;

use std::collections::BTreeMap;

use common::*;

use mago_oracle::ty::Atom;

#[track_caller]
fn expect<'arena>(f: &mut Fixture<'_, 'arena>, label: &str, input: Vec<Atom<'arena>>, expected: &[Atom<'arena>]) {
    let result = combine_default(f, input);
    let mut actual = result.clone();
    actual.sort_unstable();
    let mut expected_sorted = expected.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(actual, expected_sorted, "{label}: got {result:?}, expected {expected:?}");
}

#[test]
fn boolean_megatable() {
    fixture(|f| {
        expect(f, "true", vec![f.t_true()], &[f.t_true()]);
        expect(f, "false", vec![f.t_false()], &[f.t_false()]);
        expect(f, "bool", vec![f.t_bool()], &[f.t_bool()]);
        expect(f, "true|true", vec![f.t_true(), f.t_true()], &[f.t_true()]);
        expect(f, "true|false", vec![f.t_true(), f.t_false()], &[f.t_bool()]);
        expect(f, "true|bool", vec![f.t_true(), f.t_bool()], &[f.t_bool()]);
        expect(f, "false|true", vec![f.t_false(), f.t_true()], &[f.t_bool()]);
        expect(f, "false|false", vec![f.t_false(), f.t_false()], &[f.t_false()]);
        expect(f, "false|bool", vec![f.t_false(), f.t_bool()], &[f.t_bool()]);
        expect(f, "bool|true", vec![f.t_bool(), f.t_true()], &[f.t_bool()]);
        expect(f, "bool|false", vec![f.t_bool(), f.t_false()], &[f.t_bool()]);
        expect(f, "bool|bool", vec![f.t_bool(), f.t_bool()], &[f.t_bool()]);
        expect(f, "3true", vec![f.t_true(); 3], &[f.t_true()]);
        expect(f, "3false", vec![f.t_false(); 3], &[f.t_false()]);
        expect(f, "3bool", vec![f.t_bool(); 3], &[f.t_bool()]);
        expect(f, "true,true,false", vec![f.t_true(), f.t_true(), f.t_false()], &[f.t_bool()]);
        expect(f, "false,false,true", vec![f.t_false(), f.t_false(), f.t_true()], &[f.t_bool()]);
        expect(f, "true,bool,false", vec![f.t_true(), f.t_bool(), f.t_false()], &[f.t_bool()]);
        expect(f, "bool,bool,true", vec![f.t_bool(), f.t_bool(), f.t_true()], &[f.t_bool()]);
        expect(f, "bool,bool,false", vec![f.t_bool(), f.t_bool(), f.t_false()], &[f.t_bool()]);
        expect(f, "4true", vec![f.t_true(); 4], &[f.t_true()]);
        expect(f, "4false", vec![f.t_false(); 4], &[f.t_false()]);
        expect(f, "bool*4", vec![f.t_bool(); 4], &[f.t_bool()]);
        expect(f, "alt true,false", vec![f.t_true(), f.t_false(), f.t_true(), f.t_false()], &[f.t_bool()]);
    });
}

#[test]
fn integer_megatable_singletons() {
    fixture(|f| {
        let zero_to_ten = f.t_int_range(0, 10);
        let from_five = f.t_int_from(5);
        let to_five = f.t_int_to(5);
        expect(f, "int", vec![f.t_int()], &[f.t_int()]);
        expect(f, "int(0)", vec![f.t_lit_int(0)], &[f.t_lit_int(0)]);
        expect(f, "int(-100)", vec![f.t_lit_int(-100)], &[f.t_lit_int(-100)]);
        expect(f, "int(1000)", vec![f.t_lit_int(1000)], &[f.t_lit_int(1000)]);
        expect(f, "positive-int", vec![f.t_positive_int()], &[f.t_positive_int()]);
        expect(f, "non-negative-int", vec![f.t_non_negative_int()], &[f.t_non_negative_int()]);
        expect(f, "negative-int", vec![f.t_negative_int()], &[f.t_negative_int()]);
        expect(f, "non-positive-int", vec![f.t_non_positive_int()], &[f.t_non_positive_int()]);
        expect(f, "Range(0,10)", vec![zero_to_ten], &[zero_to_ten]);
        expect(f, "From(5)", vec![from_five], &[from_five]);
        expect(f, "To(5)", vec![to_five], &[to_five]);
        expect(f, "UnspecLit", vec![f.t_int_unspec_lit()], &[f.t_int_unspec_lit()]);
    });
}

#[test]
fn integer_megatable_dominator() {
    fixture(|f| {
        let zero_to_ten = f.t_int_range(0, 10);
        let from_five = f.t_int_from(5);
        let to_five = f.t_int_to(5);
        expect(f, "int|int(0)", vec![f.t_int(), f.t_lit_int(0)], &[f.t_int()]);
        expect(f, "int|positive", vec![f.t_int(), f.t_positive_int()], &[f.t_int()]);
        expect(f, "int|negative", vec![f.t_int(), f.t_negative_int()], &[f.t_int()]);
        expect(f, "int|Range", vec![f.t_int(), zero_to_ten], &[f.t_int()]);
        expect(f, "int|From", vec![f.t_int(), from_five], &[f.t_int()]);
        expect(f, "int|To", vec![f.t_int(), to_five], &[f.t_int()]);
        expect(f, "int|UnspecLit", vec![f.t_int(), f.t_int_unspec_lit()], &[f.t_int()]);
    });
}

#[test]
fn integer_megatable_subtype() {
    fixture(|f| {
        let zero_to_ten = f.t_int_range(0, 10);
        let one_to_max = f.t_int_from(1);
        expect(f, "positive|5", vec![f.t_positive_int(), f.t_lit_int(5)], &[f.t_positive_int()]);
        expect(f, "non-negative|0", vec![f.t_non_negative_int(), f.t_lit_int(0)], &[f.t_non_negative_int()]);
        expect(f, "negative|-5", vec![f.t_negative_int(), f.t_lit_int(-5)], &[f.t_negative_int()]);
        expect(f, "non-positive|0", vec![f.t_non_positive_int(), f.t_lit_int(0)], &[f.t_non_positive_int()]);
        expect(f, "Range(0,10)|5", vec![zero_to_ten, f.t_lit_int(5)], &[zero_to_ten]);
        expect(f, "Range(0,10)|0", vec![zero_to_ten, f.t_lit_int(0)], &[zero_to_ten]);
        expect(f, "Range(0,10)|10", vec![zero_to_ten, f.t_lit_int(10)], &[zero_to_ten]);
        expect(f, "positive|From(1)", vec![f.t_positive_int(), one_to_max], &[f.t_positive_int()]);
        expect(f, "non-negative|positive", vec![f.t_non_negative_int(), f.t_positive_int()], &[f.t_non_negative_int()]);
        expect(f, "non-positive|negative", vec![f.t_non_positive_int(), f.t_negative_int()], &[f.t_non_positive_int()]);
    });
}

#[test]
fn string_megatable_singletons() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let hi = f.t_lit_string("hi");
        let zero = f.t_lit_string("0");
        let digits = f.t_lit_string("123");
        expect(f, "string", vec![f.t_string()], &[f.t_string()]);
        expect(f, "non-empty-string", vec![f.t_non_empty_string()], &[f.t_non_empty_string()]);
        expect(f, "numeric-string", vec![f.t_numeric_string()], &[f.t_numeric_string()]);
        expect(f, "lowercase-string", vec![f.t_lower_string()], &[f.t_lower_string()]);
        expect(f, "uppercase-string", vec![f.t_upper_string()], &[f.t_upper_string()]);
        expect(f, "truthy-string", vec![f.t_truthy_string()], &[f.t_truthy_string()]);
        expect(f, "''", vec![empty], &[empty]);
        expect(f, "'hi'", vec![hi], &[hi]);
        expect(f, "'0'", vec![zero], &[zero]);
        expect(f, "'123'", vec![digits], &[digits]);
    });
}

#[test]
fn string_megatable_dominator() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let hi = f.t_lit_string("hi");
        let zero = f.t_lit_string("0");
        let letter_a = f.t_lit_string("a");
        let letter_b = f.t_lit_string("b");
        expect(f, "string|''", vec![f.t_string(), empty], &[f.t_string()]);
        expect(f, "string|'hi'", vec![f.t_string(), hi], &[f.t_string()]);
        expect(f, "string|'0'", vec![f.t_string(), zero], &[f.t_string()]);
        expect(f, "'hi'|string", vec![hi, f.t_string()], &[f.t_string()]);
        expect(f, "'a'|'b'", vec![letter_a, letter_b], &[letter_a, letter_b]);
        expect(f, "'a'|'a'", vec![letter_a, letter_a], &[letter_a]);
    });
}

#[test]
fn string_megatable_subtype() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        let abc = f.t_lit_string("abc");
        let digits = f.t_lit_string("123");
        let truthy_literal = f.t_lit_string("yes");
        let upper_literal = f.t_lit_string("ABC");
        expect(f, "non-empty|'hi'", vec![f.t_non_empty_string(), hi], &[f.t_non_empty_string()]);
        expect(f, "lowercase|'abc'", vec![f.t_lower_string(), abc], &[f.t_lower_string()]);
        expect(f, "numeric|'123'", vec![f.t_numeric_string(), digits], &[f.t_numeric_string()]);
        expect(f, "truthy|'yes'", vec![f.t_truthy_string(), truthy_literal], &[f.t_truthy_string()]);
        expect(f, "uppercase|'ABC'", vec![f.t_upper_string(), upper_literal], &[f.t_upper_string()]);
        expect(f, "non-empty|truthy", vec![f.t_non_empty_string(), f.t_truthy_string()], &[f.t_non_empty_string()]);
    });
}

#[test]
fn float_megatable() {
    fixture(|f| {
        expect(f, "float", vec![f.t_float()], &[f.t_float()]);
        expect(f, "float(0)", vec![f.t_lit_float(0.0)], &[f.t_lit_float(0.0)]);
        expect(f, "float(1.5)", vec![f.t_lit_float(1.5)], &[f.t_lit_float(1.5)]);
        expect(f, "float(-1.0)", vec![f.t_lit_float(-1.0)], &[f.t_lit_float(-1.0)]);
        expect(f, "float|float(1.5)", vec![f.t_float(), f.t_lit_float(1.5)], &[f.t_float()]);
        expect(f, "float(1.5)|float", vec![f.t_lit_float(1.5), f.t_float()], &[f.t_float()]);
        expect(
            f,
            "float(1.5)|float(2.5)",
            vec![f.t_lit_float(1.5), f.t_lit_float(2.5)],
            &[f.t_lit_float(1.5), f.t_lit_float(2.5)],
        );
        expect(f, "float(1.5)|float(1.5)", vec![f.t_lit_float(1.5), f.t_lit_float(1.5)], &[f.t_lit_float(1.5)]);
    });
}

#[test]
fn object_megatable() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let baz = f.t_named("Baz");
        let enum_e = f.t_enum("E");
        let enum_f = f.t_enum("F");
        let case_a = f.t_enum_case("E", "A");
        let case_b = f.t_enum_case("E", "B");
        expect(f, "object", vec![f.t_object_any()], &[f.t_object_any()]);
        expect(f, "Foo", vec![foo], &[foo]);
        expect(f, "E", vec![enum_e], &[enum_e]);
        expect(f, "E::A", vec![case_a], &[case_a]);
        expect(f, "object|Foo", vec![f.t_object_any(), foo], &[f.t_object_any()]);
        expect(f, "Foo|object", vec![foo, f.t_object_any()], &[f.t_object_any()]);
        expect(f, "Foo|Foo", vec![foo, foo], &[foo]);
        expect(f, "Foo|Bar", vec![foo, bar], &[bar, foo]);
        expect(f, "Foo|Bar|Baz", vec![foo, bar, baz], &[bar, baz, foo]);
        expect(f, "Foo|Bar|object", vec![foo, bar, f.t_object_any()], &[f.t_object_any()]);
        expect(f, "E|E", vec![enum_e, enum_e], &[enum_e]);
        expect(f, "E|F", vec![enum_e, enum_f], &[enum_e, enum_f]);
        expect(f, "E|E::A", vec![enum_e, case_a], &[enum_e]);
        expect(f, "E::A|E::B", vec![case_a, case_b], &[case_a, case_b]);
    });
}

#[test]
fn object_megatable_generic() {
    fixture(|f| {
        let int_argument = f.u(f.t_int());
        let string_argument = f.u(f.t_string());
        let foo_int = f.t_generic_named("Foo", vec![int_argument]);
        let foo_int_again = f.t_generic_named("Foo", vec![int_argument]);
        let bar_int = f.t_generic_named("Bar", vec![int_argument]);
        let foo_string = f.t_generic_named("Foo", vec![string_argument]);
        let foo_plain = f.t_named("Foo");
        expect(f, "Foo<int>", vec![foo_int], &[foo_int]);
        expect(f, "Foo<int>|Foo<int>", vec![foo_int, foo_int_again], &[foo_int]);
        expect(f, "Foo<int>|object", vec![foo_int, f.t_object_any()], &[f.t_object_any()]);
        expect(f, "object|Foo<int>", vec![f.t_object_any(), foo_int], &[f.t_object_any()]);
        expect(f, "Foo<int>|mixed", vec![foo_int, f.mixed()], &[f.mixed()]);
        expect(f, "Foo<int>|Bar<int>", vec![foo_int, bar_int], &[bar_int, foo_int]);
        expect(f, "Foo<int>|Foo<string>", vec![foo_int, foo_string], &[foo_int, foo_string]);
        expect(f, "Foo|Foo<int>", vec![foo_plain, foo_int], &[foo_int, foo_plain]);
    });
}

#[test]
fn array_megatable_empty() {
    fixture(|f| {
        expect(f, "array{}", vec![f.t_empty_array()], &[f.t_empty_array()]);
        expect(f, "array{}|array{}", vec![f.t_empty_array(); 2], &[f.t_empty_array()]);
        expect(f, "3*array{}", vec![f.t_empty_array(); 3], &[f.t_empty_array()]);
    });
}

#[test]
fn array_megatable_shapes() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let int_or_string = f.u_many(vec![f.t_int(), f.t_string()]);

        let list_int = f.t_list(int_type, false);
        let list_string = f.t_list(string_type, false);
        let list_int_or_string = f.t_list(int_or_string, false);
        expect(f, "list<int>|list<int>", vec![list_int, list_int], &[list_int]);
        expect(f, "list<int>|list<string>", vec![list_int, list_string], &[list_int_or_string]);

        let non_empty_list_int = f.t_list(int_type, true);
        expect(f, "non-empty-list<int>|list<int>", vec![non_empty_list_int, list_int], &[list_int]);

        let array_key_type = f.u(f.t_array_key());
        let keyed_int = f.t_keyed_unsealed(array_key_type, int_type, false);
        let array_key_type_again = f.u(f.t_array_key());
        let keyed_string = f.t_keyed_unsealed(array_key_type_again, string_type, false);
        let array_key_type_merged = f.u(f.t_array_key());
        let keyed_merged = f.t_keyed_unsealed(array_key_type_merged, int_or_string, false);
        expect(f, "array<ak,int>|array<ak,string>", vec![keyed_int, keyed_string], &[keyed_merged]);

        let mut shared_int = BTreeMap::new();
        shared_int.insert(f.ak_str("a"), (false, int_type));
        let sealed_a_int = f.t_keyed_sealed(shared_int, false);
        let mut shared_string = BTreeMap::new();
        shared_string.insert(f.ak_str("a"), (false, string_type));
        let sealed_a_string = f.t_keyed_sealed(shared_string, false);
        let mut shared_merged = BTreeMap::new();
        shared_merged.insert(f.ak_str("a"), (false, int_or_string));
        let sealed_a_merged = f.t_keyed_sealed(shared_merged, false);
        expect(f, "array{a:int}|array{a:string}", vec![sealed_a_int, sealed_a_string], &[sealed_a_merged]);

        let mut only_a = BTreeMap::new();
        only_a.insert(f.ak_str("a"), (false, int_type));
        let sealed_only_a = f.t_keyed_sealed(only_a, false);
        let mut only_b = BTreeMap::new();
        only_b.insert(f.ak_str("b"), (false, string_type));
        let sealed_only_b = f.t_keyed_sealed(only_b, false);
        expect(f, "array{a:int}|array{b:string}", vec![sealed_only_a, sealed_only_b], &[sealed_only_a, sealed_only_b]);
    });
}

#[test]
fn cross_family_megatable() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let enum_e = f.t_enum("E");
        expect(f, "int|object", vec![f.t_int(), f.t_object_any()], &[f.t_int(), f.t_object_any()]);
        expect(f, "object|int", vec![f.t_object_any(), f.t_int()], &[f.t_int(), f.t_object_any()]);
        expect(f, "int|resource", vec![f.t_int(), f.t_resource()], &[f.t_int(), f.t_resource()]);
        expect(f, "int|open", vec![f.t_int(), f.t_open_resource()], &[f.t_int(), f.t_open_resource()]);
        expect(f, "string|object", vec![f.t_string(), f.t_object_any()], &[f.t_object_any(), f.t_string()]);
        expect(f, "string|resource", vec![f.t_string(), f.t_resource()], &[f.t_resource(), f.t_string()]);
        expect(f, "string|array{}", vec![f.t_string(), f.t_empty_array()], &[f.t_empty_array(), f.t_string()]);
        expect(f, "float|object", vec![f.t_float(), f.t_object_any()], &[f.t_float(), f.t_object_any()]);
        expect(f, "bool|object", vec![f.t_bool(), f.t_object_any()], &[f.t_bool(), f.t_object_any()]);
        expect(f, "null|object", vec![f.null(), f.t_object_any()], &[f.null(), f.t_object_any()]);
        expect(f, "null|array{}", vec![f.null(), f.t_empty_array()], &[f.t_empty_array(), f.null()]);
        expect(f, "null|resource", vec![f.null(), f.t_resource()], &[f.null(), f.t_resource()]);
        expect(f, "Foo|resource", vec![foo, f.t_resource()], &[foo, f.t_resource()]);
        expect(f, "Foo|E", vec![foo, enum_e], &[foo, enum_e]);
        expect(f, "E|resource", vec![enum_e, f.t_resource()], &[enum_e, f.t_resource()]);
        expect(f, "array{}|object", vec![f.t_empty_array(), f.t_object_any()], &[f.t_empty_array(), f.t_object_any()]);
    });
}

#[test]
fn scalar_synthesis_megatable() {
    fixture(|f| {
        expect(f, "int|string|float|bool", vec![f.t_int(), f.t_string(), f.t_float(), f.t_bool()], &[f.t_scalar()]);
        expect(
            f,
            "int|string|float|true|false",
            vec![f.t_int(), f.t_string(), f.t_float(), f.t_true(), f.t_false()],
            &[f.t_scalar()],
        );
        expect(
            f,
            "bool|float|int|string reordered",
            vec![f.t_bool(), f.t_float(), f.t_int(), f.t_string()],
            &[f.t_scalar()],
        );
        expect(
            f,
            "int|string|float",
            vec![f.t_int(), f.t_string(), f.t_float()],
            &[f.t_int(), f.t_float(), f.t_string()],
        );
        expect(f, "int|string|bool", vec![f.t_int(), f.t_string(), f.t_bool()], &[f.t_int(), f.t_bool(), f.t_string()]);
        expect(
            f,
            "int|string|float|bool|null",
            vec![f.t_int(), f.t_string(), f.t_float(), f.t_bool(), f.null()],
            &[f.t_scalar(), f.null()],
        );
    });
}
