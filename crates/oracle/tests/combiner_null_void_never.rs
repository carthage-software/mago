mod common;

use common::*;

#[test]
fn null_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.null(), n);
        }
    });
}

#[test]
fn void_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            let r = combine_default(f, vec![f.void(); n]);
            assert_eq!(r, vec![f.void()]);
        }
    });
}

#[test]
fn never_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.never(), n);
        }
    });
}

#[test]
fn void_or_null_yields_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.null()], vec![f.null()]);
        assert_combines_to(f, vec![f.null(), f.void()], vec![f.null()]);
    });
}

#[test]
fn void_becomes_null_alongside_int() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.t_int()], vec![f.t_int(), f.null()]);
        assert_combines_to(f, vec![f.t_int(), f.void()], vec![f.t_int(), f.null()]);
    });
}

#[test]
fn void_becomes_null_alongside_string() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.t_string()], vec![f.t_string(), f.null()]);
        assert_combines_to(f, vec![f.t_string(), f.void()], vec![f.t_string(), f.null()]);
    });
}

#[test]
fn void_becomes_null_alongside_bool() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.t_bool()], vec![f.t_bool(), f.null()]);
        assert_combines_to(f, vec![f.t_bool(), f.void()], vec![f.t_bool(), f.null()]);
    });
}

#[test]
fn void_becomes_null_alongside_object() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.t_object_any()], vec![f.t_object_any(), f.null()]);
        let foo = f.t_named("Foo");
        assert_combines_to(f, vec![f.void(), foo], vec![foo, f.null()]);
    });
}

#[test]
fn void_becomes_null_alongside_resource() {
    fixture(|f| {
        assert_combines_to(f, vec![f.void(), f.t_resource()], vec![f.t_resource(), f.null()]);
        assert_combines_to(f, vec![f.void(), f.t_open_resource()], vec![f.t_open_resource(), f.null()]);
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
fn void_becomes_null_with_two_other_types() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.void(), f.t_int(), f.t_string()]);
        sorted.sort();
        let mut expected = vec![f.t_int(), f.t_string(), f.null()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn null_or_int_kept_separate() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.null(), f.t_int()]);
        sorted.sort();
        let mut expected = vec![f.null(), f.t_int()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn null_or_string_kept_separate() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.null(), f.t_string()]);
        sorted.sort();
        let mut expected = vec![f.null(), f.t_string()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn null_or_bool_kept_separate() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.null(), f.t_bool()]);
        sorted.sort();
        let mut expected = vec![f.null(), f.t_bool()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn null_or_object_kept_separate() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.null(), f.t_object_any()]);
        sorted.sort();
        let mut expected = vec![f.null(), f.t_object_any()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn null_absorbs_never() {
    fixture(|f| {
        assert_combines_to(f, vec![f.null(), f.never()], vec![f.null()]);
        assert_combines_to(f, vec![f.never(), f.null()], vec![f.null()]);
    });
}

#[test]
fn never_dropped_with_int() {
    fixture(|f| {
        assert_combines_to(f, vec![f.never(), f.t_int()], vec![f.t_int()]);
        assert_combines_to(f, vec![f.t_int(), f.never()], vec![f.t_int()]);
    });
}

#[test]
fn never_dropped_with_string() {
    fixture(|f| {
        assert_combines_to(f, vec![f.never(), f.t_string()], vec![f.t_string()]);
    });
}

#[test]
fn never_dropped_with_array() {
    fixture(|f| {
        assert_combines_to(f, vec![f.never(), f.t_empty_array()], vec![f.t_empty_array()]);
    });
}

#[test]
fn never_dropped_with_object() {
    fixture(|f| {
        let x = f.t_named("X");
        assert_combines_to(f, vec![f.never(), x], vec![x]);
    });
}

#[test]
fn never_dropped_with_three_atoms() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.never(), f.t_int(), f.t_string()]);
        sorted.sort();
        let mut expected = vec![f.t_int(), f.t_string()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn many_nevers_collapse() {
    fixture(|f| {
        for n in 1..=10 {
            assert_combines_to(f, vec![f.never(); n], vec![f.never()]);
        }
    });
}

#[test]
fn never_with_many_others_disappears() {
    fixture(|f| {
        let mut inputs = vec![f.never()];
        for i in 0..5 {
            inputs.push(f.t_lit_int(i * 10));
        }
        let r = combine_default(f, inputs);
        assert_eq!(r.len(), 5);
        assert!(r.iter().all(|e| *e != f.never()));
    });
}
