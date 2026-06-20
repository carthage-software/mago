mod common;

use common::*;

use std::fmt::Write as _;

use mago_allocator::LocalArena;
use mago_oracle::symbol::SymbolTable;

use mago_oracle::ty::Type;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::is_uninhabited;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
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

fn subtract_of<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> Type<'arena> {
    let mut report = LatticeReport::new();
    subtract::compute(a, b, symbols, LatticeOptions::default(), &mut report, &mut f.builder)
}

fn sealed_throwable_symbols<'arena>(f: &Fixture<'_, 'arena>) -> SymbolTable<'arena, LocalArena> {
    symbol_table(
        f.arena,
        "<?php
/** @inheritors Error|Exception */
interface Throwable {}
class Error implements Throwable {}
class Exception implements Throwable {}
class RuntimeException extends Exception {}",
    )
}

#[test]
fn subtract_throwable_by_exception_canonicalises_to_error() {
    fixture(|f| {
        let symbols = sealed_throwable_symbols(f);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let error = f.t_named("Error");

        let throwable_type = f.u(throwable);
        let exception_type = f.u(exception);
        let result = subtract_of(f, throwable_type, exception_type, &symbols);
        let expected = f.u(error);
        assert_eq!(result, expected);
    });
}

#[test]
fn subtract_throwable_by_error_canonicalises_to_exception() {
    fixture(|f| {
        let symbols = sealed_throwable_symbols(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let throwable_type = f.u(throwable);
        let error_type = f.u(error);
        let result = subtract_of(f, throwable_type, error_type, &symbols);
        let expected = f.u(exception);
        assert_eq!(result, expected);
    });
}

#[test]
fn meet_throwable_with_negated_exception_canonicalises_to_error() {
    fixture(|f| {
        let symbols = sealed_throwable_symbols(f);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let error = f.t_named("Error");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_without_exception = f.builder.intersected(throwable, &[negated_exception]);
        let input = f.u(throwable_without_exception);
        let error_type = f.u(error);
        let result = meet_of(f, input, error_type, &symbols);
        assert_eq!(result, error_type);
    });
}

#[test]
fn subtract_throwable_by_error_or_exception_is_never() {
    fixture(|f| {
        let symbols = sealed_throwable_symbols(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let union = f.u_many(vec![error, exception]);
        let throwable_type = f.u(throwable);
        assert_eq!(subtract_of(f, throwable_type, union, &symbols), well_known::TYPE_NEVER);
    });
}

#[test]
fn traversable_minus_iterator_minus_iterator_aggregate_is_never() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @inheritors Iterator|IteratorAggregate */
interface Traversable {}
interface Iterator extends Traversable {}
interface IteratorAggregate extends Traversable {}",
        );

        let traversable = f.t_named("Traversable");
        let iterator = f.t_named("Iterator");
        let iterator_aggregate = f.t_named("IteratorAggregate");

        let traversable_type = f.u(traversable);
        let iterator_type = f.u(iterator);
        let after_iterator = subtract_of(f, traversable_type, iterator_type, &symbols);
        let iterator_aggregate_type = f.u(iterator_aggregate);
        let final_result = subtract_of(f, after_iterator, iterator_aggregate_type, &symbols);
        assert_eq!(final_result, well_known::TYPE_NEVER);
    });
}

#[test]
fn partial_cover_does_not_collapse_when_residual_has_multiple() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @inheritors A|B|C */
class Foo {}
class A extends Foo {}
class B extends Foo {}
class C extends Foo {}",
        );

        let foo = f.t_named("Foo");
        let inheritor = f.t_named("A");
        let inheritor_type = f.u(inheritor);
        let negated_inheritor = f.builder.negated(inheritor_type);
        let foo_without_inheritor = f.builder.intersected(foo, &[negated_inheritor]);

        assert!(!is_uninhabited(foo_without_inheritor, &symbols, &mut f.builder));
    });
}

#[test]
fn cycle_with_direct_coverage_collapses() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @inheritors B */
class A extends B {}
/** @inheritors A */
class B extends A {}",
        );

        let head = f.t_named("A");
        let inheritor = f.t_named("B");
        let inheritor_type = f.u(inheritor);
        let negated_inheritor = f.builder.negated(inheritor_type);
        let covered = f.builder.intersected(head, &[negated_inheritor]);

        assert!(is_uninhabited(covered, &symbols, &mut f.builder));
    });
}

#[test]
fn cycle_without_direct_coverage_terminates() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @inheritors B */
class A extends B {}
/** @inheritors A */
class B extends A {}",
        );

        let head = f.t_named("A");
        let unrelated = f.t_named("Unrelated");
        let unrelated_type = f.u(unrelated);
        let negated_unrelated = f.builder.negated(unrelated_type);
        let residual = f.builder.intersected(head, &[negated_unrelated]);

        assert!(!is_uninhabited(residual, &symbols, &mut f.builder));
    });
}

#[test]
fn depth_cap_does_not_overflow() {
    fixture(|f| {
        let mut source = String::from("<?php /** @inheritors S1 */ class S0 {}");
        for index in 1..=20 {
            let _ = write!(source, " class S{index} extends S{} {{}}", index - 1);
        }
        let symbols = symbol_table(f.arena, &source);

        let root = f.t_named("S0");
        let deepest = f.t_named("S21");
        let deepest_type = f.u(deepest);
        let negated_deepest = f.builder.negated(deepest_type);
        let residual = f.builder.intersected(root, &[negated_deepest]);

        assert!(!is_uninhabited(residual, &symbols, &mut f.builder));
    });
}

#[test]
fn final_class_with_negated_self_is_never() {
    fixture(|f| {
        let final_class = f.t_named("Final");
        let final_type = f.u(final_class);
        let negated_final = f.builder.negated(final_type);
        let contradiction = f.builder.intersected(final_class, &[negated_final]);
        assert_eq!(contradiction, well_known::NEVER);
    });
}

#[test]
fn non_class_head_skips_sealed_logic() {
    fixture(|f| {
        let symbols = sealed_throwable_symbols(f);
        let exception = f.t_named("Exception");
        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let int_without_exception = f.builder.intersected(well_known::INT, &[negated_exception]);
        assert!(!is_uninhabited(int_without_exception, &symbols, &mut f.builder));
    });
}

#[test]
fn empty_symbols_returns_no_sealed_inheritors() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);
        assert!(!is_uninhabited(throwable_minus_exception, &symbols, &mut f.builder));
    });
}
