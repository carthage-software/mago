mod combiner_common;

use combiner_common::*;

use std::collections::BTreeMap;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::union::TUnion;

fn expect(label: &str, input: Vec<TAtomic>, ids: &[&str]) {
    let result = combine_default(input);
    let mut actual: Vec<String> = result.iter().map(atomic_id_string).collect();
    actual.sort();
    let mut expected: Vec<String> = ids.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(actual, expected, "{label}");
}

#[test]
fn boolean_megatable() {
    expect("true", vec![t_true()], &["true"]);
    expect("false", vec![t_false()], &["false"]);
    expect("bool", vec![t_bool()], &["bool"]);
    expect("true∨true", vec![t_true(), t_true()], &["true"]);
    expect("true∨false", vec![t_true(), t_false()], &["bool"]);
    expect("true∨bool", vec![t_true(), t_bool()], &["bool"]);
    expect("false∨true", vec![t_false(), t_true()], &["bool"]);
    expect("false∨false", vec![t_false(), t_false()], &["false"]);
    expect("false∨bool", vec![t_false(), t_bool()], &["bool"]);
    expect("bool∨true", vec![t_bool(), t_true()], &["bool"]);
    expect("bool∨false", vec![t_bool(), t_false()], &["bool"]);
    expect("bool∨bool", vec![t_bool(), t_bool()], &["bool"]);
    expect("3true", vec![t_true(); 3], &["true"]);
    expect("3false", vec![t_false(); 3], &["false"]);
    expect("3bool", vec![t_bool(); 3], &["bool"]);
    expect("true,true,false", vec![t_true(), t_true(), t_false()], &["bool"]);
    expect("false,false,true", vec![t_false(), t_false(), t_true()], &["bool"]);
    expect("true,bool,false", vec![t_true(), t_bool(), t_false()], &["bool"]);
    expect("bool,bool,true", vec![t_bool(), t_bool(), t_true()], &["bool"]);
    expect("bool,bool,false", vec![t_bool(), t_bool(), t_false()], &["bool"]);
    expect("true,true,true,true", vec![t_true(); 4], &["true"]);
    expect("false,false,false,false", vec![t_false(); 4], &["false"]);
    expect("bool*4", vec![t_bool(); 4], &["bool"]);
    expect("true,false,true,false", vec![t_true(), t_false(), t_true(), t_false()], &["bool"]);
}

#[test]
fn integer_megatable() {
    expect("int", vec![t_int()], &["int"]);
    expect("int(0)", vec![t_lit_int(0)], &["int(0)"]);
    expect("int(-100)", vec![t_lit_int(-100)], &["int(-100)"]);
    expect("int(1000)", vec![t_lit_int(1000)], &["int(1000)"]);
    expect("positive-int", vec![t_positive_int()], &["positive-int"]);
    expect("non-negative-int", vec![t_non_negative_int()], &["non-negative-int"]);
    expect("negative-int", vec![t_negative_int()], &["negative-int"]);
    expect("non-positive-int", vec![t_non_positive_int()], &["non-positive-int"]);
    expect("Range(0,10)", vec![t_int_range(0, 10)], &["int<0, 10>"]);
    expect("From(5)", vec![t_int_from(5)], &["int<5, max>"]);
    expect("To(5)", vec![t_int_to(5)], &["int<min, 5>"]);
    expect("UnspecLit", vec![t_int_unspec_lit()], &["literal-int"]);

    expect("int∨int(0)", vec![t_int(), t_lit_int(0)], &["int"]);
    expect("int∨positive", vec![t_int(), t_positive_int()], &["int"]);
    expect("int∨negative", vec![t_int(), t_negative_int()], &["int"]);
    expect("int∨Range", vec![t_int(), t_int_range(0, 10)], &["int"]);
    expect("int∨From", vec![t_int(), t_int_from(5)], &["int"]);
    expect("int∨To", vec![t_int(), t_int_to(5)], &["int"]);
    expect("int∨UnspecLit", vec![t_int(), t_int_unspec_lit()], &["int"]);

    expect("Range(0,5)∨Range(6,10)", vec![t_int_range(0, 5), t_int_range(6, 10)], &["int<0, 10>"]);
    expect("Range(0,10)∨Range(5,15)", vec![t_int_range(0, 10), t_int_range(5, 15)], &["int<0, 15>"]);
    expect("Range(0,5)∨Range(10,15)", vec![t_int_range(0, 5), t_int_range(10, 15)], &["int<0, 5>", "int<10, 15>"]);

    expect("Range(0,5)∨int(6)", vec![t_int_range(0, 5), t_lit_int(6)], &["int<0, 6>"]);
    expect("Range(0,5)∨int(-1)", vec![t_int_range(0, 5), t_lit_int(-1)], &["int<-1, 5>"]);
    expect("Range(0,5)∨int(10)", vec![t_int_range(0, 5), t_lit_int(10)], &["int(10)", "int<0, 5>"]);

    expect("From(5)∨int(4)", vec![t_int_from(5), t_lit_int(4)], &["int<4, max>"]);
    expect("From(5)∨int(0)", vec![t_int_from(5), t_lit_int(0)], &["int(0)", "int<5, max>"]);
    expect("To(5)∨int(6)", vec![t_int_to(5), t_lit_int(6)], &["int<min, 6>"]);
    expect("From(5)∨To(0)", vec![t_int_from(5), t_int_to(0)], &["int<5, max>", "non-positive-int"]);
    expect("From(1)∨To(0)", vec![t_int_from(1), t_int_to(0)], &["int"]);

    expect("positive∨int(0)", vec![t_positive_int(), t_lit_int(0)], &["non-negative-int"]);
    expect("negative∨int(0)", vec![t_negative_int(), t_lit_int(0)], &["non-positive-int"]);
    expect("positive∨negative", vec![t_positive_int(), t_negative_int()], &["negative-int", "positive-int"]);
    expect("non_negative∨non_positive", vec![t_non_negative_int(), t_non_positive_int()], &["int"]);
}

#[test]
fn string_megatable() {
    expect("string", vec![t_string()], &["string"]);
    expect("non-empty-string", vec![t_non_empty_string()], &["non-empty-string"]);
    expect("numeric-string", vec![t_numeric_string()], &["numeric-string"]);
    expect("lowercase-string", vec![t_lower_string()], &["lowercase-string"]);
    expect("uppercase-string", vec![t_upper_string()], &["uppercase-string"]);
    expect("truthy-string", vec![t_truthy_string()], &["truthy-string"]);
    expect("''", vec![t_lit_string("")], &["string('')"]);
    expect("'hi'", vec![t_lit_string("hi")], &["string('hi')"]);
    expect("'0'", vec![t_lit_string("0")], &["string('0')"]);
    expect("'123'", vec![t_lit_string("123")], &["string('123')"]);

    expect("string∨''", vec![t_string(), t_lit_string("")], &["string"]);
    expect("string∨'hi'", vec![t_string(), t_lit_string("hi")], &["string"]);
    expect("string∨'0'", vec![t_string(), t_lit_string("0")], &["string"]);
    expect("'hi'∨string", vec![t_lit_string("hi"), t_string()], &["string"]);

    expect("non-empty∨'hi'", vec![t_non_empty_string(), t_lit_string("hi")], &["non-empty-string"]);
    expect("non-empty∨'0'", vec![t_non_empty_string(), t_lit_string("0")], &["non-empty-string"]);
    expect("non-empty∨''", vec![t_non_empty_string(), t_lit_string("")], &["non-empty-string", "string('')"]);
    expect("''∨non-empty", vec![t_lit_string(""), t_non_empty_string()], &["string"]);

    expect("numeric∨'1'", vec![t_numeric_string(), t_lit_string("1")], &["numeric-string"]);
    expect("numeric∨'-5'", vec![t_numeric_string(), t_lit_string("-5")], &["numeric-string"]);
    expect("numeric∨'1.5'", vec![t_numeric_string(), t_lit_string("1.5")], &["numeric-string"]);
    expect("numeric∨'abc'", vec![t_numeric_string(), t_lit_string("abc")], &["numeric-string", "string('abc')"]);

    expect("lower∨'abc'", vec![t_lower_string(), t_lit_string("abc")], &["lowercase-string"]);
    expect("lower∨'ABC'", vec![t_lower_string(), t_lit_string("ABC")], &["lowercase-string", "string('ABC')"]);
    expect("lower∨''", vec![t_lower_string(), t_lit_string("")], &["string"]);

    expect("upper∨'ABC'", vec![t_upper_string(), t_lit_string("ABC")], &["uppercase-string"]);
    expect("upper∨'abc'", vec![t_upper_string(), t_lit_string("abc")], &["string('abc')", "uppercase-string"]);

    expect("truthy∨'hi'", vec![t_truthy_string(), t_lit_string("hi")], &["truthy-string"]);
    expect("truthy∨'1'", vec![t_truthy_string(), t_lit_string("1")], &["truthy-string"]);
    expect("truthy∨'0'", vec![t_truthy_string(), t_lit_string("0")], &["string('0')", "truthy-string"]);
    expect("truthy∨''", vec![t_truthy_string(), t_lit_string("")], &["string('')", "truthy-string"]);

    expect("'a'∨'b'", vec![t_lit_string("a"), t_lit_string("b")], &["string('a')", "string('b')"]);
    expect("'a'∨'a'", vec![t_lit_string("a"), t_lit_string("a")], &["string('a')"]);
}

#[test]
fn float_megatable() {
    expect("float", vec![t_float()], &["float"]);
    expect("float(0)", vec![t_lit_float(0.0)], &["float(0.0)"]);
    expect("float(1.5)", vec![t_lit_float(1.5)], &["float(1.5)"]);
    expect("float(-1.0)", vec![t_lit_float(-1.0)], &["float(-1.0)"]);
    expect("float∨float(1.5)", vec![t_float(), t_lit_float(1.5)], &["float"]);
    expect("float(1.5)∨float", vec![t_lit_float(1.5), t_float()], &["float"]);
    expect("float(1.5)∨float(2.5)", vec![t_lit_float(1.5), t_lit_float(2.5)], &["float(1.5)", "float(2.5)"]);
    expect("float(1.5)∨float(1.5)", vec![t_lit_float(1.5), t_lit_float(1.5)], &["float(1.5)"]);
}

#[test]
fn object_megatable() {
    expect("object", vec![t_object_any()], &["object"]);
    expect("Foo", vec![t_named("Foo")], &["Foo"]);
    expect("E", vec![t_enum("E")], &["enum(E)"]);
    expect("E::A", vec![t_enum_case("E", "A")], &["enum(E::A)"]);
    expect("object∨Foo", vec![t_object_any(), t_named("Foo")], &["object"]);
    expect("Foo∨object", vec![t_named("Foo"), t_object_any()], &["object"]);
    expect("Foo∨Foo", vec![t_named("Foo"), t_named("Foo")], &["Foo"]);
    expect("Foo∨Bar", vec![t_named("Foo"), t_named("Bar")], &["Bar", "Foo"]);
    expect("Foo∨Bar∨Baz", vec![t_named("Foo"), t_named("Bar"), t_named("Baz")], &["Bar", "Baz", "Foo"]);
    expect("Foo∨Bar∨object", vec![t_named("Foo"), t_named("Bar"), t_object_any()], &["object"]);
    expect("E∨E", vec![t_enum("E"), t_enum("E")], &["enum(E)"]);
    expect("E∨F", vec![t_enum("E"), t_enum("F")], &["enum(E)", "enum(F)"]);
    expect("E∨E::A", vec![t_enum("E"), t_enum_case("E", "A")], &["enum(E)", "enum(E::A)"]);
    expect("E::A∨E::B", vec![t_enum_case("E", "A"), t_enum_case("E", "B")], &["enum(E::A)", "enum(E::B)"]);

    expect(
        "Container<int>",
        vec![t_generic_named("Container", vec![TUnion::from_atomic(t_int())])],
        &["Container<int>"],
    );
    expect(
        "Container<int>∨Container<int>",
        vec![
            t_generic_named("Container", vec![TUnion::from_atomic(t_int())]),
            t_generic_named("Container", vec![TUnion::from_atomic(t_int())]),
        ],
        &["Container<int>"],
    );
}

#[test]
fn array_megatable() {
    expect("array{}", vec![t_empty_array()], &["array{}"]);
    expect("array{}∨array{}", vec![t_empty_array(); 2], &["array{}"]);
    expect("3×array{}", vec![t_empty_array(); 3], &["array{}"]);

    expect("list<int>", vec![t_list(u(t_int()), false)], &["list<int>"]);
    expect("non-empty-list<int>", vec![t_list(u(t_int()), true)], &["non-empty-list<int>"]);
    expect("list<string>", vec![t_list(u(t_string()), false)], &["list<string>"]);
    expect("list<mixed>", vec![t_list(u(mixed()), false)], &["list<mixed>"]);

    expect("list<int>∨list<int>", vec![t_list(u(t_int()), false), t_list(u(t_int()), false)], &["list<int>"]);
    expect(
        "list<int>∨list<string>",
        vec![t_list(u(t_int()), false), t_list(u(t_string()), false)],
        &["list<int|string>"],
    );
    expect("list<int>∨list<float>", vec![t_list(u(t_int()), false), t_list(u(t_float()), false)], &["list<float|int>"]);
    expect("ne-list∨ne-list", vec![t_list(u(t_int()), true), t_list(u(t_int()), true)], &["non-empty-list<int>"]);
    expect("ne-list∨list", vec![t_list(u(t_int()), true), t_list(u(t_int()), false)], &["list<int>"]);
    expect("list∨ne-list", vec![t_list(u(t_int()), false), t_list(u(t_int()), true)], &["list<int>"]);

    expect("empty ∨ list", vec![t_empty_array(), t_list(u(t_int()), false)], &["list<int>"]);
    expect("list ∨ empty", vec![t_list(u(t_int()), false), t_empty_array()], &["list<int>"]);
    expect("empty ∨ ne-list", vec![t_empty_array(), t_list(u(t_int()), true)], &["list<int>"]);

    expect("keyed<string,int>", vec![t_keyed_unsealed(u(t_string()), u(t_int()), false)], &["array<string, int>"]);
    expect(
        "keyed<string,int>∨keyed<string,int>",
        vec![t_keyed_unsealed(u(t_string()), u(t_int()), false), t_keyed_unsealed(u(t_string()), u(t_int()), false)],
        &["array<string, int>"],
    );
    expect(
        "keyed<string,int>∨keyed<string,string>",
        vec![t_keyed_unsealed(u(t_string()), u(t_int()), false), t_keyed_unsealed(u(t_string()), u(t_string()), false)],
        &["array<string, int|string>"],
    );

    let sa = t_sealed_list(BTreeMap::from([(0usize, (false, ui(1)))]));
    expect("sealed_a", vec![sa.clone()], &["list{int(1)}"]);
    expect("sealed_a∨unsealed", vec![sa, t_list(u(t_int()), false)], &["list<int>"]);
}

#[test]
fn cross_family_megatable() {
    expect("int∨object", vec![t_int(), t_object_any()], &["int", "object"]);
    expect("object∨int", vec![t_object_any(), t_int()], &["int", "object"]);
    expect("int∨resource", vec![t_int(), t_resource()], &["int", "resource"]);
    expect("int∨open", vec![t_int(), t_open_resource()], &["int", "open-resource"]);
    expect("string∨object", vec![t_string(), t_object_any()], &["object", "string"]);
    expect("string∨resource", vec![t_string(), t_resource()], &["resource", "string"]);
    expect("string∨array{}", vec![t_string(), t_empty_array()], &["array{}", "string"]);
    expect("string∨list", vec![t_string(), t_list(u(t_int()), false)], &["list<int>", "string"]);
    expect("float∨object", vec![t_float(), t_object_any()], &["float", "object"]);
    expect("bool∨object", vec![t_bool(), t_object_any()], &["bool", "object"]);
    expect("null∨object", vec![null(), t_object_any()], &["null", "object"]);
    expect("null∨array{}", vec![null(), t_empty_array()], &["array{}", "null"]);
    expect("null∨resource", vec![null(), t_resource()], &["null", "resource"]);
    expect("Foo∨resource", vec![t_named("Foo"), t_resource()], &["Foo", "resource"]);
    expect("Foo∨E", vec![t_named("Foo"), t_enum("E")], &["Foo", "enum(E)"]);
    expect("E∨resource", vec![t_enum("E"), t_resource()], &["enum(E)", "resource"]);
    expect("array{}∨object", vec![t_empty_array(), t_object_any()], &["array{}", "object"]);
    expect("list∨object", vec![t_list(u(t_int()), false), t_object_any()], &["list<int>", "object"]);
}

#[test]
fn scalar_synthesis_megatable() {
    expect("int∨string∨float∨bool", vec![t_int(), t_string(), t_float(), t_bool()], &["scalar"]);
    expect(
        "int∨string∨float∨true (no bool synthesis without false)",
        vec![t_int(), t_string(), t_float(), t_true()],
        &["float", "int", "string", "true"],
    );
    expect(
        "int∨string∨float∨false",
        vec![t_int(), t_string(), t_float(), t_false()],
        &["false", "float", "int", "string"],
    );
    expect(
        "int∨string∨float∨true∨false (bool synthesised → scalar)",
        vec![t_int(), t_string(), t_float(), t_true(), t_false()],
        &["scalar"],
    );
    expect("int∨string∨float∨bool∨null", vec![t_int(), t_string(), t_float(), t_bool(), null()], &["null", "scalar"]);
    expect("int∨string∨bool", vec![t_int(), t_string(), t_bool()], &["bool", "int", "string"]);
    expect(
        "lit_int∨string∨float∨bool",
        vec![t_lit_int(5), t_string(), t_float(), t_bool()],
        &["bool", "float", "int(5)", "string"],
    );
}
