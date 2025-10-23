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
    let options = FingerprintOptions::default().with_use_statements(true);

    use ahash::AHasher;
    use std::hash::Hasher;
    let mut hasher = AHasher::default();
    program.fingerprint_with_hasher(&mut hasher, &resolved_names, &options);
    hasher.finish()
}

#[test]
fn test_simple_namespace() {
    let global = get_fingerprint("<?php class Foo {}");
    let namespaced = get_fingerprint("<?php namespace App; class Foo {}");

    assert_ne!(global, namespaced, "Namespace should change fingerprint");
}

#[test]
fn test_nested_namespace() {
    let simple = get_fingerprint("<?php namespace App; class Foo {}");
    let nested = get_fingerprint("<?php namespace App\\Models; class Foo {}");

    assert_ne!(simple, nested, "Nested namespace should change fingerprint");
}

#[test]
fn test_deeply_nested_namespace() {
    let shallow = get_fingerprint("<?php namespace A; class C {}");
    let deep = get_fingerprint("<?php namespace A\\B\\C\\D\\E; class C {}");

    assert_ne!(shallow, deep, "Deep namespace nesting should change fingerprint");
}

#[test]
fn test_namespace_case_insensitivity() {
    let lower = get_fingerprint("<?php namespace app; class Foo {}");
    let upper = get_fingerprint("<?php namespace APP; class Foo {}");
    let mixed = get_fingerprint("<?php namespace App; class Foo {}");

    assert_eq!(lower, upper, "Namespaces are case-insensitive");
    assert_eq!(lower, mixed, "Namespaces are case-insensitive");
}

#[test]
fn test_braced_namespace() {
    let unbraced = get_fingerprint("<?php namespace Foo; class Bar {}");
    let braced = get_fingerprint("<?php namespace Foo { class Bar {} }");

    assert_ne!(unbraced, braced, "Braced and unbraced namespaces are syntactically different");
}

#[test]
fn test_multiple_braced_namespaces() {
    let fp = get_fingerprint(
        "<?php
        namespace Foo {
            class A {}
        }
        namespace Bar {
            class B {}
        }",
    );

    assert_ne!(fp, 0, "Multiple braced namespaces should have fingerprint");
}

#[test]
fn test_namespace_switch() {
    let single = get_fingerprint("<?php namespace Foo; class A {} class B {}");
    let switched = get_fingerprint(
        "<?php
        namespace Foo;
        class A {}
        namespace Bar;
        class B {}",
    );

    assert_ne!(single, switched, "Namespace switch should change fingerprint");
}

#[test]
fn test_multiple_namespace_switches() {
    let fp = get_fingerprint(
        "<?php
        namespace NS1;
        class C1 {}

        namespace NS2;
        class C2 {}

        namespace NS3;
        class C3 {}",
    );

    assert_ne!(fp, 0, "Multiple namespace switches should have fingerprint");
}

#[test]
fn test_return_to_same_namespace() {
    let sequential = get_fingerprint(
        "<?php
        namespace Foo;
        class A {}
        class B {}",
    );
    let switched_back = get_fingerprint(
        "<?php
        namespace Foo;
        class A {}

        namespace Bar;
        // Switch away

        namespace Foo;
        class B {}",
    );

    assert_ne!(sequential, switched_back, "Switching and returning should differ");
}

#[test]
fn test_explicit_global_namespace() {
    let implicit = get_fingerprint("<?php class Foo {}");
    let explicit = get_fingerprint("<?php namespace { class Foo {} }");

    assert_ne!(implicit, explicit, "Explicit and implicit global are syntactically different");
}

#[test]
fn test_global_then_namespaced() {
    let fp = get_fingerprint(
        "<?php
        class Global {}

        namespace App {
            class Namespaced {}
        }",
    );

    assert_ne!(fp, 0, "Global then namespaced should have fingerprint");
}

#[test]
fn test_namespace_with_class() {
    let global = get_fingerprint("<?php class User {}");
    let namespaced = get_fingerprint("<?php namespace App; class User {}");

    assert_ne!(global, namespaced, "Namespaced class should differ from global");
}

#[test]
fn test_namespace_with_interface() {
    let global = get_fingerprint("<?php interface Repository {}");
    let namespaced = get_fingerprint("<?php namespace App; interface Repository {}");

    assert_ne!(global, namespaced, "Namespaced interface should differ from global");
}

#[test]
fn test_namespace_with_trait() {
    let global = get_fingerprint("<?php trait Timestampable {}");
    let namespaced = get_fingerprint("<?php namespace App; trait Timestampable {}");

    assert_ne!(global, namespaced, "Namespaced trait should differ from global");
}

#[test]
fn test_namespace_with_enum() {
    let global = get_fingerprint("<?php enum Status {}");
    let namespaced = get_fingerprint("<?php namespace App; enum Status {}");

    assert_ne!(global, namespaced, "Namespaced enum should differ from global");
}

#[test]
fn test_namespace_with_function() {
    let global = get_fingerprint("<?php function helper() {}");
    let namespaced = get_fingerprint("<?php namespace App; function helper() {}");

    assert_ne!(global, namespaced, "Namespaced function should differ from global");
}

#[test]
fn test_namespace_with_constant() {
    let global = get_fingerprint("<?php const VERSION = '1.0';");
    let namespaced = get_fingerprint("<?php namespace App; const VERSION = '1.0';");

    assert_ne!(global, namespaced, "Namespaced constant should differ from global");
}

#[test]
fn test_multiple_classes_in_namespace() {
    let one = get_fingerprint("<?php namespace App; class A {}");
    let two = get_fingerprint("<?php namespace App; class A {} class B {}");

    assert_ne!(one, two, "Adding class in namespace should change fingerprint");
}

#[test]
fn test_mixed_symbols_in_namespace() {
    let fp = get_fingerprint(
        "<?php
        namespace App;

        class MyClass {}
        interface MyInterface {}
        trait MyTrait {}
        enum MyEnum {}
        function myFunction() {}
        const MY_CONSTANT = 1;",
    );

    assert_ne!(fp, 0, "Mixed symbols in namespace should have fingerprint");
}

#[test]
fn test_nested_function_inherits_namespace() {
    let global = get_fingerprint(
        "<?php
        function outer() {
            function inner() {}
        }",
    );
    let namespaced = get_fingerprint(
        "<?php
        namespace App;
        function outer() {
            function inner() {}
        }",
    );

    assert_ne!(global, namespaced, "Nested functions should inherit namespace");
}

#[test]
fn test_nested_function_multiple_levels() {
    let fp = get_fingerprint(
        "<?php
        namespace App\\Services;

        function level1() {
            function level2() {
                function level3() {}
            }
        }",
    );

    assert_ne!(fp, 0, "Multi-level nested functions in namespace should work");
}

#[test]
fn test_namespaced_class_extends() {
    let fp = get_fingerprint(
        "<?php
        namespace App;
        class Child extends Parent {}",
    );

    assert_ne!(fp, 0, "Namespaced class with extends should have fingerprint");
}

#[test]
fn test_namespaced_class_implements() {
    let fp = get_fingerprint(
        "<?php
        namespace App;
        class Implementation implements Interface1, Interface2 {}",
    );

    assert_ne!(fp, 0, "Namespaced class with implements should have fingerprint");
}

#[test]
fn test_namespaced_trait_usage() {
    let fp = get_fingerprint(
        "<?php
        namespace App;
        class MyClass {
            use Trait1, Trait2;
        }",
    );

    assert_ne!(fp, 0, "Namespaced class with traits should have fingerprint");
}

#[test]
fn test_use_statement_added() {
    let without = get_fingerprint("<?php namespace App; class Foo {}");
    let with = get_fingerprint("<?php namespace App; use Some\\Thing; class Foo {}");

    assert_ne!(without, with, "Adding use statement should change fingerprint");
}

#[test]
fn test_use_statement_change() {
    let use1 = get_fingerprint("<?php namespace App; use Lib\\A; class Foo {}");
    let use2 = get_fingerprint("<?php namespace App; use Lib\\B; class Foo {}");

    assert_ne!(use1, use2, "Changing use statement should change fingerprint");
}

#[test]
fn test_multiple_use_statements() {
    let one = get_fingerprint("<?php namespace App; use Lib\\A; class Foo {}");
    let two = get_fingerprint("<?php namespace App; use Lib\\A; use Lib\\B; class Foo {}");

    assert_ne!(one, two, "Adding use statement should change fingerprint");
}

#[test]
fn test_use_statement_order() {
    let order1 = get_fingerprint("<?php namespace App; use Lib\\A; use Lib\\B; class Foo {}");
    let order2 = get_fingerprint("<?php namespace App; use Lib\\B; use Lib\\A; class Foo {}");

    assert_ne!(order1, order2, "Use statement order should matter");
}

#[test]
fn test_use_with_alias() {
    let without = get_fingerprint("<?php namespace App; use Lib\\Thing; class Foo {}");
    let with = get_fingerprint("<?php namespace App; use Lib\\Thing as Alias; class Foo {}");

    assert_ne!(without, with, "Use alias should change fingerprint");
}

#[test]
fn test_use_alias_change() {
    let alias1 = get_fingerprint("<?php namespace App; use Lib\\Thing as A; class Foo {}");
    let alias2 = get_fingerprint("<?php namespace App; use Lib\\Thing as B; class Foo {}");

    assert_ne!(alias1, alias2, "Changing use alias should change fingerprint");
}

#[test]
fn test_grouped_use_statements() {
    let separate = get_fingerprint("<?php namespace App; use Lib\\A; use Lib\\B; class Foo {}");
    let grouped = get_fingerprint("<?php namespace App; use Lib\\{A, B}; class Foo {}");

    assert_ne!(separate, grouped, "Grouped and separate use are syntactically different");
}

#[test]
fn test_use_function() {
    let without = get_fingerprint("<?php namespace App; function foo() {}");
    let with = get_fingerprint("<?php namespace App; use function Lib\\helper; function foo() {}");

    assert_ne!(without, with, "Use function should change fingerprint");
}

#[test]
fn test_use_const() {
    let without = get_fingerprint("<?php namespace App; const X = 1;");
    let with = get_fingerprint("<?php namespace App; use const Lib\\CONSTANT; const X = 1;");

    assert_ne!(without, with, "Use const should change fingerprint");
}

#[test]
fn test_mixed_use_types() {
    let fp = get_fingerprint(
        "<?php
        namespace App;

        use Lib\\ClassA;
        use function Lib\\helper;
        use const Lib\\VERSION;

        class Foo {}",
    );

    assert_ne!(fp, 0, "Mixed use types should have fingerprint");
}

#[test]
fn test_namespace_with_all_features() {
    let fp = get_fingerprint(
        "<?php
        namespace App\\Models;

        use App\\Traits\\Timestampable;
        use Illuminate\\Database\\Eloquent\\Model;
        use function App\\Helpers\\formatDate;
        use const App\\Config\\VERSION;

        #[Entity]
        class User extends Model implements JsonSerializable {
            use Timestampable;

            public function __construct() {}
        }",
    );

    assert_ne!(fp, 0, "Complex namespaced code should have fingerprint");
}

#[test]
fn test_relative_namespace_references() {
    let absolute = get_fingerprint("<?php namespace App; class Foo extends \\Base\\Parent {}");
    let relative = get_fingerprint("<?php namespace App; class Foo extends Parent {}");

    assert_ne!(absolute, relative, "Absolute vs relative namespace reference should differ");
}

#[test]
fn test_namespace_keyword_in_code() {
    let fp = get_fingerprint(
        "<?php
        namespace App;

        class Foo {
            public function getNamespace() {
                return __NAMESPACE__;
            }
        }",
    );

    assert_ne!(fp, 0, "Namespace keyword in code should have fingerprint");
}

#[test]
fn test_empty_namespace_declaration() {
    let fp1 = get_fingerprint("<?php namespace Foo;");
    let fp2 = get_fingerprint("<?php namespace Bar;");

    assert_ne!(fp1, fp2, "Different empty namespaces should differ");
}

#[test]
fn test_namespace_then_empty_namespace() {
    let fp = get_fingerprint(
        "<?php
        namespace Foo;
        class A {}

        namespace Bar;
        // Empty

        namespace Baz;
        class B {}",
    );

    assert_ne!(fp, 0, "Empty namespace between non-empty should work");
}

#[test]
fn test_function_in_control_flow_inherits_namespace() {
    let fp = get_fingerprint(
        "<?php
        namespace App;

        if (true) {
            function conditionalFunc() {}
        }",
    );

    assert_ne!(fp, 0, "Function in if block should inherit namespace");
}

#[test]
fn test_nested_namespace_scoping() {
    let fp = get_fingerprint(
        "<?php
        namespace Outer;

        function outer() {
            // This function is in Outer namespace
            function inner() {
                // This is also in Outer namespace
            }
        }",
    );

    assert_ne!(fp, 0, "Nested functions should share namespace");
}
