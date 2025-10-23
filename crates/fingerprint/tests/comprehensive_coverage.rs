use bumpalo::Bump;
use mago_database::file::File;
use mago_fingerprint::FingerprintOptions;
use mago_fingerprint::Fingerprintable;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

const COMPREHENSIVE_PHP: &str = include_str!("fixtures/comprehensive.php");

fn fingerprint_code(code: &'static str) -> u64 {
    let arena = Bump::new();
    let file = File::ephemeral("test.php".into(), code.into());
    let (program, _) = parse_file(&arena, &file);
    let resolved_names = NameResolver::new(&arena).resolve(program);
    let options = FingerprintOptions::default();
    program.fingerprint(&resolved_names, &options)
}

#[test]
fn test_comprehensive_file_parses() {
    let arena = Bump::new();
    let file = File::ephemeral("comprehensive.php".into(), COMPREHENSIVE_PHP.into());
    let (program, errors) = parse_file(&arena, &file);

    assert!(errors.is_none(), "Parse should succeed without errors");

    let resolved_names = NameResolver::new(&arena).resolve(program);
    let options = FingerprintOptions::default();
    let _fingerprint = program.fingerprint(&resolved_names, &options);
}

#[test]
fn test_comprehensive_file_consistent_fingerprint() {
    let fp1 = fingerprint_code(COMPREHENSIVE_PHP);
    let fp2 = fingerprint_code(COMPREHENSIVE_PHP);

    assert_eq!(fp1, fp2, "Same code should produce same fingerprint");
}

#[test]
fn test_whitespace_normalized() {
    let code1 = "<?php $a=1;$b=2;";
    let code2 = "<?php $a = 1; $b = 2;";
    let code3 = "<?php\n$a  =  1;\n$b  =  2;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);
    let fp3 = fingerprint_code(code3);

    assert_eq!(fp1, fp2, "Whitespace should be normalized");
    assert_eq!(fp2, fp3, "Whitespace should be normalized");
}

#[test]
fn test_variable_name_change_detected() {
    let code1 = "<?php $foo = 1;";
    let code2 = "<?php $bar = 1;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Different variable names should produce different fingerprints");
}

#[test]
fn test_function_name_change_detected() {
    let code1 = "<?php function foo() {}";
    let code2 = "<?php function bar() {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Different function names should produce different fingerprints");
}

#[test]
fn test_class_name_change_detected() {
    let code1 = "<?php class Foo {}";
    let code2 = "<?php class Bar {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Different class names should produce different fingerprints");
}

#[test]
fn test_literal_value_change_detected() {
    let code1 = "<?php $a = 123;";
    let code2 = "<?php $a = 456;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Different literal values should produce different fingerprints");
}

#[test]
fn test_string_value_change_detected() {
    let code1 = "<?php $a = 'hello';";
    let code2 = "<?php $a = 'world';";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Different string values should produce different fingerprints");
}

#[test]
fn test_operator_change_detected() {
    let code1 = "<?php $a = 1 + 2;";
    let code2 = "<?php $a = 1 - 2;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Different operators should produce different fingerprints");
}

#[test]
fn test_property_addition_detected() {
    let code1 = "<?php class Foo { public $a; }";
    let code2 = "<?php class Foo { public $a; public $b; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Property addition should produce different fingerprint");
}

#[test]
fn test_method_addition_detected() {
    let code1 = "<?php class Foo { public function a() {} }";
    let code2 = "<?php class Foo { public function a() {} public function b() {} }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Method addition should produce different fingerprint");
}

#[test]
fn test_parameter_addition_detected() {
    let code1 = "<?php function foo($a) {}";
    let code2 = "<?php function foo($a, $b) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Parameter addition should produce different fingerprint");
}

#[test]
fn test_type_hint_change_detected() {
    let code1 = "<?php function foo(int $a) {}";
    let code2 = "<?php function foo(string $a) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Type hint change should produce different fingerprint");
}

#[test]
fn test_return_type_change_detected() {
    let code1 = "<?php function foo(): int { return 1; }";
    let code2 = "<?php function foo(): string { return '1'; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Return type change should produce different fingerprint");
}

#[test]
fn test_visibility_change_detected() {
    let code1 = "<?php class Foo { public $a; }";
    let code2 = "<?php class Foo { private $a; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Visibility change should produce different fingerprint");
}

#[test]
fn test_modifier_addition_detected() {
    let code1 = "<?php class Foo { function bar() {} }";
    let code2 = "<?php class Foo { static function bar() {} }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Modifier addition should produce different fingerprint");
}

#[test]
fn test_attribute_addition_detected() {
    let code1 = "<?php function foo() {}";
    let code2 = "<?php #[Pure] function foo() {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Attribute addition should produce different fingerprint");
}

#[test]
fn test_extends_change_detected() {
    let code1 = "<?php class Foo extends Bar {}";
    let code2 = "<?php class Foo extends Baz {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Extends change should produce different fingerprint");
}

#[test]
fn test_implements_addition_detected() {
    let code1 = "<?php class Foo implements Bar {}";
    let code2 = "<?php class Foo implements Bar, Baz {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Implements addition should produce different fingerprint");
}

#[test]
fn test_trait_use_change_detected() {
    let code1 = "<?php class Foo { use Bar; }";
    let code2 = "<?php class Foo { use Baz; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Trait use change should produce different fingerprint");
}

#[test]
fn test_namespace_change_detected() {
    let code1 = "<?php namespace Foo; class Bar {}";
    let code2 = "<?php namespace Baz; class Bar {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Namespace change should produce different fingerprint");
}

#[test]
fn test_use_statement_skipped() {
    let code1 = "<?php use Foo\\Bar; class Test {}";
    let code2 = "<?php use Foo\\Baz; class Test {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_eq!(fp1, fp2, "Use statements should be skipped");
}

#[test]
fn test_use_statement_change_detected_when_enabled() {
    fn fingerprint_with_use(code: &'static str) -> u64 {
        let arena = Bump::new();
        let file = File::ephemeral("test.php".into(), code.into());
        let (program, _) = parse_file(&arena, &file);
        let resolved_names = NameResolver::new(&arena).resolve(program);
        let options = FingerprintOptions::default().with_use_statements(true);
        program.fingerprint(&resolved_names, &options)
    }

    let code1 = "<?php use Foo\\Bar; class Test {}";
    let code2 = "<?php use Foo\\Baz; class Test {}";

    let fp1 = fingerprint_with_use(code1);
    let fp2 = fingerprint_with_use(code2);

    assert_ne!(fp1, fp2, "Use statement change should be detected when enabled");
}

#[test]
fn test_control_flow_change_detected() {
    let code1 = "<?php if ($a) { echo 1; }";
    let code2 = "<?php if ($a) { echo 2; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Control flow body change should produce different fingerprint");
}

#[test]
fn test_loop_type_change_detected() {
    let code1 = "<?php while ($a) { echo 1; }";
    let code2 = "<?php for (;;) { echo 1; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Loop type change should produce different fingerprint");
}

#[test]
fn test_closure_vs_arrow_function_detected() {
    let code1 = "<?php $f = function($x) { return $x; };";
    let code2 = "<?php $f = fn($x) => $x;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Closure vs arrow function should produce different fingerprint");
}

#[test]
fn test_statement_order_change_detected() {
    let code1 = "<?php $a = 1; $b = 2;";
    let code2 = "<?php $b = 2; $a = 1;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Statement order change should produce different fingerprint");
}

#[test]
fn test_expression_parentheses_normalized() {
    let code1 = "<?php $a = 1 + 2;";
    let code2 = "<?php $a = (1 + 2);";
    let code3 = "<?php $a = ((1 + 2));";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);
    let fp3 = fingerprint_code(code3);

    assert_eq!(fp1, fp2, "Expression parentheses should be normalized");
    assert_eq!(fp2, fp3, "Expression parentheses should be normalized");
}

#[test]
fn test_boolean_literal_normalized() {
    let code1 = "<?php $a = true;";
    let code2 = "<?php $a = TRUE;";
    let code3 = "<?php $a = True;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);
    let fp3 = fingerprint_code(code3);

    assert_eq!(fp1, fp2, "Boolean literals should be normalized");
    assert_eq!(fp2, fp3, "Boolean literals should be normalized");
}

#[test]
fn test_null_literal_normalized() {
    let code1 = "<?php $a = null;";
    let code2 = "<?php $a = NULL;";
    let code3 = "<?php $a = Null;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);
    let fp3 = fingerprint_code(code3);

    assert_eq!(fp1, fp2, "Null literals should be normalized");
    assert_eq!(fp2, fp3, "Null literals should be normalized");
}

#[test]
fn test_type_keyword_normalized() {
    let code1 = "<?php function foo(int $a) {}";
    let code2 = "<?php function foo(INT $a) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_eq!(fp1, fp2, "Type keywords should be normalized");
}

#[test]
fn test_not_equal_operators_equivalent() {
    let code1 = "<?php $a = 1 != 2;";
    let code2 = "<?php $a = 1 <> 2;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_eq!(fp1, fp2, "!= and <> should be equivalent");
}

#[test]
fn test_exit_die_equivalent() {
    let code1 = "<?php exit;";
    let code2 = "<?php die;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_eq!(fp1, fp2, "exit and die should be equivalent");
}

#[test]
fn test_empty_argument_list_vs_no_parens() {
    let code1 = "<?php exit;";
    let code2 = "<?php exit();";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_eq!(fp1, fp2, "exit and exit() should be equivalent in normalized mode");
}

#[test]
fn test_method_call_vs_null_safe_method_call() {
    let code1 = "<?php $a->method();";
    let code2 = "<?php $a?->method();";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Regular and null-safe method calls should differ");
}

#[test]
fn test_property_access_vs_null_safe_property_access() {
    let code1 = "<?php $a->prop;";
    let code2 = "<?php $a?->prop;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Regular and null-safe property access should differ");
}

#[test]
fn test_nested_expression_change_detected() {
    let code1 = "<?php $a = ($b + $c) * $d;";
    let code2 = "<?php $a = $b + ($c * $d);";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Nested expression structure change should be detected");
}

#[test]
fn test_array_element_order_matters() {
    let code1 = "<?php $a = [1, 2, 3];";
    let code2 = "<?php $a = [3, 2, 1];";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Array element order should matter");
}

#[test]
fn test_array_key_change_detected() {
    let code1 = "<?php $a = ['foo' => 1, 'bar' => 2];";
    let code2 = "<?php $a = ['baz' => 1, 'bar' => 2];";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Array key change should be detected");
}

#[test]
fn test_empty_vs_nonempty_array() {
    let code1 = "<?php $a = [];";
    let code2 = "<?php $a = [1];";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Empty vs non-empty array should differ");
}

#[test]
fn test_reference_vs_value() {
    let code1 = "<?php $a = $b;";
    let code2 = "<?php $a = &$b;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Reference assignment should differ from value assignment");
}

#[test]
fn test_prefix_vs_postfix_increment() {
    let code1 = "<?php $a = ++$b;";
    let code2 = "<?php $a = $b++;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Prefix vs postfix increment should differ");
}

#[test]
fn test_static_vs_nonstatic_method() {
    let code1 = "<?php class Foo { function bar() {} }";
    let code2 = "<?php class Foo { static function bar() {} }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Static vs non-static method should differ");
}

#[test]
fn test_final_vs_nonfinal_method() {
    let code1 = "<?php class Foo { function bar() {} }";
    let code2 = "<?php class Foo { final function bar() {} }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Final vs non-final method should differ");
}

#[test]
fn test_abstract_vs_concrete_method() {
    let code1 = "<?php abstract class Foo { abstract function bar(); }";
    let code2 = "<?php abstract class Foo { function bar() {} }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Abstract vs concrete method should differ");
}

#[test]
fn test_nullable_vs_nonnullable_type() {
    let code1 = "<?php function foo(string $a) {}";
    let code2 = "<?php function foo(?string $a) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Nullable vs non-nullable type should differ");
}

#[test]
fn test_union_type_order_matters() {
    let code1 = "<?php function foo(string|int $a) {}";
    let code2 = "<?php function foo(int|string $a) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Union type order should matter");
}

#[test]
fn test_default_parameter_value_change() {
    let code1 = "<?php function foo($a = 1) {}";
    let code2 = "<?php function foo($a = 2) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Default parameter value change should be detected");
}

#[test]
fn test_variadic_vs_normal_parameter() {
    let code1 = "<?php function foo($a) {}";
    let code2 = "<?php function foo(...$a) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Variadic vs normal parameter should differ");
}

#[test]
fn test_by_ref_vs_by_value_parameter() {
    let code1 = "<?php function foo($a) {}";
    let code2 = "<?php function foo(&$a) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "By-ref vs by-value parameter should differ");
}

#[test]
fn test_match_vs_switch() {
    let code1 = "<?php match($a) { 1 => 'one', default => 'other' };";
    let code2 = "<?php switch($a) { case 1: echo 'one'; break; default: echo 'other'; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Match vs switch should differ");
}

#[test]
fn test_throw_as_expression_vs_statement() {
    let code1 = "<?php $a = $b ?? throw new Exception();";
    let code2 = "<?php if (!isset($b)) { throw new Exception(); } $a = $b;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Throw expression vs statement should differ");
}

#[test]
fn test_foreach_with_key_vs_without() {
    let code1 = "<?php foreach ($a as $v) {}";
    let code2 = "<?php foreach ($a as $k => $v) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Foreach with key vs without should differ");
}

#[test]
fn test_foreach_by_ref_vs_by_value() {
    let code1 = "<?php foreach ($a as $v) {}";
    let code2 = "<?php foreach ($a as &$v) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Foreach by-ref vs by-value should differ");
}

#[test]
fn test_break_with_level_vs_without() {
    let code1 = "<?php while(true) { break; }";
    let code2 = "<?php while(true) { while(true) { break 2; } }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Break with level should differ");
}

#[test]
fn test_yield_vs_yield_from() {
    let code1 = "<?php function gen() { yield 1; }";
    let code2 = "<?php function gen() { yield from [1]; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Yield vs yield from should differ");
}

#[test]
fn test_yield_with_key_vs_without() {
    let code1 = "<?php function gen() { yield 1; }";
    let code2 = "<?php function gen() { yield 'key' => 1; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Yield with key vs without should differ");
}

#[test]
fn test_integer_literal_bases_normalized() {
    let code1 = "<?php $a = 10;";
    let code2 = "<?php $a = 0xA;";
    let code3 = "<?php $a = 0b1010;";
    let code4 = "<?php $a = 0o12;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);
    let fp3 = fingerprint_code(code3);
    let fp4 = fingerprint_code(code4);

    assert_eq!(fp1, fp2, "Different integer bases with same value should be normalized");
    assert_eq!(fp1, fp3, "Different integer bases with same value should be normalized");
    assert_eq!(fp1, fp4, "Different integer bases with same value should be normalized");
}

#[test]
fn test_string_quote_style_normalized() {
    let code1 = "<?php $a = 'hello';";
    let code2 = "<?php $a = \"hello\";";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_eq!(fp1, fp2, "String quote style should be normalized");
}

#[test]
fn test_binary_number_formatting() {
    let code1 = "<?php $a = 1000;";
    let code2 = "<?php $a = 1_000;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_eq!(fp1, fp2, "Number separators should be normalized");
}

#[test]
fn test_readonly_vs_non_readonly_property() {
    let code1 = "<?php class Foo { public string $prop; }";
    let code2 = "<?php class Foo { public readonly string $prop; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Readonly vs non-readonly property should differ");
}

#[test]
fn test_promoted_vs_regular_parameter() {
    let code1 = "<?php class Foo { public function __construct($a) {} }";
    let code2 = "<?php class Foo { public function __construct(public $a) {} }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Promoted vs regular parameter should differ");
}

#[test]
fn test_named_vs_positional_argument() {
    let code1 = "<?php foo(1, 2);";
    let code2 = "<?php foo(b: 2, a: 1);";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Named vs positional arguments should differ");
}

#[test]
fn test_try_catch_different_exception_types() {
    let code1 = "<?php try {} catch (Exception $e) {}";
    let code2 = "<?php try {} catch (Error $e) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Different exception types should differ");
}

#[test]
fn test_try_with_vs_without_finally() {
    let code1 = "<?php try {} catch (Exception $e) {}";
    let code2 = "<?php try {} catch (Exception $e) {} finally {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Try with vs without finally should differ");
}

#[test]
fn test_multiple_catch_clauses_order() {
    let code1 = "<?php try {} catch (Exception $e) {} catch (Error $e) {}";
    let code2 = "<?php try {} catch (Error $e) {} catch (Exception $e) {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Catch clause order should matter");
}

#[test]
fn test_interface_vs_class() {
    let code1 = "<?php class Foo {}";
    let code2 = "<?php interface Foo {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Interface vs class should differ");
}

#[test]
fn test_trait_vs_class() {
    let code1 = "<?php class Foo {}";
    let code2 = "<?php trait Foo {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Trait vs class should differ");
}

#[test]
fn test_enum_vs_class() {
    let code1 = "<?php class Foo {}";
    let code2 = "<?php enum Foo {}";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Enum vs class should differ");
}

#[test]
fn test_backed_vs_pure_enum() {
    let code1 = "<?php enum Foo { case A; }";
    let code2 = "<?php enum Foo: int { case A = 1; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Backed vs pure enum should differ");
}

#[test]
fn test_enum_case_value_change() {
    let code1 = "<?php enum Foo: int { case A = 1; }";
    let code2 = "<?php enum Foo: int { case A = 2; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Enum case value change should be detected");
}

#[test]
fn test_constant_vs_class_constant() {
    let code1 = "<?php const FOO = 1;";
    let code2 = "<?php class Bar { const FOO = 1; }";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Global constant vs class constant should differ");
}

#[test]
fn test_logical_and_vs_bitwise_and() {
    let code1 = "<?php $a = $b && $c;";
    let code2 = "<?php $a = $b & $c;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Logical AND vs bitwise AND should differ");
}

#[test]
fn test_short_vs_long_logical_operators() {
    let code1 = "<?php $a = $b && $c;";
    let code2 = "<?php $a = $b and $c;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Short vs long logical operators should differ");
}

#[test]
fn test_declare_directive_value_change() {
    let code1 = "<?php declare(ticks=1);";
    let code2 = "<?php declare(ticks=2);";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Declare directive value change should be detected");
}

#[test]
fn test_magic_constant_type_differs() {
    let code1 = "<?php $a = __FILE__;";
    let code2 = "<?php $a = __LINE__;";

    let fp1 = fingerprint_code(code1);
    let fp2 = fingerprint_code(code2);

    assert_ne!(fp1, fp2, "Different magic constants should differ");
}
