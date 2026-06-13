//! Soundness invariants for [`meet`]: the result must be a subset of
//! both inputs. Each test pins a regression for a soundness bug
//! previously found by audit (where `meet(a, b)` returned values
//! outside `a` or `b`).

mod common;

use core::num::NonZeroU32;

use common::*;

use mago_flags::U8Flags;
use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::KnownElement;
use mago_oracle::ty::atom::payload::array::ListAtom;
use mago_oracle::ty::atom::payload::scalar::mixed::MixedAtom;
use mago_oracle::ty::atom::payload::scalar::mixed::Truthiness;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::atom::payload::scalar::string::StringRefinementFlag;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::refines;
use mago_oracle::ty::meet;
use mago_oracle::ty::well_known;
use mago_oracle::world::World;

fn lattice_meet<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, w: &W) -> Type<'arena>
where
    W: World<'arena>,
{
    meet::compute(a, b, w, LatticeOptions::default(), &mut LatticeReport::new(), &mut f.builder)
}

fn does_refine<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, w: &W) -> bool
where
    W: World<'arena>,
{
    refines(a, b, w, LatticeOptions::default(), &mut LatticeReport::new(), &mut f.builder)
}

#[track_caller]
fn assert_meet_is_subset<'arena>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>) {
    let w = empty_world();
    let m = lattice_meet(f, a, b, &w);
    assert!(does_refine(f, m, a, &w), "meet({a}, {b}) = {m} must refine {a}");
    assert!(does_refine(f, m, b, &w), "meet({a}, {b}) = {m} must refine {b}");
}

#[test]
fn truthy_mixed_meet_literal_zero_string_is_never() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let zero = f.t_lit_string("0");
        let rhs = f.u(zero);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "truthy ∩ string('0') must be never (got {m})");
    });
}

#[test]
fn truthy_mixed_meet_literal_empty_string_is_never() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let empty = f.t_lit_string("");
        let rhs = f.u(empty);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "truthy ∩ string('') must be never (got {m})");
    });
}

#[test]
fn falsy_mixed_meet_non_empty_string_excludes_empty_literal() {
    fixture(|f| {
        let non_empty = f.builder.string(StringAtom {
            literal: StringLiteral::None,
            casing: StringCasing::Unspecified,
            flags: U8Flags::empty().with(StringRefinementFlag::NonEmpty),
        });
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let rhs = f.u(non_empty);
        assert_meet_is_subset(f, lhs, rhs);
        let empty_lit = f.us("");
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert!(
            !does_refine(f, empty_lit, m, &empty_world()),
            "string('') is empty so cannot be in falsy ∩ non-empty-string (got meet {m})"
        );
    });
}

#[test]
fn falsy_mixed_meet_truthy_string_is_never() {
    fixture(|f| {
        let truthy_string = f.builder.string(StringAtom {
            literal: StringLiteral::None,
            casing: StringCasing::Unspecified,
            flags: U8Flags::empty().with(StringRefinementFlag::Truthy).with(StringRefinementFlag::NonEmpty),
        });
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let rhs = f.u(truthy_string);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "falsy ∩ truthy-string must be never (got {m})");
    });
}

#[test]
fn falsy_mixed_meet_list_is_empty_list_only() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let rhs = f.u(list_atom);
        assert_meet_is_subset(f, lhs, rhs);
        let non_empty_atom = f.t_list(well_known::TYPE_INT, true);
        let non_empty = f.u(non_empty_atom);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert!(
            !does_refine(f, non_empty, m, &empty_world()),
            "non-empty-list<int> values are truthy so cannot be in falsy ∩ list<int> (got meet {m})"
        );
    });
}

#[test]
fn falsy_mixed_meet_non_empty_list_is_never() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let list_atom = f.t_list(well_known::TYPE_INT, true);
        let rhs = f.u(list_atom);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "non-empty-list is always truthy ; falsy meet must be never (got {m})");
    });
}

#[test]
fn falsy_mixed_meet_unsealed_array_is_empty_array_only() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let array_atom = f.t_keyed_unsealed(well_known::TYPE_ARRAY_KEY, well_known::TYPE_MIXED, false);
        let rhs = f.u(array_atom);
        assert_meet_is_subset(f, lhs, rhs);
        let non_empty_atom = f.t_keyed_unsealed(well_known::TYPE_ARRAY_KEY, well_known::TYPE_MIXED, true);
        let non_empty = f.u(non_empty_atom);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert!(
            !does_refine(f, non_empty, m, &empty_world()),
            "non-empty arrays are truthy so cannot be in falsy ∩ array (got meet {m})"
        );
    });
}

#[test]
fn falsy_mixed_meet_iterable_is_never() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let iterable_atom = f.t_iterable(well_known::TYPE_ARRAY_KEY, well_known::TYPE_MIXED);
        let rhs = f.u(iterable_atom);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "iterable is conservatively truthy ; falsy meet must be never (got {m})");
    });
}

#[test]
fn falsy_mixed_meet_class_like_string_is_never() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let class_string = f.t_class_string();
        let rhs = f.u(class_string);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(
            m,
            well_known::TYPE_NEVER,
            "class-like-string is always truthy ; falsy meet must be never (got {m})"
        );
    });
}

#[test]
fn falsy_mixed_meet_callable_is_never() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let callable = f.t_callable_any();
        let rhs = f.u(callable);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "callable is always truthy ; falsy meet must be never (got {m})");
    });
}

#[test]
fn falsy_mixed_meet_resource_is_never() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let resource = f.t_resource();
        let rhs = f.u(resource);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "resource is always truthy ; falsy meet must be never (got {m})");
    });
}

#[test]
fn falsy_mixed_meet_object_is_never() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let object = f.t_object_any();
        let rhs = f.u(object);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "objects are always truthy ; falsy meet must be never (got {m})");
    });
}

#[test]
fn mixed_meet_preserves_is_empty_flag() {
    fixture(|f| {
        let empty_mixed = Atom::Mixed(MixedAtom::EMPTY.with_is_empty(true));
        let non_null_mixed = Atom::Mixed(MixedAtom::EMPTY.with_is_non_null(true));
        let lhs = f.u(empty_mixed);
        let rhs = f.u(non_null_mixed);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        let m_atom = m.atoms[0];
        assert_eq!(m_atom.kind(), AtomKind::Mixed);
        let Atom::Mixed(m_info) = m_atom else {
            panic!("meet of mixed atoms must stay mixed, got {m_atom:?}");
        };
        assert!(m_info.is_empty(), "meet must preserve is_empty (got {m_info:?})");
        assert!(m_info.is_non_null(), "meet must preserve is_non_null (got {m_info:?})");
    });
}

#[test]
fn mixed_meet_preserves_is_isset_from_loop_flag() {
    fixture(|f| {
        let isset_mixed = Atom::Mixed(MixedAtom::EMPTY.with_is_isset_from_loop(true));
        let non_null_mixed = Atom::Mixed(MixedAtom::EMPTY.with_is_non_null(true));
        let lhs = f.u(isset_mixed);
        let rhs = f.u(non_null_mixed);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        let m_atom = m.atoms[0];
        let Atom::Mixed(m_info) = m_atom else {
            panic!("meet of mixed atoms must stay mixed, got {m_atom:?}");
        };
        assert!(m_info.is_isset_from_loop(), "meet must preserve is_isset_from_loop");
        assert!(m_info.is_non_null(), "meet must preserve is_non_null");
    });
}

#[test]
fn truthy_mixed_meet_truthy_mixed_is_truthy() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let rhs = f.u(truthy);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        let m_atom = m.atoms[0];
        let Atom::Mixed(m_info) = m_atom else {
            panic!("meet of mixed atoms must stay mixed, got {m_atom:?}");
        };
        assert!(matches!(m_info.truthiness(), Truthiness::Truthy));
    });
}

#[test]
fn truthy_mixed_meet_non_null_mixed_is_truthy_non_null() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let nonnull = f.mixed_nonnull();
        let rhs = f.u(nonnull);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        let m_atom = m.atoms[0];
        let Atom::Mixed(m_info) = m_atom else {
            panic!("meet of mixed atoms must stay mixed, got {m_atom:?}");
        };
        assert!(matches!(m_info.truthiness(), Truthiness::Truthy));
        assert!(m_info.is_non_null());
    });
}

#[test]
fn truthy_mixed_meet_object_passes_through() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let object = f.t_object_any();
        let rhs = f.u(object);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        assert!(does_refine(f, rhs, m, &w), "objects are truthy so truthy ∩ object should equal object");
    });
}

#[test]
fn falsy_mixed_meet_int_is_zero_only() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let int = f.t_int();
        let rhs = f.u(int);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        let one = f.ui(1);
        assert!(!does_refine(f, one, m, &w), "int(1) is truthy so cannot be in falsy ∩ int (got meet {m})");
        let zero = f.ui(0);
        assert!(does_refine(f, zero, m, &w), "int(0) is falsy so must be in falsy ∩ int (got meet {m})");
    });
}

#[test]
fn truthy_mixed_meet_int_excludes_zero() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let int = f.t_int();
        let rhs = f.u(int);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        let zero = f.ui(0);
        assert!(!does_refine(f, zero, m, &w), "int(0) is falsy so cannot be in truthy ∩ int (got meet {m})");
        let one = f.ui(1);
        assert!(does_refine(f, one, m, &w), "int(1) is truthy so must be in truthy ∩ int (got meet {m})");
    });
}

#[test]
fn falsy_mixed_meet_float_is_zero_only() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let float = f.t_float();
        let rhs = f.u(float);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        let one_float = f.u(Atom::float_literal(1.5));
        assert!(!does_refine(f, one_float, m, &w), "float(1.5) is truthy so cannot be in falsy ∩ float (got meet {m})");
    });
}

#[test]
fn truthy_mixed_meet_bool_is_true() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let bool_atom = f.t_bool();
        let rhs = f.u(bool_atom);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        let true_atom = f.t_true();
        let truevalue = f.u(true_atom);
        assert!(does_refine(f, truevalue, m, &w));
        let false_atom = f.t_false();
        let falsevalue = f.u(false_atom);
        assert!(!does_refine(f, falsevalue, m, &w), "false is falsy so cannot be in truthy ∩ bool (got meet {m})");
    });
}

#[test]
fn falsy_mixed_meet_bool_is_false() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let bool_atom = f.t_bool();
        let rhs = f.u(bool_atom);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        let false_atom = f.t_false();
        let falsevalue = f.u(false_atom);
        assert!(does_refine(f, falsevalue, m, &w));
        let true_atom = f.t_true();
        let truevalue = f.u(true_atom);
        assert!(!does_refine(f, truevalue, m, &w), "true is truthy so cannot be in falsy ∩ bool (got meet {m})");
    });
}

#[test]
fn falsy_mixed_meet_uppercase_string_excludes_truthy_uppercase_literals() {
    fixture(|f| {
        let upper_string = f.builder.string(StringAtom {
            literal: StringLiteral::None,
            casing: StringCasing::Uppercase,
            flags: U8Flags::empty(),
        });
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let rhs = f.u(upper_string);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        let abc = f.us("ABC");
        assert!(
            !does_refine(f, abc, m, &w),
            "string('ABC') is truthy ; falsy ∩ uppercase-string excludes it (got {m})"
        );
    });
}

#[test]
fn truthy_mixed_meet_lowercase_string_excludes_falsy_literals() {
    fixture(|f| {
        let lower_string = f.builder.string(StringAtom {
            literal: StringLiteral::None,
            casing: StringCasing::Lowercase,
            flags: U8Flags::empty(),
        });
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let rhs = f.u(lower_string);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        let empty_lit = f.us("");
        let zero_lit = f.us("0");
        assert!(!does_refine(f, empty_lit, m, &w), "'' is falsy ; truthy ∩ lowercase-string excludes it");
        assert!(!does_refine(f, zero_lit, m, &w), "'0' is falsy ; truthy ∩ lowercase-string excludes it");
    });
}

#[test]
fn falsy_mixed_meet_numeric_string_includes_zero_excludes_one() {
    fixture(|f| {
        let numeric_string = f.builder.string(StringAtom {
            literal: StringLiteral::None,
            casing: StringCasing::Unspecified,
            flags: U8Flags::empty().with(StringRefinementFlag::Numeric).with(StringRefinementFlag::NonEmpty),
        });
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let rhs = f.u(numeric_string);
        assert_meet_is_subset(f, lhs, rhs);
        let w = empty_world();
        let m = lattice_meet(f, lhs, rhs, &w);
        let one_lit = f.us("1");
        assert!(!does_refine(f, one_lit, m, &w), "'1' is truthy ; falsy ∩ numeric-string excludes it (got {m})");
    });
}

#[test]
fn nonnull_mixed_meet_null_is_never() {
    fixture(|f| {
        let nonnull = f.mixed_nonnull();
        let lhs = f.u(nonnull);
        let null = f.null();
        let rhs = f.u(null);
        assert_meet_is_subset(f, lhs, rhs);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER);
    });
}

#[test]
fn sealed_list_with_required_never_is_uninhabited() {
    fixture(|f| {
        let entries = vec![KnownElement { index: 0, value: well_known::TYPE_NEVER, optional: false }];
        let known = f.builder.known_elements(&entries);
        let bad = f.builder.list(ListAtom {
            element_type: well_known::TYPE_NEVER,
            known_elements: Some(known),
            known_count: NonZeroU32::new(1),
            flags: U8Flags::empty(),
        });

        let bad_t = f.u(bad);
        let other_atom = f.t_list(well_known::TYPE_INT, false);
        let other = f.u(other_atom);
        let m = lattice_meet(f, bad_t, other, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "list with required never element is uninhabited (got {m})");
    });
}

#[test]
fn intersected_int_excluding_zero_refines_truthy_mixed() {
    fixture(|f| {
        let zero_t = f.u(well_known::INT_ZERO);
        let neg_zero = f.builder.negated(zero_t);
        let nonzero_int = f.builder.intersected(well_known::INT, &[neg_zero]);
        let nonzero_t = f.u(nonzero_int);
        let truthy_atom = f.mixed_truthy();
        let truthy = f.u(truthy_atom);
        assert!(
            does_refine(f, nonzero_t, truthy, &empty_world()),
            "int & !int(0) is non-zero int, all values truthy ; must refine truthy-mixed (got {nonzero_t})"
        );
    });
}

#[test]
fn empty_list_singleton_refines_falsy_mixed() {
    fixture(|f| {
        let empty_list = f.builder.list(ListAtom {
            element_type: well_known::TYPE_NEVER,
            known_elements: None,
            known_count: None,
            flags: U8Flags::empty(),
        });

        let empty_t = f.u(empty_list);
        let falsy_atom = f.mixed_falsy();
        let falsy = f.u(falsy_atom);
        assert!(
            does_refine(f, empty_t, falsy, &empty_world()),
            "empty list is falsy ; must refine falsy-mixed (got {empty_t})"
        );
    });
}

#[test]
fn empty_array_refines_falsy_mixed() {
    fixture(|f| {
        let empty_array = f.t_empty_array();
        let empty_t = f.u(empty_array);
        let falsy_atom = f.mixed_falsy();
        let falsy = f.u(falsy_atom);
        assert!(does_refine(f, empty_t, falsy, &empty_world()), "empty array is falsy ; must refine falsy-mixed");
    });
}

#[test]
fn mixed_with_is_empty_implies_falsy_truthiness() {
    fixture(|f| {
        let empty_mixed = Atom::Mixed(MixedAtom::EMPTY.with_is_empty(true));
        let empty_t = f.u(empty_mixed);
        let falsy_atom = f.mixed_falsy();
        let falsy = f.u(falsy_atom);
        assert!(
            does_refine(f, empty_t, falsy, &empty_world()),
            "mixed with is_empty is by definition falsy ; must refine falsy-mixed (got {empty_t})"
        );
    });
}

#[test]
fn class_string_refines_truthy_mixed() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let cs = f.u(class_string);
        let truthy_atom = f.mixed_truthy();
        let truthy = f.u(truthy_atom);
        assert!(
            does_refine(f, cs, truthy, &empty_world()),
            "class-strings are non-empty/non-zero ; must refine truthy-mixed"
        );
    });
}

#[test]
fn callable_refines_truthy_mixed() {
    fixture(|f| {
        let callable = f.t_callable_any();
        let c = f.u(callable);
        let truthy_atom = f.mixed_truthy();
        let truthy = f.u(truthy_atom);
        assert!(does_refine(f, c, truthy, &empty_world()), "callables are objects/closures, always truthy");
    });
}

#[test]
fn resource_refines_truthy_mixed() {
    fixture(|f| {
        let resource = f.t_resource();
        let r = f.u(resource);
        let truthy_atom = f.mixed_truthy();
        let truthy = f.u(truthy_atom);
        assert!(does_refine(f, r, truthy, &empty_world()), "resources are always truthy");
    });
}

#[test]
fn intersected_mixed_meet_drops_no_constraint_axis() {
    fixture(|f| {
        let truthy_nonnull_empty = Atom::Mixed(
            MixedAtom::EMPTY.with_truthiness(Truthiness::Truthy).with_is_non_null(true).with_is_isset_from_loop(true),
        );

        let target = f.u(truthy_nonnull_empty);
        let m = lattice_meet(f, target, target, &empty_world());
        assert_eq!(m, target, "self-meet should be identity (got {m})");
    });
}

#[test]
fn intersected_truthy_int_in_falsy_mixed_meet_is_never() {
    fixture(|f| {
        let zero_t = f.u(well_known::INT_ZERO);
        let neg_zero = f.builder.negated(zero_t);
        let nonzero_int = f.builder.intersected(well_known::INT, &[neg_zero]);
        let falsy = f.mixed_falsy();
        let lhs = f.u(falsy);
        let rhs = f.u(nonzero_int);
        let m = lattice_meet(f, lhs, rhs, &empty_world());
        assert_eq!(m, well_known::TYPE_NEVER, "falsy ∩ (int & !int(0)) must be never (got {m})");
    });
}
