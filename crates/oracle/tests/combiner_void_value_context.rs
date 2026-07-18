mod common;

use common::*;

#[test]
fn void_alone_is_preserved_as_return_type_annotation() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void()], vec![f.void()]);
    });
}

#[test]
fn void_or_never_keeps_void() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.never()], vec![f.void()]);
        assert_combines_to(f, vec![f.never(), f.void()], vec![f.void()]);
    });
}

#[test]
fn void_with_only_nevers_keeps_void() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.never(), f.never()], vec![f.void()]);
    });
}

#[test]
fn void_or_null_collapses_to_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.null()], vec![f.null()]);
        assert_combines_to(f, vec![f.null(), f.void()], vec![f.null()]);
    });
}

#[test]
fn true_or_void_should_be_true_or_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_true(), f.void()], vec![f.t_true(), f.null()]);
        assert_combines_to(f, vec![f.void(), f.t_true()], vec![f.t_true(), f.null()]);
    });
}

#[test]
fn false_or_void_should_be_false_or_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_false(), f.void()], vec![f.t_false(), f.null()]);
    });
}

#[test]
fn bool_or_void_should_be_bool_or_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_bool(), f.void()], vec![f.t_bool(), f.null()]);
    });
}

#[test]
fn int_or_void_should_be_int_or_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_int(), f.void()], vec![f.t_int(), f.null()]);
        assert_combines_to(f, vec![f.void(), f.t_int()], vec![f.t_int(), f.null()]);
    });
}

#[test]
fn string_or_void_should_be_string_or_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_string(), f.void()], vec![f.t_string(), f.null()]);
    });
}

#[test]
fn object_or_void_should_be_object_or_null() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        assert_combines_to(f, vec![f.t_object_any(), f.void()], vec![f.t_object_any(), f.null()]);
        assert_combines_to(f, vec![foo, f.void()], vec![foo, f.null()]);
    });
}

#[test]
fn resource_or_void_should_be_resource_or_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_resource(), f.void()], vec![f.t_resource(), f.null()]);
    });
}

#[test]
fn literal_or_void_should_be_literal_or_null() {
    fixture(|f| {
        let hello = f.t_lit_string("hello");
        assert_combines_to(f, vec![f.t_lit_int(42), f.void()], vec![f.t_lit_int(42), f.null()]);
        assert_combines_to(f, vec![hello, f.void()], vec![hello, f.null()]);
    });
}

#[test]
fn three_way_int_string_void_should_have_null_not_void() {
    fixture(|f| {
        let result = combine_default(f, vec![f.void(), f.t_int(), f.t_string()]);
        let mut sorted = result;
        sorted.sort_unstable();
        let mut expected = vec![f.t_int(), f.t_string(), f.null()];
        expected.sort_unstable();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn void_with_multiple_others_is_replaced_by_single_null() {
    fixture(|f| {
        let null = f.null();
        let void = f.void();
        let result = combine_default(f, vec![void, f.t_int(), f.t_string(), f.t_bool()]);
        let null_count = result.iter().filter(|atom| **atom == null).count();
        let void_count = result.iter().filter(|atom| **atom == void).count();
        assert_eq!(null_count, 1, "should be exactly one null in the result");
        assert_eq!(void_count, 0, "should be no void in the result when value-types are present");
    });
}
