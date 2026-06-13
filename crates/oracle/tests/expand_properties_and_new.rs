mod common;

use std::collections::BTreeMap;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::alias::AliasAtom;
use mago_oracle::ty::atom::payload::derived::DerivedAtom;
use mago_oracle::ty::atom::payload::derived::Visibility;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use mago_oracle::ty::expand;
use mago_oracle::ty::well_known;
use mago_oracle::world::Variance;

fn t_properties_of<'arena>(
    f: &mut Fixture<'_, 'arena>,
    target: Type<'arena>,
    visibility: Option<Visibility>,
) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::PropertiesOf { target, visibility })
}

fn t_new<'arena>(f: &mut Fixture<'_, 'arena>, target: Type<'arena>) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::New(target))
}

fn t_class_string_generic<'arena>(f: &mut Fixture<'_, 'arena>, constraint: Type<'arena>) -> Atom<'arena> {
    f.builder.class_like_string(ClassLikeStringAtom {
        kind: ClassLikeKind::Class,
        specifier: ClassLikeStringSpecifier::Generic { constraint },
    })
}

fn t_alias_elem<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, alias: &str) -> Atom<'arena> {
    let class_name = f.name(class);
    let alias_name = f.name(alias);
    f.builder.alias(AliasAtom { class_name, alias_name })
}

#[test]
fn properties_of_unknown_class_yields_empty_shape() {
    fixture(|f| {
        let world = empty_world();
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let derived_atom = t_properties_of(f, foo_type, None);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let empty_shape = f.t_keyed_sealed(BTreeMap::new(), false);
        let expected = f.u(empty_shape);
        assert_eq!(
            result, expected,
            "an unknown class is observationally a class with no declared properties: both produce array{{}}"
        );
    });
}

#[test]
fn properties_of_class_with_no_properties_returns_empty_shape() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.declare("Foo");
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let derived_atom = t_properties_of(f, foo_type, None);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let empty_shape = f.t_keyed_sealed(BTreeMap::new(), false);
        let expected = f.u(empty_shape);
        assert_eq!(result, expected);
    });
}

#[test]
fn properties_of_class_with_two_properties_returns_shape() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_property("User", "name", well_known::TYPE_STRING);
        world.with_property("User", "age", well_known::TYPE_INT);
        let user = f.t_named("User");
        let user_type = f.u(user);
        let derived_atom = t_properties_of(f, user_type, None);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let name_key = f.ak_str("name");
        let age_key = f.ak_str("age");
        let shape = f.t_keyed_sealed(
            BTreeMap::from([(name_key, (false, well_known::TYPE_STRING)), (age_key, (false, well_known::TYPE_INT))]),
            false,
        );
        let expected = f.u(shape);
        assert_eq!(result, expected);
    });
}

#[test]
fn properties_of_with_public_filter_drops_private() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_visible_property("User", "name", well_known::TYPE_STRING, Visibility::Public);
        world.with_visible_property("User", "secret", well_known::TYPE_STRING, Visibility::Private);
        let user = f.t_named("User");
        let user_type = f.u(user);
        let derived_atom = t_properties_of(f, user_type, Some(Visibility::Public));
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let name_key = f.ak_str("name");
        let shape = f.t_keyed_sealed(BTreeMap::from([(name_key, (false, well_known::TYPE_STRING))]), false);
        let expected = f.u(shape);
        assert_eq!(result, expected);
    });
}

#[test]
fn properties_of_walks_inheritance() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_property("Base", "id", well_known::TYPE_INT);
        world.with_property("Sub", "name", well_known::TYPE_STRING);
        world.add_edge("Sub", "Base");
        let sub = f.t_named("Sub");
        let sub_type = f.u(sub);
        let derived_atom = t_properties_of(f, sub_type, None);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        assert_eq!(result.atoms.len(), 1);
        let Atom::Array(payload) = result.atoms[0] else { panic!("expected a keyed-array shape") };
        let Some(known) = payload.known_items else { panic!("expected known items") };
        assert_eq!(known.len(), 2);
    });
}

#[test]
fn properties_of_subclass_overrides_inherited_property() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_property("Base", "id", well_known::TYPE_INT);
        world.with_property("Sub", "id", well_known::TYPE_STRING);
        world.add_edge("Sub", "Base");
        let sub = f.t_named("Sub");
        let sub_type = f.u(sub);
        let derived_atom = t_properties_of(f, sub_type, None);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let Atom::Array(payload) = result.atoms[0] else { panic!("expected a keyed-array shape") };
        let Some(known) = payload.known_items else { panic!("expected known items") };
        assert_eq!(known.len(), 1);
        assert_eq!(known[0].value, well_known::TYPE_STRING, "the subclass declaration wins");
    });
}

#[test]
fn properties_of_target_resolves_through_alias_first() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_property("Foo", "x", well_known::TYPE_INT);
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        world.with_alias("Other", "FooAlias", foo_type);
        let alias = t_alias_elem(f, "Other", "FooAlias");
        let alias_type = f.u(alias);
        let derived_atom = t_properties_of(f, alias_type, None);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let x_key = f.ak_str("x");
        let shape = f.t_keyed_sealed(BTreeMap::from([(x_key, (false, well_known::TYPE_INT))]), false);
        let expected = f.u(shape);
        assert_eq!(result, expected);
    });
}

#[test]
fn new_with_non_generic_class_returns_object() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.declare("Foo");
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let derived_atom = t_new(f, foo_type);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let expected = f.u(foo);
        assert_eq!(result, expected);
    });
}

#[test]
fn new_with_class_string_literal_returns_named_object() {
    fixture(|f| {
        let world = empty_world();
        let class_string = f.t_lit_class_string("Foo");
        let class_string_type = f.u(class_string);
        let derived_atom = t_new(f, class_string_type);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let foo = f.t_named("Foo");
        let expected = f.u(foo);
        assert_eq!(result, expected);
    });
}

#[test]
fn new_with_generic_class_fills_args_with_constraints_or_mixed() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);
        let box_atom = f.t_named("Box");
        let box_type = f.u(box_atom);
        let derived_atom = t_new(f, box_type);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let filled = f.t_generic_named("Box", vec![well_known::TYPE_MIXED]);
        let expected = f.u(filled);
        assert_eq!(result, expected);
    });
}

#[test]
fn new_with_generic_class_uses_template_upper_bound() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);
        world.with_template_bound("Box", "T", well_known::TYPE_INT);
        let box_atom = f.t_named("Box");
        let box_type = f.u(box_atom);
        let derived_atom = t_new(f, box_type);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        let filled = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(filled);
        assert_eq!(result, expected);
    });
}

#[test]
fn new_with_unresolved_target_passes_through() {
    fixture(|f| {
        let world = empty_world();
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let union_target = f.u_many(vec![foo, bar]);
        let derived_atom = t_new(f, union_target);
        let derived = f.u(derived_atom);
        assert_eq!(
            expand::expand(derived, &world, &mut f.builder),
            derived,
            "a single class name cannot be extracted from a union"
        );
    });
}

#[test]
fn new_with_bounded_class_string_returns_constraint_instance() {
    fixture(|f| {
        let world = empty_world();
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let class_string = f.t_class_string_of(foo_type);
        let class_string_type = f.u(class_string);
        let derived_atom = t_new(f, class_string_type);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        assert_eq!(result, foo_type, "new on class-string<Foo> produces a Foo instance");
    });
}

#[test]
fn new_with_generic_class_string_returns_template_type() {
    fixture(|f| {
        let world = empty_world();
        let template = f.t_template("Factory", "T");
        let template_type = f.u(template);
        let class_string = t_class_string_generic(f, template_type);
        let class_string_type = f.u(class_string);
        let derived_atom = t_new(f, class_string_type);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        assert_eq!(result, template_type, "new on class-string<T> produces a T");
    });
}

#[test]
fn new_with_bare_class_string_returns_object() {
    fixture(|f| {
        let world = empty_world();
        let class_string = f.t_class_string();
        let class_string_type = f.u(class_string);
        let derived_atom = t_new(f, class_string_type);
        let derived = f.u(derived_atom);
        let result = expand::expand(derived, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_OBJECT, "new on a bare class-string produces an unknown object");
    });
}

#[test]
fn properties_of_inside_alias_resolves() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_property("User", "name", well_known::TYPE_STRING);
        let user = f.t_named("User");
        let user_type = f.u(user);
        let derived_atom = t_properties_of(f, user_type, None);
        let derived_type = f.u(derived_atom);
        world.with_alias("Foo", "UserShape", derived_type);
        let alias = t_alias_elem(f, "Foo", "UserShape");
        let alias_type = f.u(alias);
        let result = expand::expand(alias_type, &world, &mut f.builder);
        let name_key = f.ak_str("name");
        let shape = f.t_keyed_sealed(BTreeMap::from([(name_key, (false, well_known::TYPE_STRING))]), false);
        let expected = f.u(shape);
        assert_eq!(result, expected);
    });
}
