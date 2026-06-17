mod common;

use std::collections::BTreeMap;

use common::*;

use mago_oracle::symbol::part::generic::Variance;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::alias::AliasAtom;
use mago_oracle::ty::atom::payload::array::KnownElement;
use mago_oracle::ty::atom::payload::derived::DerivedAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::expand;
use mago_oracle::ty::well_known;

fn t_key_of<'arena>(f: &mut Fixture<'_, 'arena>, target: Type<'arena>) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::KeyOf(target))
}

fn t_value_of<'arena>(f: &mut Fixture<'_, 'arena>, target: Type<'arena>) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::ValueOf(target))
}

fn t_index_access<'arena>(f: &mut Fixture<'_, 'arena>, target: Type<'arena>, index: Type<'arena>) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::IndexAccess { target, index })
}

fn t_int_mask<'arena>(f: &mut Fixture<'_, 'arena>, operands: Vec<Type<'arena>>) -> Atom<'arena> {
    let members = f.builder.types(&operands);
    f.builder.derived(DerivedAtom::IntMask(members))
}

fn t_int_mask_of<'arena>(f: &mut Fixture<'_, 'arena>, target: Type<'arena>) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::IntMaskOf(target))
}

fn t_template_type<'arena>(f: &mut Fixture<'_, 'arena>, class: Type<'arena>, name: Type<'arena>) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::TemplateType {
        object: well_known::TYPE_MIXED,
        class_name: class,
        template_name: name,
    })
}

fn t_template_type_of<'arena>(
    f: &mut Fixture<'_, 'arena>,
    object: Type<'arena>,
    class: Type<'arena>,
    name: Type<'arena>,
) -> Atom<'arena> {
    f.builder.derived(DerivedAtom::TemplateType { object, class_name: class, template_name: name })
}

fn t_sealed_list<'arena>(f: &mut Fixture<'_, 'arena>, elements: &[Type<'arena>]) -> Atom<'arena> {
    let entries: Vec<KnownElement<'arena>> = elements
        .iter()
        .enumerate()
        .map(|(index, value)| KnownElement { index: index as u32, value: *value, optional: false })
        .collect();
    f.builder.sealed_list(&entries, !elements.is_empty())
}

fn t_alias_elem<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, alias: &str) -> Atom<'arena> {
    let class_name = f.name(class);
    let alias_name = f.builder.intern(alias.as_bytes());
    f.builder.alias(AliasAtom { class_name, alias_name })
}

#[test]
fn key_of_unsealed_list_is_non_negative_int() {
    fixture(|f| {
        let world = empty_world();
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let list = f.u(list_atom);
        let key_of = t_key_of(f, list);
        let ty = f.u(key_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        let non_negative = f.t_non_negative_int();
        let expected = f.u(non_negative);
        assert_eq!(result, expected);
    });
}

#[test]
fn key_of_sealed_list_is_index_range() {
    fixture(|f| {
        let world = empty_world();
        let elements = [well_known::TYPE_INT, well_known::TYPE_STRING, well_known::TYPE_FLOAT];
        let list_atom = t_sealed_list(f, &elements);
        let list = f.u(list_atom);
        let key_of = t_key_of(f, list);
        let ty = f.u(key_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        let atoms = result.atoms;
        assert!(!atoms.is_empty());
        let zero = f.t_lit_int(0);
        let first = atoms[0];
        let zero_refines_first = atomic_is_contained(f, zero, first, &world);
        assert!(zero_refines_first || atoms.len() > 1, "each known index must refine the result");
    });
}

#[test]
fn key_of_iterable_is_key_type() {
    fixture(|f| {
        let world = empty_world();
        let iterable_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let iterable = f.u(iterable_atom);
        let key_of = t_key_of(f, iterable);
        let ty = f.u(key_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_STRING);
    });
}

#[test]
fn key_of_keyed_array_with_param_is_key_param() {
    fixture(|f| {
        let world = empty_world();
        let array_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let array = f.u(array_atom);
        let key_of = t_key_of(f, array);
        let ty = f.u(key_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_STRING);
    });
}

#[test]
fn key_of_sealed_keyed_shape_is_union_of_literal_keys() {
    fixture(|f| {
        let world = empty_world();
        let name_key = f.ak_str("name");
        let age_key = f.ak_str("age");
        let shape_atom = f.t_keyed_sealed(
            BTreeMap::from([(name_key, (false, well_known::TYPE_STRING)), (age_key, (false, well_known::TYPE_INT))]),
            false,
        );
        let shape = f.u(shape_atom);
        let key_of = t_key_of(f, shape);
        let ty = f.u(key_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result.atoms.len(), 2);
    });
}

#[test]
fn key_of_non_container_is_mixed() {
    fixture(|f| {
        let world = empty_world();
        let key_of = t_key_of(f, well_known::TYPE_INT);
        let ty = f.u(key_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_MIXED);
    });
}

#[test]
fn value_of_list_is_element_type() {
    fixture(|f| {
        let world = empty_world();
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let list = f.u(list_atom);
        let value_of = t_value_of(f, list);
        let ty = f.u(value_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn value_of_iterable_is_value_type() {
    fixture(|f| {
        let world = empty_world();
        let iterable_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let iterable = f.u(iterable_atom);
        let value_of = t_value_of(f, iterable);
        let ty = f.u(value_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn value_of_keyed_array_with_param_is_value_param() {
    fixture(|f| {
        let world = empty_world();
        let array_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let array = f.u(array_atom);
        let value_of = t_value_of(f, array);
        let ty = f.u(value_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn index_access_on_sealed_shape_with_known_key_returns_value() {
    fixture(|f| {
        let world = empty_world();
        let id_key = f.ak_str("id");
        let shape_atom = f.t_keyed_sealed(BTreeMap::from([(id_key, (false, well_known::TYPE_INT))]), false);
        let shape = f.u(shape_atom);
        let id_index = f.us("id");
        let index_access = t_index_access(f, shape, id_index);
        let ty = f.u(index_access);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn index_access_on_sealed_shape_with_unknown_key_returns_never() {
    fixture(|f| {
        let world = empty_world();
        let id_key = f.ak_str("id");
        let shape_atom = f.t_keyed_sealed(BTreeMap::from([(id_key, (false, well_known::TYPE_INT))]), false);
        let shape = f.u(shape_atom);
        let missing_index = f.us("missing");
        let index_access = t_index_access(f, shape, missing_index);
        let ty = f.u(index_access);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_NEVER);
    });
}

#[test]
fn index_access_on_unsealed_keyed_array_returns_value_param() {
    fixture(|f| {
        let world = empty_world();
        let array_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let array = f.u(array_atom);
        let any_index = f.us("any");
        let index_access = t_index_access(f, array, any_index);
        let ty = f.u(index_access);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn index_access_on_iterable_returns_value_type() {
    fixture(|f| {
        let world = empty_world();
        let iterable_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let iterable = f.u(iterable_atom);
        let any_index = f.us("any");
        let index_access = t_index_access(f, iterable, any_index);
        let ty = f.u(index_access);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn index_access_on_list_with_literal_index_returns_known_element() {
    fixture(|f| {
        let world = empty_world();
        let elements = [well_known::TYPE_INT, well_known::TYPE_STRING, well_known::TYPE_FLOAT];
        let list_atom = t_sealed_list(f, &elements);
        let list = f.u(list_atom);
        let one = f.ui(1);
        let index_access = t_index_access(f, list, one);
        let ty = f.u(index_access);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_STRING);
    });
}

#[test]
fn int_mask_of_two_literals_yields_four_combinations() {
    fixture(|f| {
        let world = empty_world();
        let one = f.ui(1);
        let two = f.ui(2);
        let mask = t_int_mask(f, vec![one, two]);
        let ty = f.u(mask);
        let result = expand::expand(ty, &world, &mut f.builder);
        let mut got: Vec<i64> = result
            .atoms
            .iter()
            .filter_map(|atom| match atom {
                Atom::Int(IntAtom::Literal(value)) => Some(*value),
                _ => None,
            })
            .collect();
        got.sort_unstable();
        assert_eq!(got, vec![0, 1, 2, 3]);
    });
}

#[test]
fn int_mask_of_three_literals_yields_eight_combinations() {
    fixture(|f| {
        let world = empty_world();
        let one = f.ui(1);
        let two = f.ui(2);
        let four = f.ui(4);
        let mask = t_int_mask(f, vec![one, two, four]);
        let ty = f.u(mask);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result.atoms.len(), 8);
    });
}

#[test]
fn int_mask_of_widens_target_to_int_mask_set() {
    fixture(|f| {
        let world = empty_world();
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        let union_of_literals = f.u_many(vec![one, two]);
        let mask_of = t_int_mask_of(f, union_of_literals);
        let ty = f.u(mask_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        let mut got: Vec<i64> = result
            .atoms
            .iter()
            .filter_map(|atom| match atom {
                Atom::Int(IntAtom::Literal(value)) => Some(*value),
                _ => None,
            })
            .collect();
        got.sort_unstable();
        assert_eq!(got, vec![0, 1, 2, 3]);
    });
}

#[test]
fn int_mask_with_non_literal_operand_widens_to_mixed() {
    fixture(|f| {
        let world = empty_world();
        let one = f.ui(1);
        let mask = t_int_mask(f, vec![well_known::TYPE_INT, one]);
        let ty = f.u(mask);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_MIXED);
    });
}

#[test]
fn template_type_resolves_to_constraint() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Covariant)]);
        world.with_template_bound("Box", "T", well_known::TYPE_INT);
        let box_atom = f.t_named("Box");
        let class_type = f.u(box_atom);
        let template_name = f.us("T");
        let template_type = t_template_type(f, class_type, template_name);
        let ty = f.u(template_type);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn template_type_unknown_passes_through() {
    fixture(|f| {
        let world = empty_world();
        let box_atom = f.t_named("Box");
        let class_type = f.u(box_atom);
        let template_name = f.us("T");
        let template_type = t_template_type(f, class_type, template_name);
        let derived = f.u(template_type);
        assert_eq!(expand::expand(derived, &world, &mut f.builder), derived);
    });
}

#[test]
fn template_type_reads_direct_binding_from_object() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);
        let object_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let object = f.u(object_atom);
        let box_atom = f.t_named("Box");
        let class_type = f.u(box_atom);
        let template_name = f.us("T");
        let template_type = t_template_type_of(f, object, class_type, template_name);
        let ty = f.u(template_type);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT, "$object: Box<int> binds T to int");
    });
}

#[test]
fn template_type_object_binding_beats_declared_upper_bound() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Covariant)]);
        world.with_template_bound("Box", "T", well_known::TYPE_SCALAR);
        let object_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let object = f.u(object_atom);
        let box_atom = f.t_named("Box");
        let class_type = f.u(box_atom);
        let template_name = f.us("T");
        let template_type = t_template_type_of(f, object, class_type, template_name);
        let ty = f.u(template_type);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT, "the concrete binding wins over the scalar upper bound");
    });
}

#[test]
fn template_type_reads_inherited_binding_from_object() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Container", &[("T", Variance::Invariant)]);
        world.with_extended("ArrayCollection", "Container", vec![well_known::TYPE_STRING]);
        let object_atom = f.t_named("ArrayCollection");
        let object = f.u(object_atom);
        let container_atom = f.t_named("Container");
        let class_type = f.u(container_atom);
        let template_name = f.us("T");
        let template_type = t_template_type_of(f, object, class_type, template_name);
        let ty = f.u(template_type);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(
            result,
            well_known::TYPE_STRING,
            "$object: ArrayCollection extends Container<string> binds T to string"
        );
    });
}

#[test]
fn template_type_falls_back_to_upper_bound_when_object_is_raw() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Covariant)]);
        world.with_template_bound("Box", "T", well_known::TYPE_INT);
        let object_atom = f.t_named("Box");
        let object = f.u(object_atom);
        let box_atom = f.t_named("Box");
        let class_type = f.u(box_atom);
        let template_name = f.us("T");
        let template_type = t_template_type_of(f, object, class_type, template_name);
        let ty = f.u(template_type);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT, "a raw Box object exposes no binding, so the upper bound stands in");
    });
}

#[test]
fn properties_of_passes_through_for_now() {
    fixture(|f| {
        let world = empty_world();
        let derived_atom =
            f.builder.derived(DerivedAtom::PropertiesOf { target: well_known::TYPE_OBJECT, visibility: None });
        let ty = f.u(derived_atom);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), ty);
    });
}

#[test]
fn new_passes_through_for_now() {
    fixture(|f| {
        let world = empty_world();
        let derived_atom = f.builder.derived(DerivedAtom::New(well_known::TYPE_MIXED));
        let ty = f.u(derived_atom);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), ty);
    });
}

#[test]
fn nested_derived_inside_alias_resolves() {
    fixture(|f| {
        let mut world = MockWorld::new();
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let list = f.u(list_atom);
        let value_of = t_value_of(f, list);
        let value_of_list = f.u(value_of);
        world.with_alias("Foo", "ElementType", value_of_list);
        let alias = t_alias_elem(f, "Foo", "ElementType");
        let alias_type = f.u(alias);
        assert_eq!(expand::expand(alias_type, &world, &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn key_of_object_shape_is_union_of_property_name_literals() {
    fixture(|f| {
        let world = empty_world();
        let shape_atom =
            f.t_object_shape(&[("name", well_known::TYPE_STRING, false), ("age", well_known::TYPE_INT, false)], true);
        let shape = f.u(shape_atom);
        let key_of = t_key_of(f, shape);
        let ty = f.u(key_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        let age = f.t_lit_string("age");
        let name = f.t_lit_string("name");
        let expected = f.u_many(vec![age, name]);
        assert_eq!(result, expected);
    });
}

#[test]
fn key_of_empty_object_shape_is_never() {
    fixture(|f| {
        let world = empty_world();
        let shape_atom = f.t_object_shape(&[], true);
        let shape = f.u(shape_atom);
        let key_of = t_key_of(f, shape);
        let ty = f.u(key_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_NEVER);
    });
}

#[test]
fn value_of_object_shape_is_union_of_property_types() {
    fixture(|f| {
        let world = empty_world();
        let shape_atom =
            f.t_object_shape(&[("name", well_known::TYPE_STRING, false), ("age", well_known::TYPE_INT, false)], true);
        let shape = f.u(shape_atom);
        let value_of = t_value_of(f, shape);
        let ty = f.u(value_of);
        let result = expand::expand(ty, &world, &mut f.builder);
        let int = f.t_int();
        let string = f.t_string();
        let expected = f.u_many(vec![int, string]);
        assert_eq!(result, expected);
    });
}

#[test]
fn index_access_object_shape_with_literal_key_returns_property_type() {
    fixture(|f| {
        let world = empty_world();
        let shape_atom =
            f.t_object_shape(&[("name", well_known::TYPE_STRING, false), ("age", well_known::TYPE_INT, false)], true);
        let shape = f.u(shape_atom);
        let key = f.us("name");
        let index_access = t_index_access(f, shape, key);
        let ty = f.u(index_access);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_STRING);
    });
}

#[test]
fn index_access_object_shape_with_unknown_literal_key_is_never() {
    fixture(|f| {
        let world = empty_world();
        let shape_atom = f.t_object_shape(&[("name", well_known::TYPE_STRING, false)], true);
        let shape = f.u(shape_atom);
        let key = f.us("missing");
        let index_access = t_index_access(f, shape, key);
        let ty = f.u(index_access);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_NEVER);
    });
}

#[test]
fn index_access_object_shape_with_non_literal_key_widens_to_value_union() {
    fixture(|f| {
        let world = empty_world();
        let shape_atom =
            f.t_object_shape(&[("name", well_known::TYPE_STRING, false), ("age", well_known::TYPE_INT, false)], true);
        let shape = f.u(shape_atom);
        let index_access = t_index_access(f, shape, well_known::TYPE_STRING);
        let ty = f.u(index_access);
        let result = expand::expand(ty, &world, &mut f.builder);
        let int = f.t_int();
        let string = f.t_string();
        let expected = f.u_many(vec![int, string]);
        assert_eq!(result, expected);
    });
}
