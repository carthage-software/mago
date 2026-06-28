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

fn alias_atom<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, alias: &str) -> Atom<'arena> {
    let class_name = f.name(class);
    let alias_name = f.builder.intern(alias.as_bytes());
    f.builder.alias(AliasAtom { class_name, alias_name })
}

fn class_const_atom<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, name: &str) -> Atom<'arena> {
    let class_like_name = f.name(class);
    let constant = f.builder.intern(name.as_bytes());
    f.builder.member_reference(MemberReferenceAtom { class_like_name, selector: NameSelector::Identifier(constant) })
}

fn global_const_atom<'arena>(f: &mut Fixture<'_, 'arena>, name: &str) -> Atom<'arena> {
    let constant = f.builder.intern(name.as_bytes());
    f.builder.global_reference(GlobalReferenceAtom { selector: NameSelector::Identifier(constant) })
}

#[test]
fn alias_resolves_when_eval_aliases_on() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @type Body = int */ class Foo {}");
        let alias = alias_atom(f, "Foo", "Body");
        let ty = f.u(alias);
        assert_eq!(
            expand::expand_with(ty, &symbols, &ExpansionContext::default(), &mut f.builder),
            well_known::TYPE_INT
        );
    });
}

#[test]
fn alias_passes_through_when_eval_aliases_off() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @type Body = int */ class Foo {}");
        let alias = alias_atom(f, "Foo", "Body");
        let ty = f.u(alias);
        let context = ExpansionContext::default().with_evaluate_aliases(false);
        assert_eq!(expand::expand_with(ty, &symbols, &context, &mut f.builder), ty);
    });
}

#[test]
fn class_constant_resolves_when_flag_on() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo { const int BAR = 0; }");
        let constant = class_const_atom(f, "Foo", "BAR");
        let ty = f.u(constant);
        assert_eq!(
            expand::expand_with(ty, &symbols, &ExpansionContext::default(), &mut f.builder),
            well_known::TYPE_INT
        );
    });
}

#[test]
fn class_constant_passes_through_when_flag_off() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo { const int BAR = 0; }");
        let constant = class_const_atom(f, "Foo", "BAR");
        let ty = f.u(constant);
        let context = ExpansionContext::default().with_evaluate_class_constants(false);
        assert_eq!(expand::expand_with(ty, &symbols, &context, &mut f.builder), ty);
    });
}

#[test]
fn global_constant_resolves_when_flag_on() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @var string */ const VERSION = '';");
        let constant = global_const_atom(f, "VERSION");
        let ty = f.u(constant);
        assert_eq!(
            expand::expand_with(ty, &symbols, &ExpansionContext::default(), &mut f.builder),
            well_known::TYPE_STRING
        );
    });
}

#[test]
fn global_constant_passes_through_when_flag_off() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @var string */ const VERSION = '';");
        let constant = global_const_atom(f, "VERSION");
        let ty = f.u(constant);
        let context = ExpansionContext::default().with_evaluate_global_constants(false);
        assert_eq!(expand::expand_with(ty, &symbols, &context, &mut f.builder), ty);
    });
}

#[test]
fn unfilled_generic_object_filled_when_flag_on() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Box {}");
        let box_atom = f.t_named("Box");
        let unfilled = f.u(box_atom);
        let context = ExpansionContext::default().with_fill_template_defaults(true);
        let result = expand::expand_with(unfilled, &symbols, &context, &mut f.builder);
        let filled_atom = f.t_generic_named("Box", vec![well_known::TYPE_MIXED]);
        let expected = f.u(filled_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn unfilled_generic_object_uses_declared_upper_bound_when_present() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T of int */ class Box {}");
        let box_atom = f.t_named("Box");
        let unfilled = f.u(box_atom);
        let context = ExpansionContext::default().with_fill_template_defaults(true);
        let result = expand::expand_with(unfilled, &symbols, &context, &mut f.builder);
        let filled_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(filled_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn unfilled_generic_object_unchanged_when_flag_off() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Box {}");
        let box_atom = f.t_named("Box");
        let unfilled = f.u(box_atom);
        assert_eq!(expand::expand_with(unfilled, &symbols, &ExpansionContext::default(), &mut f.builder), unfilled);
    });
}

#[test]
fn template_constraint_substituted_when_flag_on() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let template = f.t_template_of("Foo", "T", well_known::TYPE_INT);
        let ty = f.u(template);
        let context = ExpansionContext::default().with_substitute_template_constraints(true);
        assert_eq!(expand::expand_with(ty, &symbols, &context, &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn template_passes_through_when_constraint_flag_off() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let template = f.t_template_of("Foo", "T", well_known::TYPE_INT);
        let ty = f.u(template);
        assert_eq!(expand::expand_with(ty, &symbols, &ExpansionContext::default(), &mut f.builder), ty);
    });
}

#[test]
fn function_is_final_collapses_static_modality_without_static_class() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let static_atom = f.t_named_static("Foo");
        let plain_atom = f.t_named("Foo");
        let plain = f.u(plain_atom);
        let ty = f.u(static_atom);
        let context = ExpansionContext::default().with_function_is_final(true);
        assert_eq!(expand::expand_with(ty, &symbols, &context, &mut f.builder), plain);
    });
}

#[test]
fn static_modality_preserved_when_function_is_final_off() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let static_atom = f.t_named_static("Foo");
        let ty = f.u(static_atom);
        assert_eq!(expand::expand_with(ty, &symbols, &ExpansionContext::default(), &mut f.builder), ty);
    });
}
