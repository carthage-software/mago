mod common;

use common::*;

use mago_allocator::LocalArena;
use mago_oracle::symbol::SymbolTable;

use mago_oracle::ty::Type;
use mago_oracle::ty::lattice;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::is_uninhabited;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::ty::well_known;

fn create_sealed_symbols<'arena>(f: &Fixture<'_, 'arena>) -> SymbolTable<'arena, LocalArena> {
    symbol_table(
        f.arena,
        "<?php
/** @inheritors Error|Exception */
interface Throwable {}
class Error implements Throwable {}
class Exception implements Throwable {}
class RuntimeException extends Exception {}
class LogicException extends Exception {}",
    )
}

fn refines_of<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> bool {
    let mut report = LatticeReport::new();
    lattice::refines(a, b, symbols, LatticeOptions::default(), &mut report, &mut f.builder)
}

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

#[test]
fn throwable_minus_exception_refines_error() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let error = f.t_named("Error");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);

        let input = f.u(throwable_minus_exception);
        let container = f.u(error);
        assert!(refines_of(f, input, container, &symbols));
    });
}

#[test]
fn error_refines_throwable_minus_exception() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let error = f.t_named("Error");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);

        let input = f.u(error);
        let container = f.u(throwable_minus_exception);
        assert!(refines_of(f, input, container, &symbols));
    });
}

#[test]
fn throwable_refines_error_or_exception_union() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let union = f.u_many(vec![error, exception]);
        let throwable_type = f.u(throwable);
        assert!(refines_of(f, throwable_type, union, &symbols));
    });
}

#[test]
fn error_or_exception_union_refines_throwable() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let union = f.u_many(vec![error, exception]);
        let throwable_type = f.u(throwable);
        assert!(refines_of(f, union, throwable_type, &symbols));
    });
}

#[test]
fn throwable_with_full_negations_is_uninhabited() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let error_type = f.u(error);
        let negated_error = f.builder.negated(error_type);
        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_without_inheritors = f.builder.intersected(throwable, &[negated_error, negated_exception]);

        assert!(is_uninhabited(throwable_without_inheritors, &symbols, &mut f.builder));
    });
}

#[test]
fn throwable_with_full_negations_meet_anything_is_never() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let error_type = f.u(error);
        let negated_error = f.builder.negated(error_type);
        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_without_inheritors = f.builder.intersected(throwable, &[negated_error, negated_exception]);

        let input = f.u(throwable_without_inheritors);
        assert_eq!(meet_of(f, input, well_known::TYPE_MIXED, &symbols), well_known::TYPE_NEVER);
    });
}

#[test]
fn subtract_throwable_by_error_or_exception_is_never() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let union = f.u_many(vec![error, exception]);
        let throwable_type = f.u(throwable);
        assert_eq!(subtract_of(f, throwable_type, union, &symbols), well_known::TYPE_NEVER);
    });
}

fn create_traversable_symbols<'arena>(f: &Fixture<'_, 'arena>) -> SymbolTable<'arena, LocalArena> {
    symbol_table(
        f.arena,
        "<?php
/** @inheritors Iterator|IteratorAggregate */
interface Traversable {}
interface Iterator extends Traversable {}
interface IteratorAggregate extends Traversable {}",
    )
}

#[test]
fn traversable_refines_iterator_or_iterator_aggregate() {
    fixture(|f| {
        let symbols = create_traversable_symbols(f);
        let traversable = f.t_named("Traversable");
        let iterator = f.t_named("Iterator");
        let iterator_aggregate = f.t_named("IteratorAggregate");

        let union = f.u_many(vec![iterator, iterator_aggregate]);
        let traversable_type = f.u(traversable);
        assert!(refines_of(f, traversable_type, union, &symbols));
    });
}

#[test]
fn traversable_with_full_negations_is_uninhabited() {
    fixture(|f| {
        let symbols = create_traversable_symbols(f);
        let traversable = f.t_named("Traversable");
        let iterator = f.t_named("Iterator");
        let iterator_aggregate = f.t_named("IteratorAggregate");

        let iterator_type = f.u(iterator);
        let negated_iterator = f.builder.negated(iterator_type);
        let iterator_aggregate_type = f.u(iterator_aggregate);
        let negated_iterator_aggregate = f.builder.negated(iterator_aggregate_type);
        let traversable_without_inheritors =
            f.builder.intersected(traversable, &[negated_iterator, negated_iterator_aggregate]);

        assert!(is_uninhabited(traversable_without_inheritors, &symbols, &mut f.builder));
    });
}

#[test]
fn partial_cover_does_not_collapse_when_residual_has_multiple() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);

        assert!(
            !is_uninhabited(throwable_minus_exception, &symbols, &mut f.builder),
            "Throwable & !Exception is not uninhabited: Error survives"
        );
    });
}

#[test]
fn transitive_negation_via_descendant() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let runtime_exception = f.t_named("RuntimeException");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);
        let conjoined = f.builder.intersected(throwable_minus_exception, &[runtime_exception]);

        assert!(
            is_uninhabited(conjoined, &symbols, &mut f.builder),
            "RuntimeException refines Exception, so !Exception excludes it"
        );
    });
}

#[test]
fn transitive_sealing_collapses_to_never() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @inheritors Bar|Baz */
class Foo {}
/** @inheritors Bar1|Bar2 */
class Bar extends Foo {}
class Baz extends Foo {}
class Bar1 extends Bar {}
class Bar2 extends Bar {}",
        );

        let foo = f.t_named("Foo");
        let bar1 = f.t_named("Bar1");
        let bar2 = f.t_named("Bar2");
        let baz = f.t_named("Baz");

        let bar1_type = f.u(bar1);
        let negated_bar1 = f.builder.negated(bar1_type);
        let bar2_type = f.u(bar2);
        let negated_bar2 = f.builder.negated(bar2_type);
        let baz_type = f.u(baz);
        let negated_baz = f.builder.negated(baz_type);
        let covered = f.builder.intersected(foo, &[negated_bar1, negated_bar2, negated_baz]);

        assert!(is_uninhabited(covered, &symbols, &mut f.builder));
    });
}

#[test]
fn unrelated_negation_does_not_affect_cover() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
        let throwable = f.t_named("Throwable");

        let negated_int = f.builder.negated(well_known::TYPE_INT);
        let throwable_without_int = f.builder.intersected(throwable, &[negated_int]);

        assert!(
            !is_uninhabited(throwable_without_int, &symbols, &mut f.builder),
            "int is not in the sealed cover, so coverage is unaffected"
        );
    });
}

#[test]
fn non_class_head_skips_sealed_logic() {
    fixture(|f| {
        let symbols = create_sealed_symbols(f);
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

        assert!(
            !is_uninhabited(throwable_minus_exception, &symbols, &mut f.builder),
            "with an empty symbol table, Throwable is not sealed"
        );
    });
}

#[test]
fn final_class_with_negated_self_is_being_uninhabited_by_existing_rules() {
    fixture(|f| {
        let _symbols = symbol_table(f.arena, "<?php final class Final {}");
        let final_class = f.t_named("Final");
        let final_type = f.u(final_class);
        let negated_final = f.builder.negated(final_type);
        let contradiction = f.builder.intersected(final_class, &[negated_final]);
        assert_eq!(contradiction, well_known::NEVER, "self-negation is always empty via intersected construction");
    });
}
