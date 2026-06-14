mod common;

use common::assert_subtype;
use common::empty_world;
use common::fixture;
use common::is_contained;
use common::overlaps;

use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::meet;

#[test]
fn closure_signature_refines_closure_class() {
    fixture(|f| {
        let closure_atom = f.t_closure_mixed();
        let closure = f.u(closure_atom);
        let closure_class_atom = f.t_named("Closure");
        let closure_class = f.u(closure_class_atom);

        assert_subtype(f, closure, closure_class);
    });
}

#[test]
fn closure_class_match_is_case_insensitive() {
    fixture(|f| {
        let closure_atom = f.t_closure_mixed();
        let closure = f.u(closure_atom);
        let lowercase_atom = f.t_named("closure");
        let lowercase = f.u(lowercase_atom);

        assert_subtype(f, closure, lowercase);
    });
}

#[test]
fn anonymous_callable_signature_is_not_a_closure_instance() {
    fixture(|f| {
        let void = f.u(f.void());
        let int = f.u(f.t_int());
        let signature_atom = f.t_callable(&[int], void);
        let signature = f.u(signature_atom);
        let closure_class_atom = f.t_named("Closure");
        let closure_class = f.u(closure_class_atom);

        let world = empty_world();
        assert!(
            !is_contained(f, signature, closure_class, &world),
            "a bare callable(...) signature is not necessarily a \\Closure (could be a string or array)"
        );
    });
}

#[test]
fn bare_callable_is_not_a_closure_instance() {
    fixture(|f| {
        let callable_atom = f.t_callable_any();
        let callable = f.u(callable_atom);
        let closure_class_atom = f.t_named("Closure");
        let closure_class = f.u(closure_class_atom);

        let world = empty_world();
        assert!(!is_contained(f, callable, closure_class, &world), "bare `callable` is not a \\Closure instance");
    });
}

#[test]
fn closure_does_not_refine_unrelated_class() {
    fixture(|f| {
        let closure_atom = f.t_closure_mixed();
        let closure = f.u(closure_atom);
        let other_atom = f.t_named("Foo");
        let other = f.u(other_atom);

        let world = empty_world();
        assert!(!is_contained(f, closure, other, &world), "a closure is not an instance of an unrelated class");
    });
}

#[test]
fn closure_overlaps_closure_class() {
    fixture(|f| {
        let closure_atom = f.t_closure_mixed();
        let closure = f.u(closure_atom);
        let closure_class_atom = f.t_named("Closure");
        let closure_class = f.u(closure_class_atom);

        let world = empty_world();
        assert!(
            overlaps(f, closure, closure_class, &world),
            "a closure value inhabits both the signature and \\Closure"
        );
    });
}

#[test]
fn meet_of_closure_and_closure_class_keeps_the_signature() {
    fixture(|f| {
        let closure_atom = f.t_closure_mixed();
        let closure = f.u(closure_atom);
        let closure_class_atom = f.t_named("Closure");
        let closure_class = f.u(closure_class_atom);

        let world = empty_world();
        let mut report = LatticeReport::new();
        let result =
            meet::compute(closure, closure_class, &world, LatticeOptions::default(), &mut report, &mut f.builder);

        assert!(is_contained(f, result, closure, &world), "meet should refine the closure signature");
        assert!(!result.is_never(), "a closure and \\Closure are not disjoint");
    });
}
