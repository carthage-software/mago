mod common;

use std::collections::BTreeMap;

use common::*;

use mago_oracle::symbol::part::generic::Variance;
use mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity;
use mago_oracle::ty::template;
use mago_oracle::ty::template::BoundKind;
use mago_oracle::ty::template::StandinOptions;
use mago_oracle::ty::template::TemplateKey;
use mago_oracle::ty::template::TemplateState;
use mago_oracle::ty::well_known;

fn key_for<'arena>(f: &mut Fixture<'_, 'arena>, class: &'static str, name: &'static str) -> TemplateKey<'arena> {
    TemplateKey {
        defining_entity: DefiningEntity::ClassLike(f.builder.intern_class_like_path(class.as_bytes())),
        name: name.as_bytes(),
    }
}

#[test]
fn keyed_array_value_param_records_lower_bound() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, template_type, false);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f, "F", "T"));
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].kind, BoundKind::Lower);
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn keyed_array_known_item_walked_when_arg_has_matching_key() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let name_key = f.ak_str("name");
        let parameter_atom = f.t_keyed_sealed(BTreeMap::from([(name_key, (false, template_type))]), false);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_keyed_sealed(BTreeMap::from([(name_key, (false, well_known::TYPE_STRING))]), false);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f, "F", "T"));
        assert_eq!(bounds[0].ty, well_known::TYPE_STRING);
    });
}

#[test]
fn keyed_array_against_iterable_walks_key_and_value_params() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let key_template = f.t_template("F", "K");
        let value_template = f.t_template("F", "V");
        let key_type = f.u(key_template);
        let value_type = f.u(value_template);
        let parameter_atom = f.t_keyed_unsealed(key_type, value_type, false);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        assert_eq!(state.bounds_for(key_for(f, "F", "K"))[0].ty, well_known::TYPE_STRING);
        assert_eq!(state.bounds_for(key_for(f, "F", "V"))[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn callable_return_walked_covariantly() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_callable(&[], template_type);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_callable(&[], well_known::TYPE_INT);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f, "F", "T"));
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].kind, BoundKind::Lower);
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn callable_parameter_walked_contravariantly() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_callable(&[template_type], well_known::TYPE_VOID);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_callable(&[well_known::TYPE_INT], well_known::TYPE_VOID);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f, "F", "T"));
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].kind, BoundKind::Upper);
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn callable_records_both_param_and_return_bounds() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let parameter_template = f.t_template("F", "P");
        let return_template = f.t_template("F", "R");
        let parameter_type = f.u(parameter_template);
        let return_type = f.u(return_template);
        let parameter_atom = f.t_callable(&[parameter_type], return_type);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_callable(&[well_known::TYPE_INT], well_known::TYPE_STRING);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        assert_eq!(state.bounds_for(key_for(f, "F", "P"))[0].kind, BoundKind::Upper);
        assert_eq!(state.bounds_for(key_for(f, "F", "P"))[0].ty, well_known::TYPE_INT);
        assert_eq!(state.bounds_for(key_for(f, "F", "R"))[0].kind, BoundKind::Lower);
        assert_eq!(state.bounds_for(key_for(f, "F", "R"))[0].ty, well_known::TYPE_STRING);
    });
}

#[test]
fn descendant_class_arg_threads_through_extension_binding() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Covariant)]);
        world.declare("B");
        world.with_extended("B", "A", vec![well_known::TYPE_INT]);

        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("A", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_named("B");
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f, "F", "T"));
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn descendant_class_arg_substitutes_own_template_args() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Covariant)]);
        world.with_templates("B", &[("U", Variance::Covariant)]);
        let b_u = f.t_template("B", "U");
        let b_u_type = f.u(b_u);
        world.with_extended("B", "A", vec![b_u_type]);

        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("A", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("B", vec![well_known::TYPE_STRING]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f, "F", "T"));
        assert_eq!(bounds[0].ty, well_known::TYPE_STRING);
    });
}

#[test]
fn iteration_depth_cutoff_replaces_template_with_constraint() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default().with_max_depth(0);
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_list(template_type, false);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_list(well_known::TYPE_INT, false);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        assert_eq!(state.iter().count(), 0, "walking past depth 0 collapses to the constraint without recording");
    });
}

#[test]
fn iteration_depth_zero_records_top_level_binding() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default().with_max_depth(0);
        let template = f.t_template("F", "T");
        let parameter = f.u(template);
        template::standin(parameter, well_known::TYPE_INT, &world, &mut state, &options, &mut f.builder);
        assert_eq!(state.bounds_for(key_for(f, "F", "T")).len(), 1, "the depth 0 walk fires before the cutoff check");
    });
}

#[test]
fn keyed_array_unchanged_when_no_template() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let parameter_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let argument = f.u(argument_atom);
        let result = template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        assert_eq!(result, parameter);
        assert_eq!(state.iter().count(), 0);
    });
}

#[test]
fn callable_unchanged_when_no_template() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let parameter_atom = f.t_callable(&[well_known::TYPE_INT], well_known::TYPE_STRING);
        let parameter = f.u(parameter_atom);
        let result = template::standin(parameter, parameter, &world, &mut state, &options, &mut f.builder);
        assert_eq!(result, parameter);
        assert_eq!(state.iter().count(), 0);
    });
}

#[test]
fn descendant_with_no_extension_binding_passes_through() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Covariant)]);
        world.declare("B");
        world.add_edge("B", "A");

        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("A", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_named("B");
        let argument = f.u(argument_atom);
        let result = template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        assert_eq!(result, parameter, "the world has no inherited binding for B extends A");
        assert_eq!(state.iter().count(), 0);
    });
}

#[test]
fn keyed_array_known_value_against_lit_walks_to_lit_bound() {
    fixture(|f| {
        let world = empty_world();
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("F", "T");
        let template_type = f.u(template);
        let value_key = f.ak_str("v");
        let parameter_atom = f.t_keyed_sealed(BTreeMap::from([(value_key, (false, template_type))]), false);
        let parameter = f.u(parameter_atom);
        let forty_two = f.ui(42);
        let argument_atom = f.t_keyed_sealed(BTreeMap::from([(value_key, (false, forty_two))]), false);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &world, &mut state, &options, &mut f.builder);
        assert_eq!(state.bounds_for(key_for(f, "F", "T"))[0].ty, forty_two);
    });
}
