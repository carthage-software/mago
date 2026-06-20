mod common;

use common::*;

use mago_oracle::ty::lattice::CoercionCause;
use mago_oracle::ty::lattice::LatticeReport;

#[test]
fn mixed_to_int_sets_coerced_and_from_nested() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mixed = f.mixed();
        let int = f.t_int();
        let (verdict, report) = atomic_is_contained_capturing(f, mixed, int, &symbols);
        assert!(!verdict);
        assert!(report.coerced());
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
        assert!(report.causes.contains(CoercionCause::NestedMixed));
    });
}

#[test]
fn mixed_to_string_sets_coerced_and_from_nested() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mixed = f.mixed();
        let string = f.t_string();
        let (verdict, report) = atomic_is_contained_capturing(f, mixed, string, &symbols);
        assert!(!verdict);
        assert!(report.coerced());
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
        assert!(report.causes.contains(CoercionCause::NestedMixed));
    });
}

#[test]
fn array_key_to_int_sets_coerced() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let array_key = f.t_array_key();
        let int = f.t_int();
        let (verdict, report) = atomic_is_contained_capturing(f, array_key, int, &symbols);
        assert!(!verdict);
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
        assert!(!report.causes.contains(CoercionCause::NestedMixed));
    });
}

#[test]
fn array_key_to_string_sets_coerced() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let array_key = f.t_array_key();
        let string = f.t_string();
        let (verdict, report) = atomic_is_contained_capturing(f, array_key, string, &symbols);
        assert!(!verdict);
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
    });
}

#[test]
fn object_to_named_sets_coerced_and_object_any_down() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let object = f.t_object_any();
        let named = f.t_named("Foo");
        let (verdict, report) = atomic_is_contained_capturing(f, object, named, &symbols);
        assert!(!verdict);
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
        assert!(report.causes.contains(CoercionCause::ObjectAnyDown));
    });
}

#[test]
fn bool_to_true_sets_coerced() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let boolean = f.t_bool();
        let true_atom = f.t_true();
        let (verdict, report) = atomic_is_contained_capturing(f, boolean, true_atom, &symbols);
        assert!(!verdict);
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
    });
}

#[test]
fn bool_to_false_sets_coerced() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let boolean = f.t_bool();
        let false_atom = f.t_false();
        let (verdict, report) = atomic_is_contained_capturing(f, boolean, false_atom, &symbols);
        assert!(!verdict);
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
    });
}

#[test]
fn lit_int_in_int_no_flags_set() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let literal = f.t_lit_int(5);
        let int = f.t_int();
        let (verdict, report) = atomic_is_contained_capturing(f, literal, int, &symbols);
        assert!(verdict);
        assert!(!report.coerced());
        assert_eq!(report.replacement, None);
    });
}

#[test]
fn int_to_lit_int_no_coerced_flag() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let literal = f.t_lit_int(5);
        let (verdict, report) = atomic_is_contained_capturing(f, int, literal, &symbols);
        assert!(!verdict);
        assert!(!report.coerced());
    });
}

#[test]
fn int_to_positive_int_no_coerced_flag() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let positive = f.t_positive_int();
        let (verdict, report) = atomic_is_contained_capturing(f, int, positive, &symbols);
        assert!(!verdict);
        assert!(!report.coerced());
    });
}

#[test]
fn equal_atoms_no_flags() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        for atom in [f.t_int(), f.t_string(), f.t_bool(), f.t_float(), f.null(), f.t_object_any(), f.mixed()] {
            let (verdict, report) = atomic_is_contained_capturing(f, atom, atom, &symbols);
            assert!(verdict, "{atom:?} should equal itself");
            assert!(!report.coerced(), "no causes for {atom:?} == itself");
            assert_eq!(report.replacement, None);
        }
    });
}

#[test]
fn never_to_anything_no_flags() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let never = f.never();
        for atom in [f.t_int(), f.t_string(), f.t_object_any(), f.mixed()] {
            let (verdict, report) = atomic_is_contained_capturing(f, never, atom, &symbols);
            assert!(verdict, "never <: {atom:?}");
            assert!(!report.coerced());
        }
    });
}

#[test]
fn mixed_to_never_sets_coerced() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mixed = f.mixed();
        let never = f.never();
        let (verdict, report) = atomic_is_contained_capturing(f, mixed, never, &symbols);
        assert!(!verdict);
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
        assert!(report.causes.contains(CoercionCause::NestedMixed));
    });
}

#[test]
fn concrete_to_never_does_not_set_coerced() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let never = f.never();
        for atom in [f.t_int(), f.t_string()] {
            let (verdict, report) = atomic_is_contained_capturing(f, atom, never, &symbols);
            assert!(!verdict);
            assert!(!report.coerced());
        }
    });
}

#[test]
fn literal_int_to_other_literal_no_coerced_flag() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        let (verdict, report) = atomic_is_contained_capturing(f, one, two, &symbols);
        assert!(!verdict);
        assert!(!report.coerced());
    });
}

#[test]
fn int_does_not_refine_float_under_strict_subtype() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let float = f.t_float();
        let (verdict, _report) = atomic_is_contained_capturing(f, int, float, &symbols);
        assert!(!verdict);
    });
}

#[test]
fn template_constrained_to_mixed_sets_from_as_mixed_on_rejection() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let template = f.t_template("Foo", "T");
        let int = f.t_int();
        let (verdict, report) = atomic_is_contained_capturing(f, template, int, &symbols);
        assert!(!verdict);
        assert!(report.causes.contains(CoercionCause::FromAsMixed));
        assert!(report.causes.contains(CoercionCause::TrueUnionNarrow));
        assert!(!report.causes.contains(CoercionCause::NestedMixed));
    });
}

#[test]
fn template_to_mixed_does_not_set_from_as_mixed() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let template = f.t_template("Foo", "T");
        let mixed = f.mixed();
        let (verdict, report) = atomic_is_contained_capturing(f, template, mixed, &symbols);
        assert!(verdict);
        assert!(!report.causes.contains(CoercionCause::FromAsMixed));
    });
}

#[test]
fn fresh_report_has_empty_bounds_and_no_replacements() {
    let report = LatticeReport::new();
    assert!(report.replacement.is_none());
    assert!(report.replacement_atom.is_none());
    assert!(report.bounds.is_empty());
}

#[test]
fn nullable_int_to_int_with_ignore_null_passes() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let null = f.null();
        let nullable = f.u_many(vec![int, null]);
        let int_only = f.u(int);
        assert!(is_contained_with(f, nullable, int_only, &symbols, true, false, false));
    });
}

#[test]
fn int_or_false_to_int_with_ignore_false_passes() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let false_atom = f.t_false();
        let int_or_false = f.u_many(vec![int, false_atom]);
        let int_only = f.u(int);
        assert!(is_contained_with(f, int_or_false, int_only, &symbols, false, true, false));
    });
}

#[test]
fn nullable_int_to_int_without_ignore_null_fails() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let int = f.t_int();
        let null = f.null();
        let nullable = f.u_many(vec![int, null]);
        let int_only = f.u(int);
        assert!(!is_contained_with(f, nullable, int_only, &symbols, false, false, false));
    });
}
