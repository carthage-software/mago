mod combiner_common;

use combiner_common::*;

use mago_codex::ttype::atomic::TAtomic;

fn check(label: &str, input: Vec<TAtomic>, expected: &[&str]) {
    let result = combine_default(input);
    let mut actual: Vec<String> = result.iter().map(atomic_id_string).collect();
    actual.sort();
    let mut expected_sorted: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    expected_sorted.sort();
    assert_eq!(actual, expected_sorted, "case `{label}`");
}

#[test]
fn bool_triples() {
    check("true,false,bool", vec![t_true(), t_false(), t_bool()], &["bool"]);
    check("bool,true,false", vec![t_bool(), t_true(), t_false()], &["bool"]);
    check("false,bool,true", vec![t_false(), t_bool(), t_true()], &["bool"]);
    check("true,true,false", vec![t_true(), t_true(), t_false()], &["bool"]);
    check("false,false,true", vec![t_false(), t_false(), t_true()], &["bool"]);
    check("bool,bool,bool", vec![t_bool(), t_bool(), t_bool()], &["bool"]);
    check("true,true,true", vec![t_true(), t_true(), t_true()], &["true"]);
    check("false,false,false", vec![t_false(), t_false(), t_false()], &["false"]);
}

#[test]
fn int_lit_range_triples() {
    check("int,int(0),int(1)", vec![t_int(), t_lit_int(0), t_lit_int(1)], &["int"]);
    check("int(0),int,int(1)", vec![t_lit_int(0), t_int(), t_lit_int(1)], &["int"]);
    check("int(0),int(1),int(2)", vec![t_lit_int(0), t_lit_int(1), t_lit_int(2)], &["int(0)", "int(1)", "int(2)"]);
    check("Range(0,5),int(6),int(7)", vec![t_int_range(0, 5), t_lit_int(6), t_lit_int(7)], &["int<0, 7>"]);
    check(
        "Range(0,5),int(10),int(20)",
        vec![t_int_range(0, 5), t_lit_int(10), t_lit_int(20)],
        &["int(10)", "int(20)", "int<0, 5>"],
    );
    check("positive,int(0),int(-1)", vec![t_positive_int(), t_lit_int(0), t_lit_int(-1)], &["int<-1, max>"]);
    check(
        "From(5),To(0),int(2)",
        vec![t_int_from(5), t_int_to(0), t_lit_int(2)],
        &["int(2)", "int<5, max>", "non-positive-int"],
    );
}

#[test]
fn string_triples() {
    check("string,'a','b'", vec![t_string(), t_lit_string("a"), t_lit_string("b")], &["string"]);
    check(
        "'a','b','c'",
        vec![t_lit_string("a"), t_lit_string("b"), t_lit_string("c")],
        &["string('a')", "string('b')", "string('c')"],
    );
    check("non-empty,'a','b'", vec![t_non_empty_string(), t_lit_string("a"), t_lit_string("b")], &["non-empty-string"]);
    check(
        "non-empty,'a',''",
        vec![t_non_empty_string(), t_lit_string("a"), t_lit_string("")],
        &["non-empty-string", "string('')"],
    );
    check(
        "lower,'hi','world'",
        vec![t_lower_string(), t_lit_string("hi"), t_lit_string("world")],
        &["lowercase-string"],
    );
    check(
        "lower,'hi','HI'",
        vec![t_lower_string(), t_lit_string("hi"), t_lit_string("HI")],
        &["lowercase-string", "string('HI')"],
    );
    check("string,non-empty,'hi'", vec![t_string(), t_non_empty_string(), t_lit_string("hi")], &["string"]);
}

#[test]
fn null_triples() {
    check("null,int,string", vec![null(), t_int(), t_string()], &["int", "null", "string"]);
    check("null,never,int", vec![null(), never(), t_int()], &["int", "null"]);
    check("null,void,int", vec![null(), void(), t_int()], &["int", "null"]);
    check("null,null,int", vec![null(), null(), t_int()], &["int", "null"]);
    check("null,object,Foo", vec![null(), t_object_any(), t_named("Foo")], &["null", "object"]);
    check("null,Foo,Bar", vec![null(), t_named("Foo"), t_named("Bar")], &["Bar", "Foo", "null"]);
}

#[test]
fn never_triples_absorbed() {
    check("never,int,string", vec![never(), t_int(), t_string()], &["int", "string"]);
    check("int,never,string", vec![t_int(), never(), t_string()], &["int", "string"]);
    check("int,string,never", vec![t_int(), t_string(), never()], &["int", "string"]);
    check("never,never,int", vec![never(), never(), t_int()], &["int"]);
    check("never,never,never", vec![never(), never(), never()], &["never"]);
    check("never,Foo,Bar", vec![never(), t_named("Foo"), t_named("Bar")], &["Bar", "Foo"]);
    check("never,object,enum", vec![never(), t_object_any(), t_enum("E")], &["enum(E)", "object"]);
}

#[test]
fn void_triples() {
    check("void,int,string", vec![void(), t_int(), t_string()], &["int", "string"]);
    check("void,null,int", vec![void(), null(), t_int()], &["int", "null"]);
    check("void,never,int", vec![void(), never(), t_int()], &["int"]);
    check("void,never,never", vec![void(), never(), never()], &["never"]);
    check("void,void,int", vec![void(), void(), t_int()], &["int"]);
    check("void,void,void", vec![void(), void(), void()], &["void"]);
    check("void,Foo,Bar", vec![void(), t_named("Foo"), t_named("Bar")], &["Bar", "Foo"]);
}

#[test]
fn mixed_triples_dominate() {
    check("mixed,int,string", vec![mixed(), t_int(), t_string()], &["mixed"]);
    check("int,mixed,string", vec![t_int(), mixed(), t_string()], &["mixed"]);
    check("int,string,mixed", vec![t_int(), t_string(), mixed()], &["mixed"]);
    check("mixed,Foo,Bar", vec![mixed(), t_named("Foo"), t_named("Bar")], &["mixed"]);
    check("mixed,never,int", vec![mixed(), never(), t_int()], &["mixed"]);
}

#[test]
fn array_triples() {
    check(
        "array{},list,int",
        vec![t_empty_array(), t_list(u(t_int()), false), t_int()],
        &["array{}", "int", "list<int>"],
    );
    check(
        "list,list,int",
        vec![t_list(u(t_int()), false), t_list(u(t_string()), false), t_int()],
        &["int", "list<int|string>"],
    );
    check("array{},array{},array{}", vec![t_empty_array(), t_empty_array(), t_empty_array()], &["array{}"]);
    check(
        "list,list,list",
        vec![t_list(u(t_int()), false), t_list(u(t_int()), false), t_list(u(t_int()), false)],
        &["list<int>"],
    );
    check(
        "list,list_string,list_float",
        vec![t_list(u(t_int()), false), t_list(u(t_string()), false), t_list(u(t_float()), false)],
        &["list<float|int|string>"],
    );
}

#[test]
fn object_triples() {
    check("object,Foo,Bar", vec![t_object_any(), t_named("Foo"), t_named("Bar")], &["object"]);
    check("Foo,Bar,object", vec![t_named("Foo"), t_named("Bar"), t_object_any()], &["object"]);
    check("Foo,Bar,Baz", vec![t_named("Foo"), t_named("Bar"), t_named("Baz")], &["Bar", "Baz", "Foo"]);
    check("Foo,Foo,Bar", vec![t_named("Foo"), t_named("Foo"), t_named("Bar")], &["Bar", "Foo"]);
    check(
        "E,E::A,E::B",
        vec![t_enum("E"), t_enum_case("E", "A"), t_enum_case("E", "B")],
        &["enum(E)", "enum(E::A)", "enum(E::B)"],
    );
}

#[test]
fn scalar_subtype_triples() {
    check("scalar,int,string", vec![t_scalar(), t_int(), t_string()], &["scalar"]);
    check("int,string,scalar", vec![t_int(), t_string(), t_scalar()], &["scalar"]);
    check("int,scalar,string", vec![t_int(), t_scalar(), t_string()], &["scalar"]);
    check("array-key,int,string", vec![t_array_key(), t_int(), t_string()], &["array-key"]);
    check("int,string,array-key", vec![t_int(), t_string(), t_array_key()], &["array-key"]);
    check("array-key,float,bool", vec![t_array_key(), t_float(), t_bool()], &["array-key", "bool", "float"]);
    check("scalar,bool,true", vec![t_scalar(), t_bool(), t_true()], &["bool", "scalar"]);
    check("scalar,true,false", vec![t_scalar(), t_true(), t_false()], &["bool", "scalar"]);
    check("bool,true,scalar", vec![t_bool(), t_true(), t_scalar()], &["scalar"]);
}

#[test]
fn resource_triples() {
    check("open,closed,resource", vec![t_open_resource(), t_closed_resource(), t_resource()], &["resource"]);
    check("open,open,closed", vec![t_open_resource(), t_open_resource(), t_closed_resource()], &["resource"]);
    check(
        "closed,closed,closed",
        vec![t_closed_resource(), t_closed_resource(), t_closed_resource()],
        &["closed-resource"],
    );
    check("open,open,open", vec![t_open_resource(), t_open_resource(), t_open_resource()], &["open-resource"]);
    check("open,int,closed", vec![t_open_resource(), t_int(), t_closed_resource()], &["int", "resource"]);
}

#[test]
fn four_atoms() {
    check(
        "int,string,float,bool",
        vec![t_int(), t_string(), t_float(), t_bool()],
        &["scalar"], // generalisation triggered
    );
    check("int,string,bool,null", vec![t_int(), t_string(), t_bool(), null()], &["bool", "int", "null", "string"]);
    check(
        "Foo,Bar,Baz,Qux",
        vec![t_named("Foo"), t_named("Bar"), t_named("Baz"), t_named("Qux")],
        &["Bar", "Baz", "Foo", "Qux"],
    );
    check("object,Foo,Bar,Baz", vec![t_object_any(), t_named("Foo"), t_named("Bar"), t_named("Baz")], &["object"]);
    check("true,false,bool,bool", vec![t_true(), t_false(), t_bool(), t_bool()], &["bool"]);
}
