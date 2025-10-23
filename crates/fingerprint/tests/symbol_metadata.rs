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
fn test_class_attribute_addition() {
    let without = get_fingerprint("<?php class Foo {}");
    let with = get_fingerprint("<?php #[Attribute] class Foo {}");

    assert_ne!(without, with, "Adding class attribute should change fingerprint");
}

#[test]
fn test_class_attribute_change() {
    let attr1 = get_fingerprint("<?php #[Attribute1] class Foo {}");
    let attr2 = get_fingerprint("<?php #[Attribute2] class Foo {}");

    assert_ne!(attr1, attr2, "Changing class attribute should change fingerprint");
}

#[test]
fn test_class_multiple_attributes() {
    let one = get_fingerprint("<?php #[Attr1] class Foo {}");
    let two = get_fingerprint("<?php #[Attr1] #[Attr2] class Foo {}");

    assert_ne!(one, two, "Adding second attribute should change fingerprint");
}

#[test]
fn test_class_attribute_order() {
    let order1 = get_fingerprint("<?php #[Attr1] #[Attr2] class Foo {}");
    let order2 = get_fingerprint("<?php #[Attr2] #[Attr1] class Foo {}");

    assert_ne!(order1, order2, "Attribute order should change fingerprint");
}

#[test]
fn test_class_attribute_with_arguments() {
    let without_args = get_fingerprint("<?php #[Route] class Foo {}");
    let with_args = get_fingerprint("<?php #[Route('/path')] class Foo {}");

    assert_ne!(without_args, with_args, "Attribute arguments should change fingerprint");
}

#[test]
fn test_class_attribute_argument_change() {
    let arg1 = get_fingerprint("<?php #[Route('/path1')] class Foo {}");
    let arg2 = get_fingerprint("<?php #[Route('/path2')] class Foo {}");

    assert_ne!(arg1, arg2, "Changing attribute argument should change fingerprint");
}

#[test]
fn test_class_attribute_multiple_arguments() {
    let one_arg = get_fingerprint("<?php #[Route('/path')] class Foo {}");
    let two_args = get_fingerprint("<?php #[Route('/path', 'GET')] class Foo {}");

    assert_ne!(one_arg, two_args, "Adding attribute argument should change fingerprint");
}

#[test]
fn test_method_attribute_addition() {
    let without = get_fingerprint("<?php class A { function foo() {} }");
    let with = get_fingerprint("<?php class A { #[Route] function foo() {} }");

    assert_ne!(without, with, "Adding method attribute should change fingerprint");
}

#[test]
fn test_method_attribute_change() {
    let attr1 = get_fingerprint("<?php class A { #[Get] function foo() {} }");
    let attr2 = get_fingerprint("<?php class A { #[Post] function foo() {} }");

    assert_ne!(attr1, attr2, "Changing method attribute should change fingerprint");
}

#[test]
fn test_method_multiple_attributes() {
    let one = get_fingerprint("<?php class A { #[Route('/path')] function foo() {} }");
    let two = get_fingerprint("<?php class A { #[Route('/path')] #[Auth] function foo() {} }");

    assert_ne!(one, two, "Adding method attribute should change fingerprint");
}

#[test]
fn test_method_attribute_with_named_args() {
    let positional = get_fingerprint("<?php class A { #[Route('/path', 'GET')] function foo() {} }");
    let named = get_fingerprint("<?php class A { #[Route(path: '/path', method: 'GET')] function foo() {} }");

    assert_ne!(positional, named, "Named vs positional args should change fingerprint");
}

#[test]
fn test_property_attribute_addition() {
    let without = get_fingerprint("<?php class A { public $x; }");
    let with = get_fingerprint("<?php class A { #[Required] public $x; }");

    assert_ne!(without, with, "Adding property attribute should change fingerprint");
}

#[test]
fn test_property_attribute_change() {
    let attr1 = get_fingerprint("<?php class A { #[Min(1)] public $x; }");
    let attr2 = get_fingerprint("<?php class A { #[Max(100)] public $x; }");

    assert_ne!(attr1, attr2, "Changing property attribute should change fingerprint");
}

#[test]
fn test_property_multiple_attributes() {
    let one = get_fingerprint("<?php class A { #[Required] public $x; }");
    let two = get_fingerprint("<?php class A { #[Required] #[Email] public $x; }");

    assert_ne!(one, two, "Adding property attribute should change fingerprint");
}

#[test]
fn test_parameter_attribute_addition() {
    let without = get_fingerprint("<?php function foo($x) {}");
    let with = get_fingerprint("<?php function foo(#[Inject] $x) {}");

    assert_ne!(without, with, "Adding parameter attribute should change fingerprint");
}

#[test]
fn test_parameter_attribute_change() {
    let attr1 = get_fingerprint("<?php function foo(#[Inject] $x) {}");
    let attr2 = get_fingerprint("<?php function foo(#[Autowire] $x) {}");

    assert_ne!(attr1, attr2, "Changing parameter attribute should change fingerprint");
}

#[test]
fn test_parameter_multiple_attributes() {
    let one = get_fingerprint("<?php function foo(#[Inject] $x) {}");
    let two = get_fingerprint("<?php function foo(#[Inject] #[Required] $x) {}");

    assert_ne!(one, two, "Adding parameter attribute should change fingerprint");
}

#[test]
fn test_different_parameter_attributes() {
    let first = get_fingerprint("<?php function foo(#[Inject] $x, $y) {}");
    let second = get_fingerprint("<?php function foo($x, #[Inject] $y) {}");

    assert_ne!(first, second, "Attribute on different parameter should differ");
}

#[test]
fn test_enum_case_attribute() {
    let without = get_fingerprint("<?php enum Status { case Active; }");
    let with = get_fingerprint("<?php enum Status { #[Label('Active')] case Active; }");

    assert_ne!(without, with, "Enum case attribute should change fingerprint");
}

#[test]
fn test_enum_case_attribute_different_cases() {
    let first = get_fingerprint("<?php enum Status { #[Label('A')] case Active; case Inactive; }");
    let second = get_fingerprint("<?php enum Status { case Active; #[Label('I')] case Inactive; }");

    assert_ne!(first, second, "Attribute on different case should differ");
}

#[test]
fn test_class_abstract_modifier() {
    let concrete = get_fingerprint("<?php class Foo {}");
    let abstract_class = get_fingerprint("<?php abstract class Foo {}");

    assert_ne!(concrete, abstract_class, "Abstract modifier should change fingerprint");
}

#[test]
fn test_class_final_modifier() {
    let regular = get_fingerprint("<?php class Foo {}");
    let final_class = get_fingerprint("<?php final class Foo {}");

    assert_ne!(regular, final_class, "Final modifier should change fingerprint");
}

#[test]
fn test_class_readonly_modifier() {
    let regular = get_fingerprint("<?php class Foo {}");
    let readonly = get_fingerprint("<?php readonly class Foo {}");

    assert_ne!(regular, readonly, "Readonly modifier should change fingerprint");
}

#[test]
fn test_method_visibility_public() {
    let implicit = get_fingerprint("<?php class A { function foo() {} }");
    let explicit = get_fingerprint("<?php class A { public function foo() {} }");

    assert_eq!(implicit, explicit, "Implicit public should equal explicit");
}

#[test]
fn test_method_visibility_private() {
    let public_method = get_fingerprint("<?php class A { public function foo() {} }");
    let private_method = get_fingerprint("<?php class A { private function foo() {} }");

    assert_ne!(public_method, private_method, "Visibility should change fingerprint");
}

#[test]
fn test_method_visibility_protected() {
    let public_method = get_fingerprint("<?php class A { public function foo() {} }");
    let protected_method = get_fingerprint("<?php class A { protected function foo() {} }");

    assert_ne!(public_method, protected_method, "Visibility should change fingerprint");
}

#[test]
fn test_method_static_modifier() {
    let instance = get_fingerprint("<?php class A { public function foo() {} }");
    let static_method = get_fingerprint("<?php class A { public static function foo() {} }");

    assert_ne!(instance, static_method, "Static modifier should change fingerprint");
}

#[test]
fn test_method_abstract_modifier() {
    let concrete = get_fingerprint("<?php abstract class A { public function foo() {} }");
    let abstract_method = get_fingerprint("<?php abstract class A { abstract public function foo(); }");

    assert_ne!(concrete, abstract_method, "Abstract modifier should change fingerprint");
}

#[test]
fn test_method_final_modifier() {
    let regular = get_fingerprint("<?php class A { public function foo() {} }");
    let final_method = get_fingerprint("<?php class A { final public function foo() {} }");

    assert_ne!(regular, final_method, "Final modifier should change fingerprint");
}

#[test]
fn test_property_visibility_public() {
    let public_prop = get_fingerprint("<?php class A { public $x; }");
    let private_prop = get_fingerprint("<?php class A { private $x; }");

    assert_ne!(public_prop, private_prop, "Property visibility should change fingerprint");
}

#[test]
fn test_property_static_modifier() {
    let instance = get_fingerprint("<?php class A { public $x; }");
    let static_prop = get_fingerprint("<?php class A { public static $x; }");

    assert_ne!(instance, static_prop, "Static property should change fingerprint");
}

#[test]
fn test_property_readonly_modifier() {
    let mutable = get_fingerprint("<?php class A { public string $x; }");
    let readonly = get_fingerprint("<?php class A { public readonly string $x; }");

    assert_ne!(mutable, readonly, "Readonly modifier should change fingerprint");
}

#[test]
fn test_method_multiple_modifiers() {
    let simple = get_fingerprint("<?php class A { function foo() {} }");
    let complex = get_fingerprint("<?php class A { final public static function foo() {} }");

    assert_ne!(simple, complex, "Multiple modifiers should change fingerprint");
}

#[test]
fn test_property_multiple_modifiers() {
    let simple = get_fingerprint("<?php class A { $x; }");
    let complex = get_fingerprint("<?php class A { private static readonly int $x; }");

    assert_ne!(simple, complex, "Multiple property modifiers should change fingerprint");
}

#[test]
fn test_class_constant_visibility() {
    let public_const = get_fingerprint("<?php class A { const X = 1; }");
    let private_const = get_fingerprint("<?php class A { private const X = 1; }");

    assert_ne!(public_const, private_const, "Constant visibility should change fingerprint");
}

#[test]
fn test_class_constant_final() {
    let regular = get_fingerprint("<?php class A { const X = 1; }");
    let final_const = get_fingerprint("<?php class A { final const X = 1; }");

    assert_ne!(regular, final_const, "Final constant should change fingerprint");
}

#[test]
fn test_nested_attribute_arguments() {
    let simple = get_fingerprint("<?php #[Route('/path')] class Foo {}");
    let nested = get_fingerprint("<?php #[Route('/path', options: ['auth' => true])] class Foo {}");

    assert_ne!(simple, nested, "Nested attribute arguments should change fingerprint");
}

#[test]
fn test_attribute_with_class_constant() {
    let literal = get_fingerprint("<?php #[Route('/path')] class Foo {}");
    let constant = get_fingerprint("<?php #[Route(MyClass::PATH)] class Foo {}");

    assert_ne!(literal, constant, "Attribute with constant should differ from literal");
}

#[test]
fn test_attribute_with_array() {
    let without = get_fingerprint("<?php #[Middleware] class Foo {}");
    let with_array = get_fingerprint("<?php #[Middleware(['auth', 'csrf'])] class Foo {}");

    assert_ne!(without, with_array, "Attribute with array should change fingerprint");
}

#[test]
fn test_same_attribute_different_targets() {
    let on_class = get_fingerprint("<?php #[Inject] class Foo { public $x; }");
    let on_property = get_fingerprint("<?php class Foo { #[Inject] public $x; }");

    assert_ne!(on_class, on_property, "Same attribute on different targets should differ");
}

#[test]
fn test_attribute_on_promoted_property() {
    let regular = get_fingerprint("<?php class A { public function __construct(public $x) {} }");
    let with_attr = get_fingerprint("<?php class A { public function __construct(#[Inject] public $x) {} }");

    assert_ne!(regular, with_attr, "Attribute on promoted property should change fingerprint");
}

#[test]
fn test_all_method_visibility_options() {
    let public = get_fingerprint("<?php class A { public function foo() {} }");
    let protected = get_fingerprint("<?php class A { protected function foo() {} }");
    let private = get_fingerprint("<?php class A { private function foo() {} }");

    assert_ne!(public, protected, "Public and protected should differ");
    assert_ne!(public, private, "Public and private should differ");
    assert_ne!(protected, private, "Protected and private should differ");
}

#[test]
fn test_all_property_visibility_options() {
    let public = get_fingerprint("<?php class A { public $x; }");
    let protected = get_fingerprint("<?php class A { protected $x; }");
    let private = get_fingerprint("<?php class A { private $x; }");

    assert_ne!(public, protected, "Public and protected should differ");
    assert_ne!(public, private, "Public and private should differ");
    assert_ne!(protected, private, "Protected and private should differ");
}

#[test]
fn test_modifier_order_normalization() {
    let order1 = get_fingerprint("<?php class A { public static function foo() {} }");
    let order2 = get_fingerprint("<?php class A { static public function foo() {} }");

    assert_eq!(order1, order2, "Modifier order should be normalized");
}

#[test]
fn test_final_public_vs_public_final() {
    let order1 = get_fingerprint("<?php class A { final public function foo() {} }");
    let order2 = get_fingerprint("<?php class A { public final function foo() {} }");

    assert_eq!(order1, order2, "Modifier order should be normalized");
}

#[test]
fn test_class_with_all_metadata() {
    let minimal = get_fingerprint("<?php class Foo {}");
    let maximal = get_fingerprint(
        "<?php
        #[Entity]
        #[Table('users')]
        final readonly class Foo {}",
    );

    assert_ne!(minimal, maximal, "Full metadata should differ from minimal");
}

#[test]
fn test_method_with_all_metadata() {
    let minimal = get_fingerprint("<?php class A { function foo() {} }");
    let maximal = get_fingerprint(
        "<?php
        class A {
            #[Route('/path')]
            #[Auth('admin')]
            final public static function foo(): void {}
        }",
    );

    assert_ne!(minimal, maximal, "Full method metadata should differ from minimal");
}

#[test]
fn test_property_with_all_metadata() {
    let minimal = get_fingerprint("<?php class A { $x; }");
    let maximal = get_fingerprint(
        "<?php
        class A {
            #[Required]
            #[Email]
            private static readonly string $x = 'default';
        }",
    );

    assert_ne!(minimal, maximal, "Full property metadata should differ from minimal");
}

#[test]
fn test_interface_with_attributes() {
    let without = get_fingerprint("<?php interface Repository {}");
    let with = get_fingerprint("<?php #[InterfaceType] interface Repository {}");

    assert_ne!(without, with, "Interface attribute should change fingerprint");
}

#[test]
fn test_trait_with_attributes() {
    let without = get_fingerprint("<?php trait Timestampable {}");
    let with = get_fingerprint("<?php #[TraitType] trait Timestampable {}");

    assert_ne!(without, with, "Trait attribute should change fingerprint");
}

#[test]
fn test_enum_with_attributes() {
    let without = get_fingerprint("<?php enum Status {}");
    let with = get_fingerprint("<?php #[EnumType] enum Status {}");

    assert_ne!(without, with, "Enum attribute should change fingerprint");
}

#[test]
fn test_enum_implements_with_attributes() {
    let without = get_fingerprint("<?php enum Status implements Stringable {}");
    let with = get_fingerprint("<?php #[JsonSerializable] enum Status implements Stringable {}");

    assert_ne!(without, with, "Enum attribute with implements should change fingerprint");
}
