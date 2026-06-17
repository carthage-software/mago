mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::alias::AliasAtom;
use mago_oracle::ty::atom::payload::reference::GlobalReferenceAtom;
use mago_oracle::ty::atom::payload::reference::MemberReferenceAtom;
use mago_oracle::ty::atom::payload::reference::NameSelector;
use mago_oracle::ty::atom::payload::reference::SymbolReferenceAtom;
use mago_oracle::ty::expand;
use mago_oracle::ty::well_known;

fn t_reference<'arena>(f: &mut Fixture<'_, 'arena>, name: &str) -> Atom<'arena> {
    let name = f.name(name);
    f.builder.reference(SymbolReferenceAtom { name, type_arguments: None })
}

fn t_reference_generic<'arena>(f: &mut Fixture<'_, 'arena>, name: &str, args: Vec<Type<'arena>>) -> Atom<'arena> {
    let name = f.name(name);
    let type_arguments = Some(f.builder.types(&args));
    f.builder.reference(SymbolReferenceAtom { name, type_arguments })
}

fn t_reference_intersected<'arena>(
    f: &mut Fixture<'_, 'arena>,
    name: &str,
    conjuncts: &[Atom<'arena>],
) -> Atom<'arena> {
    let head = t_reference(f, name);
    f.builder.intersected(head, conjuncts)
}

fn t_member_ref<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, member: &str) -> Atom<'arena> {
    let class_like_name = f.name(class);
    let member = f.builder.intern(member.as_bytes());
    f.builder.member_reference(MemberReferenceAtom { class_like_name, selector: NameSelector::Identifier(member) })
}

fn t_member_ref_wildcard<'arena>(f: &mut Fixture<'_, 'arena>, class: &str) -> Atom<'arena> {
    let class_like_name = f.name(class);
    f.builder.member_reference(MemberReferenceAtom { class_like_name, selector: NameSelector::Wildcard })
}

fn t_member_ref_prefix<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, prefix: &str) -> Atom<'arena> {
    let class_like_name = f.name(class);
    let prefix = f.builder.intern(prefix.as_bytes());
    f.builder.member_reference(MemberReferenceAtom { class_like_name, selector: NameSelector::StartsWith(prefix) })
}

fn t_member_ref_suffix<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, suffix: &str) -> Atom<'arena> {
    let class_like_name = f.name(class);
    let suffix = f.builder.intern(suffix.as_bytes());
    f.builder.member_reference(MemberReferenceAtom { class_like_name, selector: NameSelector::EndsWith(suffix) })
}

fn t_member_ref_contains<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, needle: &str) -> Atom<'arena> {
    let class_like_name = f.name(class);
    let needle = f.builder.intern(needle.as_bytes());
    f.builder.member_reference(MemberReferenceAtom { class_like_name, selector: NameSelector::Contains(needle) })
}

fn t_global_ref<'arena>(f: &mut Fixture<'_, 'arena>, name: &str) -> Atom<'arena> {
    let name = f.builder.intern(name.as_bytes());
    f.builder.global_reference(GlobalReferenceAtom { selector: NameSelector::Identifier(name) })
}

#[test]
fn symbol_reference_resolves_to_named_object() {
    fixture(|f| {
        let world = empty_world();
        let reference = t_reference(f, "Foo");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        let foo = f.t_named("Foo");
        let expected = f.u(foo);
        assert_eq!(result, expected);
    });
}

#[test]
fn symbol_reference_with_type_args_resolves_to_generic_object() {
    fixture(|f| {
        let world = empty_world();
        let reference = t_reference_generic(f, "Box", vec![well_known::TYPE_INT]);
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        let generic = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(generic);
        assert_eq!(result, expected);
    });
}

#[test]
fn nested_alias_inside_reference_type_args_expands() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_alias("Foo", "Id", well_known::TYPE_INT);
        let alias = t_alias_via_world(f, "Foo", "Id");
        let alias_type = f.u(alias);
        let reference = t_reference_generic(f, "Box", vec![alias_type]);
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        let generic = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(generic);
        assert_eq!(result, expected);
    });
}

#[test]
fn intersected_reference_resolves_to_intersected_object() {
    fixture(|f| {
        let world = empty_world();
        let bar = f.t_named("Bar");
        let reference = t_reference_intersected(f, "Foo", &[bar]);
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        let foo_and_bar_atom = f.t_named_intersected("Foo", &[bar]);
        let foo_and_bar = f.u(foo_and_bar_atom);
        assert_eq!(result, foo_and_bar);
    });
}

#[test]
fn member_reference_resolves_to_constant_type() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Status", "ACTIVE", well_known::TYPE_INT);
        let reference = t_member_ref(f, "Status", "ACTIVE");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn unknown_member_reference_passes_through() {
    fixture(|f| {
        let world = empty_world();
        let reference = t_member_ref(f, "Status", "UNKNOWN");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), ty);
    });
}

#[test]
fn member_reference_inherits_from_ancestor() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Base", "ID", well_known::TYPE_STRING);
        world.add_edge("Sub", "Base");
        let reference = t_member_ref(f, "Sub", "ID");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), well_known::TYPE_STRING);
    });
}

#[test]
fn member_reference_wildcard_unions_all_constants() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Status", "ACTIVE", well_known::TYPE_INT);
        world.with_class_constant(&mut f.builder, "Status", "LABEL", well_known::TYPE_STRING);
        let reference = t_member_ref_wildcard(f, "Status");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn member_reference_wildcard_single_constant_resolves_to_that_type() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Status", "ACTIVE", well_known::TYPE_INT);
        let reference = t_member_ref_wildcard(f, "Status");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn member_reference_wildcard_unknown_class_passes_through() {
    fixture(|f| {
        let world = empty_world();
        let reference = t_member_ref_wildcard(f, "Status");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), ty);
    });
}

#[test]
fn member_reference_prefix_selects_matching_constants() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Status", "STATUS_ON", well_known::TYPE_INT);
        world.with_class_constant(&mut f.builder, "Status", "STATUS_OFF", well_known::TYPE_STRING);
        world.with_class_constant(&mut f.builder, "Status", "OTHER", well_known::TYPE_FLOAT);
        let reference = t_member_ref_prefix(f, "Status", "STATUS_");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn member_reference_suffix_selects_matching_constants() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Flags", "READ_FLAG", well_known::TYPE_INT);
        world.with_class_constant(&mut f.builder, "Flags", "WRITE_FLAG", well_known::TYPE_STRING);
        world.with_class_constant(&mut f.builder, "Flags", "VERSION", well_known::TYPE_FLOAT);
        let reference = t_member_ref_suffix(f, "Flags", "_FLAG");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn member_reference_contains_selects_matching_constants() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Config", "DB_HOST_NAME", well_known::TYPE_INT);
        world.with_class_constant(&mut f.builder, "Config", "CACHE_HOST_PORT", well_known::TYPE_STRING);
        world.with_class_constant(&mut f.builder, "Config", "TIMEOUT", well_known::TYPE_FLOAT);
        let reference = t_member_ref_contains(f, "Config", "HOST");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn member_reference_prefix_no_match_passes_through() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Status", "ACTIVE", well_known::TYPE_INT);
        let reference = t_member_ref_prefix(f, "Status", "MISSING_");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), ty);
    });
}

#[test]
fn member_reference_wildcard_unions_enum_cases() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_pure_enum("Suit");
        world.with_enum_case(&mut f.builder, "Suit", "Hearts");
        world.with_enum_case(&mut f.builder, "Suit", "Spades");
        let reference = t_member_ref_wildcard(f, "Suit");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        let hearts = f.t_enum_case("Suit", "Hearts");
        let spades = f.t_enum_case("Suit", "Spades");
        let expected = f.u_many(vec![hearts, spades]);
        assert_eq!(result, expected);
    });
}

#[test]
fn member_reference_prefix_selects_matching_enum_cases() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_pure_enum("Event");
        world.with_enum_case(&mut f.builder, "Event", "OnOpen");
        world.with_enum_case(&mut f.builder, "Event", "OnClose");
        world.with_enum_case(&mut f.builder, "Event", "Reset");
        let reference = t_member_ref_prefix(f, "Event", "On");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        let on_open = f.t_enum_case("Event", "OnOpen");
        let on_close = f.t_enum_case("Event", "OnClose");
        let expected = f.u_many(vec![on_open, on_close]);
        assert_eq!(result, expected);
    });
}

#[test]
fn member_reference_wildcard_enum_unions_cases_and_constants() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_pure_enum("Mode");
        world.with_enum_case(&mut f.builder, "Mode", "Fast");
        world.with_class_constant(&mut f.builder, "Mode", "DEFAULT_TIMEOUT", well_known::TYPE_INT);
        let reference = t_member_ref_wildcard(f, "Mode");
        let ty = f.u(reference);
        let result = expand::expand(ty, &world, &mut f.builder);
        let fast = f.t_enum_case("Mode", "Fast");
        let expected = f.u_many(vec![fast, well_known::INT]);
        assert_eq!(result, expected);
    });
}

#[test]
fn global_reference_resolves_to_constant_type() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_global_constant("PHP_INT_MAX", well_known::TYPE_INT);
        let reference = t_global_ref(f, "PHP_INT_MAX");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn unknown_global_reference_passes_through() {
    fixture(|f| {
        let world = empty_world();
        let reference = t_global_ref(f, "UNKNOWN");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), ty);
    });
}

#[test]
fn member_reference_to_union_constant_flat_merges() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Foo", "MIXED", well_known::TYPE_INT_OR_STRING);
        let reference = t_member_ref(f, "Foo", "MIXED");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &world, &mut f.builder), well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn reference_inside_list_expands() {
    fixture(|f| {
        let world = empty_world();
        let reference = t_reference(f, "Foo");
        let inner = f.u(reference);
        let list_atom = f.t_list(inner, false);
        let list = f.u(list_atom);
        let result = expand::expand(list, &world, &mut f.builder);
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let expected_atom = f.t_list(foo_type, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn member_reference_inside_iterable_expands() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_class_constant(&mut f.builder, "Foo", "K", well_known::TYPE_STRING);
        let reference = t_member_ref(f, "Foo", "K");
        let key = f.u(reference);
        let iterable_atom = f.t_iterable(key, well_known::TYPE_INT);
        let iterable = f.u(iterable_atom);
        let result = expand::expand(iterable, &world, &mut f.builder);
        let expected_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn chained_alias_then_reference_resolves() {
    fixture(|f| {
        let mut world = MockWorld::new();
        let bar_reference = t_reference(f, "Bar");
        let bar_reference_type = f.u(bar_reference);
        world.with_alias("Foo", "Id", bar_reference_type);
        let alias = t_alias_via_world(f, "Foo", "Id");
        let alias_type = f.u(alias);
        let bar = f.t_named("Bar");
        let expected = f.u(bar);
        assert_eq!(expand::expand(alias_type, &world, &mut f.builder), expected);
    });
}

fn t_alias_via_world<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, alias: &str) -> Atom<'arena> {
    let class_name = f.name(class);
    let alias_name = f.builder.intern(alias.as_bytes());
    f.builder.alias(AliasAtom { class_name, alias_name })
}
