mod combiner_common;

use combiner_common::*;

#[test]
fn smoke_int_int_collapses() {
    assert_combines_to(vec![t_int(), t_int()], vec![t_int()]);
}

#[test]
fn smoke_true_false_becomes_bool() {
    assert_combines_to(vec![t_true(), t_false()], vec![t_bool()]);
}

#[test]
fn smoke_never_is_absorbed() {
    assert_combines_to(vec![never(), t_int()], vec![t_int()]);
}

#[test]
fn smoke_mixed_dominates() {
    assert_combines_to(vec![mixed(), t_int(), t_string()], vec![mixed()]);
}

#[test]
fn smoke_empty_array_alone() {
    assert_combines_to(vec![t_empty_array()], vec![t_empty_array()]);
}
