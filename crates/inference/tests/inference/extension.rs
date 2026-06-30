use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::variable::Variable;
use mago_inference::extension::AssertionSink;
use mago_inference::extension::AssertionTiming;
use mago_inference::extension::ExtensionAssertion;
use mago_inference::extension::ExtensionContext;
use mago_inference::extension::ExtensionInference;
use mago_inference::extension::Extensions;
use mago_inference::extension::StdlibInference;
use mago_inference::extension::semantics::Bits32Extension;
use mago_inference::flow::Flow;
use mago_oracle::assertion::Assertion;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::INT;
use mago_oracle::ty::well_known::STRING;
use mago_oracle::var::Var;

use crate::harness::*;

#[test]
fn bits32_overflows_integer_arithmetic_to_float() {
    let test = Test::new();
    let extension = Bits32Extension;
    let inference: [&dyn ExtensionInference<LocalArena>; 1] = [&extension];

    let ir = test.infer_with("<?php", "<?php 2 ** 31;", Extensions { inference: &inference, assertion: &[] });
    assert_eq!(get_last_expression(ir).meta.to_string(), "float(2147483648)");
}

#[test]
fn bits32_keeps_in_range_results_as_int() {
    let test = Test::new();
    let extension = Bits32Extension;
    let inference: [&dyn ExtensionInference<LocalArena>; 1] = [&extension];

    let ir = test.infer_with("<?php", "<?php 2 ** 30;", Extensions { inference: &inference, assertion: &[] });
    assert_eq!(get_last_expression(ir).meta.to_string(), "int(1073741824)");
}

#[test]
fn bits32_out_of_range_literal_is_float() {
    let test = Test::new();
    let extension = Bits32Extension;
    let inference: [&dyn ExtensionInference<LocalArena>; 1] = [&extension];

    let ir = test.infer_with("<?php", "<?php 9999999999;", Extensions { inference: &inference, assertion: &[] });
    assert_eq!(get_last_expression(ir).meta.to_string(), "float(9999999999)");
}

#[test]
fn bits32_in_range_literal_stays_int() {
    let test = Test::new();
    let extension = Bits32Extension;
    let inference: [&dyn ExtensionInference<LocalArena>; 1] = [&extension];

    let ir = test.infer_with("<?php", "<?php 2147483647;", Extensions { inference: &inference, assertion: &[] });
    assert_eq!(get_last_expression(ir).meta.to_string(), "int(2147483647)");
}

#[test]
fn without_bits32_arithmetic_stays_64bit() {
    let test = Test::new();
    let ir = test.infer("<?php", "<?php 2 ** 31;");
    assert_eq!(get_last_expression(ir).meta.to_string(), "int(2147483648)");
}

/// Narrows `$flag` to `int` unconditionally wherever it is read.
struct AlwaysFlagIsInt;

impl<A: Arena> ExtensionAssertion<A> for AlwaysFlagIsInt {
    fn assertions<'ctx, 'source, 'arena>(
        &self,
        _context: &mut ExtensionContext<'ctx, 'source, 'arena, A>,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        out: &mut AssertionSink<'ctx, 'source, 'arena, A>,
    ) {
        if let ExpressionKind::Variable(Variable::Direct(direct)) = &expression.kind
            && direct.name == b"$flag"
        {
            out.push(Var::new(direct.name), Assertion::IsType(INT), AssertionTiming::Always);
        }
    }
}

#[test]
fn always_assertion_narrows_the_environment() {
    let test = Test::new();
    let extension = AlwaysFlagIsInt;
    let assertion: [&dyn ExtensionAssertion<LocalArena>; 1] = [&extension];

    let ir =
        test.infer_with("<?php", "<?php $flag; $flag === null;", Extensions { inference: &[], assertion: &assertion });
    assert_eq!(get_last_expression(ir).meta.to_string(), "false");
}

#[test]
fn without_the_assertion_the_comparison_is_unknown() {
    let test = Test::new();
    let ir = test.infer("<?php", "<?php $flag; $flag === null;");
    assert_eq!(get_last_expression(ir).meta.to_string(), "bool");
}

/// When `$gate` is the condition, narrows the unrelated `$payload` to `int` on
/// the truthy branch and `string` on the falsy branch.
struct GateNarrowsPayload;

impl<A: Arena> ExtensionAssertion<A> for GateNarrowsPayload {
    fn assertions<'ctx, 'source, 'arena>(
        &self,
        _context: &mut ExtensionContext<'ctx, 'source, 'arena, A>,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        out: &mut AssertionSink<'ctx, 'source, 'arena, A>,
    ) {
        if let ExpressionKind::Variable(Variable::Direct(direct)) = &expression.kind
            && direct.name == b"$gate"
        {
            let payload = Var::new(b"$payload");
            out.push(payload, Assertion::IsType(INT), AssertionTiming::WhenTrue);
            out.push(payload, Assertion::IsType(STRING), AssertionTiming::WhenFalse);
        }
    }
}

#[test]
fn conditional_assertions_narrow_each_branch() {
    let test = Test::new();
    let extension = GateNarrowsPayload;
    let assertion: [&dyn ExtensionAssertion<LocalArena>; 1] = [&extension];

    let ir = test.infer_with(
        "<?php",
        "<?php $gate ? $payload : $payload;",
        Extensions { inference: &[], assertion: &assertion },
    );
    assert_eq!(get_last_expression(ir).meta.to_string(), "int|string");
}

#[test]
fn stdlib_constant_folds_literal_arguments() {
    let test = Test::new();
    let extension = StdlibInference;
    let inference: [&dyn ExtensionInference<LocalArena>; 1] = [&extension];

    let cases = [
        ("<?php strlen('hello');", "int(5)"),
        ("<?php mb_strlen('héllo');", "int(5)"),
        ("<?php count([1, 2, 3]);", "int(3)"),
        ("<?php count([]);", "int(0)"),
        ("<?php strtoupper('aBc');", "string('ABC')"),
        ("<?php strtolower('aBc');", "string('abc')"),
        ("<?php ucfirst('abc');", "string('Abc')"),
        ("<?php ord('A');", "int(65)"),
        ("<?php chr(65);", "string('A')"),
        ("<?php abs(-7);", "int(7)"),
        ("<?php abs(-7.5);", "float(7.5)"),
        ("<?php strrev('abc');", "string('cba')"),
        ("<?php str_repeat('ab', 3);", "string('ababab')"),
        ("<?php str_contains('hello', 'ell');", "true"),
        ("<?php str_contains('hello', 'xyz');", "false"),
        ("<?php str_starts_with('hello', 'he');", "true"),
        ("<?php str_ends_with('hello', 'lo');", "true"),
        ("<?php dechex(255);", "string('ff')"),
        ("<?php decbin(5);", "string('101')"),
        ("<?php decoct(8);", "string('10')"),
        ("<?php bin2hex('AB');", "string('4142')"),
        ("<?php intdiv(7, 2);", "int(3)"),
    ];

    for (code, expected) in cases {
        let ir = test.infer_with("<?php", code, Extensions { inference: &inference, assertion: &[] });
        assert_eq!(get_last_expression(ir).meta.to_string(), expected, "for code: {code}");
    }
}

#[test]
fn stdlib_refines_non_literal_arguments() {
    let test = Test::new();
    let extension = StdlibInference;
    let inference: [&dyn ExtensionInference<LocalArena>; 1] = [&extension];

    let cases = [
        ("<?php /** @var string */ $s = ''; strlen($s);", "non-negative-int"),
        ("<?php /** @var array<int> */ $a = []; count($a);", "non-negative-int"),
    ];

    for (code, expected) in cases {
        let ir = test.infer_with("<?php", code, Extensions { inference: &inference, assertion: &[] });
        assert_eq!(get_last_expression(ir).meta.to_string(), expected, "for code: {code}");
    }
}

#[test]
fn stdlib_count_keeps_lower_bound_for_non_empty_shapes() {
    let test = Test::new();
    let extension = StdlibInference;
    let inference: [&dyn ExtensionInference<LocalArena>; 1] = [&extension];

    let cases = [
        ("<?php /** @var non-empty-list<int> */ $a = [1]; count($a);", "positive-int"),
        ("<?php /** @var non-empty-array<string, int> */ $a = []; count($a);", "positive-int"),
        ("<?php /** @var array{a: int, b?: string} */ $a = []; count($a);", "int<1, 2>"),
        ("<?php /** @var array{a: int} */ $a = []; count($a);", "int(1)"),
        ("<?php /** @var list<int> */ $a = []; count($a);", "non-negative-int"),
    ];

    for (code, expected) in cases {
        let ir = test.infer_with("<?php", code, Extensions { inference: &inference, assertion: &[] });
        assert_eq!(get_last_expression(ir).meta.to_string(), expected, "for code: {code}");
    }
}

#[test]
fn stdlib_abs_refines_int_ranges_and_floats() {
    let test = Test::new();
    let extension = StdlibInference;
    let inference: [&dyn ExtensionInference<LocalArena>; 1] = [&extension];

    let cases = [
        ("<?php /** @var int<-5, 3> */ $n = 0; abs($n);", "int<0, 5>"),
        ("<?php /** @var int<2, 10> */ $n = 2; abs($n);", "int<2, 10>"),
        ("<?php /** @var int */ $n = 0; abs($n);", "non-negative-int"),
        ("<?php /** @var float */ $f = 0.0; abs($f);", "float"),
    ];

    for (code, expected) in cases {
        let ir = test.infer_with("<?php", code, Extensions { inference: &inference, assertion: &[] });
        assert_eq!(get_last_expression(ir).meta.to_string(), expected, "for code: {code}");
    }
}
