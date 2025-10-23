use bumpalo::Bump;

use mago_database::file::File;
use mago_fingerprint::FingerprintOptions;
use mago_fingerprint::Fingerprintable;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

fn get_fingerprint(code: &'static str) -> u64 {
    let arena = Bump::new();
    let file = File::ephemeral("test.php".into(), code.into());
    let (program, error) = parse_file(&arena, &file);
    assert!(error.is_none(), "Parse error: {:?}", error);

    let resolved_names = NameResolver::new(&arena).resolve(program);
    let options = FingerprintOptions::default();

    program.fingerprint(&resolved_names, &options)
}

#[test]
fn test_class_extends_addition() {
    let without = get_fingerprint("<?php class Child {}");
    let with = get_fingerprint("<?php class Child extends Parent {}");

    assert_ne!(without, with, "Adding extends should change fingerprint");
}

#[test]
fn test_class_extends_change() {
    let parent_a = get_fingerprint("<?php class Child extends ParentA {}");
    let parent_b = get_fingerprint("<?php class Child extends ParentB {}");

    assert_ne!(parent_a, parent_b, "Changing parent class should change fingerprint");
}

#[test]
fn test_class_extends_removal() {
    let with = get_fingerprint("<?php class Child extends Parent {}");
    let without = get_fingerprint("<?php class Child {}");

    assert_ne!(with, without, "Removing extends should change fingerprint");
}

#[test]
fn test_class_extends_with_namespace() {
    let without_ns = get_fingerprint("<?php class Child extends Parent {}");
    let with_ns = get_fingerprint("<?php class Child extends \\Namespace\\Parent {}");

    assert_ne!(without_ns, with_ns, "Namespace qualifier on parent should change fingerprint");
}

#[test]
fn test_class_extends_different_namespaces() {
    let ns1 = get_fingerprint("<?php class Child extends \\Ns1\\Parent {}");
    let ns2 = get_fingerprint("<?php class Child extends \\Ns2\\Parent {}");

    assert_ne!(ns1, ns2, "Different parent namespaces should change fingerprint");
}

#[test]
fn test_class_extends_case_sensitivity() {
    let lower = get_fingerprint("<?php class Child extends parent {}");
    let upper = get_fingerprint("<?php class Child extends PARENT {}");
    let mixed = get_fingerprint("<?php class Child extends Parent {}");

    assert_eq!(lower, upper, "Parent class name case should not matter");
    assert_eq!(lower, mixed, "Parent class name case should not matter");
}

#[test]
fn test_class_implements_addition() {
    let without = get_fingerprint("<?php class A {}");
    let with = get_fingerprint("<?php class A implements I {}");

    assert_ne!(without, with, "Adding implements should change fingerprint");
}

#[test]
fn test_class_implements_multiple() {
    let one = get_fingerprint("<?php class A implements I1 {}");
    let two = get_fingerprint("<?php class A implements I1, I2 {}");

    assert_ne!(one, two, "Adding interface should change fingerprint");
}

#[test]
fn test_class_implements_order() {
    let order1 = get_fingerprint("<?php class A implements I1, I2 {}");
    let order2 = get_fingerprint("<?php class A implements I2, I1 {}");

    assert_ne!(order1, order2, "Interface order should change fingerprint");
}

#[test]
fn test_class_implements_change() {
    let i1 = get_fingerprint("<?php class A implements I1 {}");
    let i2 = get_fingerprint("<?php class A implements I2 {}");

    assert_ne!(i1, i2, "Changing interface should change fingerprint");
}

#[test]
fn test_class_implements_removal() {
    let two = get_fingerprint("<?php class A implements I1, I2 {}");
    let one = get_fingerprint("<?php class A implements I1 {}");

    assert_ne!(two, one, "Removing interface should change fingerprint");
}

#[test]
fn test_class_implements_namespaced() {
    let global = get_fingerprint("<?php class A implements I {}");
    let namespaced = get_fingerprint("<?php class A implements \\Ns\\I {}");

    assert_ne!(global, namespaced, "Namespaced interface should change fingerprint");
}

#[test]
fn test_class_implements_three_interfaces() {
    let two = get_fingerprint("<?php class A implements I1, I2 {}");
    let three = get_fingerprint("<?php class A implements I1, I2, I3 {}");

    assert_ne!(two, three, "Adding third interface should change fingerprint");
}

#[test]
fn test_class_implements_replacement() {
    let original = get_fingerprint("<?php class A implements I1, I2 {}");
    let replaced = get_fingerprint("<?php class A implements I3, I4 {}");

    assert_ne!(original, replaced, "Replacing all interfaces should change fingerprint");
}

#[test]
fn test_class_extends_and_implements() {
    let only_extends = get_fingerprint("<?php class A extends B {}");
    let both = get_fingerprint("<?php class A extends B implements I {}");

    assert_ne!(only_extends, both, "Adding implements to extends should change fingerprint");
}

#[test]
fn test_class_complex_hierarchy() {
    let simple = get_fingerprint("<?php class A {}");
    let complex = get_fingerprint("<?php class A extends B implements I1, I2, I3 {}");

    assert_ne!(simple, complex, "Complex hierarchy should differ from simple class");
}

#[test]
fn test_class_hierarchy_change() {
    let hier1 = get_fingerprint("<?php class A extends B implements I1, I2 {}");
    let hier2 = get_fingerprint("<?php class A extends C implements I1, I3 {}");

    assert_ne!(hier1, hier2, "Changing parent and interface should change fingerprint");
}

#[test]
fn test_interface_extends_addition() {
    let without = get_fingerprint("<?php interface I {}");
    let with = get_fingerprint("<?php interface I extends Base {}");

    assert_ne!(without, with, "Adding interface extends should change fingerprint");
}

#[test]
fn test_interface_extends_multiple() {
    let one = get_fingerprint("<?php interface I extends I1 {}");
    let two = get_fingerprint("<?php interface I extends I1, I2 {}");

    assert_ne!(one, two, "Adding parent interface should change fingerprint");
}

#[test]
fn test_interface_extends_change() {
    let i1 = get_fingerprint("<?php interface I extends I1 {}");
    let i2 = get_fingerprint("<?php interface I extends I2 {}");

    assert_ne!(i1, i2, "Changing parent interface should change fingerprint");
}

#[test]
fn test_interface_extends_order() {
    let order1 = get_fingerprint("<?php interface I extends I1, I2 {}");
    let order2 = get_fingerprint("<?php interface I extends I2, I1 {}");

    assert_ne!(order1, order2, "Parent interface order should change fingerprint");
}

#[test]
fn test_interface_extends_namespaced() {
    let global = get_fingerprint("<?php interface I extends Base {}");
    let namespaced = get_fingerprint("<?php interface I extends \\Ns\\Base {}");

    assert_ne!(global, namespaced, "Namespaced parent interface should change fingerprint");
}

#[test]
fn test_interface_multiple_parents() {
    let two = get_fingerprint("<?php interface I extends I1, I2 {}");
    let three = get_fingerprint("<?php interface I extends I1, I2, I3 {}");

    assert_ne!(two, three, "Adding third parent interface should change fingerprint");
}

#[test]
fn test_trait_use_addition() {
    let without = get_fingerprint("<?php class A {}");
    let with = get_fingerprint("<?php class A { use T; }");

    assert_ne!(without, with, "Adding trait use should change fingerprint");
}

#[test]
fn test_trait_use_multiple() {
    let one = get_fingerprint("<?php class A { use T1; }");
    let two = get_fingerprint("<?php class A { use T1, T2; }");

    assert_ne!(one, two, "Adding trait should change fingerprint");
}

#[test]
fn test_trait_use_order() {
    let order1 = get_fingerprint("<?php class A { use T1, T2; }");
    let order2 = get_fingerprint("<?php class A { use T2, T1; }");

    assert_ne!(order1, order2, "Trait order should change fingerprint");
}

#[test]
fn test_trait_use_change() {
    let t1 = get_fingerprint("<?php class A { use T1; }");
    let t2 = get_fingerprint("<?php class A { use T2; }");

    assert_ne!(t1, t2, "Changing trait should change fingerprint");
}

#[test]
fn test_trait_use_aliasing() {
    let without_alias = get_fingerprint("<?php class A { use T; }");
    let with_alias = get_fingerprint("<?php class A { use T { foo as bar; } }");

    assert_ne!(without_alias, with_alias, "Trait aliasing should change fingerprint");
}

#[test]
fn test_trait_use_multiple_aliases() {
    let one_alias = get_fingerprint("<?php class A { use T { foo as bar; } }");
    let two_aliases = get_fingerprint("<?php class A { use T { foo as bar; baz as qux; } }");

    assert_ne!(one_alias, two_aliases, "Adding alias should change fingerprint");
}

#[test]
fn test_trait_use_insteadof() {
    let without = get_fingerprint("<?php class A { use T1, T2; }");
    let with = get_fingerprint("<?php class A { use T1, T2 { T1::foo insteadof T2; } }");

    assert_ne!(without, with, "Trait insteadof should change fingerprint");
}

#[test]
fn test_trait_use_visibility_change() {
    let without = get_fingerprint("<?php class A { use T; }");
    let with_private = get_fingerprint("<?php class A { use T { foo as private; } }");
    let with_public = get_fingerprint("<?php class A { use T { foo as public; } }");

    assert_ne!(without, with_private, "Trait visibility change should change fingerprint");
    assert_ne!(with_private, with_public, "Different visibility should differ");
}

#[test]
fn test_trait_use_combined_adaptations() {
    let simple = get_fingerprint("<?php class A { use T; }");
    let complex = get_fingerprint("<?php class A { use T { foo as private bar; baz as public; } }");

    assert_ne!(simple, complex, "Combined trait adaptations should change fingerprint");
}

#[test]
fn test_trait_use_namespaced() {
    let global = get_fingerprint("<?php class A { use T; }");
    let namespaced = get_fingerprint("<?php class A { use \\Ns\\T; }");

    assert_ne!(global, namespaced, "Namespaced trait should change fingerprint");
}

#[test]
fn test_trait_in_trait() {
    let without = get_fingerprint("<?php trait T1 {}");
    let with = get_fingerprint("<?php trait T1 { use T2; }");

    assert_ne!(without, with, "Trait using trait should change fingerprint");
}

#[test]
fn test_enum_case_addition() {
    let one = get_fingerprint("<?php enum E { case A; }");
    let two = get_fingerprint("<?php enum E { case A; case B; }");

    assert_ne!(one, two, "Adding enum case should change fingerprint");
}

#[test]
fn test_enum_case_removal() {
    let two = get_fingerprint("<?php enum E { case A; case B; }");
    let one = get_fingerprint("<?php enum E { case A; }");

    assert_ne!(two, one, "Removing enum case should change fingerprint");
}

#[test]
fn test_enum_case_name_change() {
    let case_a = get_fingerprint("<?php enum E { case A; }");
    let case_b = get_fingerprint("<?php enum E { case B; }");

    assert_ne!(case_a, case_b, "Enum case name change should change fingerprint");
}

#[test]
fn test_enum_case_order() {
    let order1 = get_fingerprint("<?php enum E { case A; case B; }");
    let order2 = get_fingerprint("<?php enum E { case B; case A; }");

    assert_ne!(order1, order2, "Enum case order should change fingerprint");
}

#[test]
fn test_enum_backed_type_addition() {
    let pure = get_fingerprint("<?php enum E { case A; }");
    let backed = get_fingerprint("<?php enum E: string { case A = 'a'; }");

    assert_ne!(pure, backed, "Adding backed type should change fingerprint");
}

#[test]
fn test_enum_backed_type_change() {
    let string_backed = get_fingerprint("<?php enum E: string { case A = 'a'; }");
    let int_backed = get_fingerprint("<?php enum E: int { case A = 1; }");

    assert_ne!(string_backed, int_backed, "Changing backed type should change fingerprint");
}

#[test]
fn test_enum_backed_value_change() {
    let value_a = get_fingerprint("<?php enum E: string { case A = 'a'; }");
    let value_b = get_fingerprint("<?php enum E: string { case A = 'b'; }");

    assert_ne!(value_a, value_b, "Enum backed value change should change fingerprint");
}

#[test]
fn test_enum_multiple_backed_cases() {
    let one = get_fingerprint("<?php enum E: int { case A = 1; }");
    let two = get_fingerprint("<?php enum E: int { case A = 1; case B = 2; }");

    assert_ne!(one, two, "Adding backed case should change fingerprint");
}

#[test]
fn test_enum_implements() {
    let without = get_fingerprint("<?php enum E {}");
    let with = get_fingerprint("<?php enum E implements I {}");

    assert_ne!(without, with, "Enum implements should change fingerprint");
}

#[test]
fn test_enum_implements_multiple() {
    let one = get_fingerprint("<?php enum E implements I1 {}");
    let two = get_fingerprint("<?php enum E implements I1, I2 {}");

    assert_ne!(one, two, "Adding interface to enum should change fingerprint");
}

#[test]
fn test_enum_with_methods() {
    let without = get_fingerprint("<?php enum E { case A; }");
    let with = get_fingerprint("<?php enum E { case A; public function foo() {} }");

    assert_ne!(without, with, "Adding method to enum should change fingerprint");
}

#[test]
fn test_enum_backed_mixed_values() {
    let sequential = get_fingerprint("<?php enum E: int { case A = 0; case B = 1; }");
    let non_sequential = get_fingerprint("<?php enum E: int { case A = 10; case B = 20; }");

    assert_ne!(sequential, non_sequential, "Different enum values should change fingerprint");
}

#[test]
fn test_constructor_property_promotion() {
    let regular = get_fingerprint("<?php class A { public function __construct($x) {} }");
    let promoted = get_fingerprint("<?php class A { public function __construct(public $x) {} }");

    assert_ne!(regular, promoted, "Property promotion should change fingerprint");
}

#[test]
fn test_constructor_property_promotion_type() {
    let without_type = get_fingerprint("<?php class A { public function __construct(public $x) {} }");
    let with_type = get_fingerprint("<?php class A { public function __construct(public int $x) {} }");

    assert_ne!(without_type, with_type, "Promoted property type should change fingerprint");
}

#[test]
fn test_constructor_property_promotion_readonly() {
    let regular = get_fingerprint("<?php class A { public function __construct(public $x) {} }");
    let readonly = get_fingerprint("<?php class A { public function __construct(public readonly $x) {} }");

    assert_ne!(regular, readonly, "Promoted readonly property should change fingerprint");
}

#[test]
fn test_constructor_mixed_params() {
    let all_regular = get_fingerprint("<?php class A { public function __construct($x, $y) {} }");
    let one_promoted = get_fingerprint("<?php class A { public function __construct(public $x, $y) {} }");
    let both_promoted = get_fingerprint("<?php class A { public function __construct(public $x, public $y) {} }");

    assert_ne!(all_regular, one_promoted, "Promoting one param should change fingerprint");
    assert_ne!(one_promoted, both_promoted, "Promoting both params should change fingerprint");
}

#[test]
fn test_constructor_promotion_visibility() {
    let public = get_fingerprint("<?php class A { public function __construct(public $x) {} }");
    let private = get_fingerprint("<?php class A { public function __construct(private $x) {} }");
    let protected = get_fingerprint("<?php class A { public function __construct(protected $x) {} }");

    assert_ne!(public, private, "Different visibility should change fingerprint");
    assert_ne!(public, protected, "Different visibility should change fingerprint");
    assert_ne!(private, protected, "Different visibility should change fingerprint");
}

#[test]
fn test_constructor_promotion_with_defaults() {
    let without_default = get_fingerprint("<?php class A { public function __construct(public int $x) {} }");
    let with_default = get_fingerprint("<?php class A { public function __construct(public int $x = 0) {} }");

    assert_ne!(without_default, with_default, "Default value should change fingerprint");
}

#[test]
fn test_class_attribute_addition() {
    let without = get_fingerprint("<?php class A {}");
    let with = get_fingerprint("<?php #[Attribute] class A {}");

    assert_ne!(without, with, "Adding class attribute should change fingerprint");
}

#[test]
fn test_method_attribute_addition() {
    let without = get_fingerprint("<?php class A { function foo() {} }");
    let with = get_fingerprint("<?php class A { #[Route] function foo() {} }");

    assert_ne!(without, with, "Adding method attribute should change fingerprint");
}

#[test]
fn test_property_attribute_addition() {
    let without = get_fingerprint("<?php class A { public $x; }");
    let with = get_fingerprint("<?php class A { #[Required] public $x; }");

    assert_ne!(without, with, "Adding property attribute should change fingerprint");
}

#[test]
fn test_parameter_attribute_addition() {
    let without = get_fingerprint("<?php function foo($x) {}");
    let with = get_fingerprint("<?php function foo(#[Inject] $x) {}");

    assert_ne!(without, with, "Adding parameter attribute should change fingerprint");
}

#[test]
fn test_attribute_argument_change() {
    let arg1 = get_fingerprint("<?php #[Route('/path1')] class A {}");
    let arg2 = get_fingerprint("<?php #[Route('/path2')] class A {}");

    assert_ne!(arg1, arg2, "Attribute argument change should change fingerprint");
}

#[test]
fn test_multiple_attributes_same_target() {
    let one = get_fingerprint("<?php #[Attr1] class A {}");
    let two = get_fingerprint("<?php #[Attr1] #[Attr2] class A {}");

    assert_ne!(one, two, "Adding second attribute should change fingerprint");
}

#[test]
fn test_attribute_order() {
    let order1 = get_fingerprint("<?php #[Attr1] #[Attr2] class A {}");
    let order2 = get_fingerprint("<?php #[Attr2] #[Attr1] class A {}");

    assert_ne!(order1, order2, "Attribute order should change fingerprint");
}

#[test]
fn test_attribute_on_enum_case() {
    let without = get_fingerprint("<?php enum E { case A; }");
    let with = get_fingerprint("<?php enum E { #[Label('A')] case A; }");

    assert_ne!(without, with, "Attribute on enum case should change fingerprint");
}

#[test]
fn test_anonymous_class_extends() {
    let without = get_fingerprint("<?php $x = new class {};");
    let with = get_fingerprint("<?php $x = new class extends Parent {};");

    assert_ne!(without, with, "Anonymous class extends should change fingerprint");
}

#[test]
fn test_anonymous_class_implements() {
    let without = get_fingerprint("<?php $x = new class {};");
    let with = get_fingerprint("<?php $x = new class implements I {};");

    assert_ne!(without, with, "Anonymous class implements should change fingerprint");
}

#[test]
fn test_anonymous_class_uses_trait() {
    let without = get_fingerprint("<?php $x = new class {};");
    let with = get_fingerprint("<?php $x = new class { use T; };");

    assert_ne!(without, with, "Anonymous class using trait should change fingerprint");
}

#[test]
fn test_anonymous_class_full_hierarchy() {
    let simple = get_fingerprint("<?php $x = new class {};");
    let complex = get_fingerprint("<?php $x = new class extends Base implements I1, I2 { use T; };");

    assert_ne!(simple, complex, "Complex anonymous class should differ from simple");
}

#[test]
fn test_complex_class_hierarchy_all_features() {
    let fp = get_fingerprint(
        "<?php
        #[Entity]
        abstract class User extends BaseModel implements JsonSerializable, ArrayAccess {
            use Timestampable, SoftDeletes;

            #[Column]
            private string $name;

            public function __construct(
                #[Inject] private UserRepository $repository,
                public readonly string $id
            ) {}
        }",
    );

    assert_ne!(fp, 0, "Complex hierarchy with all features should have fingerprint");
}

#[test]
fn test_interface_full_hierarchy() {
    let fp = get_fingerprint(
        "<?php
        #[InterfaceType]
        interface AdvancedRepository extends BaseRepository, Cacheable, Loggable {
            public function complexQuery(array $criteria): Collection;
        }",
    );

    assert_ne!(fp, 0, "Interface with multiple parents should have fingerprint");
}

#[test]
fn test_enum_full_features() {
    let fp = get_fingerprint(
        "<?php
        #[EnumType]
        enum HttpStatus: int implements Stringable {
            case OK = 200;
            case NOT_FOUND = 404;
            case SERVER_ERROR = 500;

            public function message(): string {
                return match($this) {
                    self::OK => 'Success',
                    self::NOT_FOUND => 'Not Found',
                    self::SERVER_ERROR => 'Server Error',
                };
            }
        }",
    );

    assert_ne!(fp, 0, "Enum with all features should have fingerprint");
}

#[test]
fn test_trait_full_features() {
    let fp = get_fingerprint(
        "<?php
        trait ComplexTrait {
            use BaseTrait, HelperTrait;

            #[Property]
            private $data;

            abstract public function required(): void;

            public function helper() {}
        }",
    );

    assert_ne!(fp, 0, "Trait with all features should have fingerprint");
}
