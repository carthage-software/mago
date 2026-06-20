mod common;

use common::*;

use mago_allocator::LocalArena;
use mago_span::Span;

use mago_oracle::path::Path;
use mago_oracle::symbol::part::generic::Variance;
use mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity;
use mago_oracle::ty::atom::payload::reference::SymbolReferenceAtom;
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

#[test]
fn top_level_template_records_invariant_bound() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Box", "T");
        let ty = f.u(template);
        let result = template::standin(ty, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        assert_eq!(result, well_known::TYPE_MIXED, "the refined parameter is T's constraint (mixed by default)");
        let key = key_for(f.arena, "Box", "T");
        let bounds = state.bounds_for(key);
        assert_eq!(bounds.len(), 1);
        assert_eq!(
            bounds[0],
            Bound {
                kind: BoundKind::Equality,
                ty: well_known::TYPE_INT,
                argument_offset: 0,
                depth: 0,
                equality_bound_classlike: None,
                span: None,
            }
        );
    });
}

#[test]
fn top_level_template_with_int_constraint_emits_int_standin() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template_of("Box", "T", well_known::TYPE_INT);
        let ty = f.u(template);
        let argument = f.ui(42);
        let result = template::standin(ty, argument, &symbols, &mut state, &options, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn covariant_default_records_lower_bound() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default().with_default_variance(Variance::Covariant);
        let template = f.t_template("Box", "T");
        let ty = f.u(template);
        template::standin(ty, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(bounds[0].kind, BoundKind::Lower);
    });
}

#[test]
fn contravariant_default_records_upper_bound() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default().with_default_variance(Variance::Contravariant);
        let template = f.t_template("Box", "T");
        let ty = f.u(template);
        template::standin(ty, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(bounds[0].kind, BoundKind::Upper);
    });
}

#[test]
fn argument_offset_is_recorded() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default().with_argument_offset(3);
        let template = f.t_template("Box", "T");
        let ty = f.u(template);
        template::standin(ty, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(bounds[0].argument_offset, 3);
    });
}

#[test]
fn template_inside_list_records_lower_bound_at_depth_one() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Box", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_list(template_type, false);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_list(well_known::TYPE_INT, false);
        let argument = f.u(argument_atom);
        let result = template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let expected_atom = f.t_list(well_known::TYPE_MIXED, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(
            bounds[0],
            Bound {
                kind: BoundKind::Lower,
                ty: well_known::TYPE_INT,
                argument_offset: 0,
                depth: 1,
                equality_bound_classlike: None,
                span: None,
            }
        );
    });
}

#[test]
fn template_inside_list_against_iterable_arg_walks_value() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Box", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_list(template_type, false);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_iterable(well_known::TYPE_INT, well_known::TYPE_STRING);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(bounds[0].ty, well_known::TYPE_STRING);
    });
}

#[test]
fn template_inside_iterable_records_both_key_and_value() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let key_template = f.t_template("M", "K");
        let value_template = f.t_template("M", "V");
        let key_type = f.u(key_template);
        let value_type = f.u(value_template);
        let parameter_atom = f.t_iterable(key_type, value_type);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let key_bound = state.bounds_for(key_for(f.arena, "M", "K"));
        let value_bound = state.bounds_for(key_for(f.arena, "M", "V"));
        assert_eq!(key_bound[0].ty, well_known::TYPE_STRING);
        assert_eq!(value_bound[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn template_inside_object_uses_symbols_variance() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template-covariant T */ class Container {}");
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Container", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("Container", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Container", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Container", "T"));
        assert_eq!(bounds[0].kind, BoundKind::Lower);
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn template_inside_object_with_invariant_records_equality_bound() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Cell {}");
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Cell", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("Cell", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Cell", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Cell", "T"));
        assert_eq!(bounds[0].kind, BoundKind::Equality);
    });
}

#[test]
fn object_with_unrelated_arg_passes_parameter_through() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template-covariant T */ class Box {}");
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Box", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("Box", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Bag", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);
        let result = template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        assert_eq!(result, parameter, "an unrelated argument class drives no inference");
        assert!(state.bounds_for(key_for(f.arena, "Box", "T")).is_empty());
    });
}

#[test]
fn nested_object_template_records_at_correct_depth() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template-covariant T */ class Box {}");
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Box", "T");
        let template_type = f.u(template);
        let inner_parameter_atom = f.t_list(template_type, false);
        let inner_parameter = f.u(inner_parameter_atom);
        let parameter_atom = f.t_generic_named("Box", vec![inner_parameter]);
        let parameter = f.u(parameter_atom);
        let inner_argument_atom = f.t_list(well_known::TYPE_INT, false);
        let inner_argument = f.u(inner_argument_atom);
        let argument_atom = f.t_generic_named("Box", vec![inner_argument]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(bounds[0].depth, 2, "object args walk at depth 1; list element walk at depth 2");
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn equal_param_and_argument_short_circuits_no_changes() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let result =
            template::standin(well_known::TYPE_INT, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
        assert_eq!(state.iter().count(), 0, "no template parameter mentioned means no bounds recorded");
    });
}

#[test]
fn invariant_object_walk_records_introducing_class_on_equality_bound() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Cell {}");
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Cell", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("Cell", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Cell", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Cell", "T"));
        assert_eq!(bounds[0].kind, BoundKind::Equality);
        assert_eq!(bounds[0].equality_bound_classlike, Some(Path::class_like(f.arena, b"Cell")));
    });
}

#[test]
fn covariant_object_walk_does_not_set_equality_classlike() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template-covariant T */ class Container {}");
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Container", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_generic_named("Container", vec![template_type]);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Container", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Container", "T"));
        assert_eq!(bounds[0].kind, BoundKind::Lower);
        assert_eq!(bounds[0].equality_bound_classlike, None);
    });
}

#[test]
fn top_level_invariant_walk_outside_class_has_no_classlike() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Free", "T");
        let ty = f.u(template);
        template::standin(ty, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Free", "T"));
        assert_eq!(bounds[0].kind, BoundKind::Equality);
        assert_eq!(bounds[0].equality_bound_classlike, None);
    });
}

#[test]
fn span_from_options_propagates_to_recorded_bound() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let span = Span::dummy(10, 20);
        let options = StandinOptions::default().with_span(span);
        let template = f.t_template("Box", "T");
        let ty = f.u(template);
        template::standin(ty, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(bounds[0].span, Some(span));
    });
}

#[test]
fn walk_auto_declares_encountered_template() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Box", "T");
        let ty = f.u(template);
        template::standin(ty, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        let key = key_for(f.arena, "Box", "T");
        assert!(state.is_declared(key));
        let Some(declaration) = state.declaration(key) else { panic!("the walk must auto-declare T") };
        assert_eq!(declaration.constraint, well_known::TYPE_MIXED);
    });
}

#[test]
fn walk_preserves_existing_declaration_constraint() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let key = key_for(f.arena, "Box", "T");
        state.declare(key, well_known::TYPE_INT);
        let options = StandinOptions::default();
        let template = f.t_template("Box", "T");
        let ty = f.u(template);
        template::standin(ty, well_known::TYPE_STRING, &symbols, &mut state, &options, &mut f.builder);
        let Some(declaration) = state.declaration(key) else { panic!("T must stay declared") };
        assert_eq!(declaration.constraint, well_known::TYPE_INT);
    });
}

#[test]
fn declared_but_unbound_distinguishable_from_undeclared() {
    fixture(|f| {
        let mut state = TemplateState::new();
        let bound_key = key_for(f.arena, "Box", "T");
        let unbound_key = key_for(f.arena, "Box", "U");
        let absent_key = key_for(f.arena, "Box", "Z");

        state.declare(bound_key, well_known::TYPE_MIXED);
        state.declare(unbound_key, well_known::TYPE_MIXED);

        assert!(state.is_declared(bound_key));
        assert!(state.is_declared(unbound_key));
        assert!(!state.is_declared(absent_key));

        assert!(state.bounds_for(bound_key).is_empty());
        assert!(state.bounds_for(unbound_key).is_empty());
    });
}

#[test]
fn bounds_in_scope_filters_by_defining_entity() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let foo_template = f.t_template("Foo", "T");
        let foo_t = f.u(foo_template);
        let bar_template = f.t_template("Bar", "U");
        let bar_u = f.u(bar_template);
        template::standin(foo_t, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        template::standin(bar_u, well_known::TYPE_STRING, &symbols, &mut state, &options, &mut f.builder);

        let foo_entity = key_for(f.arena, "Foo", "T").defining_entity;
        let scoped: Vec<_> = state.bounds_in_scope(foo_entity).map(|(key, _)| key.name).collect();
        assert_eq!(scoped, vec![b"T".as_slice()]);
    });
}

#[test]
fn declarations_in_scope_filters_by_defining_entity() {
    fixture(|f| {
        let mut state = TemplateState::new();
        state.declare(key_for(f.arena, "Foo", "T"), well_known::TYPE_MIXED);
        state.declare(key_for(f.arena, "Bar", "U"), well_known::TYPE_INT);
        let foo_entity = key_for(f.arena, "Foo", "T").defining_entity;
        let scoped: Vec<_> = state.declarations_in_scope(foo_entity).map(|(key, _)| key.name).collect();
        assert_eq!(scoped, vec![b"T".as_slice()]);
    });
}

#[test]
fn merge_scope_re_keys_declarations_and_bounds() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();

        let foo_template = f.t_template("Foo", "T");
        let foo_t = f.u(foo_template);
        template::standin(foo_t, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);

        let foo_entity = key_for(f.arena, "Foo", "T").defining_entity;
        let bar_entity = key_for(f.arena, "Bar", "Z").defining_entity;

        state.merge_scope(foo_entity, bar_entity);

        let bar_t_key = TemplateKey { defining_entity: bar_entity, name: b"T" };
        assert!(state.is_declared(bar_t_key), "after merge, T's declaration lives under Bar");
        assert_eq!(state.bounds_for(bar_t_key).len(), 1);

        assert_eq!(state.bounds_in_scope(foo_entity).count(), 0, "Foo entity has nothing left");
        assert_eq!(state.declarations_in_scope(foo_entity).count(), 0);
    });
}

#[test]
fn merge_scope_appends_when_target_already_has_bounds_for_same_name() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();

        let foo_template = f.t_template("Foo", "T");
        let foo_t = f.u(foo_template);
        template::standin(foo_t, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        let bar_template = f.t_template("Bar", "T");
        let bar_t = f.u(bar_template);
        template::standin(bar_t, well_known::TYPE_STRING, &symbols, &mut state, &options, &mut f.builder);

        let foo_entity = key_for(f.arena, "Foo", "T").defining_entity;
        let bar_entity = key_for(f.arena, "Bar", "T").defining_entity;

        state.merge_scope(foo_entity, bar_entity);

        let bar_t_key = TemplateKey { defining_entity: bar_entity, name: b"T" };
        assert_eq!(state.bounds_for(bar_t_key).len(), 2);
    });
}

#[test]
fn freeze_preserves_declarations_bounds_and_anti_bounds() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let foo_template = f.t_template("Foo", "T");
        let foo_t = f.u(foo_template);
        template::standin(foo_t, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);
        state.add_anti_bound(key_for(f.arena, "Foo", "T"), well_known::TYPE_STRING);

        let result = state.freeze();
        let key = key_for(f.arena, "Foo", "T");
        assert!(result.is_declared(key));
        assert_eq!(result.bounds_for(key).len(), 1);
        assert_eq!(result.anti_bounds_for(key), &[well_known::TYPE_STRING]);
    });
}

#[test]
fn freeze_consumes_state() {
    let state: TemplateState<'static> = TemplateState::new();
    let _result = state.freeze();
}

#[test]
fn frozen_result_supports_scope_filtered_queries() {
    fixture(|f| {
        let mut state = TemplateState::new();
        state.declare(key_for(f.arena, "Foo", "T"), well_known::TYPE_MIXED);
        state.declare(key_for(f.arena, "Bar", "U"), well_known::TYPE_INT);
        state.add_anti_bound(key_for(f.arena, "Foo", "T"), well_known::TYPE_STRING);
        let result = state.freeze();
        let foo_entity = key_for(f.arena, "Foo", "T").defining_entity;
        assert_eq!(result.declarations_in_scope(foo_entity).count(), 1);
        assert_eq!(result.anti_bounds_in_scope(foo_entity).count(), 1);
    });
}

#[test]
fn anti_bound_recorded_and_queryable() {
    fixture(|f| {
        let mut state = TemplateState::new();
        let key = key_for(f.arena, "Foo", "T");
        state.add_anti_bound(key, well_known::TYPE_INT);
        state.add_anti_bound(key, well_known::TYPE_STRING);
        let anti_bounds = state.anti_bounds_for(key);
        assert_eq!(anti_bounds.len(), 2);
        assert!(anti_bounds.contains(&well_known::TYPE_INT));
        assert!(anti_bounds.contains(&well_known::TYPE_STRING));
    });
}

#[test]
fn anti_bound_unset_returns_empty_slice() {
    fixture(|f| {
        let state = TemplateState::new();
        assert!(state.anti_bounds_for(key_for(f.arena, "Foo", "T")).is_empty());
    });
}

#[test]
fn anti_bounds_in_scope_filters_by_defining_entity() {
    fixture(|f| {
        let mut state = TemplateState::new();
        state.add_anti_bound(key_for(f.arena, "Foo", "T"), well_known::TYPE_INT);
        state.add_anti_bound(key_for(f.arena, "Bar", "U"), well_known::TYPE_STRING);
        let foo_entity = key_for(f.arena, "Foo", "T").defining_entity;
        let scoped: Vec<_> = state.anti_bounds_in_scope(foo_entity).map(|(key, _)| key.name).collect();
        assert_eq!(scoped, vec![b"T".as_slice()]);
    });
}

#[test]
fn merge_scope_re_keys_anti_bounds() {
    fixture(|f| {
        let mut state = TemplateState::new();
        state.add_anti_bound(key_for(f.arena, "Foo", "T"), well_known::TYPE_INT);
        let foo_entity = key_for(f.arena, "Foo", "T").defining_entity;
        let bar_entity = key_for(f.arena, "Bar", "Z").defining_entity;
        state.merge_scope(foo_entity, bar_entity);
        let bar_t_key = TemplateKey { defining_entity: bar_entity, name: b"T" };
        assert_eq!(state.anti_bounds_for(bar_t_key), &[well_known::TYPE_INT]);
        assert!(state.anti_bounds_for(key_for(f.arena, "Foo", "T")).is_empty());
    });
}

#[test]
fn merge_scope_into_self_is_noop() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let foo_template = f.t_template("Foo", "T");
        let foo_t = f.u(foo_template);
        template::standin(foo_t, well_known::TYPE_INT, &symbols, &mut state, &options, &mut f.builder);

        let foo_entity = key_for(f.arena, "Foo", "T").defining_entity;
        state.merge_scope(foo_entity, foo_entity);

        assert_eq!(state.bounds_for(key_for(f.arena, "Foo", "T")).len(), 1);
        assert!(state.is_declared(key_for(f.arena, "Foo", "T")));
    });
}

#[test]
fn declarations_iter_yields_every_declared_template() {
    fixture(|f| {
        let mut state = TemplateState::new();
        state.declare(key_for(f.arena, "Foo", "T"), well_known::TYPE_MIXED);
        state.declare(key_for(f.arena, "Bar", "U"), well_known::TYPE_INT);
        let names: Vec<_> = state.declarations().map(|(key, _)| key.name).collect();
        assert!(names.contains(&b"T".as_slice()));
        assert!(names.contains(&b"U".as_slice()));
    });
}

#[test]
fn span_threads_through_nested_walk() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template-covariant T */ class Box {}");
        let mut state = TemplateState::new();
        let span = Span::dummy(100, 110);
        let options = StandinOptions::default().with_span(span);
        let template = f.t_template("Box", "T");
        let template_type = f.u(template);
        let template_list_atom = f.t_list(template_type, false);
        let template_list = f.u(template_list_atom);
        let parameter_atom = f.t_generic_named("Box", vec![template_list]);
        let parameter = f.u(parameter_atom);
        let int_list_atom = f.t_list(well_known::TYPE_INT, false);
        let int_list = f.u(int_list_atom);
        let argument_atom = f.t_generic_named("Box", vec![int_list]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(bounds[0].span, Some(span));
        assert_eq!(bounds[0].depth, 2);
    });
}

#[test]
fn parameter_without_templates_passes_through_unchanged() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let result = template::standin(
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            &symbols,
            &mut state,
            &options,
            &mut f.builder,
        );
        assert_eq!(result, well_known::TYPE_INT);
        assert_eq!(state.iter().count(), 0);
    });
}

#[test]
fn multiple_arguments_share_state_and_accumulate_bounds() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();

        let template = f.t_template("F", "T");
        let parameter = f.u(template);

        let options0 = StandinOptions::default().with_argument_offset(0).with_default_variance(Variance::Covariant);
        template::standin(parameter, well_known::TYPE_INT, &symbols, &mut state, &options0, &mut f.builder);

        let options1 = StandinOptions::default().with_argument_offset(1).with_default_variance(Variance::Covariant);
        template::standin(parameter, well_known::TYPE_STRING, &symbols, &mut state, &options1, &mut f.builder);

        let bounds = state.bounds_for(key_for(f.arena, "F", "T"));
        assert_eq!(bounds.len(), 2);
        assert_eq!(bounds[0].argument_offset, 0);
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
        assert_eq!(bounds[1].argument_offset, 1);
        assert_eq!(bounds[1].ty, well_known::TYPE_STRING);
    });
}

#[test]
fn distinct_template_parameters_recorded_separately() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let t_template = f.t_template("F", "T");
        let u_template = f.t_template("F", "U");
        let t_type = f.u(t_template);
        let u_type = f.u(u_template);
        let parameter_atom = f.t_iterable(t_type, u_type);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);

        let t_bounds = state.bounds_for(key_for(f.arena, "F", "T"));
        let u_bounds = state.bounds_for(key_for(f.arena, "F", "U"));
        assert_eq!(t_bounds.len(), 1);
        assert_eq!(u_bounds.len(), 1);
        assert_eq!(t_bounds[0].ty, well_known::TYPE_STRING);
        assert_eq!(u_bounds[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn iter_returns_all_recorded_keys() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let t_template = f.t_template("F", "T");
        let u_template = f.t_template("F", "U");
        let t_type = f.u(t_template);
        let u_type = f.u(u_template);
        let parameter_atom = f.t_iterable(t_type, u_type);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);

        assert_eq!(state.iter().count(), 2);
    });
}

#[test]
fn template_inside_reference_binds_like_object() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template-covariant T */ class Box {}");
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Box", "T");
        let template_type = f.u(template);
        let box_name = f.name("Box");
        let parameter_arguments = f.builder.types(&[template_type]);
        let parameter_atom =
            f.builder.reference(SymbolReferenceAtom { name: box_name, type_arguments: Some(parameter_arguments) });
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Box", "T"));
        assert_eq!(bounds.len(), 1, "an un-expanded Foo<T> reference still drives inference");
        assert_eq!(bounds[0].kind, BoundKind::Lower);
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
    });
}

#[test]
fn template_inside_class_string_binds_from_literal_argument() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Factory", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_class_string_of(template_type);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_lit_class_string("Foo");
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let bounds = state.bounds_for(key_for(f.arena, "Factory", "T"));
        assert_eq!(bounds.len(), 1, "class-string<T> vs Foo::class binds T");
        assert_eq!(bounds[0].kind, BoundKind::Lower);
        assert_eq!(bounds[0].ty, foo_type, "T is bound to the Foo instance type");
    });
}

#[test]
fn template_inside_class_string_binds_from_of_type_argument() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Factory", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_class_string_of(template_type);
        let parameter = f.u(parameter_atom);
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let argument_atom = f.t_class_string_of(foo_type);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Factory", "T"));
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].ty, foo_type, "class-string<T> vs class-string<Foo> binds T = Foo");
    });
}

#[test]
fn template_inside_object_shape_binds_per_property() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let mut state = TemplateState::new();
        let options = StandinOptions::default();
        let template = f.t_template("Wrap", "T");
        let template_type = f.u(template);
        let parameter_atom = f.t_object_shape(&[("x", template_type, false)], true);
        let parameter = f.u(parameter_atom);
        let argument_atom = f.t_object_shape(&[("x", well_known::TYPE_INT, false)], true);
        let argument = f.u(argument_atom);
        template::standin(parameter, argument, &symbols, &mut state, &options, &mut f.builder);
        let bounds = state.bounds_for(key_for(f.arena, "Wrap", "T"));
        assert_eq!(bounds.len(), 1, "object{{x: T}} vs object{{x: int}} binds T per property");
        assert_eq!(bounds[0].kind, BoundKind::Lower);
        assert_eq!(bounds[0].ty, well_known::TYPE_INT);
    });
}
