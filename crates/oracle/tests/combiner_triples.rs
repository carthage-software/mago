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
fn bool_triples() {
    fixture(|f| {
        check(f, "true,false,bool", vec![f.t_true(), f.t_false(), f.t_bool()], &[f.t_bool()]);
        check(f, "bool,true,false", vec![f.t_bool(), f.t_true(), f.t_false()], &[f.t_bool()]);
        check(f, "false,bool,true", vec![f.t_false(), f.t_bool(), f.t_true()], &[f.t_bool()]);
        check(f, "true,true,false", vec![f.t_true(), f.t_true(), f.t_false()], &[f.t_bool()]);
        check(f, "false,false,true", vec![f.t_false(), f.t_false(), f.t_true()], &[f.t_bool()]);
        check(f, "bool,bool,bool", vec![f.t_bool(), f.t_bool(), f.t_bool()], &[f.t_bool()]);
        check(f, "true,true,true", vec![f.t_true(), f.t_true(), f.t_true()], &[f.t_true()]);
        check(f, "false,false,false", vec![f.t_false(), f.t_false(), f.t_false()], &[f.t_false()]);
    });
}

#[test]
fn int_lit_range_triples() {
    fixture(|f| {
        let range_0_3 = f.t_int_range(0, 3);
        let range_5_10 = f.t_int_range(5, 10);
        let range_0_2 = f.t_int_range(0, 2);
        let range_0_5 = f.t_int_range(0, 5);
        check(f, "int(0),int(1),int(2)", vec![f.t_lit_int(0), f.t_lit_int(1), f.t_lit_int(2)], &[range_0_2]);
        check(f, "int(5),int<0,3>,int(4)", vec![f.t_lit_int(5), range_0_3, f.t_lit_int(4)], &[range_0_5]);
        check(f, "int,int(0),int<5,10>", vec![f.t_int(), f.t_lit_int(0), range_5_10], &[f.t_int()]);
        check(f, "int<0,2>,int(1),int(0)", vec![range_0_2, f.t_lit_int(1), f.t_lit_int(0)], &[range_0_2]);
        check(
            f,
            "int(0),int(2),int(4)",
            vec![f.t_lit_int(0), f.t_lit_int(2), f.t_lit_int(4)],
            &[f.t_lit_int(0), f.t_lit_int(2), f.t_lit_int(4)],
        );
    });
}

#[test]
fn string_triples() {
    fixture(|f| {
        let letter_a = f.t_lit_string("a");
        let letter_b = f.t_lit_string("b");
        let letter_c = f.t_lit_string("c");
        check(f, "string,'a',non-empty", vec![f.t_string(), letter_a, f.t_non_empty_string()], &[f.t_string()]);
        check(f, "lower,upper,string", vec![f.t_lower_string(), f.t_upper_string(), f.t_string()], &[f.t_string()]);
        check(
            f,
            "non-empty,truthy,numeric",
            vec![f.t_non_empty_string(), f.t_truthy_string(), f.t_numeric_string()],
            &[f.t_string()],
        );
        check(f, "'a','b','c'", vec![letter_a, letter_b, letter_c], &[letter_a, letter_b, letter_c]);
        check(f, "non-empty,'a','b'", vec![f.t_non_empty_string(), letter_a, letter_b], &[f.t_non_empty_string()]);
    });
}

#[test]
fn null_triples() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        check(f, "null,int,string", vec![f.null(), f.t_int(), f.t_string()], &[f.t_int(), f.null(), f.t_string()]);
        check(f, "null,never,int", vec![f.null(), f.never(), f.t_int()], &[f.t_int(), f.null()]);
        check(f, "null,void,int", vec![f.null(), f.void(), f.t_int()], &[f.t_int(), f.null()]);
        check(f, "null,null,int", vec![f.null(), f.null(), f.t_int()], &[f.t_int(), f.null()]);
        check(f, "null,object,Foo", vec![f.null(), f.t_object_any(), foo], &[f.null(), f.t_object_any()]);
        check(f, "null,Foo,Bar", vec![f.null(), foo, bar], &[bar, foo, f.null()]);
    });
}

#[test]
fn never_triples_absorbed() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let enum_e = f.t_enum("E");
        check(f, "never,int,string", vec![f.never(), f.t_int(), f.t_string()], &[f.t_int(), f.t_string()]);
        check(f, "int,never,string", vec![f.t_int(), f.never(), f.t_string()], &[f.t_int(), f.t_string()]);
        check(f, "int,string,never", vec![f.t_int(), f.t_string(), f.never()], &[f.t_int(), f.t_string()]);
        check(f, "never,never,int", vec![f.never(), f.never(), f.t_int()], &[f.t_int()]);
        check(f, "never,never,never", vec![f.never(), f.never(), f.never()], &[f.never()]);
        check(f, "never,Foo,Bar", vec![f.never(), foo, bar], &[bar, foo]);
        check(
            f,
            "never,object,enum (suffete absorbs enum)",
            vec![f.never(), f.t_object_any(), enum_e],
            &[f.t_object_any()],
        );
    });
}

#[test]
fn void_triples() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        check(f, "void,int,string", vec![f.void(), f.t_int(), f.t_string()], &[f.t_int(), f.t_string(), f.null()]);
        check(f, "void,null,int", vec![f.void(), f.null(), f.t_int()], &[f.t_int(), f.null()]);
        check(f, "void,never,int", vec![f.void(), f.never(), f.t_int()], &[f.t_int(), f.null()]);
        check(f, "void,never,never", vec![f.void(), f.never(), f.never()], &[f.void()]);
        check(f, "void,void,int", vec![f.void(), f.void(), f.t_int()], &[f.t_int(), f.null()]);
        check(f, "void,void,void", vec![f.void(), f.void(), f.void()], &[f.void()]);
        check(f, "void,Foo,Bar", vec![f.void(), foo, bar], &[bar, foo, f.null()]);
    });
}

#[test]
fn mixed_triples_dominate() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        check(f, "mixed,int,string", vec![f.mixed(), f.t_int(), f.t_string()], &[f.mixed()]);
        check(f, "int,mixed,string", vec![f.t_int(), f.mixed(), f.t_string()], &[f.mixed()]);
        check(f, "int,string,mixed", vec![f.t_int(), f.t_string(), f.mixed()], &[f.mixed()]);
        check(f, "mixed,Foo,Bar", vec![f.mixed(), foo, bar], &[f.mixed()]);
        check(f, "mixed,never,int", vec![f.mixed(), f.never(), f.t_int()], &[f.mixed()]);
    });
}

#[test]
fn array_triples() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let float_type = f.u(f.t_float());

        let list_int = f.t_list(int_type, false);
        let list_string = f.t_list(string_type, false);
        let list_float = f.t_list(float_type, false);
        let merged_element = f.u_many(vec![f.t_int(), f.t_string(), f.t_float()]);
        let list_merged = f.t_list(merged_element, false);
        check(f, "list<int>,list<string>,list<float>", vec![list_int, list_string, list_float], &[list_merged]);

        let non_empty_list_int = f.t_list(int_type, true);
        check(f, "array{},list<int>,ne-list<int>", vec![f.t_empty_array(), list_int, non_empty_list_int], &[list_int]);

        let keyed_string_int = f.t_keyed_unsealed(string_type, int_type, false);
        let keyed_string_string = f.t_keyed_unsealed(string_type, string_type, false);
        let keyed_string_float = f.t_keyed_unsealed(string_type, float_type, false);
        let keyed_merged = f.t_keyed_unsealed(string_type, merged_element, false);
        check(
            f,
            "keyed<s,int>,keyed<s,string>,keyed<s,float>",
            vec![keyed_string_int, keyed_string_string, keyed_string_float],
            &[keyed_merged],
        );

        check(f, "list<int>,array{},int", vec![list_int, f.t_empty_array(), f.t_int()], &[f.t_int(), list_int]);
    });
}

#[test]
fn object_triples() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let baz = f.t_named("Baz");
        let enum_e = f.t_enum("E");
        let case_a = f.t_enum_case("E", "A");
        let case_b = f.t_enum_case("E", "B");
        check(f, "object,Foo,Bar", vec![f.t_object_any(), foo, bar], &[f.t_object_any()]);
        check(f, "Foo,Bar,object", vec![foo, bar, f.t_object_any()], &[f.t_object_any()]);
        check(f, "Foo,Bar,Baz", vec![foo, bar, baz], &[bar, baz, foo]);
        check(f, "Foo,Foo,Bar", vec![foo, foo, bar], &[bar, foo]);
        check(f, "E,E::A,E::B", vec![enum_e, case_a, case_b], &[enum_e]);
    });
}

#[test]
fn scalar_subtype_triples() {
    fixture(|f| {
        check(f, "int,string,scalar", vec![f.t_int(), f.t_string(), f.t_scalar()], &[f.t_scalar()]);
        check(f, "numeric,int,float", vec![f.t_numeric(), f.t_int(), f.t_float()], &[f.t_numeric()]);
        check(f, "array-key,int,string", vec![f.t_array_key(), f.t_int(), f.t_string()], &[f.t_array_key()]);
        check(f, "scalar,numeric,array-key", vec![f.t_scalar(), f.t_numeric(), f.t_array_key()], &[f.t_scalar()]);
        check(f, "scalar,int,bool", vec![f.t_scalar(), f.t_int(), f.t_bool()], &[f.t_scalar()]);
        check(f, "numeric,array-key,scalar", vec![f.t_numeric(), f.t_array_key(), f.t_scalar()], &[f.t_scalar()]);
    });
}

#[test]
fn resource_triples() {
    fixture(|f| {
        check(
            f,
            "open,closed,resource",
            vec![f.t_open_resource(), f.t_closed_resource(), f.t_resource()],
            &[f.t_resource()],
        );
        check(
            f,
            "open,open,closed",
            vec![f.t_open_resource(), f.t_open_resource(), f.t_closed_resource()],
            &[f.t_resource()],
        );
        check(
            f,
            "closed,closed,closed",
            vec![f.t_closed_resource(), f.t_closed_resource(), f.t_closed_resource()],
            &[f.t_closed_resource()],
        );
        check(
            f,
            "open,open,open",
            vec![f.t_open_resource(), f.t_open_resource(), f.t_open_resource()],
            &[f.t_open_resource()],
        );
        check(
            f,
            "open,int,closed",
            vec![f.t_open_resource(), f.t_int(), f.t_closed_resource()],
            &[f.t_int(), f.t_resource()],
        );
    });
}

#[test]
fn four_atoms() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_int(), f.t_string(), f.t_float(), f.t_bool()]);
        assert_eq!(result, vec![f.t_scalar()]);

        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let baz = f.t_named("Baz");
        let qux = f.t_named("Qux");
        check(
            f,
            "int,string,bool,null",
            vec![f.t_int(), f.t_string(), f.t_bool(), f.null()],
            &[f.t_bool(), f.t_int(), f.null(), f.t_string()],
        );
        check(f, "Foo,Bar,Baz,Qux", vec![foo, bar, baz, qux], &[bar, baz, foo, qux]);
        check(f, "object,Foo,Bar,Baz", vec![f.t_object_any(), foo, bar, baz], &[f.t_object_any()]);
        check(f, "true,false,bool,bool", vec![f.t_true(), f.t_false(), f.t_bool(), f.t_bool()], &[f.t_bool()]);
    });
}
