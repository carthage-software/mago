mod common;

use common::*;

use mago_oracle::ty::Atom;

#[track_caller]
fn check<'arena>(f: &mut Fixture<'_, 'arena>, label: &str, input: Vec<Atom<'arena>>, expected: &[Atom<'arena>]) {
    let result = combine_default(f, input);
    let mut actual = result.clone();
    actual.sort_unstable();
    let mut expected_sorted = expected.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(actual, expected_sorted, "case `{label}` failed: got {result:?}, expected {expected:?}");
}

#[test]
fn primitive_pairs_int() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, "int | int", vec![f.t_int(), f.t_int()], &[f.t_int()]);
        check(f, "int | string", vec![f.t_int(), f.t_string()], &[f.t_int(), f.t_string()]);
        check(f, "int | float", vec![f.t_int(), f.t_float()], &[f.t_float(), f.t_int()]);
        check(f, "int | bool", vec![f.t_int(), f.t_bool()], &[f.t_bool(), f.t_int()]);
        check(f, "int | true", vec![f.t_int(), f.t_true()], &[f.t_int(), f.t_true()]);
        check(f, "int | false", vec![f.t_int(), f.t_false()], &[f.t_false(), f.t_int()]);
        check(f, "int | null", vec![f.t_int(), f.null()], &[f.t_int(), f.null()]);
        check(f, "int | object", vec![f.t_int(), f.t_object_any()], &[f.t_int(), f.t_object_any()]);
        check(f, "int | Foo", vec![f.t_int(), foo], &[foo, f.t_int()]);
        check(f, "int | resource", vec![f.t_int(), f.t_resource()], &[f.t_int(), f.t_resource()]);
        check(f, "int | array{}", vec![f.t_int(), f.t_empty_array()], &[f.t_empty_array(), f.t_int()]);
        check(f, "int | class-string", vec![f.t_int(), f.t_class_string()], &[f.t_class_string(), f.t_int()]);
        check(f, "int | never", vec![f.t_int(), f.never()], &[f.t_int()]);
        check(f, "int | void", vec![f.t_int(), f.void()], &[f.t_int(), f.null()]);
        check(f, "int | mixed", vec![f.t_int(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_string() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, "string | string", vec![f.t_string(), f.t_string()], &[f.t_string()]);
        check(f, "string | int", vec![f.t_string(), f.t_int()], &[f.t_int(), f.t_string()]);
        check(f, "string | float", vec![f.t_string(), f.t_float()], &[f.t_float(), f.t_string()]);
        check(f, "string | bool", vec![f.t_string(), f.t_bool()], &[f.t_bool(), f.t_string()]);
        check(f, "string | null", vec![f.t_string(), f.null()], &[f.null(), f.t_string()]);
        check(f, "string | object", vec![f.t_string(), f.t_object_any()], &[f.t_object_any(), f.t_string()]);
        check(f, "string | Foo", vec![f.t_string(), foo], &[foo, f.t_string()]);
        check(f, "string | resource", vec![f.t_string(), f.t_resource()], &[f.t_resource(), f.t_string()]);
        check(f, "string | array{}", vec![f.t_string(), f.t_empty_array()], &[f.t_empty_array(), f.t_string()]);
        check(f, "string | class-string", vec![f.t_string(), f.t_class_string()], &[f.t_string()]);
        check(f, "string | never", vec![f.t_string(), f.never()], &[f.t_string()]);
        check(f, "string | void", vec![f.t_string(), f.void()], &[f.t_string(), f.null()]);
        check(f, "string | mixed", vec![f.t_string(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_float() {
    fixture(|f| {
        check(f, "float | float", vec![f.t_float(), f.t_float()], &[f.t_float()]);
        check(f, "float | int", vec![f.t_float(), f.t_int()], &[f.t_float(), f.t_int()]);
        check(f, "float | string", vec![f.t_float(), f.t_string()], &[f.t_float(), f.t_string()]);
        check(f, "float | bool", vec![f.t_float(), f.t_bool()], &[f.t_bool(), f.t_float()]);
        check(f, "float | null", vec![f.t_float(), f.null()], &[f.t_float(), f.null()]);
        check(f, "float | object", vec![f.t_float(), f.t_object_any()], &[f.t_float(), f.t_object_any()]);
        check(f, "float | resource", vec![f.t_float(), f.t_resource()], &[f.t_float(), f.t_resource()]);
        check(f, "float | never", vec![f.t_float(), f.never()], &[f.t_float()]);
        check(f, "float | void", vec![f.t_float(), f.void()], &[f.t_float(), f.null()]);
        check(f, "float | mixed", vec![f.t_float(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_bool() {
    fixture(|f| {
        check(f, "bool | bool", vec![f.t_bool(), f.t_bool()], &[f.t_bool()]);
        check(f, "bool | int", vec![f.t_bool(), f.t_int()], &[f.t_bool(), f.t_int()]);
        check(f, "bool | string", vec![f.t_bool(), f.t_string()], &[f.t_bool(), f.t_string()]);
        check(f, "bool | float", vec![f.t_bool(), f.t_float()], &[f.t_bool(), f.t_float()]);
        check(f, "bool | null", vec![f.t_bool(), f.null()], &[f.t_bool(), f.null()]);
        check(f, "bool | object", vec![f.t_bool(), f.t_object_any()], &[f.t_bool(), f.t_object_any()]);
        check(f, "bool | resource", vec![f.t_bool(), f.t_resource()], &[f.t_bool(), f.t_resource()]);
        check(f, "bool | true", vec![f.t_bool(), f.t_true()], &[f.t_bool()]);
        check(f, "bool | false", vec![f.t_bool(), f.t_false()], &[f.t_bool()]);
        check(f, "bool | array{}", vec![f.t_bool(), f.t_empty_array()], &[f.t_empty_array(), f.t_bool()]);
        check(f, "bool | never", vec![f.t_bool(), f.never()], &[f.t_bool()]);
        check(f, "bool | void", vec![f.t_bool(), f.void()], &[f.t_bool(), f.null()]);
        check(f, "bool | mixed", vec![f.t_bool(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_null() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, "null | null", vec![f.null(), f.null()], &[f.null()]);
        check(f, "null | int", vec![f.null(), f.t_int()], &[f.t_int(), f.null()]);
        check(f, "null | string", vec![f.null(), f.t_string()], &[f.null(), f.t_string()]);
        check(f, "null | float", vec![f.null(), f.t_float()], &[f.t_float(), f.null()]);
        check(f, "null | bool", vec![f.null(), f.t_bool()], &[f.t_bool(), f.null()]);
        check(f, "null | object", vec![f.null(), f.t_object_any()], &[f.null(), f.t_object_any()]);
        check(f, "null | Foo", vec![f.null(), foo], &[foo, f.null()]);
        check(f, "null | resource", vec![f.null(), f.t_resource()], &[f.null(), f.t_resource()]);
        check(f, "null | array{}", vec![f.null(), f.t_empty_array()], &[f.t_empty_array(), f.null()]);
        check(f, "null | class-string", vec![f.null(), f.t_class_string()], &[f.t_class_string(), f.null()]);
        check(f, "null | never", vec![f.null(), f.never()], &[f.null()]);
        check(f, "null | void", vec![f.null(), f.void()], &[f.null()]);
        check(f, "null | mixed", vec![f.null(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_void() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, "void | void", vec![f.void(), f.void()], &[f.void()]);
        check(f, "void | int", vec![f.void(), f.t_int()], &[f.t_int(), f.null()]);
        check(f, "void | string", vec![f.void(), f.t_string()], &[f.t_string(), f.null()]);
        check(f, "void | float", vec![f.void(), f.t_float()], &[f.t_float(), f.null()]);
        check(f, "void | bool", vec![f.void(), f.t_bool()], &[f.t_bool(), f.null()]);
        check(f, "void | null", vec![f.void(), f.null()], &[f.null()]);
        check(f, "void | object", vec![f.void(), f.t_object_any()], &[f.t_object_any(), f.null()]);
        check(f, "void | Foo", vec![f.void(), foo], &[foo, f.null()]);
        check(f, "void | resource", vec![f.void(), f.t_resource()], &[f.t_resource(), f.null()]);
        check(f, "void | array{}", vec![f.void(), f.t_empty_array()], &[f.t_empty_array(), f.null()]);
        check(f, "void | never", vec![f.void(), f.never()], &[f.void()]);
        check(f, "void | mixed", vec![f.void(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_never() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, "never | never", vec![f.never(), f.never()], &[f.never()]);
        check(f, "never | int", vec![f.never(), f.t_int()], &[f.t_int()]);
        check(f, "never | string", vec![f.never(), f.t_string()], &[f.t_string()]);
        check(f, "never | float", vec![f.never(), f.t_float()], &[f.t_float()]);
        check(f, "never | bool", vec![f.never(), f.t_bool()], &[f.t_bool()]);
        check(f, "never | null", vec![f.never(), f.null()], &[f.null()]);
        check(f, "never | void", vec![f.never(), f.void()], &[f.void()]);
        check(f, "never | object", vec![f.never(), f.t_object_any()], &[f.t_object_any()]);
        check(f, "never | Foo", vec![f.never(), foo], &[foo]);
        check(f, "never | resource", vec![f.never(), f.t_resource()], &[f.t_resource()]);
        check(f, "never | array{}", vec![f.never(), f.t_empty_array()], &[f.t_empty_array()]);
        check(f, "never | mixed", vec![f.never(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_mixed() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, "mixed | mixed", vec![f.mixed(), f.mixed()], &[f.mixed()]);
        check(f, "mixed | int", vec![f.mixed(), f.t_int()], &[f.mixed()]);
        check(f, "mixed | string", vec![f.mixed(), f.t_string()], &[f.mixed()]);
        check(f, "mixed | float", vec![f.mixed(), f.t_float()], &[f.mixed()]);
        check(f, "mixed | bool", vec![f.mixed(), f.t_bool()], &[f.mixed()]);
        check(f, "mixed | null", vec![f.mixed(), f.null()], &[f.mixed()]);
        check(f, "mixed | void", vec![f.mixed(), f.void()], &[f.mixed()]);
        check(f, "mixed | object", vec![f.mixed(), f.t_object_any()], &[f.mixed()]);
        check(f, "mixed | Foo", vec![f.mixed(), foo], &[f.mixed()]);
        check(f, "mixed | resource", vec![f.mixed(), f.t_resource()], &[f.mixed()]);
        check(f, "mixed | array{}", vec![f.mixed(), f.t_empty_array()], &[f.mixed()]);
        check(f, "mixed | never", vec![f.mixed(), f.never()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_array_key() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        check(f, "array-key | array-key", vec![f.t_array_key(), f.t_array_key()], &[f.t_array_key()]);
        check(f, "array-key | int", vec![f.t_array_key(), f.t_int()], &[f.t_array_key()]);
        check(f, "array-key | string", vec![f.t_array_key(), f.t_string()], &[f.t_array_key()]);
        check(f, "array-key | int(5)", vec![f.t_array_key(), f.t_lit_int(5)], &[f.t_array_key()]);
        check(f, "array-key | 'hi'", vec![f.t_array_key(), hi], &[f.t_array_key()]);
        check(f, "array-key | class-string", vec![f.t_array_key(), f.t_class_string()], &[f.t_array_key()]);
        check(f, "array-key | float", vec![f.t_array_key(), f.t_float()], &[f.t_array_key(), f.t_float()]);
        check(f, "array-key | bool", vec![f.t_array_key(), f.t_bool()], &[f.t_array_key(), f.t_bool()]);
        check(f, "array-key | null", vec![f.t_array_key(), f.null()], &[f.t_array_key(), f.null()]);
        check(f, "array-key | numeric", vec![f.t_array_key(), f.t_numeric()], &[f.t_array_key(), f.t_numeric()]);
        check(f, "array-key | object", vec![f.t_array_key(), f.t_object_any()], &[f.t_array_key(), f.t_object_any()]);
        check(f, "array-key | scalar", vec![f.t_array_key(), f.t_scalar()], &[f.t_scalar()]);
        check(f, "array-key | never", vec![f.t_array_key(), f.never()], &[f.t_array_key()]);
        check(f, "array-key | mixed", vec![f.t_array_key(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_scalar() {
    fixture(|f| {
        check(f, "scalar | scalar", vec![f.t_scalar(), f.t_scalar()], &[f.t_scalar()]);
        check(f, "scalar | int", vec![f.t_scalar(), f.t_int()], &[f.t_scalar()]);
        check(f, "scalar | string", vec![f.t_scalar(), f.t_string()], &[f.t_scalar()]);
        check(f, "scalar | float", vec![f.t_scalar(), f.t_float()], &[f.t_scalar()]);
        check(f, "scalar | bool", vec![f.t_scalar(), f.t_bool()], &[f.t_scalar()]);
        check(f, "scalar | true", vec![f.t_scalar(), f.t_true()], &[f.t_scalar()]);
        check(f, "scalar | numeric", vec![f.t_scalar(), f.t_numeric()], &[f.t_scalar()]);
        check(f, "scalar | array-key", vec![f.t_scalar(), f.t_array_key()], &[f.t_scalar()]);
        check(f, "scalar | class-string", vec![f.t_scalar(), f.t_class_string()], &[f.t_scalar()]);
        check(f, "scalar | null", vec![f.t_scalar(), f.null()], &[f.t_scalar(), f.null()]);
        check(f, "scalar | object", vec![f.t_scalar(), f.t_object_any()], &[f.t_scalar(), f.t_object_any()]);
        check(f, "scalar | resource", vec![f.t_scalar(), f.t_resource()], &[f.t_scalar(), f.t_resource()]);
        check(f, "scalar | array{}", vec![f.t_scalar(), f.t_empty_array()], &[f.t_scalar(), f.t_empty_array()]);
        check(f, "scalar | never", vec![f.t_scalar(), f.never()], &[f.t_scalar()]);
        check(f, "scalar | mixed", vec![f.t_scalar(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_object() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let enum_e = f.t_enum("E");
        check(f, "object | object", vec![f.t_object_any(), f.t_object_any()], &[f.t_object_any()]);
        check(f, "object | Foo", vec![f.t_object_any(), foo], &[f.t_object_any()]);
        check(f, "Foo | object", vec![foo, f.t_object_any()], &[f.t_object_any()]);
        check(f, "object | Bar", vec![f.t_object_any(), bar], &[f.t_object_any()]);
        check(f, "object | enum (suffete absorbs)", vec![f.t_object_any(), enum_e], &[f.t_object_any()]);
        check(f, "object | int", vec![f.t_object_any(), f.t_int()], &[f.t_int(), f.t_object_any()]);
        check(f, "object | resource", vec![f.t_object_any(), f.t_resource()], &[f.t_object_any(), f.t_resource()]);
        check(f, "object | array{}", vec![f.t_object_any(), f.t_empty_array()], &[f.t_empty_array(), f.t_object_any()]);
        check(f, "object | never", vec![f.t_object_any(), f.never()], &[f.t_object_any()]);
        check(f, "object | mixed", vec![f.t_object_any(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_resource() {
    fixture(|f| {
        check(f, "resource | resource", vec![f.t_resource(), f.t_resource()], &[f.t_resource()]);
        check(f, "resource | open", vec![f.t_resource(), f.t_open_resource()], &[f.t_resource()]);
        check(f, "resource | closed", vec![f.t_resource(), f.t_closed_resource()], &[f.t_resource()]);
        check(f, "open | closed", vec![f.t_open_resource(), f.t_closed_resource()], &[f.t_resource()]);
        check(f, "resource | int", vec![f.t_resource(), f.t_int()], &[f.t_int(), f.t_resource()]);
        check(f, "open | int", vec![f.t_open_resource(), f.t_int()], &[f.t_int(), f.t_open_resource()]);
        check(f, "closed | string", vec![f.t_closed_resource(), f.t_string()], &[f.t_closed_resource(), f.t_string()]);
        check(f, "resource | never", vec![f.t_resource(), f.never()], &[f.t_resource()]);
        check(f, "resource | mixed", vec![f.t_resource(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_array() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let list_int = f.t_list(int_type, false);
        let keyed_string_int = f.t_keyed_unsealed(string_type, int_type, false);

        check(f, "array{} | array{}", vec![f.t_empty_array(), f.t_empty_array()], &[f.t_empty_array()]);
        check(f, "list<int> | list<int>", vec![list_int, list_int], &[list_int]);
        check(f, "array{} | list<int>", vec![f.t_empty_array(), list_int], &[list_int]);
        check(f, "array{} | keyed<string,int>", vec![f.t_empty_array(), keyed_string_int], &[keyed_string_int]);
        check(f, "array{} | int", vec![f.t_empty_array(), f.t_int()], &[f.t_empty_array(), f.t_int()]);
        check(f, "array{} | string", vec![f.t_empty_array(), f.t_string()], &[f.t_empty_array(), f.t_string()]);
        check(f, "array{} | null", vec![f.t_empty_array(), f.null()], &[f.t_empty_array(), f.null()]);
        check(f, "array{} | object", vec![f.t_empty_array(), f.t_object_any()], &[f.t_empty_array(), f.t_object_any()]);
        check(f, "list<int> | int", vec![list_int, f.t_int()], &[list_int, f.t_int()]);
        check(f, "keyed<string,int> | string", vec![keyed_string_int, f.t_string()], &[keyed_string_int, f.t_string()]);
        check(f, "array{} | never", vec![f.t_empty_array(), f.never()], &[f.t_empty_array()]);
        check(f, "array{} | mixed", vec![f.t_empty_array(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn primitive_pairs_class_like() {
    fixture(|f| {
        check(f, "class-string | class-string", vec![f.t_class_string(), f.t_class_string()], &[f.t_class_string()]);
        check(
            f,
            "interface-string | interface-string",
            vec![f.t_interface_string(), f.t_interface_string()],
            &[f.t_interface_string()],
        );
        check(
            f,
            "class-string | interface-string",
            vec![f.t_class_string(), f.t_interface_string()],
            &[f.t_class_string(), f.t_interface_string()],
        );
        check(
            f,
            "class-string | enum-string",
            vec![f.t_class_string(), f.t_enum_string()],
            &[f.t_class_string(), f.t_enum_string()],
        );
        check(
            f,
            "class-string | trait-string",
            vec![f.t_class_string(), f.t_trait_string()],
            &[f.t_class_string(), f.t_trait_string()],
        );
        check(f, "class-string | string", vec![f.t_class_string(), f.t_string()], &[f.t_string()]);
        check(f, "class-string | int", vec![f.t_class_string(), f.t_int()], &[f.t_class_string(), f.t_int()]);
        check(f, "class-string | null", vec![f.t_class_string(), f.null()], &[f.t_class_string(), f.null()]);
        check(f, "class-string | never", vec![f.t_class_string(), f.never()], &[f.t_class_string()]);
        check(f, "class-string | mixed", vec![f.t_class_string(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn numeric_pairs() {
    fixture(|f| {
        let lit_float = f.t_lit_float(1.5);
        check(f, "numeric | numeric", vec![f.t_numeric(), f.t_numeric()], &[f.t_numeric()]);
        check(f, "numeric | int", vec![f.t_numeric(), f.t_int()], &[f.t_numeric()]);
        check(f, "numeric | float", vec![f.t_numeric(), f.t_float()], &[f.t_numeric()]);
        check(f, "numeric | int(5)", vec![f.t_numeric(), f.t_lit_int(5)], &[f.t_numeric()]);
        check(f, "numeric | float(1.5)", vec![f.t_numeric(), lit_float], &[f.t_numeric()]);
        check(f, "numeric | numeric-string", vec![f.t_numeric(), f.t_numeric_string()], &[f.t_numeric()]);
        check(f, "numeric | string", vec![f.t_numeric(), f.t_string()], &[f.t_numeric(), f.t_string()]);
        check(f, "numeric | bool", vec![f.t_numeric(), f.t_bool()], &[f.t_numeric(), f.t_bool()]);
        check(f, "numeric | null", vec![f.t_numeric(), f.null()], &[f.t_numeric(), f.null()]);
        check(f, "numeric | array-key", vec![f.t_numeric(), f.t_array_key()], &[f.t_numeric(), f.t_array_key()]);
        check(f, "numeric | object", vec![f.t_numeric(), f.t_object_any()], &[f.t_numeric(), f.t_object_any()]);
        check(f, "numeric | scalar", vec![f.t_numeric(), f.t_scalar()], &[f.t_scalar()]);
        check(f, "numeric | never", vec![f.t_numeric(), f.never()], &[f.t_numeric()]);
        check(f, "numeric | mixed", vec![f.t_numeric(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn string_refinement_pairs() {
    fixture(|f| {
        check(
            f,
            "non-empty | non-empty",
            vec![f.t_non_empty_string(), f.t_non_empty_string()],
            &[f.t_non_empty_string()],
        );
        check(f, "string | non-empty", vec![f.t_string(), f.t_non_empty_string()], &[f.t_string()]);
        check(f, "string | numeric", vec![f.t_string(), f.t_numeric_string()], &[f.t_string()]);
        check(f, "string | lowercase", vec![f.t_string(), f.t_lower_string()], &[f.t_string()]);
        check(f, "string | uppercase", vec![f.t_string(), f.t_upper_string()], &[f.t_string()]);
        check(f, "string | truthy", vec![f.t_string(), f.t_truthy_string()], &[f.t_string()]);
        check(f, "non-empty | truthy", vec![f.t_non_empty_string(), f.t_truthy_string()], &[f.t_non_empty_string()]);
        check(f, "truthy | non-empty", vec![f.t_truthy_string(), f.t_non_empty_string()], &[f.t_non_empty_string()]);
        check(f, "lowercase | uppercase", vec![f.t_lower_string(), f.t_upper_string()], &[f.t_string()]);
        check(f, "non-empty | lowercase", vec![f.t_non_empty_string(), f.t_lower_string()], &[f.t_string()]);
        check(f, "lowercase | numeric", vec![f.t_lower_string(), f.t_numeric_string()], &[f.t_string()]);
        check(f, "callable-string | truthy", vec![f.t_callable_string(), f.t_truthy_string()], &[f.t_truthy_string()]);
    });
}
