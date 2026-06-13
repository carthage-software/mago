mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::atom::payload::alias::AliasAtom;
use mago_oracle::ty::atom::payload::reference::GlobalReferenceAtom;
use mago_oracle::ty::atom::payload::reference::MemberReferenceAtom;
use mago_oracle::ty::atom::payload::reference::NameSelector;
use mago_oracle::ty::expand;
use mago_oracle::ty::expand::ExpansionContext;
use mago_oracle::ty::well_known;
use mago_oracle::world::Variance;

fn alias_atom<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, alias: &str) -> Atom<'arena> {
    let class_name = f.name(class);
    let alias_name = f.name(alias);
    f.builder.alias(AliasAtom { class_name, alias_name })
}

fn class_const_atom<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, name: &str) -> Atom<'arena> {
    let class_like_name = f.name(class);
    let constant = f.name(name);
    f.builder.member_reference(MemberReferenceAtom { class_like_name, selector: NameSelector::Identifier(constant) })
}

fn global_const_atom<'arena>(f: &mut Fixture<'_, 'arena>, name: &str) -> Atom<'arena> {
    let constant = f.name(name);
    f.builder.global_reference(GlobalReferenceAtom { selector: NameSelector::Identifier(constant) })
}

#[test]
fn alias_resolves_when_eval_aliases_on() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Body", well_known::TYPE_INT);
        let alias = alias_atom(f, "Foo", "Body");
        let ty = f.u(alias);
        assert_eq!(expand::expand_with(ty, &world, &ExpansionContext::default(), &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn alias_passes_through_when_eval_aliases_off() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Body", well_known::TYPE_INT);
        let alias = alias_atom(f, "Foo", "Body");
        let ty = f.u(alias);
        let context = ExpansionContext::default().with_evaluate_aliases(false);
        assert_eq!(expand::expand_with(ty, &world, &context, &mut f.builder), ty);
    });
}

#[test]
fn class_constant_resolves_when_flag_on() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant("Foo", "BAR", well_known::TYPE_INT);
        let constant = class_const_atom(f, "Foo", "BAR");
        let ty = f.u(constant);
        assert_eq!(expand::expand_with(ty, &world, &ExpansionContext::default(), &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn class_constant_passes_through_when_flag_off() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant("Foo", "BAR", well_known::TYPE_INT);
        let constant = class_const_atom(f, "Foo", "BAR");
        let ty = f.u(constant);
        let context = ExpansionContext::default().with_evaluate_class_constants(false);
        assert_eq!(expand::expand_with(ty, &world, &context, &mut f.builder), ty);
    });
}

#[test]
fn global_constant_resolves_when_flag_on() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_global_constant("VERSION", well_known::TYPE_STRING);
        let constant = global_const_atom(f, "VERSION");
        let ty = f.u(constant);
        assert_eq!(
            expand::expand_with(ty, &world, &ExpansionContext::default(), &mut f.builder),
            well_known::TYPE_STRING
        );
    });
}

#[test]
fn global_constant_passes_through_when_flag_off() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_global_constant("VERSION", well_known::TYPE_STRING);
        let constant = global_const_atom(f, "VERSION");
        let ty = f.u(constant);
        let context = ExpansionContext::default().with_evaluate_global_constants(false);
        assert_eq!(expand::expand_with(ty, &world, &context, &mut f.builder), ty);
    });
}

#[test]
fn unfilled_generic_object_filled_when_flag_on() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);
        let box_atom = f.t_named("Box");
        let unfilled = f.u(box_atom);
        let context = ExpansionContext::default().with_fill_template_defaults(true);
        let result = expand::expand_with(unfilled, &world, &context, &mut f.builder);
        let filled_atom = f.t_generic_named("Box", vec![well_known::TYPE_MIXED]);
        let expected = f.u(filled_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn unfilled_generic_object_uses_declared_upper_bound_when_present() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);
        world.with_template_bound("Box", "T", well_known::TYPE_INT);
        let box_atom = f.t_named("Box");
        let unfilled = f.u(box_atom);
        let context = ExpansionContext::default().with_fill_template_defaults(true);
        let result = expand::expand_with(unfilled, &world, &context, &mut f.builder);
        let filled_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(filled_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn unfilled_generic_object_unchanged_when_flag_off() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);
        let box_atom = f.t_named("Box");
        let unfilled = f.u(box_atom);
        assert_eq!(expand::expand_with(unfilled, &world, &ExpansionContext::default(), &mut f.builder), unfilled);
    });
}

#[test]
fn template_constraint_substituted_when_flag_on() {
    fixture(|f| {
        let world = empty_world();
        let template = f.t_template_of("Foo", "T", well_known::TYPE_INT);
        let ty = f.u(template);
        let context = ExpansionContext::default().with_substitute_template_constraints(true);
        assert_eq!(expand::expand_with(ty, &world, &context, &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn template_passes_through_when_constraint_flag_off() {
    fixture(|f| {
        let world = empty_world();
        let template = f.t_template_of("Foo", "T", well_known::TYPE_INT);
        let ty = f.u(template);
        assert_eq!(expand::expand_with(ty, &world, &ExpansionContext::default(), &mut f.builder), ty);
    });
}

#[test]
fn function_is_final_collapses_static_modality_without_static_class() {
    fixture(|f| {
        let world = empty_world();
        let static_atom = f.t_named_static("Foo");
        let plain_atom = f.t_named("Foo");
        let plain = f.u(plain_atom);
        let ty = f.u(static_atom);
        let context = ExpansionContext::default().with_function_is_final(true);
        assert_eq!(expand::expand_with(ty, &world, &context, &mut f.builder), plain);
    });
}

#[test]
fn static_modality_preserved_when_function_is_final_off() {
    fixture(|f| {
        let world = empty_world();
        let static_atom = f.t_named_static("Foo");
        let ty = f.u(static_atom);
        assert_eq!(expand::expand_with(ty, &world, &ExpansionContext::default(), &mut f.builder), ty);
    });
}
