mod common;

use common::*;

#[test]
fn vanilla_mixed_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.mixed(), n);
        }
    });
}

#[test]
fn truthy_mixed_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.mixed_truthy(), n);
        }
    });
}

#[test]
fn falsy_mixed_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.mixed_falsy(), n);
        }
    });
}

#[test]
fn nonnull_mixed_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.mixed_nonnull(), n);
        }
    });
}

#[test]
fn vanilla_dominates_int() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.t_int()], vec![f.mixed()]);
        assert_combines_to(f, vec![f.t_int(), f.mixed()], vec![f.mixed()]);
    });
}

#[test]
fn vanilla_dominates_string() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.t_string()], vec![f.mixed()]);
        assert_combines_to(f, vec![f.t_string(), f.mixed()], vec![f.mixed()]);
    });
}

#[test]
fn vanilla_dominates_object() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.t_object_any()], vec![f.mixed()]);
    });
}

#[test]
fn vanilla_dominates_array() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.t_empty_array()], vec![f.mixed()]);
    });
}

#[test]
fn vanilla_dominates_null() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.null()], vec![f.mixed()]);
        assert_combines_to(f, vec![f.null(), f.mixed()], vec![f.mixed()]);
    });
}

#[test]
fn vanilla_dominates_void() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.void()], vec![f.mixed()]);
        assert_combines_to(f, vec![f.void(), f.mixed()], vec![f.mixed()]);
    });
}

#[test]
fn vanilla_dominates_never() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.never()], vec![f.mixed()]);
        assert_combines_to(f, vec![f.never(), f.mixed()], vec![f.mixed()]);
    });
}

#[test]
fn vanilla_then_truthy_mixed_yields_vanilla() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.mixed_truthy()], vec![f.mixed()]);
    });
}

#[test]
fn truthy_mixed_then_vanilla_yields_nonnull() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_truthy(), f.mixed()], vec![f.mixed_nonnull()]);
    });
}

#[test]
fn vanilla_then_falsy_mixed_yields_vanilla() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.mixed_falsy()], vec![f.mixed()]);
    });
}

#[test]
fn falsy_mixed_then_vanilla_yields_vanilla() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_falsy(), f.mixed()], vec![f.mixed()]);
    });
}

#[test]
fn vanilla_then_nonnull_mixed_yields_vanilla() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed(), f.mixed_nonnull()], vec![f.mixed()]);
    });
}

#[test]
fn nonnull_mixed_then_vanilla_yields_nonnull() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_nonnull(), f.mixed()], vec![f.mixed_nonnull()]);
    });
}

#[test]
fn vanilla_dominates_many_atoms() {
    fixture(|f| {
        let inputs = vec![f.mixed(), f.t_int(), f.t_string(), f.t_bool(), f.t_float(), f.null()];
        assert_combines_to(f, inputs, vec![f.mixed()]);
    });
}

#[test]
fn truthy_or_falsy_mixed_yields_nonnull() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_truthy(), f.mixed_falsy()], vec![f.mixed_nonnull()]);
    });
}

#[test]
fn truthy_mixed_then_nontruthy_int_yields_nonnull() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_truthy(), f.t_int()], vec![f.mixed_nonnull()]);
    });
}

#[test]
fn nontruthy_int_then_truthy_mixed_yields_truthy_mixed() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_int(), f.mixed_truthy()], vec![f.mixed_truthy()]);
    });
}

#[test]
fn truthy_mixed_then_falsy_string_literal_yields_nonnull() {
    fixture(|f| {
        let zero = f.t_lit_string("0");
        assert_combines_to(f, vec![f.mixed_truthy(), zero], vec![f.mixed_nonnull()]);
    });
}

#[test]
fn truthy_mixed_then_truthy_literal_preserves_truthy() {
    fixture(|f| {
        let hello = f.t_lit_string("hello");
        assert_combines_to(f, vec![f.mixed_truthy(), hello], vec![f.mixed_truthy()]);
    });
}

#[test]
fn nonnull_mixed_with_null_becomes_vanilla() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_nonnull(), f.null()], vec![f.mixed()]);
        assert_combines_to(f, vec![f.null(), f.mixed_nonnull()], vec![f.mixed()]);
    });
}

#[test]
fn falsy_mixed_with_null_preserves_falsy() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_falsy(), f.null()], vec![f.mixed_falsy()]);
        assert_combines_to(f, vec![f.null(), f.mixed_falsy()], vec![f.mixed_falsy()]);
    });
}

#[test]
fn truthy_mixed_first_then_null_yields_nonnull() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_truthy(), f.null()], vec![f.mixed_nonnull()]);
    });
}

#[test]
fn null_first_then_truthy_mixed_yields_vanilla() {
    fixture(|f| {
        assert_combines_to(f, vec![f.null(), f.mixed_truthy()], vec![f.mixed()]);
    });
}

#[test]
fn truthy_or_nonnull_mixed_collapses_to_nonnull() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_truthy(), f.mixed_nonnull()], vec![f.mixed_nonnull()]);
    });
}

#[test]
fn never_then_truthy_mixed_yields_truthy() {
    fixture(|f| {
        let r = combine_default(f, vec![f.never(), f.mixed_truthy()]);
        assert_eq!(r.len(), 1);
        assert!(r[0] == f.mixed() || r[0] == f.mixed_truthy(), "expected mixed or truthy_mixed, got {:?}", r[0]);
    });
}

#[test]
fn truthy_mixed_then_never_yields_nonnull() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_truthy(), f.never()], vec![f.mixed_nonnull()]);
    });
}

#[test]
fn never_dominated_by_mixed() {
    fixture(|f| {
        assert_combines_to(f, vec![f.never(), f.mixed()], vec![f.mixed()]);
        assert_combines_to(f, vec![f.mixed(), f.never()], vec![f.mixed()]);
    });
}

#[test]
fn many_truthy_mixed_collapse() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_truthy(); 10], vec![f.mixed_truthy()]);
    });
}

#[test]
fn many_falsy_mixed_collapse() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_falsy(); 10], vec![f.mixed_falsy()]);
    });
}

#[test]
fn many_nonnull_mixed_collapse() {
    fixture(|f| {
        assert_combines_to(f, vec![f.mixed_nonnull(); 10], vec![f.mixed_nonnull()]);
    });
}
