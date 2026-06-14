mod common;

use common::*;

use mago_oracle::name::Name;
use mago_oracle::ty::template;
use mago_oracle::ty::template::StandinOptions;
use mago_oracle::ty::template::TemplateState;
use mago_oracle::ty::well_known;
use mago_oracle::world::Variance;

fn marker_for<'arena>(state: &TemplateState<'arena>, class: &'static str, name: &'static str) -> Option<Name<'arena>> {
    let key = mago_oracle::ty::template::TemplateKey {
        defining_entity: mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity::ClassLike(Name::new(
            class.as_bytes(),
        )),
        name: Name::new(name.as_bytes()),
    };
    let bounds = state.bounds_for(key);
    assert_eq!(bounds.len(), 1, "expected exactly one bound for {class}::{name}");

    bounds[0].equality_bound_classlike
}

#[test]
fn invariance_propagates_through_nested_covariant_position() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Cell", &[("V", Variance::Invariant)]);
        world.with_templates("Box", &[("T", Variance::Covariant)]);

        let template = f.t_template("Box", "T");
        let template_type = f.u(template);
        let inner_parameter = f.t_generic_named("Box", vec![template_type]);
        let inner_parameter_type = f.u(inner_parameter);
        let parameter_atom = f.t_generic_named("Cell", vec![inner_parameter_type]);
        let parameter = f.u(parameter_atom);

        let inner_argument = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let inner_argument_type = f.u(inner_argument);
        let argument_atom = f.t_generic_named("Cell", vec![inner_argument_type]);
        let argument = f.u(argument_atom);

        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);

        assert_eq!(
            marker_for(&state, "Box", "T"),
            Some(Name::new(b"Cell")),
            "invariance introduced at Cell must mark the bound on T even though Box's position is covariant"
        );
    });
}

#[test]
fn pure_covariant_nesting_records_no_marker() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Bag", &[("E", Variance::Covariant)]);
        world.with_templates("Box", &[("T", Variance::Covariant)]);

        let template = f.t_template("Box", "T");
        let template_type = f.u(template);
        let inner_parameter = f.t_generic_named("Box", vec![template_type]);
        let inner_parameter_type = f.u(inner_parameter);
        let parameter_atom = f.t_generic_named("Bag", vec![inner_parameter_type]);
        let parameter = f.u(parameter_atom);

        let inner_argument = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let inner_argument_type = f.u(inner_argument);
        let argument_atom = f.t_generic_named("Bag", vec![inner_argument_type]);
        let argument = f.u(argument_atom);

        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);

        assert_eq!(
            marker_for(&state, "Box", "T"),
            None,
            "no invariant position is crossed, so the bound must carry no invariant marker"
        );
    });
}

#[test]
fn invariance_at_outer_position_marks_direct_template() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Cell", &[("V", Variance::Invariant)]);

        let template = f.t_template("Cell", "V");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("Cell", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Cell", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);

        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);

        assert_eq!(marker_for(&state, "Cell", "V"), Some(Name::new(b"Cell")));
    });
}
