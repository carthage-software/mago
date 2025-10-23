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
fn test_anonymous_class_inside_anonymous_class() {
    let simple = get_fingerprint("<?php $x = new class {};");
    let nested = get_fingerprint("<?php $x = new class { public function foo() { $y = new class {}; } };");

    assert_ne!(simple, nested, "Nested anonymous class should change fingerprint");
}

#[test]
fn test_nested_anonymous_class_with_extends() {
    let without = get_fingerprint("<?php $x = new class { function m() { return new class {}; } };");
    let with = get_fingerprint("<?php $x = new class { function m() { return new class extends Base {}; } };");

    assert_ne!(without, with, "Extends in nested anonymous class should change fingerprint");
}

#[test]
fn test_deeply_nested_anonymous_classes() {
    let fp = get_fingerprint(
        "<?php
        $a = new class {
            function level1() {
                $b = new class {
                    function level2() {
                        $c = new class extends Deep {};
                    }
                };
            }
        };",
    );

    assert_ne!(fp, 0, "Deeply nested anonymous classes should have fingerprint");
}

#[test]
fn test_function_inside_function() {
    let simple = get_fingerprint("<?php function outer() {}");
    let nested = get_fingerprint("<?php function outer() { function inner() {} }");

    assert_ne!(simple, nested, "Nested function should change fingerprint");
}

#[test]
fn test_deeply_nested_functions() {
    let fp = get_fingerprint(
        "<?php
        function level1() {
            function level2() {
                function level3() {
                    function level4() {}
                }
            }
        }",
    );

    assert_ne!(fp, 0, "Deeply nested functions should have fingerprint");
}

#[test]
fn test_function_in_if_block() {
    let without = get_fingerprint("<?php if (true) {}");
    let with = get_fingerprint("<?php if (true) { function foo() {} }");

    assert_ne!(without, with, "Function in if block should change fingerprint");
}

#[test]
fn test_function_in_foreach_block() {
    let without = get_fingerprint("<?php foreach ($items as $item) {}");
    let with = get_fingerprint("<?php foreach ($items as $item) { function process() {} }");

    assert_ne!(without, with, "Function in foreach should change fingerprint");
}

#[test]
fn test_function_in_try_catch() {
    let fp = get_fingerprint(
        "<?php
        try {
            function tryFunc() {}
        } catch (Exception $e) {
            function catchFunc() {}
        } finally {
            function finallyFunc() {}
        }",
    );

    assert_ne!(fp, 0, "Functions in try-catch-finally should have fingerprint");
}

#[test]
fn test_anonymous_class_in_array() {
    let without = get_fingerprint("<?php $arr = [];");
    let with = get_fingerprint("<?php $arr = [new class {}, new class {}];");

    assert_ne!(without, with, "Anonymous classes in array should change fingerprint");
}

#[test]
fn test_anonymous_class_as_array_value() {
    let simple = get_fingerprint("<?php $arr = ['key' => null];");
    let with_anon = get_fingerprint("<?php $arr = ['key' => new class {}];");

    assert_ne!(simple, with_anon, "Anonymous class as array value should change fingerprint");
}

#[test]
fn test_anonymous_class_in_function_call() {
    let simple = get_fingerprint("<?php foo(null);");
    let with_anon = get_fingerprint("<?php foo(new class {});");

    assert_ne!(simple, with_anon, "Anonymous class as argument should change fingerprint");
}

#[test]
fn test_anonymous_class_in_return() {
    let simple = get_fingerprint("<?php function factory() { return null; }");
    let with_anon = get_fingerprint("<?php function factory() { return new class {}; }");

    assert_ne!(simple, with_anon, "Returning anonymous class should change fingerprint");
}

#[test]
fn test_anonymous_class_in_closure() {
    let simple = get_fingerprint("<?php $fn = function() { return null; };");
    let with_anon = get_fingerprint("<?php $fn = function() { return new class {}; };");

    assert_ne!(simple, with_anon, "Anonymous class in closure should change fingerprint");
}

#[test]
fn test_anonymous_class_in_arrow_function() {
    let simple = get_fingerprint("<?php $fn = fn() => null;");
    let with_anon = get_fingerprint("<?php $fn = fn() => new class {};");

    assert_ne!(simple, with_anon, "Anonymous class in arrow function should change fingerprint");
}

#[test]
fn test_global_then_namespaced_code() {
    let fp = get_fingerprint(
        "<?php
        class GlobalClass {}
        namespace App;
        class NamespacedClass {}",
    );

    assert_ne!(fp, 0, "Mixed global and namespaced code should have fingerprint");
}

#[test]
fn test_multiple_namespace_declarations() {
    let fp = get_fingerprint(
        "<?php
        namespace App\\Models;
        class User {}

        namespace App\\Controllers;
        class UserController {}",
    );

    assert_ne!(fp, 0, "Multiple namespaces should have fingerprint");
}

#[test]
fn test_braced_namespaces() {
    let fp = get_fingerprint(
        "<?php
        namespace App\\A {
            class ClassA {}
        }

        namespace App\\B {
            class ClassB {}
        }",
    );

    assert_ne!(fp, 0, "Braced namespaces should have fingerprint");
}

#[test]
fn test_anonymous_class_in_namespace() {
    let global = get_fingerprint("<?php $x = new class extends Parent {};");
    let namespaced = get_fingerprint("<?php namespace Foo; $x = new class extends Parent {};");

    assert_ne!(global, namespaced, "Namespace affects anonymous class parent resolution");
}

#[test]
fn test_empty_class() {
    let fp = get_fingerprint("<?php class Empty {}");
    assert_ne!(fp, 0, "Empty class should have fingerprint");
}

#[test]
fn test_empty_interface() {
    let fp = get_fingerprint("<?php interface Empty {}");
    assert_ne!(fp, 0, "Empty interface should have fingerprint");
}

#[test]
fn test_empty_trait() {
    let fp = get_fingerprint("<?php trait Empty {}");
    assert_ne!(fp, 0, "Empty trait should have fingerprint");
}

#[test]
fn test_empty_enum() {
    let fp = get_fingerprint("<?php enum Empty {}");
    assert_ne!(fp, 0, "Empty enum should have fingerprint");
}

#[test]
fn test_empty_function() {
    let fp = get_fingerprint("<?php function empty() {}");
    assert_ne!(fp, 0, "Empty function should have fingerprint");
}

#[test]
fn test_function_with_only_semicolon() {
    let fp = get_fingerprint("<?php function foo() { ; }");
    assert_ne!(fp, 0, "Function with semicolon should have fingerprint");
}

#[test]
fn test_class_with_method_with_nested_function_with_anonymous_class() {
    let fp = get_fingerprint(
        "<?php
        class Outer {
            public function method() {
                function nested() {
                    $anon = new class {
                        public function deep() {}
                    };
                }
            }
        }",
    );

    assert_ne!(fp, 0, "Complex nesting should have fingerprint");
}

#[test]
fn test_extreme_nesting_depth() {
    let fp = get_fingerprint(
        "<?php
        namespace Level1;
        class C1 {
            function m1() {
                function f1() {
                    $x = new class {
                        function m2() {
                            function f2() {
                                $y = new class extends Base {};
                            }
                        }
                    };
                }
            }
        }",
    );

    assert_ne!(fp, 0, "Extreme nesting should have fingerprint");
}

#[test]
fn test_five_level_function_nesting() {
    let fp = get_fingerprint(
        "<?php
        function level1() {
            function level2() {
                function level3() {
                    function level4() {
                        function level5() {}
                    }
                }
            }
        }",
    );

    assert_ne!(fp, 0, "Five levels of function nesting should have fingerprint");
}

#[test]
fn test_identical_code_same_fingerprint() {
    let code = "<?php namespace App; class Test { function method() {} }";
    let fp1 = get_fingerprint(code);
    let fp2 = get_fingerprint(code);

    assert_eq!(fp1, fp2, "Identical code should produce identical fingerprints");
}

#[test]
fn test_whitespace_ignored_in_complex_nesting() {
    let compact = get_fingerprint("<?php namespace A;class B{function c(){function d(){$x=new class{};}}}");
    let spaced = get_fingerprint(
        "<?php
        namespace A;

        class B {
            function c() {
                function d() {
                    $x = new class {};
                }
            }
        }",
    );

    assert_eq!(compact, spaced, "Whitespace should be ignored in complex nesting");
}

#[test]
fn test_trailing_comma_in_arrays() {
    let without = get_fingerprint("<?php $x = [1, 2, 3];");
    let with = get_fingerprint("<?php $x = [1, 2, 3,];");

    assert_eq!(without, with, "Trailing comma in array should be ignored");
}

#[test]
fn test_trailing_comma_in_parameters() {
    let without = get_fingerprint("<?php function foo($a, $b) {}");
    let with = get_fingerprint("<?php function foo($a, $b,) {}");

    assert_eq!(without, with, "Trailing comma in parameters should be ignored");
}

#[test]
fn test_fingerprint_deterministic_multiple_runs() {
    let code = "<?php class Foo { public function bar() {} }";
    let fingerprints: Vec<u64> = (0..10).map(|_| get_fingerprint(code)).collect();

    for fp in &fingerprints[1..] {
        assert_eq!(*fp, fingerprints[0], "Fingerprints should be deterministic");
    }
}

#[test]
fn test_complex_code_stability() {
    let code = "<?php
        namespace App\\Models;

        use App\\Traits\\Timestampable;

        #[Entity]
        class User extends BaseModel implements JsonSerializable {
            use Timestampable;

            /** @var string */
            private $name;

            public function __construct(
                public readonly string $id,
                private int $age = 0
            ) {}

            /** @return string */
            public function getName(): string {
                return $this->name;
            }
        }";

    let fp1 = get_fingerprint(code);
    let fp2 = get_fingerprint(code);
    let fp3 = get_fingerprint(code);

    assert_eq!(fp1, fp2, "Complex code should have stable fingerprint");
    assert_eq!(fp2, fp3, "Complex code should have stable fingerprint");
}

#[test]
fn test_function_in_while_block() {
    let without = get_fingerprint("<?php while (true) {}");
    let with = get_fingerprint("<?php while (true) { function loopFunc() {} }");

    assert_ne!(without, with, "Function in while should change fingerprint");
}

#[test]
fn test_function_in_switch_case() {
    let fp = get_fingerprint(
        "<?php
        switch ($x) {
            case 1:
                function case1() {}
                break;
            case 2:
                function case2() {}
                break;
        }",
    );

    assert_ne!(fp, 0, "Functions in switch cases should have fingerprint");
}

#[test]
fn test_function_in_match_arms() {
    let fp = get_fingerprint(
        "<?php
        match ($x) {
            1 => (function() { function foo() {} })(),
            default => null
        };",
    );

    assert_ne!(fp, 0, "Function in match should have fingerprint");
}

#[test]
fn test_anonymous_class_in_method_chain() {
    let simple = get_fingerprint("<?php $obj->method(null);");
    let with_anon = get_fingerprint("<?php $obj->method(new class {});");

    assert_ne!(simple, with_anon, "Anonymous class in method chain should change fingerprint");
}

#[test]
fn test_anonymous_class_chained_calls() {
    let fp = get_fingerprint("<?php (new class {})->method()->another();");
    assert_ne!(fp, 0, "Chained calls on anonymous class should have fingerprint");
}

#[test]
fn test_multiple_anonymous_classes_in_function() {
    let one = get_fingerprint("<?php function f() { $a = new class {}; }");
    let two = get_fingerprint("<?php function f() { $a = new class {}; $b = new class {}; }");

    assert_ne!(one, two, "Adding anonymous class should change fingerprint");
}

#[test]
fn test_different_anonymous_classes_same_function() {
    let extends_a = get_fingerprint("<?php function f() { $a = new class extends A {}; $b = new class {}; }");
    let extends_b = get_fingerprint("<?php function f() { $a = new class {}; $b = new class extends B {}; }");

    assert_ne!(extends_a, extends_b, "Different anonymous class extends should differ");
}

#[test]
fn test_multiple_classes_same_name_different_namespaces() {
    let fp = get_fingerprint(
        "<?php
        namespace A {
            class Foo {}
        }
        namespace B {
            class Foo {}
        }",
    );

    assert_ne!(fp, 0, "Same class name in different namespaces should work");
}

#[test]
fn test_function_and_method_same_name() {
    let fp = get_fingerprint(
        "<?php
        function foo() {}
        class A {
            public function foo() {}
        }",
    );

    assert_ne!(fp, 0, "Function and method with same name should work");
}

#[test]
fn test_global_constant_and_class_constant() {
    let fp = get_fingerprint(
        "<?php
        const X = 1;
        class A {
            const X = 2;
        }",
    );

    assert_ne!(fp, 0, "Global and class constant with same name should work");
}

#[test]
fn test_nested_ternary_operators() {
    let simple = get_fingerprint("<?php $x = $a ? $b : $c;");
    let nested = get_fingerprint("<?php $x = $a ? ($b ? $c : $d) : ($e ? $f : $g);");

    assert_ne!(simple, nested, "Nested ternary should change fingerprint");
}

#[test]
fn test_anonymous_class_in_ternary() {
    let simple = get_fingerprint("<?php $x = $a ? null : null;");
    let with_anon = get_fingerprint("<?php $x = $a ? new class {} : null;");

    assert_ne!(simple, with_anon, "Anonymous class in ternary should change fingerprint");
}

#[test]
fn test_enum_single_case() {
    let fp = get_fingerprint("<?php enum Single { case OnlyOne; }");
    assert_ne!(fp, 0, "Enum with single case should have fingerprint");
}

#[test]
fn test_enum_many_cases() {
    let fp = get_fingerprint(
        "<?php enum Many {
            case A; case B; case C; case D; case E;
            case F; case G; case H; case I; case J;
        }",
    );

    assert_ne!(fp, 0, "Enum with many cases should have fingerprint");
}

#[test]
fn test_backed_enum_negative_values() {
    let fp = get_fingerprint(
        "<?php enum Status: int {
            case Error = -1;
            case Unknown = 0;
            case Success = 1;
        }",
    );

    assert_ne!(fp, 0, "Backed enum with negative values should have fingerprint");
}

#[test]
fn test_variadic_by_ref_parameter() {
    let regular = get_fingerprint("<?php function foo(...$args) {}");
    let by_ref = get_fingerprint("<?php function foo(&...$args) {}");

    assert_ne!(regular, by_ref, "By-ref variadic should differ from regular variadic");
}

#[test]
fn test_typed_variadic_parameter() {
    let untyped = get_fingerprint("<?php function foo(...$args) {}");
    let typed = get_fingerprint("<?php function foo(int ...$args) {}");

    assert_ne!(untyped, typed, "Typed variadic should differ from untyped");
}

#[test]
fn test_mixed_parameter_styles() {
    let fp = get_fingerprint("<?php function foo($a, int $b, int &$c, int $d = 0, ...$rest) {}");
    assert_ne!(fp, 0, "Mixed parameter styles should have fingerprint");
}
