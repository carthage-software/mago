use bumpalo::Bump;
use mago_database::file::File;
use mago_fingerprint::FingerprintOptions;
use mago_fingerprint::Fingerprintable;

fn get_fingerprint(code: &'static str) -> u64 {
    let arena = Bump::new();
    let file = File::ephemeral("test.php".into(), code.into());
    let (program, error) = mago_syntax::parser::parse_file(&arena, &file);
    assert!(error.is_none(), "Parse error: {:?}", error);

    let resolved_names = mago_names::resolver::NameResolver::new(&arena).resolve(program);
    let options = FingerprintOptions::default();

    use ahash::AHasher;
    use std::hash::Hasher;
    let mut hasher = AHasher::default();
    for statement in program.statements.iter() {
        statement.fingerprint_with_hasher(&mut hasher, &resolved_names, &options);
    }
    hasher.finish()
}

#[test]
fn test_parameter_name_change() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo($y) {}");

    assert_ne!(sig1, sig2, "Parameter name change should change fingerprint");
}

#[test]
fn test_parameter_addition() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo($x, $y) {}");

    assert_ne!(sig1, sig2, "Adding parameter should change fingerprint");
}

#[test]
fn test_parameter_removal() {
    let sig1 = get_fingerprint("<?php function foo($x, $y) {}");
    let sig2 = get_fingerprint("<?php function foo($x) {}");

    assert_ne!(sig1, sig2, "Removing parameter should change fingerprint");
}

#[test]
fn test_parameter_order_change() {
    let sig1 = get_fingerprint("<?php function foo($x, $y) {}");
    let sig2 = get_fingerprint("<?php function foo($y, $x) {}");

    assert_ne!(sig1, sig2, "Parameter order should change fingerprint");
}

#[test]
fn test_parameter_default_value_addition() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo($x = 1) {}");

    assert_ne!(sig1, sig2, "Adding default value should change fingerprint");
}

#[test]
fn test_parameter_default_value_change() {
    let sig1 = get_fingerprint("<?php function foo($x = 1) {}");
    let sig2 = get_fingerprint("<?php function foo($x = 2) {}");

    assert_ne!(sig1, sig2, "Changing default value should change fingerprint");
}

#[test]
fn test_parameter_type_addition() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo(int $x) {}");

    assert_ne!(sig1, sig2, "Adding parameter type should change fingerprint");
}

#[test]
fn test_parameter_type_change() {
    let sig1 = get_fingerprint("<?php function foo(int $x) {}");
    let sig2 = get_fingerprint("<?php function foo(string $x) {}");

    assert_ne!(sig1, sig2, "Parameter type change should change fingerprint");
}

#[test]
fn test_parameter_type_removal() {
    let sig1 = get_fingerprint("<?php function foo(int $x) {}");
    let sig2 = get_fingerprint("<?php function foo($x) {}");

    assert_ne!(sig1, sig2, "Removing parameter type should change fingerprint");
}

#[test]
fn test_parameter_nullable_type() {
    let sig1 = get_fingerprint("<?php function foo(string $x) {}");
    let sig2 = get_fingerprint("<?php function foo(?string $x) {}");

    assert_ne!(sig1, sig2, "Making parameter nullable should change fingerprint");
}

#[test]
fn test_parameter_union_type() {
    let sig1 = get_fingerprint("<?php function foo(string $x) {}");
    let sig2 = get_fingerprint("<?php function foo(string|int $x) {}");

    assert_ne!(sig1, sig2, "Adding union type should change fingerprint");
}

#[test]
fn test_parameter_intersection_type() {
    let sig1 = get_fingerprint("<?php function foo(A $x) {}");
    let sig2 = get_fingerprint("<?php function foo(A&B $x) {}");

    assert_ne!(sig1, sig2, "Adding intersection type should change fingerprint");
}

#[test]
fn test_parameter_array_type() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo(array $x) {}");

    assert_ne!(sig1, sig2, "Adding array type should change fingerprint");
}

#[test]
fn test_parameter_callable_type() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo(callable $x) {}");

    assert_ne!(sig1, sig2, "Adding callable type should change fingerprint");
}

#[test]
fn test_return_type_addition() {
    let sig1 = get_fingerprint("<?php function foo() {}");
    let sig2 = get_fingerprint("<?php function foo(): int {}");

    assert_ne!(sig1, sig2, "Adding return type should change fingerprint");
}

#[test]
fn test_return_type_change() {
    let sig1 = get_fingerprint("<?php function foo(): int {}");
    let sig2 = get_fingerprint("<?php function foo(): string {}");

    assert_ne!(sig1, sig2, "Return type change should change fingerprint");
}

#[test]
fn test_return_type_removal() {
    let sig1 = get_fingerprint("<?php function foo(): int {}");
    let sig2 = get_fingerprint("<?php function foo() {}");

    assert_ne!(sig1, sig2, "Removing return type should change fingerprint");
}

#[test]
fn test_return_type_nullable() {
    let sig1 = get_fingerprint("<?php function foo(): string {}");
    let sig2 = get_fingerprint("<?php function foo(): ?string {}");

    assert_ne!(sig1, sig2, "Making return type nullable should change fingerprint");
}

#[test]
fn test_return_type_void() {
    let sig1 = get_fingerprint("<?php function foo() {}");
    let sig2 = get_fingerprint("<?php function foo(): void {}");

    assert_ne!(sig1, sig2, "Adding void return type should change fingerprint");
}

#[test]
fn test_return_type_never() {
    let sig1 = get_fingerprint("<?php function foo(): void {}");
    let sig2 = get_fingerprint("<?php function foo(): never {}");

    assert_ne!(sig1, sig2, "Changing to never return type should change fingerprint");
}

#[test]
fn test_return_type_union() {
    let sig1 = get_fingerprint("<?php function foo(): int {}");
    let sig2 = get_fingerprint("<?php function foo(): int|string {}");

    assert_ne!(sig1, sig2, "Adding union return type should change fingerprint");
}

#[test]
fn test_return_type_mixed() {
    let sig1 = get_fingerprint("<?php function foo() {}");
    let sig2 = get_fingerprint("<?php function foo(): mixed {}");

    assert_ne!(sig1, sig2, "Adding mixed return type should change fingerprint");
}

#[test]
fn test_parameter_by_ref_addition() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo(&$x) {}");

    assert_ne!(sig1, sig2, "Making parameter by-ref should change fingerprint");
}

#[test]
fn test_parameter_by_ref_removal() {
    let sig1 = get_fingerprint("<?php function foo(&$x) {}");
    let sig2 = get_fingerprint("<?php function foo($x) {}");

    assert_ne!(sig1, sig2, "Removing by-ref should change fingerprint");
}

#[test]
fn test_multiple_by_ref_parameters() {
    let sig1 = get_fingerprint("<?php function foo(&$x, $y) {}");
    let sig2 = get_fingerprint("<?php function foo($x, &$y) {}");

    assert_ne!(sig1, sig2, "By-ref position should change fingerprint");
}

#[test]
fn test_by_ref_with_type() {
    let sig1 = get_fingerprint("<?php function foo(int $x) {}");
    let sig2 = get_fingerprint("<?php function foo(int &$x) {}");

    assert_ne!(sig1, sig2, "By-ref with type should change fingerprint");
}

#[test]
fn test_variadic_parameter_addition() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo(...$x) {}");

    assert_ne!(sig1, sig2, "Making parameter variadic should change fingerprint");
}

#[test]
fn test_variadic_with_type() {
    let sig1 = get_fingerprint("<?php function foo(...$x) {}");
    let sig2 = get_fingerprint("<?php function foo(int ...$x) {}");

    assert_ne!(sig1, sig2, "Adding type to variadic should change fingerprint");
}

#[test]
fn test_variadic_by_ref() {
    let sig1 = get_fingerprint("<?php function foo(...$x) {}");
    let sig2 = get_fingerprint("<?php function foo(&...$x) {}");

    assert_ne!(sig1, sig2, "Making variadic by-ref should change fingerprint");
}

#[test]
fn test_method_visibility_public_to_private() {
    let sig1 = get_fingerprint("<?php class A { public function foo() {} }");
    let sig2 = get_fingerprint("<?php class A { private function foo() {} }");

    assert_ne!(sig1, sig2, "Changing method visibility should change fingerprint");
}

#[test]
fn test_method_visibility_public_to_protected() {
    let sig1 = get_fingerprint("<?php class A { public function foo() {} }");
    let sig2 = get_fingerprint("<?php class A { protected function foo() {} }");

    assert_ne!(sig1, sig2, "Changing method visibility should change fingerprint");
}

#[test]
fn test_method_static_addition() {
    let sig1 = get_fingerprint("<?php class A { public function foo() {} }");
    let sig2 = get_fingerprint("<?php class A { public static function foo() {} }");

    assert_ne!(sig1, sig2, "Making method static should change fingerprint");
}

#[test]
fn test_method_static_removal() {
    let sig1 = get_fingerprint("<?php class A { public static function foo() {} }");
    let sig2 = get_fingerprint("<?php class A { public function foo() {} }");

    assert_ne!(sig1, sig2, "Removing static should change fingerprint");
}

#[test]
fn test_method_abstract_addition() {
    let sig1 = get_fingerprint("<?php abstract class A { public function foo() {} }");
    let sig2 = get_fingerprint("<?php abstract class A { abstract public function foo(); }");

    assert_ne!(sig1, sig2, "Making method abstract should change fingerprint");
}

#[test]
fn test_method_final_addition() {
    let sig1 = get_fingerprint("<?php class A { public function foo() {} }");
    let sig2 = get_fingerprint("<?php class A { final public function foo() {} }");

    assert_ne!(sig1, sig2, "Making method final should change fingerprint");
}

#[test]
fn test_property_type_addition() {
    let sig1 = get_fingerprint("<?php class A { public $x; }");
    let sig2 = get_fingerprint("<?php class A { public int $x; }");

    assert_ne!(sig1, sig2, "Adding property type should change fingerprint");
}

#[test]
fn test_property_type_change() {
    let sig1 = get_fingerprint("<?php class A { public int $x; }");
    let sig2 = get_fingerprint("<?php class A { public string $x; }");

    assert_ne!(sig1, sig2, "Property type change should change fingerprint");
}

#[test]
fn test_property_nullable_type() {
    let sig1 = get_fingerprint("<?php class A { public string $x; }");
    let sig2 = get_fingerprint("<?php class A { public ?string $x; }");

    assert_ne!(sig1, sig2, "Making property nullable should change fingerprint");
}

#[test]
fn test_property_readonly_addition() {
    let sig1 = get_fingerprint("<?php class A { public string $x; }");
    let sig2 = get_fingerprint("<?php class A { public readonly string $x; }");

    assert_ne!(sig1, sig2, "Making property readonly should change fingerprint");
}

#[test]
fn test_property_static_addition() {
    let sig1 = get_fingerprint("<?php class A { public $x; }");
    let sig2 = get_fingerprint("<?php class A { public static $x; }");

    assert_ne!(sig1, sig2, "Making property static should change fingerprint");
}

#[test]
fn test_property_visibility_change() {
    let sig1 = get_fingerprint("<?php class A { public $x; }");
    let sig2 = get_fingerprint("<?php class A { private $x; }");

    assert_ne!(sig1, sig2, "Changing property visibility should change fingerprint");
}

#[test]
fn test_property_default_value() {
    let sig1 = get_fingerprint("<?php class A { public $x; }");
    let sig2 = get_fingerprint("<?php class A { public $x = 1; }");

    assert_ne!(sig1, sig2, "Adding property default value should change fingerprint");
}

#[test]
fn test_constructor_property_promotion() {
    let sig1 = get_fingerprint("<?php class A { public function __construct($x) {} }");
    let sig2 = get_fingerprint("<?php class A { public function __construct(public $x) {} }");

    assert_ne!(sig1, sig2, "Property promotion should change fingerprint");
}

#[test]
fn test_constructor_property_promotion_type() {
    let sig1 = get_fingerprint("<?php class A { public function __construct(public $x) {} }");
    let sig2 = get_fingerprint("<?php class A { public function __construct(public int $x) {} }");

    assert_ne!(sig1, sig2, "Promoted property type should change fingerprint");
}

#[test]
fn test_constructor_property_promotion_readonly() {
    let sig1 = get_fingerprint("<?php class A { public function __construct(public $x) {} }");
    let sig2 = get_fingerprint("<?php class A { public function __construct(public readonly $x) {} }");

    assert_ne!(sig1, sig2, "Promoted readonly property should change fingerprint");
}

#[test]
fn test_constructor_property_promotion_visibility() {
    let sig1 = get_fingerprint("<?php class A { public function __construct(public $x) {} }");
    let sig2 = get_fingerprint("<?php class A { public function __construct(private $x) {} }");

    assert_ne!(sig1, sig2, "Promoted property visibility should change fingerprint");
}

#[test]
fn test_all_parameter_features() {
    let sig1 = get_fingerprint("<?php function foo($x) {}");
    let sig2 = get_fingerprint("<?php function foo(int|string &...$x = []) {}");

    assert_ne!(sig1, sig2, "Combined parameter features should change fingerprint");
}

#[test]
fn test_mixed_parameter_types() {
    let sig1 = get_fingerprint("<?php function foo($a, $b, $c) {}");
    let sig2 = get_fingerprint("<?php function foo(int $a, string $b, array $c) {}");

    assert_ne!(sig1, sig2, "Adding types to multiple params should change fingerprint");
}

#[test]
fn test_mixed_by_ref_parameters() {
    let sig1 = get_fingerprint("<?php function foo($a, $b, $c) {}");
    let sig2 = get_fingerprint("<?php function foo(&$a, $b, &$c) {}");

    assert_ne!(sig1, sig2, "Making some params by-ref should change fingerprint");
}

#[test]
fn test_mixed_default_values() {
    let sig1 = get_fingerprint("<?php function foo($a, $b, $c) {}");
    let sig2 = get_fingerprint("<?php function foo($a, $b = 1, $c = 2) {}");

    assert_ne!(sig1, sig2, "Adding some default values should change fingerprint");
}

#[test]
fn test_class_abstract_addition() {
    let sig1 = get_fingerprint("<?php class A {}");
    let sig2 = get_fingerprint("<?php abstract class A {}");

    assert_ne!(sig1, sig2, "Making class abstract should change fingerprint");
}

#[test]
fn test_class_final_addition() {
    let sig1 = get_fingerprint("<?php class A {}");
    let sig2 = get_fingerprint("<?php final class A {}");

    assert_ne!(sig1, sig2, "Making class final should change fingerprint");
}

#[test]
fn test_class_readonly_addition() {
    let sig1 = get_fingerprint("<?php class A {}");
    let sig2 = get_fingerprint("<?php readonly class A {}");

    assert_ne!(sig1, sig2, "Making class readonly should change fingerprint");
}

#[test]
fn test_void_vs_no_return_type() {
    let no_type = get_fingerprint("<?php function foo() {}");
    let void_type = get_fingerprint("<?php function foo(): void {}");

    assert_ne!(no_type, void_type, "Void return type should differ from no type");
}

#[test]
fn test_null_vs_nullable() {
    let nullable = get_fingerprint("<?php function foo(?int $x) {}");
    let union = get_fingerprint("<?php function foo(int|null $x) {}");

    assert_ne!(nullable, union, "?int and int|null are syntactically different");
}

#[test]
fn test_parameter_names_only_differ() {
    let sig1 = get_fingerprint("<?php function foo(int $x, string $y) {}");
    let sig2 = get_fingerprint("<?php function foo(int $a, string $b) {}");

    assert_ne!(sig1, sig2, "Parameter names should matter even with same types");
}

#[test]
fn test_property_name_order() {
    let sig1 = get_fingerprint("<?php class A { public $x; public $y; }");
    let sig2 = get_fingerprint("<?php class A { public $y; public $x; }");

    assert_ne!(sig1, sig2, "Property order should matter");
}

#[test]
fn test_method_order() {
    let sig1 = get_fingerprint("<?php class A { function foo() {} function bar() {} }");
    let sig2 = get_fingerprint("<?php class A { function bar() {} function foo() {} }");

    assert_ne!(sig1, sig2, "Method order should matter");
}
