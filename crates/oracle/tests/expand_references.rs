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
        let symbols = empty_symbol_table(f.arena);
        let reference = t_reference(f, "Foo");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        let foo = f.t_named("Foo");
        let expected = f.u(foo);
        assert_eq!(result, expected);
    });
}

#[test]
fn symbol_reference_with_type_args_resolves_to_generic_object() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let reference = t_reference_generic(f, "Box", vec![well_known::TYPE_INT]);
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        let generic = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(generic);
        assert_eq!(result, expected);
    });
}

#[test]
fn nested_alias_inside_reference_type_args_expands() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @type Id = int */ class Foo {}");
        let alias = t_alias_via_symbols(f, "Foo", "Id");
        let alias_type = f.u(alias);
        let reference = t_reference_generic(f, "Box", vec![alias_type]);
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        let generic = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(generic);
        assert_eq!(result, expected);
    });
}

#[test]
fn intersected_reference_resolves_to_intersected_object() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let bar = f.t_named("Bar");
        let reference = t_reference_intersected(f, "Foo", &[bar]);
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        let foo_and_bar_atom = f.t_named_intersected("Foo", &[bar]);
        let foo_and_bar = f.u(foo_and_bar_atom);
        assert_eq!(result, foo_and_bar);
    });
}

#[test]
fn member_reference_resolves_to_constant_type() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Status { const int ACTIVE = 0; }");
        let reference = t_member_ref(f, "Status", "ACTIVE");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn unknown_member_reference_passes_through() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let reference = t_member_ref(f, "Status", "UNKNOWN");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &symbols, &mut f.builder), ty);
    });
}

#[test]
fn member_reference_inherits_from_ancestor() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Base { const string ID = ''; } class Sub extends Base {}");
        let reference = t_member_ref(f, "Sub", "ID");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &symbols, &mut f.builder), well_known::TYPE_STRING);
    });
}

#[test]
fn member_reference_wildcard_unions_all_constants() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php class Status {
                const int ACTIVE = 0;
                const string LABEL = '';
            }",
        );
        let reference = t_member_ref_wildcard(f, "Status");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn member_reference_wildcard_single_constant_resolves_to_that_type() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Status { const int ACTIVE = 0; }");
        let reference = t_member_ref_wildcard(f, "Status");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn member_reference_wildcard_unknown_class_passes_through() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let reference = t_member_ref_wildcard(f, "Status");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &symbols, &mut f.builder), ty);
    });
}

#[test]
fn member_reference_prefix_selects_matching_constants() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php class Status {
                const int STATUS_ON = 0;
                const string STATUS_OFF = '';
                const float OTHER = 0.0;
            }",
        );
        let reference = t_member_ref_prefix(f, "Status", "STATUS_");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn member_reference_suffix_selects_matching_constants() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php class Flags {
                const int READ_FLAG = 0;
                const string WRITE_FLAG = '';
                const float VERSION = 0.0;
            }",
        );
        let reference = t_member_ref_suffix(f, "Flags", "_FLAG");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn member_reference_contains_selects_matching_constants() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php class Config {
                const int DB_HOST_NAME = 0;
                const string CACHE_HOST_PORT = '';
                const float TIMEOUT = 0.0;
            }",
        );
        let reference = t_member_ref_contains(f, "Config", "HOST");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn member_reference_prefix_no_match_passes_through() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Status { const int ACTIVE = 0; }");
        let reference = t_member_ref_prefix(f, "Status", "MISSING_");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &symbols, &mut f.builder), ty);
    });
}

#[test]
fn member_reference_wildcard_unions_enum_cases() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Suit { case Hearts; case Spades; }");
        let reference = t_member_ref_wildcard(f, "Suit");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        let hearts = f.t_enum_case("Suit", "Hearts");
        let spades = f.t_enum_case("Suit", "Spades");
        let expected = f.u_many(vec![hearts, spades]);
        assert_eq!(result, expected);
    });
}

#[test]
fn member_reference_prefix_selects_matching_enum_cases() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Event { case OnOpen; case OnClose; case Reset; }");
        let reference = t_member_ref_prefix(f, "Event", "On");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        let on_open = f.t_enum_case("Event", "OnOpen");
        let on_close = f.t_enum_case("Event", "OnClose");
        let expected = f.u_many(vec![on_open, on_close]);
        assert_eq!(result, expected);
    });
}

#[test]
fn member_reference_wildcard_enum_unions_cases_and_constants() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Mode { case Fast; const int DEFAULT_TIMEOUT = 0; }");
        let reference = t_member_ref_wildcard(f, "Mode");
        let ty = f.u(reference);
        let result = expand::expand(ty, &symbols, &mut f.builder);
        let fast = f.t_enum_case("Mode", "Fast");
        let expected = f.u_many(vec![fast, well_known::INT]);
        assert_eq!(result, expected);
    });
}

#[test]
fn global_reference_resolves_to_constant_type() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @var int */ const PHP_INT_MAX = 0;");
        let reference = t_global_ref(f, "PHP_INT_MAX");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &symbols, &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn unknown_global_reference_passes_through() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let reference = t_global_ref(f, "UNKNOWN");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &symbols, &mut f.builder), ty);
    });
}

#[test]
fn member_reference_to_union_constant_flat_merges() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo { const int|string MIXED = 0; }");
        let reference = t_member_ref(f, "Foo", "MIXED");
        let ty = f.u(reference);
        assert_eq!(expand::expand(ty, &symbols, &mut f.builder), well_known::TYPE_INT_OR_STRING);
    });
}

#[test]
fn reference_inside_list_expands() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let reference = t_reference(f, "Foo");
        let inner = f.u(reference);
        let list_atom = f.t_list(inner, false);
        let list = f.u(list_atom);
        let result = expand::expand(list, &symbols, &mut f.builder);
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
        let symbols = symbol_table(f.arena, "<?php class Foo { const string K = ''; }");
        let reference = t_member_ref(f, "Foo", "K");
        let key = f.u(reference);
        let iterable_atom = f.t_iterable(key, well_known::TYPE_INT);
        let iterable = f.u(iterable_atom);
        let result = expand::expand(iterable, &symbols, &mut f.builder);
        let expected_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn chained_alias_then_reference_resolves() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Bar {} /** @type Id = Bar */ class Foo {}");
        let alias = t_alias_via_symbols(f, "Foo", "Id");
        let alias_type = f.u(alias);
        let bar = f.t_named("Bar");
        let expected = f.u(bar);
        assert_eq!(expand::expand(alias_type, &symbols, &mut f.builder), expected);
    });
}

fn t_alias_via_symbols<'arena>(f: &mut Fixture<'_, 'arena>, class: &str, alias: &str) -> Atom<'arena> {
    let class_name = f.name(class);
    let alias_name = f.builder.intern(alias.as_bytes());
    f.builder.alias(AliasAtom { class_name, alias_name })
}
