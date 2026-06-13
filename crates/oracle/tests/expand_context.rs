mod common;

use common::*;

use mago_flags::U8Flags;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::alias::AliasAtom;
use mago_oracle::ty::atom::payload::conditional::ConditionalAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectFlag;
use mago_oracle::ty::expand;
use mago_oracle::ty::expand::ExpansionContext;
use mago_oracle::ty::well_known;

fn t_object_with_flags<'arena>(
    f: &mut Fixture<'_, 'arena>,
    name: &str,
    is_static: bool,
    is_this: bool,
) -> Atom<'arena> {
    let name = f.name(name);
    let mut flags = U8Flags::empty();
    flags.set_value(ObjectFlag::IsStatic, is_static);
    flags.set_value(ObjectFlag::IsThis, is_this);
    f.builder.object(ObjectAtom { name, type_arguments: None, flags })
}

fn t_conditional<'arena>(
    f: &mut Fixture<'_, 'arena>,
    subject: Type<'arena>,
    target: Type<'arena>,
    then: Type<'arena>,
    otherwise: Type<'arena>,
    negated: bool,
) -> Atom<'arena> {
    f.builder.conditional(ConditionalAtom { subject, target, then, otherwise, negated })
}

#[test]
fn self_keyword_substitutes_with_self_class() {
    fixture(|f| {
        let world = empty_world();
        let self_atom = f.t_named("self");
        let self_object = f.u(self_atom);
        let foo = f.name("Foo");
        let context = ExpansionContext::default().with_self_class(foo);
        let foo_atom = f.t_named("Foo");
        let expected = f.u(foo_atom);
        assert_eq!(expand::expand_with(self_object, &world, &context, &mut f.builder), expected);
    });
}

#[test]
fn static_keyword_substitutes_with_static_class() {
    fixture(|f| {
        let world = empty_world();
        let static_atom = f.t_named("static");
        let static_object = f.u(static_atom);
        let foo = f.name("Foo");
        let context = ExpansionContext::default().with_static_class(foo);
        let foo_atom = f.t_named("Foo");
        let expected = f.u(foo_atom);
        assert_eq!(expand::expand_with(static_object, &world, &context, &mut f.builder), expected);
    });
}

#[test]
fn parent_keyword_substitutes_with_parent_class() {
    fixture(|f| {
        let world = empty_world();
        let parent_atom = f.t_named("parent");
        let parent_object = f.u(parent_atom);
        let animal = f.name("Animal");
        let context = ExpansionContext::default().with_parent_class(animal);
        let animal_atom = f.t_named("Animal");
        let expected = f.u(animal_atom);
        assert_eq!(expand::expand_with(parent_object, &world, &context, &mut f.builder), expected);
    });
}

#[test]
fn is_static_flag_resolved_with_static_class() {
    fixture(|f| {
        let world = empty_world();
        let element = t_object_with_flags(f, "Foo", true, false);
        let ty = f.u(element);
        let sub = f.name("Sub");
        let context = ExpansionContext::default().with_static_class(sub);
        let result = expand::expand_with(ty, &world, &context, &mut f.builder);
        let sub_atom = f.t_named("Sub");
        let expected = f.u(sub_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn is_this_flag_resolved_with_static_class() {
    fixture(|f| {
        let world = empty_world();
        let element = t_object_with_flags(f, "Foo", false, true);
        let ty = f.u(element);
        let sub = f.name("Sub");
        let context = ExpansionContext::default().with_static_class(sub);
        let result = expand::expand_with(ty, &world, &context, &mut f.builder);
        let sub_atom = f.t_named("Sub");
        let expected = f.u(sub_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn keyword_without_context_passes_through() {
    fixture(|f| {
        let world = empty_world();
        let self_atom = f.t_named("self");
        let self_object = f.u(self_atom);
        let context = ExpansionContext::default();
        assert_eq!(expand::expand_with(self_object, &world, &context, &mut f.builder), self_object);
    });
}

#[test]
fn plain_named_object_unaffected_by_context() {
    fixture(|f| {
        let world = empty_world();
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let bar = f.name("Bar");
        let context = ExpansionContext::default().with_self_class(bar);
        assert_eq!(expand::expand_with(foo, &world, &context, &mut f.builder), foo);
    });
}

#[test]
fn keyword_inside_list_resolves() {
    fixture(|f| {
        let world = empty_world();
        let self_atom = f.t_named("self");
        let self_object = f.u(self_atom);
        let list_atom = f.t_list(self_object, false);
        let list = f.u(list_atom);
        let foo = f.name("Foo");
        let context = ExpansionContext::default().with_self_class(foo);
        let result = expand::expand_with(list, &world, &context, &mut f.builder);
        let foo_atom = f.t_named("Foo");
        let foo_type = f.u(foo_atom);
        let expected_atom = f.t_list(foo_type, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn conditional_passes_through_when_eval_off() {
    fixture(|f| {
        let world = empty_world();
        let conditional = t_conditional(
            f,
            well_known::TYPE_INT,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
            false,
        );
        let ty = f.u(conditional);
        let context = ExpansionContext::default();
        assert_eq!(expand::expand_with(ty, &world, &context, &mut f.builder), ty);
    });
}

#[test]
fn conditional_picks_then_branch_when_test_passes() {
    fixture(|f| {
        let world = empty_world();
        let conditional = t_conditional(
            f,
            well_known::TYPE_INT,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
            false,
        );
        let ty = f.u(conditional);
        let context = ExpansionContext::default().with_evaluate_conditional(true);
        assert_eq!(
            expand::expand_with(ty, &world, &context, &mut f.builder),
            well_known::TYPE_STRING,
            "(int <: int) ? string : float picks string"
        );
    });
}

#[test]
fn conditional_picks_otherwise_when_test_disjoint() {
    fixture(|f| {
        let world = empty_world();
        let conditional = t_conditional(
            f,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
            false,
        );
        let ty = f.u(conditional);
        let context = ExpansionContext::default().with_evaluate_conditional(true);
        assert_eq!(
            expand::expand_with(ty, &world, &context, &mut f.builder),
            well_known::TYPE_FLOAT,
            "int and string are disjoint so the otherwise branch wins"
        );
    });
}

#[test]
fn conditional_widens_to_union_when_undecidable() {
    fixture(|f| {
        let world = empty_world();
        let int = f.t_int();
        let string = f.t_string();
        let mixed_input = f.u_many(vec![int, string]);
        let conditional =
            t_conditional(f, mixed_input, well_known::TYPE_INT, well_known::TYPE_FLOAT, well_known::TYPE_BOOL, false);
        let ty = f.u(conditional);
        let context = ExpansionContext::default().with_evaluate_conditional(true);
        let result = expand::expand_with(ty, &world, &context, &mut f.builder);
        let expected = f.u_many(vec![well_known::FLOAT, well_known::BOOL]);
        assert_eq!(result, expected, "int|string neither refines int nor is disjoint from it");
    });
}

#[test]
fn negated_conditional_swaps_branches_on_pass() {
    fixture(|f| {
        let world = empty_world();
        let conditional = t_conditional(
            f,
            well_known::TYPE_INT,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
            true,
        );
        let ty = f.u(conditional);
        let context = ExpansionContext::default().with_evaluate_conditional(true);
        assert_eq!(
            expand::expand_with(ty, &world, &context, &mut f.builder),
            well_known::TYPE_FLOAT,
            "(int is not int) fails so the otherwise branch wins"
        );
    });
}

#[test]
fn negated_conditional_swaps_branches_on_disjoint() {
    fixture(|f| {
        let world = empty_world();
        let conditional = t_conditional(
            f,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
            true,
        );
        let ty = f.u(conditional);
        let context = ExpansionContext::default().with_evaluate_conditional(true);
        assert_eq!(
            expand::expand_with(ty, &world, &context, &mut f.builder),
            well_known::TYPE_STRING,
            "(int is not string) holds so the then branch wins"
        );
    });
}

#[test]
fn conditional_with_alias_inside_branch_resolves_after_pick() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Result", well_known::TYPE_STRING);
        let class_name = f.name("Foo");
        let alias_name = f.name("Result");
        let alias = f.builder.alias(AliasAtom { class_name, alias_name });
        let alias_type = f.u(alias);
        let conditional =
            t_conditional(f, well_known::TYPE_INT, well_known::TYPE_INT, alias_type, well_known::TYPE_FLOAT, false);
        let ty = f.u(conditional);
        let context = ExpansionContext::default().with_evaluate_conditional(true);
        assert_eq!(expand::expand_with(ty, &world, &context, &mut f.builder), well_known::TYPE_STRING);
    });
}

#[test]
fn expand_default_wrapper_uses_default_context() {
    fixture(|f| {
        let world = empty_world();
        let conditional = t_conditional(
            f,
            well_known::TYPE_INT,
            well_known::TYPE_INT,
            well_known::TYPE_STRING,
            well_known::TYPE_FLOAT,
            false,
        );
        let ty = f.u(conditional);
        assert_eq!(
            expand::expand(ty, &world, &mut f.builder),
            ty,
            "without an explicit context, conditionals do not evaluate"
        );
    });
}

#[test]
fn keyword_inside_generic_object_args_resolves() {
    fixture(|f| {
        let world = empty_world();
        let self_atom = f.t_named("self");
        let self_object = f.u(self_atom);
        let box_of_self_atom = f.t_generic_named("Box", vec![self_object]);
        let box_of_self = f.u(box_of_self_atom);
        let foo = f.name("Foo");
        let context = ExpansionContext::default().with_self_class(foo);
        let result = expand::expand_with(box_of_self, &world, &context, &mut f.builder);
        let foo_atom = f.t_named("Foo");
        let foo_type = f.u(foo_atom);
        let expected_atom = f.t_generic_named("Box", vec![foo_type]);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn keyword_in_conditional_branch_resolves() {
    fixture(|f| {
        let world = empty_world();
        let self_atom = f.t_named("self");
        let self_object = f.u(self_atom);
        let conditional =
            t_conditional(f, well_known::TYPE_INT, well_known::TYPE_INT, self_object, well_known::TYPE_FLOAT, false);
        let ty = f.u(conditional);
        let bar = f.name("Bar");
        let context = ExpansionContext::default().with_evaluate_conditional(true).with_self_class(bar);
        let bar_atom = f.t_named("Bar");
        let expected = f.u(bar_atom);
        assert_eq!(expand::expand_with(ty, &world, &context, &mut f.builder), expected);
    });
}
