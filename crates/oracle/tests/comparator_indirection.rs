mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::alias::AliasAtom;
use mago_oracle::ty::atom::payload::conditional::ConditionalAtom;
use mago_oracle::ty::atom::payload::derived::DerivedAtom;
use mago_oracle::ty::atom::payload::reference::GlobalReferenceAtom;
use mago_oracle::ty::atom::payload::reference::MemberReferenceAtom;
use mago_oracle::ty::atom::payload::reference::NameSelector;
use mago_oracle::ty::atom::payload::reference::SymbolReferenceAtom;
use mago_oracle::ty::atom::payload::variable::VariableAtom;
use mago_oracle::ty::well_known;

fn t_variable<'arena>(f: &mut Fixture<'_, 'arena>, name: &str) -> Atom<'arena> {
    let name = f.name(name);
    Atom::Variable(VariableAtom { name })
}

fn t_reference<'arena>(f: &mut Fixture<'_, 'arena>, name: &str) -> Atom<'arena> {
    let name = f.name(name);
    f.builder.reference(SymbolReferenceAtom { name, type_arguments: None })
}

fn t_member_reference<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, member: &str) -> Atom<'arena> {
    let class_like_name = f.name(class);
    let member_name = f.name(member);
    f.builder.member_reference(MemberReferenceAtom { class_like_name, selector: NameSelector::Identifier(member_name) })
}

fn t_global_reference<'arena>(f: &mut Fixture<'_, 'arena>, name: &str) -> Atom<'arena> {
    let name = f.name(name);
    f.builder.global_reference(GlobalReferenceAtom { selector: NameSelector::Identifier(name) })
}

fn t_alias<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, alias: &str) -> Atom<'arena> {
    let class_name = f.name(class);
    let alias_name = f.name(alias);
    f.builder.alias(AliasAtom { class_name, alias_name })
}

fn t_conditional<'arena>(
    f: &mut Fixture<'_, 'arena>,
    subject: Type<'arena>,
    target: Type<'arena>,
    then: Type<'arena>,
    otherwise: Type<'arena>,
) -> Atom<'arena> {
    f.builder.conditional(ConditionalAtom { subject, target, then, otherwise, negated: false })
}

fn t_key_of<'arena>(f: &mut Fixture<'_, 'arena>, target: Type<'arena>) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::KeyOf(target))
}

#[test]
fn variable_reflexive() {
    fixture(|f| {
        let variable = t_variable(f, "T");
        assert!(atomic_is_contained(f, variable, variable, &empty_world()));
    });
}

#[test]
fn distinct_variables_dont_refine() {
    fixture(|f| {
        let t = t_variable(f, "T");
        let u = t_variable(f, "U");
        assert!(!atomic_is_contained(f, t, u, &empty_world()));
        assert!(!atomic_is_contained(f, u, t, &empty_world()));
    });
}

#[test]
fn reference_handle_equal_refines_via_interning() {
    fixture(|f| {
        let first = t_reference(f, "Foo");
        let second = t_reference(f, "Foo");
        assert!(atomic_is_contained(f, first, second, &empty_world()));
    });
}

#[test]
fn distinct_references_dont_refine_without_resolution() {
    fixture(|f| {
        let foo = t_reference(f, "Foo");
        let bar = t_reference(f, "Bar");
        assert!(!atomic_is_contained(f, foo, bar, &empty_world()));
    });
}

#[test]
fn concrete_input_does_not_refine_unresolved_reference() {
    fixture(|f| {
        let foo_reference = t_reference(f, "Foo");
        let int = f.t_int();
        let foo_named = f.t_named("Foo");
        assert!(!atomic_is_contained(f, int, foo_reference, &empty_world()));
        assert!(!atomic_is_contained(f, foo_named, foo_reference, &empty_world()));
    });
}

#[test]
fn reference_input_does_not_refine_concrete_container() {
    fixture(|f| {
        let foo_reference = t_reference(f, "Foo");
        let int = f.t_int();
        let foo_named = f.t_named("Foo");
        assert!(!atomic_is_contained(f, foo_reference, int, &empty_world()));
        assert!(!atomic_is_contained(f, foo_reference, foo_named, &empty_world()));
    });
}

#[test]
fn reference_refines_mixed_via_top() {
    fixture(|f| {
        let foo_reference = t_reference(f, "Foo");
        let mixed = f.mixed();
        assert!(atomic_is_contained(f, foo_reference, mixed, &empty_world()));
    });
}

#[test]
fn never_refines_reference_via_bot() {
    fixture(|f| {
        let foo_reference = t_reference(f, "Foo");
        let never = f.never();
        assert!(atomic_is_contained(f, never, foo_reference, &empty_world()));
    });
}

#[test]
fn member_reference_handle_equality() {
    fixture(|f| {
        let first = t_member_reference(f, "Foo", "BAR");
        let second = t_member_reference(f, "Foo", "BAR");
        let third = t_member_reference(f, "Foo", "BAZ");
        assert!(atomic_is_contained(f, first, second, &empty_world()));
        assert!(!atomic_is_contained(f, first, third, &empty_world()));
    });
}

#[test]
fn global_reference_handle_equality() {
    fixture(|f| {
        let first = t_global_reference(f, "PHP_INT_MAX");
        let second = t_global_reference(f, "PHP_INT_MAX");
        let third = t_global_reference(f, "PHP_INT_MIN");
        assert!(atomic_is_contained(f, first, second, &empty_world()));
        assert!(!atomic_is_contained(f, first, third, &empty_world()));
    });
}

#[test]
fn alias_handle_equality() {
    fixture(|f| {
        let first = t_alias(f, "Foo", "MyAlias");
        let second = t_alias(f, "Foo", "MyAlias");
        let third = t_alias(f, "Foo", "OtherAlias");
        assert!(atomic_is_contained(f, first, second, &empty_world()));
        assert!(!atomic_is_contained(f, first, third, &empty_world()));
    });
}

#[test]
fn conditional_handle_equality() {
    fixture(|f| {
        let first = t_conditional(
            f,
            well_known::TYPE_INT,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
        );
        let second = t_conditional(
            f,
            well_known::TYPE_INT,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
        );
        let third = t_conditional(
            f,
            well_known::TYPE_STRING,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
        );
        assert!(atomic_is_contained(f, first, second, &empty_world()));
        assert!(!atomic_is_contained(f, first, third, &empty_world()));
    });
}

#[test]
fn derived_handle_equality() {
    fixture(|f| {
        let first = t_key_of(f, well_known::TYPE_INT);
        let second = t_key_of(f, well_known::TYPE_INT);
        let third = t_key_of(f, well_known::TYPE_STRING);
        assert!(atomic_is_contained(f, first, second, &empty_world()));
        assert!(!atomic_is_contained(f, first, third, &empty_world()));
    });
}

#[test]
fn distinct_indirection_kinds_dont_cross() {
    fixture(|f| {
        let variable = t_variable(f, "T");
        let reference = t_reference(f, "T");
        assert!(!atomic_is_contained(f, variable, reference, &empty_world()));
        assert!(!atomic_is_contained(f, reference, variable, &empty_world()));
    });
}
