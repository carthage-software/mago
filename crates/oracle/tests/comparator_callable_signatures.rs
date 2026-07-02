mod common;

use common::*;

use mago_flags::U8Flags;
use mago_oracle::ty::Atom;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::atom::payload::callable::Signature;
use mago_oracle::ty::well_known;

#[test]
fn signature_reflexive() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let signature = f.t_callable(&[int_type], string_type);
        assert!(atomic_is_contained(f, signature, signature, &symbols));
    });
}

#[test]
fn return_covariance_holds() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let forty_two = f.ui(42);
        let int = f.t_int();
        let int_type = f.u(int);
        let returns_lit = f.t_callable(&[], forty_two);
        let returns_int = f.t_callable(&[], int_type);
        assert!(atomic_is_contained(f, returns_lit, returns_int, &symbols));
    });
}

#[test]
fn return_covariance_failure() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let forty_two = f.ui(42);
        let returns_int = f.t_callable(&[], int_type);
        let returns_lit = f.t_callable(&[], forty_two);
        assert!(!atomic_is_contained(f, returns_int, returns_lit, &symbols));
    });
}

#[test]
fn return_widens_into_mixed() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let returns_int = f.t_callable(&[], int_type);
        let returns_mixed = f.t_callable(&[], well_known::TYPE_MIXED);
        assert!(atomic_is_contained(f, returns_int, returns_mixed, &symbols));
    });
}

#[test]
fn parameter_contravariance_holds() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let forty_two = f.ui(42);
        let takes_int = f.t_callable(&[int_type], int_type);
        let takes_lit = f.t_callable(&[forty_two], int_type);
        assert!(atomic_is_contained(f, takes_int, takes_lit, &symbols));
    });
}

#[test]
fn parameter_contravariance_failure() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let forty_two = f.ui(42);
        let int = f.t_int();
        let int_type = f.u(int);
        let takes_lit = f.t_callable(&[forty_two], int_type);
        let takes_int = f.t_callable(&[int_type], int_type);
        assert!(!atomic_is_contained(f, takes_lit, takes_int, &symbols));
    });
}

#[test]
fn parameter_contravariance_widens_via_mixed() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let takes_mixed = f.t_callable(&[well_known::TYPE_MIXED], int_type);
        let takes_int = f.t_callable(&[int_type], int_type);
        assert!(atomic_is_contained(f, takes_mixed, takes_int, &symbols));
    });
}

#[test]
fn arity_mismatch_more_required_input() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let takes_two = f.t_callable(&[int_type, string_type], int_type);
        let takes_one = f.t_callable(&[int_type], int_type);
        assert!(!atomic_is_contained(f, takes_two, takes_one, &symbols));
    });
}

#[test]
fn arity_mismatch_more_required_container() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let takes_one = f.t_callable(&[int_type], int_type);
        let takes_two = f.t_callable(&[int_type, string_type], int_type);
        assert!(!atomic_is_contained(f, takes_one, takes_two, &symbols));
    });
}

#[test]
fn input_with_default_satisfies_smaller_arity_container() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        let with_default =
            f.t_callable_sig(&[(int_type, false, false, false), (string_type, true, false, false)], int_type, false);
        let takes_one = f.t_callable(&[int_type], int_type);
        assert!(atomic_is_contained(f, with_default, takes_one, &symbols));
    });
}

#[test]
fn pure_container_rejects_impure_input() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let impure = f.t_callable_sig(&[(int_type, false, false, false)], int_type, false);
        let pure = f.t_callable_sig(&[(int_type, false, false, false)], int_type, true);
        assert!(!atomic_is_contained(f, impure, pure, &symbols));
    });
}

#[test]
fn pure_input_satisfies_pure_container() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let pure = f.t_callable_sig(&[(int_type, false, false, false)], int_type, true);
        assert!(atomic_is_contained(f, pure, pure, &symbols));
    });
}

#[test]
fn pure_input_satisfies_impure_container() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let pure = f.t_callable_sig(&[(int_type, false, false, false)], int_type, true);
        let impure = f.t_callable_sig(&[(int_type, false, false, false)], int_type, false);
        assert!(atomic_is_contained(f, pure, impure, &symbols));
    });
}

#[test]
fn variadic_input_absorbs_extra_container_param() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let variadic_in = f.t_callable_sig(&[(int_type, false, false, true)], int_type, false);
        let two_int = f.t_callable(&[int_type, int_type], int_type);
        assert!(atomic_is_contained(f, variadic_in, two_int, &symbols));
    });
}

#[test]
fn variadic_container_requires_variadic_input() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let one_in = f.t_callable(&[int_type], int_type);
        let variadic_out = f.t_callable_sig(&[(int_type, false, false, true)], int_type, false);
        assert!(!atomic_is_contained(f, one_in, variadic_out, &symbols));
    });
}

#[test]
fn variadic_to_variadic_with_contravariant_type() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let in_takes_mixed = f.t_callable_sig(&[(well_known::TYPE_MIXED, false, false, true)], int_type, false);
        let out_takes_int = f.t_callable_sig(&[(int_type, false, false, true)], int_type, false);
        assert!(atomic_is_contained(f, in_takes_mixed, out_takes_int, &symbols));
    });
}

#[test]
fn unspecified_container_accepts_any_signature() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let specific = f.t_callable(&[int_type], int_type);
        let callable_mixed = f.t_callable_mixed();
        assert!(atomic_is_contained(f, specific, callable_mixed, &symbols));
    });
}

#[test]
fn unspecified_input_does_not_refine_specific_container() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let specific = f.t_callable(&[int_type], int_type);
        let callable_mixed = f.t_callable_mixed();
        assert!(!atomic_is_contained(f, callable_mixed, specific, &symbols));
    });
}

#[test]
fn closure_refines_signature_with_compatible_shape() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let payload = f.builder.signature(Signature {
            parameters: None,
            return_type: int_type,
            throws: None,
            flags: U8Flags::empty(),
        });
        let closure = Atom::Callable(CallableAtom::Closure(payload));
        let signature = Atom::Callable(CallableAtom::Signature(payload));
        assert!(atomic_is_contained(f, closure, signature, &symbols));
        assert!(!atomic_is_contained(f, signature, closure, &symbols));
    });
}

#[test]
fn any_callable_does_not_refine_specific() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let specific = f.t_callable(&[int_type], int_type);
        let callable_any = f.t_callable_any();
        assert!(!atomic_is_contained(f, callable_any, specific, &symbols));
    });
}

#[test]
fn anything_refines_any_callable() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let int_type = f.u(int);
        let specific = f.t_callable(&[int_type], int_type);
        let callable_any = f.t_callable_any();
        let callable_mixed = f.t_callable_mixed();
        assert!(atomic_is_contained(f, specific, callable_any, &symbols));
        assert!(atomic_is_contained(f, callable_mixed, callable_any, &symbols));
    });
}
