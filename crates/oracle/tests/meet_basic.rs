mod common;

use common::*;

use mago_allocator::LocalArena;
use mago_oracle::symbol::SymbolTable;

use mago_flags::U8Flags;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::atom::payload::scalar::string::StringRefinementFlag;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::refines;
use mago_oracle::ty::meet;
use mago_oracle::ty::well_known;

fn meet_of<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> Type<'arena> {
    let mut report = LatticeReport::new();
    meet::compute(a, b, symbols, LatticeOptions::default(), &mut report, &mut f.builder)
}

fn refines_of<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> bool {
    let mut report = LatticeReport::new();
    refines(a, b, symbols, LatticeOptions::default(), &mut report, &mut f.builder)
}

#[track_caller]
fn assert_lower_bound<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) {
    let m = meet_of(f, a, b, symbols);
    assert!(refines_of(f, m, a, symbols), "meet({a:?}, {b:?}) = {m:?} does not refine {a:?}");
    assert!(refines_of(f, m, b, symbols), "meet({a:?}, {b:?}) = {m:?} does not refine {b:?}");
}

#[test]
fn reflexive_meet() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        assert_eq!(meet_of(f, well_known::TYPE_INT, well_known::TYPE_INT, &cb), well_known::TYPE_INT);
    });
}

#[test]
fn meet_with_mixed_yields_other() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        assert_eq!(meet_of(f, well_known::TYPE_MIXED, well_known::TYPE_INT, &cb), well_known::TYPE_INT);
        assert_eq!(meet_of(f, well_known::TYPE_INT, well_known::TYPE_MIXED, &cb), well_known::TYPE_INT);
    });
}

#[test]
fn meet_with_never_yields_never() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        assert_eq!(meet_of(f, well_known::TYPE_NEVER, well_known::TYPE_INT, &cb), well_known::TYPE_NEVER);
        assert_eq!(meet_of(f, well_known::TYPE_INT, well_known::TYPE_NEVER, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn subsumption_picks_more_specific_side() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let lit = f.ui(42);
        let int = well_known::TYPE_INT;
        assert_eq!(meet_of(f, lit, int, &cb), lit);
        assert_eq!(meet_of(f, int, lit, &cb), lit);
    });
}

#[test]
fn distinct_kinds_meet_to_never() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        assert_eq!(meet_of(f, well_known::TYPE_INT, well_known::TYPE_STRING, &cb), well_known::TYPE_NEVER);
        assert_eq!(meet_of(f, well_known::TYPE_INT, well_known::TYPE_NULL, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn overlapping_int_ranges_meet_to_intersection() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let range_a = f.t_int_range(0, 10);
        let a = f.u(range_a);
        let range_b = f.t_int_range(5, 15);
        let b = f.u(range_b);
        let m = meet_of(f, a, b, &cb);
        let expected_range = f.t_int_range(5, 10);
        let expected = f.u(expected_range);
        assert_eq!(m, expected);
        assert_lower_bound(f, a, b, &cb);
    });
}

#[test]
fn touching_int_ranges_collapse_to_literal() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let range_a = f.t_int_range(0, 10);
        let a = f.u(range_a);
        let range_b = f.t_int_range(10, 20);
        let b = f.u(range_b);
        let m = meet_of(f, a, b, &cb);
        let expected = f.ui(10);
        assert_eq!(m, expected);
    });
}

#[test]
fn disjoint_int_ranges_meet_to_never() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let range_a = f.t_int_range(0, 10);
        let a = f.u(range_a);
        let range_b = f.t_int_range(20, 30);
        let b = f.u(range_b);
        assert_eq!(meet_of(f, a, b, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn lit_int_in_range_meets_to_lit() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let lit = f.ui(5);
        let range = f.t_int_range(0, 10);
        let r = f.u(range);
        assert_eq!(meet_of(f, lit, r, &cb), lit);
    });
}

#[test]
fn lit_int_outside_range_meets_to_never() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let lit = f.ui(20);
        let range = f.t_int_range(0, 10);
        let r = f.u(range);
        assert_eq!(meet_of(f, lit, r, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn distinct_int_literals_meet_to_never() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let a = f.ui(1);
        let b = f.ui(2);
        assert_eq!(meet_of(f, a, b, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn open_lower_meets_open_upper_into_bounded_range() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let from_zero_atom = f.t_int_from(0);
        let from_zero = f.u(from_zero_atom);
        let to_ten_atom = f.t_int_to(10);
        let to_ten = f.u(to_ten_atom);
        let m = meet_of(f, from_zero, to_ten, &cb);
        let expected_range = f.t_int_range(0, 10);
        let expected = f.u(expected_range);
        assert_eq!(m, expected);
    });
}

#[test]
fn nominal_subsumption_meet_picks_descendant() {
    fixture(|f| {
        let cb = symbol_table(f.arena, "<?php class Animal {} class Dog extends Animal {}");
        let dog_atom = f.t_named("Dog");
        let dog = f.u(dog_atom);
        let animal_atom = f.t_named("Animal");
        let animal = f.u(animal_atom);
        assert_eq!(meet_of(f, dog, animal, &cb), dog);
        assert_eq!(meet_of(f, animal, dog, &cb), dog);
    });
}

#[test]
fn unrelated_interfaces_compose_intersection() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php interface Foo {} interface Bar {}");
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let bar_atom = f.t_named("Bar");
        let bar = f.u(bar_atom);
        let m = meet_of(f, foo, bar, &w);
        assert!(refines_of(f, m, foo, &w));
        assert!(refines_of(f, m, bar, &w));
        assert!(!m.is_never(), "a class can implement two unrelated interfaces, so Foo & Bar is inhabited");
        assert_lower_bound(f, foo, bar, &w);
    });
}

#[test]
fn unrelated_concrete_classes_meet_is_never() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php class Foo {} class Bar {}");
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let bar_atom = f.t_named("Bar");
        let bar = f.u(bar_atom);
        let m = meet_of(f, foo, bar, &w);
        assert!(m.is_never(), "PHP single inheritance gives two unrelated concrete classes no common instance");
    });
}

#[test]
fn union_meet_distributes() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        let int_or_string = f.u_many(vec![int, string]);
        let int_or_null = f.u_many(vec![int, null]);
        assert_eq!(meet_of(f, int_or_string, int_or_null, &cb), well_known::TYPE_INT);
    });
}

#[test]
fn union_meet_yields_union_when_multiple_pairs_survive() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        let m = meet_of(f, int_or_string, int_or_string, &cb);
        assert_eq!(m, int_or_string);
    });
}

#[test]
fn nullable_int_meet_int_drops_null() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let null = f.null();
        let int_atom = f.t_int();
        let nullable_int = f.u_many(vec![null, int_atom]);
        let int = well_known::TYPE_INT;
        assert_eq!(meet_of(f, nullable_int, int, &cb), int);
    });
}

#[test]
fn class_like_string_meet_string_picks_class_like_string() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let cls_atom = f.t_class_string();
        let cls = f.u(cls_atom);
        let s = well_known::TYPE_STRING;
        assert_eq!(meet_of(f, cls, s, &cb), cls);
    });
}

#[test]
fn non_empty_lowercase_meet_non_empty_uppercase_is_never() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let nel = f.builder.string(StringAtom {
            literal: StringLiteral::None,
            casing: StringCasing::Lowercase,
            flags: U8Flags::empty().with(StringRefinementFlag::NonEmpty),
        });
        let neu = f.builder.string(StringAtom {
            literal: StringLiteral::None,
            casing: StringCasing::Uppercase,
            flags: U8Flags::empty().with(StringRefinementFlag::NonEmpty),
        });
        let a = f.u(nel);
        let b = f.u(neu);

        assert_eq!(meet_of(f, a, b, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn plain_lowercase_meet_uppercase_is_empty_string() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let a = f.u(well_known::LOWERCASE_STRING);
        let b = f.u(well_known::UPPERCASE_STRING);
        let m = meet_of(f, a, b, &cb);
        let expected = f.us("");
        assert_eq!(m, expected);
    });
}

#[test]
fn intersected_partition_cover_with_shared_head() {
    fixture(|f| {
        let a = f.t_named("A");
        let d = f.t_named("D");
        let e = f.t_named("E");

        let a_d = f.builder.intersected(a, &[d]);
        let a_e = f.builder.intersected(a, &[e]);
        let a_d_e = f.builder.intersected(a, &[d, e]);

        let negand = f.u(a_e);
        let neg_a_e = f.builder.negated(negand);
        let s = f.builder.intersected(a_d, &[neg_a_e]);

        let union = f.u_many(vec![s, a_d_e]);

        let probe = f.u(a_d);
        let w = empty_symbol_table(f.arena);
        assert!(refines_of(f, probe, union, &w), "A&D should refine (A&D & !(A&E)) | (A&D&E)");
    });
}
