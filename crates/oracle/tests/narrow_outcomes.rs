mod common;

use common::*;

use mago_oracle::ty::Type;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::meet;
use mago_oracle::ty::meet::MeetOutcome;
use mago_oracle::ty::subtract;
use mago_oracle::ty::subtract::SubtractOutcome;
use mago_oracle::ty::well_known;

fn meet_narrow<'arena>(
    f: &mut Fixture<'_, 'arena>,
    input: Type<'arena>,
    narrowing: Type<'arena>,
) -> MeetOutcome<'arena> {
    let cb = empty_world();
    let mut report = LatticeReport::new();
    meet::narrow(input, narrowing, &cb, LatticeOptions::default(), &mut report, &mut f.builder)
}

fn subtract_narrow<'arena>(
    f: &mut Fixture<'_, 'arena>,
    input: Type<'arena>,
    narrowing: Type<'arena>,
) -> SubtractOutcome<'arena> {
    let cb = empty_world();
    let mut report = LatticeReport::new();
    subtract::narrow(input, narrowing, &cb, LatticeOptions::default(), &mut report, &mut f.builder)
}

#[test]
fn meet_narrow_redundant_when_input_equals_narrowing() {
    fixture(|f| {
        let r = meet_narrow(f, well_known::TYPE_INT, well_known::TYPE_INT);
        assert_eq!(r, MeetOutcome::Redundant(well_known::TYPE_INT));
    });
}

#[test]
fn meet_narrow_redundant_when_input_refines_narrowing() {
    fixture(|f| {
        let lit = f.ui(42);
        let r = meet_narrow(f, lit, well_known::TYPE_INT);
        assert_eq!(r, MeetOutcome::Redundant(lit));
    });
}

#[test]
fn meet_narrow_narrowed_when_strictly_smaller() {
    fixture(|f| {
        let null = f.null();
        let int = f.t_int();
        let nullable_int = f.u_many(vec![null, int]);
        let r = meet_narrow(f, nullable_int, well_known::TYPE_INT);
        assert_eq!(r, MeetOutcome::Narrowed(well_known::TYPE_INT));
    });
}

#[test]
fn meet_narrow_narrowed_via_subsumption_other_direction() {
    fixture(|f| {
        let lit = f.ui(42);
        let r = meet_narrow(f, well_known::TYPE_INT, lit);
        assert_eq!(r, MeetOutcome::Narrowed(lit));
    });
}

#[test]
fn meet_narrow_impossible_when_disjoint() {
    fixture(|f| {
        let r = meet_narrow(f, well_known::TYPE_INT, well_known::TYPE_STRING);
        assert_eq!(r, MeetOutcome::Impossible);
    });
}

#[test]
fn meet_narrow_impossible_for_unrelated_named_objects_with_no_descend() {
    fixture(|f| {
        let cb = empty_world();
        let mut report = LatticeReport::new();
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let list_ty = f.u(list_atom);
        let r =
            meet::narrow(list_ty, well_known::TYPE_STRING, &cb, LatticeOptions::default(), &mut report, &mut f.builder);
        assert_eq!(r, MeetOutcome::Impossible);
    });
}

#[test]
fn meet_narrow_int_range_overlap_is_narrowed() {
    fixture(|f| {
        let range_a = f.t_int_range(0, 10);
        let a = f.u(range_a);
        let range_b = f.t_int_range(5, 15);
        let b = f.u(range_b);
        let r = meet_narrow(f, a, b);
        let expected_range = f.t_int_range(5, 10);
        let expected = f.u(expected_range);
        assert_eq!(r, MeetOutcome::Narrowed(expected));
    });
}

#[test]
fn meet_narrow_int_range_disjoint_is_impossible() {
    fixture(|f| {
        let range_a = f.t_int_range(0, 10);
        let a = f.u(range_a);
        let range_b = f.t_int_range(20, 30);
        let b = f.u(range_b);
        assert_eq!(meet_narrow(f, a, b), MeetOutcome::Impossible);
    });
}

#[test]
fn meet_compute_returns_never_when_impossible() {
    fixture(|f| {
        let r = meet::compute(
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            &empty_world(),
            LatticeOptions::default(),
            &mut LatticeReport::new(),
            &mut f.builder,
        );
        assert_eq!(r, well_known::TYPE_NEVER);
    });
}

#[test]
fn meet_compute_unwraps_redundant() {
    fixture(|f| {
        let r = meet::compute(
            well_known::TYPE_INT,
            well_known::TYPE_INT,
            &empty_world(),
            LatticeOptions::default(),
            &mut LatticeReport::new(),
            &mut f.builder,
        );
        assert_eq!(r, well_known::TYPE_INT);
    });
}

#[test]
fn subtract_narrow_impossible_when_input_equals_narrowing() {
    fixture(|f| {
        let r = subtract_narrow(f, well_known::TYPE_INT, well_known::TYPE_INT);
        assert_eq!(r, SubtractOutcome::Impossible);
    });
}

#[test]
fn subtract_narrow_impossible_when_input_refines_narrowing() {
    fixture(|f| {
        let lit = f.ui(42);
        let r = subtract_narrow(f, lit, well_known::TYPE_INT);
        assert_eq!(r, SubtractOutcome::Impossible);
    });
}

#[test]
fn subtract_narrow_redundant_when_disjoint() {
    fixture(|f| {
        let r = subtract_narrow(f, well_known::TYPE_INT, well_known::TYPE_STRING);
        assert_eq!(r, SubtractOutcome::Redundant(well_known::TYPE_INT));
    });
}

#[test]
fn subtract_narrow_narrowed_when_strictly_smaller() {
    fixture(|f| {
        let null = f.null();
        let int = f.t_int();
        let nullable_int = f.u_many(vec![null, int]);
        let r = subtract_narrow(f, nullable_int, well_known::TYPE_NULL);
        assert_eq!(r, SubtractOutcome::Narrowed(well_known::TYPE_INT));
    });
}

#[test]
fn subtract_narrow_narrowed_for_int_range_split() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let r_ty = f.u(range);
        let mid = f.ui(5);
        let r = subtract_narrow(f, r_ty, mid);
        let low = f.t_int_range(0, 4);
        let high = f.t_int_range(6, 10);
        let expected = f.u_many(vec![low, high]);
        assert_eq!(r, SubtractOutcome::Narrowed(expected));
    });
}

#[test]
fn subtract_int_minus_zero_splits_into_negative_and_positive_ranges() {
    fixture(|f| {
        let zero = f.ui(0);
        let r = subtract_narrow(f, well_known::TYPE_INT, zero);
        let negative = f.t_negative_int();
        let positive = f.t_positive_int();
        let expected = f.u_many(vec![negative, positive]);
        assert_eq!(r, SubtractOutcome::Narrowed(expected));
    });
}

#[test]
fn subtract_narrow_impossible_when_int_range_fully_covered() {
    fixture(|f| {
        let small_range = f.t_int_range(5, 7);
        let small = f.u(small_range);
        let big_range = f.t_int_range(0, 10);
        let big = f.u(big_range);
        assert_eq!(subtract_narrow(f, small, big), SubtractOutcome::Impossible);
    });
}

#[test]
fn subtract_compute_returns_never_when_impossible() {
    fixture(|f| {
        let r = subtract::compute(
            well_known::TYPE_INT,
            well_known::TYPE_INT,
            &empty_world(),
            LatticeOptions::default(),
            &mut LatticeReport::new(),
            &mut f.builder,
        );
        assert_eq!(r, well_known::TYPE_NEVER);
    });
}

#[test]
fn subtract_compute_unwraps_redundant() {
    fixture(|f| {
        let r = subtract::compute(
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            &empty_world(),
            LatticeOptions::default(),
            &mut LatticeReport::new(),
            &mut f.builder,
        );
        assert_eq!(r, well_known::TYPE_INT);
    });
}

#[test]
fn meet_narrow_unrelated_final_objects_are_impossible() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_final("Foo");
        w.with_final("Bar");
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let bar_atom = f.t_named("Bar");
        let bar = f.u(bar_atom);
        let mut report = LatticeReport::new();
        let r = meet::narrow(foo, bar, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        match r {
            MeetOutcome::Impossible => {}
            other => panic!("expected Impossible, got {other:?}"),
        }
    });
}

#[test]
fn meet_outcome_into_type_round_trip() {
    fixture(|f| {
        let lit = f.ui(42);
        assert_eq!(MeetOutcome::Redundant(lit).into_type(), lit);
        assert_eq!(MeetOutcome::Narrowed(well_known::TYPE_INT).into_type(), well_known::TYPE_INT);
        assert_eq!(MeetOutcome::Impossible.into_type(), well_known::TYPE_NEVER);
    });
}

#[test]
fn subtract_outcome_into_type_round_trip() {
    fixture(|f| {
        let lit = f.ui(42);
        assert_eq!(SubtractOutcome::Redundant(lit).into_type(), lit);
        assert_eq!(SubtractOutcome::Narrowed(well_known::TYPE_INT).into_type(), well_known::TYPE_INT);
        assert_eq!(SubtractOutcome::Impossible.into_type(), well_known::TYPE_NEVER);
    });
}
