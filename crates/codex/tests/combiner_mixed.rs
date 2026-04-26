mod combiner_common;

use combiner_common::*;

#[test]
fn vanilla_mixed_idempotent() {
    for n in 1..=10 {
        assert_self_idempotent(mixed(), n);
    }
}

#[test]
fn truthy_mixed_idempotent() {
    for n in 1..=10 {
        let r = combine_default(vec![mixed_truthy(); n]);
        assert_eq!(r.len(), 1);
        assert_eq!(atomic_id_string(&r[0]), "truthy-mixed");
    }
}

#[test]
fn falsy_mixed_idempotent() {
    for n in 1..=10 {
        let r = combine_default(vec![mixed_falsy(); n]);
        assert_eq!(r.len(), 1);
        assert_eq!(atomic_id_string(&r[0]), "falsy-mixed");
    }
}

#[test]
fn nonnull_mixed_idempotent() {
    for n in 1..=10 {
        let r = combine_default(vec![mixed_nonnull(); n]);
        assert_eq!(r.len(), 1);
        assert_eq!(atomic_id_string(&r[0]), "nonnull");
    }
}

#[test]
fn vanilla_dominates_int() {
    assert_combines_to(vec![mixed(), t_int()], vec![mixed()]);
    assert_combines_to(vec![t_int(), mixed()], vec![mixed()]);
}

#[test]
fn vanilla_dominates_string() {
    assert_combines_to(vec![mixed(), t_string()], vec![mixed()]);
    assert_combines_to(vec![t_string(), mixed()], vec![mixed()]);
}

#[test]
fn vanilla_dominates_object() {
    assert_combines_to(vec![mixed(), t_object_any()], vec![mixed()]);
}

#[test]
fn vanilla_dominates_array() {
    assert_combines_to(vec![mixed(), t_empty_array()], vec![mixed()]);
}

#[test]
fn vanilla_dominates_null() {
    assert_combines_to(vec![mixed(), null()], vec![mixed()]);
    assert_combines_to(vec![null(), mixed()], vec![mixed()]);
}

#[test]
fn vanilla_dominates_void() {
    assert_combines_to(vec![mixed(), void()], vec![mixed()]);
    assert_combines_to(vec![void(), mixed()], vec![mixed()]);
}

#[test]
fn vanilla_dominates_never() {
    assert_combines_to(vec![mixed(), never()], vec![mixed()]);
    assert_combines_to(vec![never(), mixed()], vec![mixed()]);
}

#[test]
fn vanilla_then_truthy_mixed_yields_vanilla() {
    assert_combines_to(vec![mixed(), mixed_truthy()], vec![mixed()]);
}

#[test]
fn truthy_mixed_then_vanilla_yields_nonnull() {
    let r = combine_default(vec![mixed_truthy(), mixed()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "nonnull");
}

#[test]
fn vanilla_then_falsy_mixed_yields_vanilla() {
    assert_combines_to(vec![mixed(), mixed_falsy()], vec![mixed()]);
}

#[test]
fn falsy_mixed_then_vanilla_yields_vanilla() {
    assert_combines_to(vec![mixed_falsy(), mixed()], vec![mixed()]);
}

#[test]
fn vanilla_then_nonnull_mixed_yields_vanilla() {
    assert_combines_to(vec![mixed(), mixed_nonnull()], vec![mixed()]);
}

#[test]
fn nonnull_mixed_then_vanilla_yields_nonnull() {
    let r = combine_default(vec![mixed_nonnull(), mixed()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "nonnull");
}

#[test]
fn vanilla_dominates_many_atoms() {
    let r = combine_default(vec![mixed(), t_int(), t_string(), t_bool(), t_float(), null()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "mixed");
}

#[test]
fn truthy_or_falsy_mixed_yields_nonnull() {
    let r = combine_default(vec![mixed_truthy(), mixed_falsy()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "nonnull");
}

#[test]
fn falsy_or_truthy_mixed_order_matters_for_truthiness() {
    let r = combine_default(vec![mixed_falsy(), mixed_truthy()]);
    assert_eq!(r.len(), 1);
    let id = atomic_id_string(&r[0]);
    assert!(matches!(id.as_str(), "mixed" | "nonnull"), "got {id}");
}

#[test]
fn truthy_mixed_then_nontruthy_int_yields_nonnull() {
    let r = combine_default(vec![mixed_truthy(), t_int()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "nonnull");
}

#[test]
fn nontruthy_int_then_truthy_mixed_yields_truthy_mixed() {
    let r = combine_default(vec![t_int(), mixed_truthy()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "truthy-mixed");
}

#[test]
fn truthy_mixed_then_falsy_string_literal_yields_nonnull() {
    let r = combine_default(vec![mixed_truthy(), t_lit_string("0")]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "nonnull");
}

#[test]
fn truthy_mixed_then_truthy_literal_preserves_truthy() {
    let r = combine_default(vec![mixed_truthy(), t_lit_string("hello")]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "truthy-mixed");
}

#[test]
fn nonnull_mixed_with_null_becomes_vanilla() {
    assert_combines_to(vec![mixed_nonnull(), null()], vec![mixed()]);
    assert_combines_to(vec![null(), mixed_nonnull()], vec![mixed()]);
}

#[test]
fn falsy_mixed_with_null_preserves_falsy() {
    let r = combine_default(vec![mixed_falsy(), null()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "falsy-mixed");

    let r = combine_default(vec![null(), mixed_falsy()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "falsy-mixed");
}

#[test]
fn truthy_mixed_first_then_null_yields_nonnull() {
    let r = combine_default(vec![mixed_truthy(), null()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "nonnull");
}

#[test]
fn null_first_then_truthy_mixed_yields_vanilla() {
    let r = combine_default(vec![null(), mixed_truthy()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "mixed");
}

#[test]
fn truthy_or_nonnull_mixed_collapses_to_nonnull() {
    let r = combine_default(vec![mixed_truthy(), mixed_nonnull()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "nonnull");
}

#[test]
fn never_then_truthy_mixed_yields_truthy() {
    let r = combine_default(vec![never(), mixed_truthy()]);
    assert_eq!(r.len(), 1);
    let id = atomic_id_string(&r[0]);
    assert!(matches!(id.as_str(), "mixed" | "truthy-mixed"));
}

#[test]
fn truthy_mixed_then_never_yields_nonnull() {
    let r = combine_default(vec![mixed_truthy(), never()]);
    assert_eq!(r.len(), 1);
    assert_eq!(atomic_id_string(&r[0]), "nonnull");
}

#[test]
fn never_dominated_by_mixed() {
    assert_combines_to(vec![never(), mixed()], vec![mixed()]);
    assert_combines_to(vec![mixed(), never()], vec![mixed()]);
}

#[test]
fn many_truthy_mixed_collapse() {
    assert_combines_to(vec![mixed_truthy(); 10], vec![mixed_truthy()]);
}

#[test]
fn many_falsy_mixed_collapse() {
    assert_combines_to(vec![mixed_falsy(); 10], vec![mixed_falsy()]);
}

#[test]
fn many_nonnull_mixed_collapse() {
    assert_combines_to(vec![mixed_nonnull(); 10], vec![mixed_nonnull()]);
}
