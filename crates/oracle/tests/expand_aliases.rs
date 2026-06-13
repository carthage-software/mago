mod common;

use std::collections::BTreeMap;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::atom::payload::alias::AliasAtom;
use mago_oracle::ty::expand;
use mago_oracle::ty::well_known;

fn t_alias<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, alias: &str) -> Atom<'arena> {
    let class_name = f.name(class);
    let alias_name = f.name(alias);
    f.builder.alias(AliasAtom { class_name, alias_name })
}

#[test]
fn alias_to_int_expands() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Id", well_known::TYPE_INT);
        let alias = t_alias(f, "Foo", "Id");
        let ty = f.u(alias);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn unknown_alias_passes_through_unchanged() {
    fixture(|f| {
        let world = empty_world();
        let alias = t_alias(f, "Foo", "Id");
        let ty = f.u(alias);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), ty);
    });
}

#[test]
fn alias_to_union_flat_merges() {
    fixture(|f| {
        let mut world = MockWorld::new();
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        world.with_alias("Foo", "Key", int_or_string);
        let alias = t_alias(f, "Foo", "Key");
        let ty = f.u(alias);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn nested_alias_expands_recursively() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "A", well_known::TYPE_INT);
        let inner_alias = t_alias(f, "Foo", "A");
        let inner_type = f.u(inner_alias);
        world.with_alias("Foo", "B", inner_type);
        let alias = t_alias(f, "Foo", "B");
        let ty = f.u(alias);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn alias_inside_list_expands() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Id", well_known::TYPE_INT);
        let alias = t_alias(f, "Foo", "Id");
        let alias_type = f.u(alias);
        let list_atom = f.t_list(alias_type, false);
        let list = f.u(list_atom);
        let result = expand::expand(list, &world, &mut f.builder);
        let expected_atom = f.t_list(well_known::TYPE_INT, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn alias_inside_object_type_args_expands() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Id", well_known::TYPE_INT);
        let alias = t_alias(f, "Foo", "Id");
        let alias_type = f.u(alias);
        let generic_atom = f.t_generic_named("Box", vec![alias_type]);
        let generic = f.u(generic_atom);
        let result = expand::expand(generic, &world, &mut f.builder);
        let expected_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn alias_inside_iterable_expands() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "K", well_known::TYPE_STRING);
        world.with_alias("Foo", "V", well_known::TYPE_INT);
        let key_alias = t_alias(f, "Foo", "K");
        let key = f.u(key_alias);
        let value_alias = t_alias(f, "Foo", "V");
        let value = f.u(value_alias);
        let iterable_atom = f.t_iterable(key, value);
        let iterable = f.u(iterable_atom);
        let result = expand::expand(iterable, &world, &mut f.builder);
        let expected_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn alias_inside_keyed_array_value_expands() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Id", well_known::TYPE_INT);
        let alias = t_alias(f, "Foo", "Id");
        let alias_type = f.u(alias);
        let id_key = f.ak_str("id");
        let shape_atom = f.t_keyed_sealed(BTreeMap::from([(id_key, (false, alias_type))]), false);
        let shape = f.u(shape_atom);
        let result = expand::expand(shape, &world, &mut f.builder);
        let expected_atom = f.t_keyed_sealed(BTreeMap::from([(id_key, (false, well_known::TYPE_INT))]), false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn alias_to_alias_chain_expands_to_terminal() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "A", well_known::TYPE_INT);
        let a_alias = t_alias(f, "Foo", "A");
        let a_type = f.u(a_alias);
        world.with_alias("Foo", "B", a_type);
        let b_alias = t_alias(f, "Foo", "B");
        let b_type = f.u(b_alias);
        world.with_alias("Foo", "C", b_type);
        let c_alias = t_alias(f, "Foo", "C");
        let ty = f.u(c_alias);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn distinct_aliases_in_union_each_expand() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "I", well_known::TYPE_INT);
        world.with_alias("Foo", "S", well_known::TYPE_STRING);
        let int_alias = t_alias(f, "Foo", "I");
        let string_alias = t_alias(f, "Foo", "S");
        let ty = f.u_many(vec![int_alias, string_alias]);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn no_alias_no_change() {
    fixture(|f| {
        let world = empty_world();
        assert_eq!(expand::expand(well_known::TYPE_INT, &world, &mut f.builder), well_known::TYPE_INT);
        assert_eq!(
            expand::expand(well_known::TYPE_INT_OR_STRING, &world, &mut f.builder),
            well_known::TYPE_INT_OR_STRING
        );
    });
}

#[test]
fn expanded_handle_is_stable() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Id", well_known::TYPE_INT);
        let alias = t_alias(f, "Foo", "Id");
        let ty = f.u(alias);
        let first = expand::expand(ty, &world, &mut f.builder);
        let second = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(first, second);
    });
}

#[test]
fn fully_structural_input_returns_same_handle() {
    fixture(|f| {
        let world = empty_world();
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let ty = f.u(list_atom);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, ty);
    });
}

#[test]
fn deeply_nested_alias_in_box_of_list_expands() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Id", well_known::TYPE_INT);
        let alias = t_alias(f, "Foo", "Id");
        let alias_type = f.u(alias);
        let list_of_alias_atom = f.t_list(alias_type, false);
        let list_of_alias = f.u(list_of_alias_atom);
        let box_of_list_atom = f.t_generic_named("Box", vec![list_of_alias]);
        let box_of_list = f.u(box_of_list_atom);
        let result = expand::expand(box_of_list, &world, &mut f.builder);
        let expected_inner_atom = f.t_list(well_known::TYPE_INT, false);
        let expected_inner = f.u(expected_inner_atom);
        let expected_atom = f.t_generic_named("Box", vec![expected_inner]);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}
