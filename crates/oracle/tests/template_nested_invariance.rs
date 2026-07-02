mod common;

use common::*;

use mago_oracle::path::Path;
use mago_oracle::ty::template;
use mago_oracle::ty::template::StandinOptions;
use mago_oracle::ty::template::TemplateState;
use mago_oracle::ty::well_known;

fn marker_for<'arena>(
    f: &mut Fixture<'_, 'arena>,
    state: &TemplateState<'arena>,
    class: &'static str,
    name: &'static str,
) -> Option<Path<'arena>> {
    let key = mago_oracle::ty::template::TemplateKey {
        defining_entity: mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity::ClassLike(
            f.builder.intern_class_like_path(class.as_bytes()),
        ),
        name: name.as_bytes(),
    };
    let bounds = state.bounds_for(key);
    assert_eq!(bounds.len(), 1, "expected exactly one bound for {class}::{name}");

    bounds[0].equality_bound_classlike
}

#[test]
fn invariance_propagates_through_nested_covariant_position() {
    fixture(|f| {
        let symbols =
            symbol_table(f.arena, "<?php /** @template V */ class Cell {} /** @template-covariant T */ class Box {}");

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
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);

        let marked = marker_for(f, &state, "Box", "T");
        let expected = f.name("Cell");
        assert_eq!(
            marked,
            Some(expected),
            "invariance introduced at Cell must mark the bound on T even though Box's position is covariant"
        );
    });
}

#[test]
fn pure_covariant_nesting_records_no_marker() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php /** @template-covariant E */ class Bag {} /** @template-covariant T */ class Box {}",
        );

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
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);

        let marked = marker_for(f, &state, "Box", "T");
        assert_eq!(marked, None, "no invariant position is crossed, so the bound must carry no invariant marker");
    });
}

#[test]
fn invariance_at_outer_position_marks_direct_template() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template V */ class Cell {}");

        let template = f.t_template("Cell", "V");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("Cell", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Cell", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);

        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);

        let marked = marker_for(f, &state, "Cell", "V");
        let expected = f.name("Cell");
        assert_eq!(marked, Some(expected));
    });
}
