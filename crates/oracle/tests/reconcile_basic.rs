mod common;

use common::*;

use mago_allocator::LocalArena;
use mago_oracle::path::Path;
use mago_oracle::symbol::part::generic::Variance;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity;
use mago_oracle::ty::template;
use mago_oracle::ty::template::Bound;
use mago_oracle::ty::template::BoundKind;
use mago_oracle::ty::template::StandinOptions;
use mago_oracle::ty::template::TemplateKey;
use mago_oracle::ty::template::TemplateState;
use mago_oracle::ty::well_known;

fn key_for<'arena>(arena: &'arena LocalArena, class: &'static str, name: &'static str) -> TemplateKey<'arena> {
    TemplateKey {
        defining_entity: DefiningEntity::ClassLike(Path::class_like(arena, class.as_bytes())),
        name: name.as_bytes(),
    }
}

fn bound<'arena>(
    arena: &'arena LocalArena,
    kind: BoundKind,
    ty: Type<'arena>,
    depth: u32,
    offset: u32,
) -> Bound<'arena> {
    let equality_bound_classlike = match kind {
        BoundKind::Equality => Some(Path::class_like(arena, b"C")),
        BoundKind::Lower | BoundKind::Upper => None,
    };

    Bound { kind, ty, depth, argument_offset: offset, equality_bound_classlike, span: None }
}

#[test]
fn empty_bounds_returns_none() {
    fixture(|f| {
        assert!(template::reconcile(&[], &mut f.builder).is_none());
    });
}

#[test]
fn single_bound_yields_that_type() {
    fixture(|f| {
        let result =
            template::reconcile(&[bound(f.arena, BoundKind::Lower, well_known::TYPE_INT, 0, 0)], &mut f.builder);
        assert_eq!(result, Some(well_known::TYPE_INT));
    });
}

#[test]
fn two_shallow_bounds_at_same_depth_union() {
    fixture(|f| {
        let result = template::reconcile(
            &[
                bound(f.arena, BoundKind::Lower, well_known::TYPE_INT, 0, 0),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_STRING, 0, 1),
            ],
            &mut f.builder,
        );
        assert_eq!(result, Some(well_known::TYPE_INT_OR_STRING));
    });
}

#[test]
fn deeper_bound_discarded_without_equality_marker() {
    fixture(|f| {
        let result = template::reconcile(
            &[
                bound(f.arena, BoundKind::Lower, well_known::TYPE_INT, 0, 0),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_STRING, 1, 0),
            ],
            &mut f.builder,
        );
        assert_eq!(result, Some(well_known::TYPE_INT), "without an equality marker only the shallowest bound counts");
    });
}

#[test]
fn deeper_bound_included_when_baseline_is_equality_marker_and_offset_matches() {
    fixture(|f| {
        let result = template::reconcile(
            &[
                bound(f.arena, BoundKind::Equality, well_known::TYPE_INT, 0, 0),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_STRING, 1, 0),
            ],
            &mut f.builder,
        );
        assert_eq!(result, Some(well_known::TYPE_INT_OR_STRING));
    });
}

#[test]
fn deeper_bound_discarded_when_equality_marker_present_but_offset_differs() {
    fixture(|f| {
        let result = template::reconcile(
            &[
                bound(f.arena, BoundKind::Equality, well_known::TYPE_INT, 0, 0),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_STRING, 1, 1),
            ],
            &mut f.builder,
        );
        assert_eq!(result, Some(well_known::TYPE_INT));
    });
}

#[test]
fn equality_marker_propagates_to_further_depths() {
    fixture(|f| {
        let result = template::reconcile(
            &[
                bound(f.arena, BoundKind::Equality, well_known::TYPE_INT, 0, 0),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_STRING, 1, 0),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_FLOAT, 2, 0),
            ],
            &mut f.builder,
        );
        let Some(witness) = result else { panic!("expected a witness") };
        assert_eq!(witness.atoms.len(), 3);
    });
}

#[test]
fn unsorted_input_is_handled_correctly() {
    fixture(|f| {
        let result = template::reconcile(
            &[
                bound(f.arena, BoundKind::Lower, well_known::TYPE_FLOAT, 2, 0),
                bound(f.arena, BoundKind::Equality, well_known::TYPE_INT, 0, 0),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_STRING, 1, 0),
            ],
            &mut f.builder,
        );
        let Some(witness) = result else { panic!("expected a witness") };
        assert_eq!(witness.atoms.len(), 3);
    });
}

#[test]
fn multiple_baseline_bounds_with_equality_propagate() {
    fixture(|f| {
        let result = template::reconcile(
            &[
                bound(f.arena, BoundKind::Lower, well_known::TYPE_INT, 0, 0),
                bound(f.arena, BoundKind::Equality, well_known::TYPE_STRING, 0, 1),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_FLOAT, 1, 0),
            ],
            &mut f.builder,
        );
        let Some(witness) = result else { panic!("expected a witness") };
        assert_eq!(witness.atoms.len(), 3);
    });
}

#[test]
fn witness_falls_back_when_no_bound_collected() {
    fixture(|f| {
        let state = TemplateState::new();
        let key = key_for(f.arena, "Box", "T");
        let result = state.witness(key, well_known::TYPE_MIXED, &mut f.builder);
        assert_eq!(result, well_known::TYPE_MIXED);
    });
}

#[test]
fn witness_uses_recorded_bounds_when_present() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default().with_default_variance(Variance::Covariant);
        let template = f.t_template("Box", "T");
        let ty = f.u(template);
        template::standin(ty, well_known::TYPE_INT, &world, &mut state, &options, &mut f.builder);
        let result = state.witness(key_for(f.arena, "Box", "T"), well_known::TYPE_MIXED, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn witness_after_two_arguments_unions_bounds() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let template = f.t_template("F", "T");
        let ty = f.u(template);

        let options0 = StandinOptions::default().with_argument_offset(0).with_default_variance(Variance::Covariant);
        template::standin(ty, well_known::TYPE_INT, &world, &mut state, &options0, &mut f.builder);
        let options1 = StandinOptions::default().with_argument_offset(1).with_default_variance(Variance::Covariant);
        template::standin(ty, well_known::TYPE_STRING, &world, &mut state, &options1, &mut f.builder);

        let result = state.witness(key_for(f.arena, "F", "T"), well_known::TYPE_MIXED, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn witness_after_invariant_then_nested_arg_keeps_deep_bound() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Cell", &[("T", Variance::Invariant)]);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("Cell", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Cell", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        let result = state.witness(key_for(f.arena, "F", "T"), well_known::TYPE_MIXED, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn covariant_only_bounds_keep_shallowest_only() {
    fixture(|f| {
        let result = template::reconcile(
            &[
                bound(f.arena, BoundKind::Lower, well_known::TYPE_INT, 0, 0),
                bound(f.arena, BoundKind::Lower, well_known::TYPE_STRING, 5, 0),
            ],
            &mut f.builder,
        );
        assert_eq!(result, Some(well_known::TYPE_INT));
    });
}

#[test]
fn upper_bound_alone_is_returned_as_witness() {
    fixture(|f| {
        let result =
            template::reconcile(&[bound(f.arena, BoundKind::Upper, well_known::TYPE_STRING, 0, 0)], &mut f.builder);
        assert_eq!(result, Some(well_known::TYPE_STRING));
    });
}
