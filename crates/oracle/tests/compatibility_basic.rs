mod common;

use common::*;

use mago_allocator::LocalArena;
use mago_oracle::symbol::SymbolTable;

use mago_oracle::ty::Type;
use mago_oracle::ty::compatibility::runtime_compatible;
use mago_oracle::ty::compatibility::statically_compatible;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;

fn statically<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> bool {
    let mut r = LatticeReport::new();
    statically_compatible(a, b, symbols, LatticeOptions::default(), &mut r, &mut f.builder)
}

fn at_runtime<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> bool {
    let mut r = LatticeReport::new();
    runtime_compatible(a, b, symbols, LatticeOptions::default(), &mut r, &mut f.builder)
}

#[test]
fn primitives_int_and_string_are_incompatible_under_both() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let int = f.t_int();
        let a = f.u(int);
        let string = f.t_string();
        let b = f.u(string);
        assert!(!statically(f, a, b, &cb));
        assert!(!at_runtime(f, a, b, &cb));
    });
}

#[test]
fn array_key_and_string_compatible_under_both() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let array_key = f.t_array_key();
        let a = f.u(array_key);
        let string = f.t_string();
        let b = f.u(string);
        assert!(statically(f, a, b, &cb));
        assert!(at_runtime(f, a, b, &cb));
    });
}

#[test]
fn numeric_and_string_compatible_under_both() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let numeric = f.t_numeric();
        let a = f.u(numeric);
        let string = f.t_string();
        let b = f.u(string);
        assert!(statically(f, a, b, &cb));
        assert!(at_runtime(f, a, b, &cb));
    });
}

#[test]
fn never_and_anything_incompatible_under_both() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let never = f.never();
        let n = f.u(never);
        let int = f.t_int();
        let i = f.u(int);
        assert!(!statically(f, n, i, &cb));
        assert!(!at_runtime(f, n, i, &cb));
    });
}

#[test]
fn cell_int_and_cell_string_diverge_static_vs_runtime() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php /** @template T */ class Cell {}");

        let int = f.t_int();
        let int_ty = f.u(int);
        let cell_int_atom = f.t_generic_named("Cell", vec![int_ty]);
        let cell_int = f.u(cell_int_atom);
        let string = f.t_string();
        let string_ty = f.u(string);
        let cell_string_atom = f.t_generic_named("Cell", vec![string_ty]);
        let cell_string = f.u(cell_string_atom);

        assert!(!statically(f, cell_int, cell_string, &w));
        assert!(at_runtime(f, cell_int, cell_string, &w));
    });
}

#[test]
fn cell_int_and_box_string_unrelated_classes_incompatible() {
    fixture(|f| {
        let w =
            symbol_table(f.arena, "<?php /** @template T */ final class Cell {} /** @template T */ final class Box {}");

        let int = f.t_int();
        let int_ty = f.u(int);
        let cell_int_atom = f.t_generic_named("Cell", vec![int_ty]);
        let cell_int = f.u(cell_int_atom);
        let string = f.t_string();
        let string_ty = f.u(string);
        let box_string_atom = f.t_generic_named("Box", vec![string_ty]);
        let box_string = f.u(box_string_atom);

        assert!(!statically(f, cell_int, box_string, &w));
        assert!(!at_runtime(f, cell_int, box_string, &w));
    });
}

#[test]
fn intersection_runtime_compatible_with_each_conjunct() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php class Foo {} class Bar {}");

        let bar_atom = f.t_named("Bar");
        let foo_and_bar_atom = f.t_named_intersected("Foo", &[bar_atom]);
        let foo_and_bar = f.u(foo_and_bar_atom);
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let bar = f.u(bar_atom);

        assert!(at_runtime(f, foo_and_bar, foo, &w));
        assert!(at_runtime(f, foo_and_bar, bar, &w));
    });
}

#[test]
fn descendant_classes_compatible_under_both() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php class Animal {} class Dog extends Animal {}");

        let dog_atom = f.t_named("Dog");
        let dog = f.u(dog_atom);
        let animal_atom = f.t_named("Animal");
        let animal = f.u(animal_atom);

        assert!(statically(f, dog, animal, &w));
        assert!(at_runtime(f, dog, animal, &w));
    });
}

#[test]
fn object_any_runtime_compatible_with_any_named_class() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php class Foo {}");

        let object = f.t_object_any();
        let any = f.u(object);
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);

        assert!(at_runtime(f, any, foo, &w));
        assert!(at_runtime(f, foo, any, &w));
    });
}

#[test]
fn has_method_runtime_compatible_with_any_object() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php class Foo {}");

        let has_method = f.t_has_method("doStuff");
        let h = f.u(has_method);
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);

        assert!(at_runtime(f, h, foo, &w));
    });
}

#[test]
fn cross_family_object_vs_int_incompatible_at_runtime() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php class Foo {}");

        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let int_atom = f.t_int();
        let int = f.u(int_atom);
        assert!(!at_runtime(f, foo, int, &w));
        assert!(!at_runtime(f, int, foo, &w));
    });
}

#[test]
fn enum_and_named_class_unrelated_incompatible() {
    fixture(|f| {
        let w = symbol_table(f.arena, "<?php enum Status {} class Foo {}");

        let status_atom = f.t_enum("Status");
        let status = f.u(status_atom);
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);

        assert!(!statically(f, status, foo, &w));
        assert!(!at_runtime(f, status, foo, &w));
    });
}

#[test]
fn union_distribution_static_and_runtime() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        let int_only = f.u(int);

        assert!(statically(f, int_or_string, int_only, &cb));
        assert!(at_runtime(f, int_or_string, int_only, &cb));
    });
}
