mod common;

use common::*;

#[test]
fn smoke_int_int_collapses() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_int(), f.t_int()], vec![f.t_int()]);
    });
}

#[test]
fn smoke_true_false_becomes_bool() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_true(), f.t_false()], vec![f.t_bool()]);
    });
}

#[test]
fn smoke_never_is_absorbed() {
    fixture(|f| {
        assert_combines_to(f, vec![f.never(), f.t_int()], vec![f.t_int()]);
    });
}

#[test]
fn smoke_mixed_dominates() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.t_int(), f.t_string()], vec![f.mixed()]);
    });
}

#[test]
fn smoke_empty_array_alone() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_empty_array()], vec![f.t_empty_array()]);
    });
}
