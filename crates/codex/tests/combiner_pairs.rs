mod combiner_common;

use combiner_common::*;

use mago_codex::ttype::atomic::TAtomic;

/// Asserts that `combine(input)` produces atoms whose ids (as a sorted vec) equal `expected`.
fn check(label: &str, input: Vec<TAtomic>, expected: &[&str]) {
    let result = combine_default(input);
    let mut actual: Vec<String> = result.iter().map(atomic_id_string).collect();
    actual.sort();
    let mut expected_sorted: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    expected_sorted.sort();
    assert_eq!(actual, expected_sorted, "case `{label}` failed");
}

#[test]
fn primitive_pairs_int() {
    check("int ∨ int", vec![t_int(), t_int()], &["int"]);
    check("int ∨ string", vec![t_int(), t_string()], &["int", "string"]);
    check("int ∨ float", vec![t_int(), t_float()], &["float", "int"]);
    check("int ∨ bool", vec![t_int(), t_bool()], &["bool", "int"]);
    check("int ∨ true", vec![t_int(), t_true()], &["int", "true"]);
    check("int ∨ false", vec![t_int(), t_false()], &["false", "int"]);
    check("int ∨ null", vec![t_int(), null()], &["int", "null"]);
    check("int ∨ object", vec![t_int(), t_object_any()], &["int", "object"]);
    check("int ∨ Foo", vec![t_int(), t_named("Foo")], &["Foo", "int"]);
    check("int ∨ resource", vec![t_int(), t_resource()], &["int", "resource"]);
    check("int ∨ array{}", vec![t_int(), t_empty_array()], &["array{}", "int"]);
    check("int ∨ list<int>", vec![t_int(), t_list(u(t_int()), false)], &["int", "list<int>"]);
    check("int ∨ class-string", vec![t_int(), t_class_string()], &["class-string", "int"]);
    check("int ∨ array-key", vec![t_int(), t_array_key()], &["array-key"]);
    check("int ∨ scalar", vec![t_int(), t_scalar()], &["scalar"]);
    check("int ∨ never", vec![t_int(), never()], &["int"]);
    check("int ∨ void", vec![t_int(), void()], &["int", "null"]);
    check("int ∨ mixed", vec![t_int(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_string() {
    check("string ∨ string", vec![t_string(), t_string()], &["string"]);
    check("string ∨ int", vec![t_string(), t_int()], &["int", "string"]);
    check("string ∨ float", vec![t_string(), t_float()], &["float", "string"]);
    check("string ∨ bool", vec![t_string(), t_bool()], &["bool", "string"]);
    check("string ∨ null", vec![t_string(), null()], &["null", "string"]);
    check("string ∨ object", vec![t_string(), t_object_any()], &["object", "string"]);
    check("string ∨ Foo", vec![t_string(), t_named("Foo")], &["Foo", "string"]);
    check("string ∨ resource", vec![t_string(), t_resource()], &["resource", "string"]);
    check("string ∨ array{}", vec![t_string(), t_empty_array()], &["array{}", "string"]);
    check("string ∨ list<int>", vec![t_string(), t_list(u(t_int()), false)], &["list<int>", "string"]);
    check("string ∨ class-string", vec![t_string(), t_class_string()], &["class-string", "string"]);
    check("string ∨ array-key", vec![t_string(), t_array_key()], &["array-key"]);
    check("string ∨ scalar", vec![t_string(), t_scalar()], &["scalar"]);
    check("string ∨ never", vec![t_string(), never()], &["string"]);
    check("string ∨ void", vec![t_string(), void()], &["null", "string"]);
    check("string ∨ mixed", vec![t_string(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_float() {
    check("float ∨ float", vec![t_float(), t_float()], &["float"]);
    check("float ∨ int", vec![t_float(), t_int()], &["float", "int"]);
    check("float ∨ string", vec![t_float(), t_string()], &["float", "string"]);
    check("float ∨ bool", vec![t_float(), t_bool()], &["bool", "float"]);
    check("float ∨ null", vec![t_float(), null()], &["float", "null"]);
    check("float ∨ object", vec![t_float(), t_object_any()], &["float", "object"]);
    check("float ∨ resource", vec![t_float(), t_resource()], &["float", "resource"]);
    check("float ∨ scalar", vec![t_float(), t_scalar()], &["scalar"]);
    check("float ∨ never", vec![t_float(), never()], &["float"]);
    check("float ∨ void", vec![t_float(), void()], &["float", "null"]);
    check("float ∨ mixed", vec![t_float(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_bool() {
    check("bool ∨ bool", vec![t_bool(), t_bool()], &["bool"]);
    check("bool ∨ int", vec![t_bool(), t_int()], &["bool", "int"]);
    check("bool ∨ string", vec![t_bool(), t_string()], &["bool", "string"]);
    check("bool ∨ float", vec![t_bool(), t_float()], &["bool", "float"]);
    check("bool ∨ null", vec![t_bool(), null()], &["bool", "null"]);
    check("bool ∨ object", vec![t_bool(), t_object_any()], &["bool", "object"]);
    check("bool ∨ resource", vec![t_bool(), t_resource()], &["bool", "resource"]);
    check("bool ∨ scalar", vec![t_bool(), t_scalar()], &["scalar"]);
    check("bool ∨ true", vec![t_bool(), t_true()], &["bool"]);
    check("bool ∨ false", vec![t_bool(), t_false()], &["bool"]);
    check("bool ∨ array{}", vec![t_bool(), t_empty_array()], &["array{}", "bool"]);
    check("bool ∨ never", vec![t_bool(), never()], &["bool"]);
    check("bool ∨ void", vec![t_bool(), void()], &["bool", "null"]);
    check("bool ∨ mixed", vec![t_bool(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_null() {
    check("null ∨ null", vec![null(), null()], &["null"]);
    check("null ∨ int", vec![null(), t_int()], &["int", "null"]);
    check("null ∨ string", vec![null(), t_string()], &["null", "string"]);
    check("null ∨ float", vec![null(), t_float()], &["float", "null"]);
    check("null ∨ bool", vec![null(), t_bool()], &["bool", "null"]);
    check("null ∨ object", vec![null(), t_object_any()], &["null", "object"]);
    check("null ∨ Foo", vec![null(), t_named("Foo")], &["Foo", "null"]);
    check("null ∨ resource", vec![null(), t_resource()], &["null", "resource"]);
    check("null ∨ array{}", vec![null(), t_empty_array()], &["array{}", "null"]);
    check("null ∨ list<int>", vec![null(), t_list(u(t_int()), false)], &["list<int>", "null"]);
    check("null ∨ class-string", vec![null(), t_class_string()], &["class-string", "null"]);
    check("null ∨ never", vec![null(), never()], &["null"]);
    check("null ∨ void", vec![null(), void()], &["null"]);
    check("null ∨ mixed", vec![null(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_void() {
    check("void ∨ void", vec![void(), void()], &["void"]);
    check("void ∨ int", vec![void(), t_int()], &["int", "null"]);
    check("void ∨ string", vec![void(), t_string()], &["null", "string"]);
    check("void ∨ float", vec![void(), t_float()], &["float", "null"]);
    check("void ∨ bool", vec![void(), t_bool()], &["bool", "null"]);
    check("void ∨ null", vec![void(), null()], &["null"]);
    check("void ∨ object", vec![void(), t_object_any()], &["null", "object"]);
    check("void ∨ Foo", vec![void(), t_named("Foo")], &["Foo", "null"]);
    check("void ∨ resource", vec![void(), t_resource()], &["null", "resource"]);
    check("void ∨ array{}", vec![void(), t_empty_array()], &["array{}", "null"]);
    check("void ∨ never", vec![void(), never()], &["null"]);
    check("void ∨ mixed", vec![void(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_never() {
    check("never ∨ never", vec![never(), never()], &["never"]);
    check("never ∨ int", vec![never(), t_int()], &["int"]);
    check("never ∨ string", vec![never(), t_string()], &["string"]);
    check("never ∨ float", vec![never(), t_float()], &["float"]);
    check("never ∨ bool", vec![never(), t_bool()], &["bool"]);
    check("never ∨ null", vec![never(), null()], &["null"]);
    check("never ∨ void", vec![never(), void()], &["null"]);
    check("never ∨ object", vec![never(), t_object_any()], &["object"]);
    check("never ∨ Foo", vec![never(), t_named("Foo")], &["Foo"]);
    check("never ∨ resource", vec![never(), t_resource()], &["resource"]);
    check("never ∨ array{}", vec![never(), t_empty_array()], &["array{}"]);
    check("never ∨ mixed", vec![never(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_mixed() {
    check("mixed ∨ mixed", vec![mixed(), mixed()], &["mixed"]);
    check("mixed ∨ int", vec![mixed(), t_int()], &["mixed"]);
    check("mixed ∨ string", vec![mixed(), t_string()], &["mixed"]);
    check("mixed ∨ float", vec![mixed(), t_float()], &["mixed"]);
    check("mixed ∨ bool", vec![mixed(), t_bool()], &["mixed"]);
    check("mixed ∨ null", vec![mixed(), null()], &["mixed"]);
    check("mixed ∨ void", vec![mixed(), void()], &["mixed"]);
    check("mixed ∨ object", vec![mixed(), t_object_any()], &["mixed"]);
    check("mixed ∨ Foo", vec![mixed(), t_named("Foo")], &["mixed"]);
    check("mixed ∨ resource", vec![mixed(), t_resource()], &["mixed"]);
    check("mixed ∨ array{}", vec![mixed(), t_empty_array()], &["mixed"]);
    check("mixed ∨ list<int>", vec![mixed(), t_list(u(t_int()), false)], &["mixed"]);
    check("mixed ∨ never", vec![mixed(), never()], &["mixed"]);
}

#[test]
fn primitive_pairs_array_key() {
    check("array-key ∨ array-key", vec![t_array_key(), t_array_key()], &["array-key"]);
    check("array-key ∨ int", vec![t_array_key(), t_int()], &["array-key"]);
    check("array-key ∨ string", vec![t_array_key(), t_string()], &["array-key"]);
    check("array-key ∨ float", vec![t_array_key(), t_float()], &["array-key", "float"]);
    check("array-key ∨ bool", vec![t_array_key(), t_bool()], &["array-key", "bool"]);
    check("array-key ∨ null", vec![t_array_key(), null()], &["array-key", "null"]);
    check("array-key ∨ object", vec![t_array_key(), t_object_any()], &["array-key", "object"]);
    check("array-key ∨ resource", vec![t_array_key(), t_resource()], &["array-key", "resource"]);
    check("array-key ∨ scalar", vec![t_array_key(), t_scalar()], &["scalar"]);
    check("array-key ∨ never", vec![t_array_key(), never()], &["array-key"]);
    check("array-key ∨ void", vec![t_array_key(), void()], &["array-key", "null"]);
    check("array-key ∨ mixed", vec![t_array_key(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_scalar() {
    check("scalar ∨ scalar", vec![t_scalar(), t_scalar()], &["scalar"]);
    check("scalar ∨ int", vec![t_scalar(), t_int()], &["scalar"]);
    check("scalar ∨ string", vec![t_scalar(), t_string()], &["scalar"]);
    check("scalar ∨ float", vec![t_scalar(), t_float()], &["scalar"]);
    check("scalar ∨ bool", vec![t_scalar(), t_bool()], &["bool", "scalar"]);
    check("bool ∨ scalar", vec![t_bool(), t_scalar()], &["scalar"]);
    check("scalar ∨ true", vec![t_scalar(), t_true()], &["scalar", "true"]);
    check("scalar ∨ false", vec![t_scalar(), t_false()], &["false", "scalar"]);
    check("scalar ∨ null", vec![t_scalar(), null()], &["null", "scalar"]);
    check("scalar ∨ object", vec![t_scalar(), t_object_any()], &["object", "scalar"]);
    check("scalar ∨ resource", vec![t_scalar(), t_resource()], &["resource", "scalar"]);
    check("scalar ∨ array{}", vec![t_scalar(), t_empty_array()], &["array{}", "scalar"]);
    check("scalar ∨ class-string", vec![t_scalar(), t_class_string()], &["class-string", "scalar"]);
    check("scalar ∨ array-key", vec![t_scalar(), t_array_key()], &["scalar"]);
    check("scalar ∨ numeric (numeric first)", vec![t_numeric(), t_scalar()], &["scalar"]);
    check("scalar ∨ numeric (scalar first)", vec![t_scalar(), t_numeric()], &["numeric", "scalar"]);
    check("scalar ∨ never", vec![t_scalar(), never()], &["scalar"]);
    check("scalar ∨ mixed", vec![t_scalar(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_object() {
    check("object ∨ object", vec![t_object_any(), t_object_any()], &["object"]);
    check("object ∨ Foo", vec![t_object_any(), t_named("Foo")], &["object"]);
    check("Foo ∨ object", vec![t_named("Foo"), t_object_any()], &["object"]);
    check("object ∨ Bar", vec![t_object_any(), t_named("Bar")], &["object"]);
    check("object ∨ enum", vec![t_object_any(), t_enum("E")], &["enum(E)", "object"]);
    check("object ∨ int", vec![t_object_any(), t_int()], &["int", "object"]);
    check("object ∨ resource", vec![t_object_any(), t_resource()], &["object", "resource"]);
    check("object ∨ array{}", vec![t_object_any(), t_empty_array()], &["array{}", "object"]);
    check("object ∨ never", vec![t_object_any(), never()], &["object"]);
    check("object ∨ mixed", vec![t_object_any(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_resource() {
    check("resource ∨ resource", vec![t_resource(), t_resource()], &["resource"]);
    check("resource ∨ open", vec![t_resource(), t_open_resource()], &["resource"]);
    check("resource ∨ closed", vec![t_resource(), t_closed_resource()], &["resource"]);
    check("open ∨ closed", vec![t_open_resource(), t_closed_resource()], &["resource"]);
    check("resource ∨ int", vec![t_resource(), t_int()], &["int", "resource"]);
    check("open ∨ int", vec![t_open_resource(), t_int()], &["int", "open-resource"]);
    check("closed ∨ string", vec![t_closed_resource(), t_string()], &["closed-resource", "string"]);
    check("resource ∨ never", vec![t_resource(), never()], &["resource"]);
    check("resource ∨ mixed", vec![t_resource(), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_array() {
    check("array{} ∨ array{}", vec![t_empty_array(), t_empty_array()], &["array{}"]);
    check("list ∨ list", vec![t_list(u(t_int()), false), t_list(u(t_int()), false)], &["list<int>"]);
    check("ne_list ∨ ne_list", vec![t_list(u(t_int()), true), t_list(u(t_int()), true)], &["non-empty-list<int>"]);
    check("ne_list ∨ list", vec![t_list(u(t_int()), true), t_list(u(t_int()), false)], &["list<int>"]);
    check(
        "list_int ∨ list_string",
        vec![t_list(u(t_int()), false), t_list(u(t_string()), false)],
        &["list<int|string>"],
    );
    check("array{} ∨ list", vec![t_empty_array(), t_list(u(t_int()), false)], &["array{}", "list<int>"]);
    check("list ∨ array{}", vec![t_list(u(t_int()), false), t_empty_array()], &["list<int>"]);
    check("list ∨ int", vec![t_list(u(t_int()), false), t_int()], &["int", "list<int>"]);
    check("list ∨ object", vec![t_list(u(t_int()), false), t_object_any()], &["list<int>", "object"]);
    check("list ∨ never", vec![t_list(u(t_int()), false), never()], &["list<int>"]);
    check("list ∨ mixed", vec![t_list(u(t_int()), false), mixed()], &["mixed"]);
}

#[test]
fn primitive_pairs_class_like() {
    check("class-string ∨ class-string", vec![t_class_string(), t_class_string()], &["class-string"]);
    check(
        "interface-string ∨ interface-string",
        vec![t_interface_string(), t_interface_string()],
        &["interface-string"],
    );
    check(
        "class-string ∨ interface-string",
        vec![t_class_string(), t_interface_string()],
        &["class-string", "interface-string"],
    );
    check("class-string ∨ enum-string", vec![t_class_string(), t_enum_string()], &["class-string", "enum-string"]);
    check("class-string ∨ trait-string", vec![t_class_string(), t_trait_string()], &["class-string", "trait-string"]);
    check("class-string ∨ string", vec![t_class_string(), t_string()], &["class-string", "string"]);
    check("class-string ∨ array-key", vec![t_class_string(), t_array_key()], &["array-key", "class-string"]);
    check("class-string ∨ scalar", vec![t_class_string(), t_scalar()], &["class-string", "scalar"]);
    check("class-string ∨ int", vec![t_class_string(), t_int()], &["class-string", "int"]);
    check("class-string ∨ null", vec![t_class_string(), null()], &["class-string", "null"]);
    check("class-string ∨ never", vec![t_class_string(), never()], &["class-string"]);
    check("class-string ∨ mixed", vec![t_class_string(), mixed()], &["mixed"]);
}

#[test]
fn numeric_pairs() {
    check("numeric ∨ numeric", vec![t_numeric(), t_numeric()], &["numeric"]);
    check("numeric ∨ int", vec![t_numeric(), t_int()], &["numeric"]);
    check("numeric ∨ float", vec![t_numeric(), t_float()], &["numeric"]);
    check("numeric ∨ lit_int", vec![t_numeric(), t_lit_int(5)], &["numeric"]);
    check("numeric ∨ lit_float", vec![t_numeric(), t_lit_float(1.5)], &["numeric"]);
    check("int ∨ numeric", vec![t_int(), t_numeric()], &["int", "numeric"]);
    check("float ∨ numeric", vec![t_float(), t_numeric()], &["float", "numeric"]);
    check("lit_int ∨ numeric", vec![t_lit_int(5), t_numeric()], &["int(5)", "numeric"]);
    check("lit_float ∨ numeric", vec![t_lit_float(1.5), t_numeric()], &["float(1.5)", "numeric"]);
    check("numeric ∨ string", vec![t_numeric(), t_string()], &["numeric", "string"]);
    check("string ∨ numeric", vec![t_string(), t_numeric()], &["numeric", "string"]);
    check("numeric ∨ bool", vec![t_numeric(), t_bool()], &["bool", "numeric"]);
    check("numeric ∨ null", vec![t_numeric(), null()], &["null", "numeric"]);
    check("numeric ∨ scalar", vec![t_numeric(), t_scalar()], &["scalar"]);
    check("scalar ∨ numeric", vec![t_scalar(), t_numeric()], &["numeric", "scalar"]);
    check("numeric ∨ never", vec![t_numeric(), never()], &["numeric"]);
    check("numeric ∨ void", vec![t_numeric(), void()], &["null", "numeric"]);
    check("numeric ∨ mixed", vec![t_numeric(), mixed()], &["mixed"]);
}

#[test]
fn string_refinement_pairs() {
    check("non-empty ∨ non-empty", vec![t_non_empty_string(), t_non_empty_string()], &["non-empty-string"]);
    check("non-empty ∨ string", vec![t_non_empty_string(), t_string()], &["string"]);
    check("string ∨ non-empty", vec![t_string(), t_non_empty_string()], &["string"]);
    check("non-empty ∨ truthy", vec![t_non_empty_string(), t_truthy_string()], &["non-empty-string"]);
    check("truthy ∨ non-empty", vec![t_truthy_string(), t_non_empty_string()], &["non-empty-string"]);
    check("non-empty ∨ numeric", vec![t_non_empty_string(), t_numeric_string()], &["non-empty-string"]);
    check("numeric ∨ non-empty", vec![t_numeric_string(), t_non_empty_string()], &["non-empty-string"]);
    check("lower ∨ upper", vec![t_lower_string(), t_upper_string()], &["string"]);
    check("upper ∨ lower", vec![t_upper_string(), t_lower_string()], &["string"]);
    check("lower ∨ non-empty", vec![t_lower_string(), t_non_empty_string()], &["string"]);
    check("lower ∨ truthy", vec![t_lower_string(), t_truthy_string()], &["string"]);
    check("lower ∨ numeric", vec![t_lower_string(), t_numeric_string()], &["string"]);
}
