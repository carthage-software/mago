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
    program.fingerprint_with_hasher(&mut hasher, &resolved_names, &options);
    hasher.finish()
}

fn get_fingerprint_with_options(code: &'static str, options: FingerprintOptions) -> u64 {
    let arena = Bump::new();
    let file = File::ephemeral("test.php".into(), code.into());
    let (program, error) = mago_syntax::parser::parse_file(&arena, &file);
    assert!(error.is_none(), "Parse error: {:?}", error);

    let resolved_names = mago_names::resolver::NameResolver::new(&arena).resolve(program);

    use ahash::AHasher;
    use std::hash::Hasher;
    let mut hasher = AHasher::default();
    program.fingerprint_with_hasher(&mut hasher, &resolved_names, &options);
    hasher.finish()
}

#[test]
fn test_basic_class_fingerprint() {
    let fp = get_fingerprint("<?php class Foo {}");
    assert_ne!(fp, 0, "Fingerprint should be non-zero");
}

#[test]
fn test_basic_function_fingerprint() {
    let fp = get_fingerprint("<?php function bar() {}");
    assert_ne!(fp, 0, "Fingerprint should be non-zero");
}

#[test]
fn test_empty_file_fingerprint() {
    let fp = get_fingerprint("<?php");
    assert_ne!(fp, 0, "Even empty file should have fingerprint");
}

#[test]
fn test_multiple_symbols() {
    let fp = get_fingerprint("<?php class Foo {} function bar() {}");
    assert_ne!(fp, 0, "Multiple symbols should have fingerprint");
}

#[test]
fn test_identical_code_same_fingerprint() {
    let code = "<?php class Foo { public function bar() {} }";
    let fp1 = get_fingerprint(code);
    let fp2 = get_fingerprint(code);

    assert_eq!(fp1, fp2, "Identical code should produce identical fingerprints");
}

#[test]
fn test_fingerprint_deterministic() {
    let code = "<?php namespace App; class User { public function getName(): string {} }";

    let fingerprints: Vec<u64> = (0..5).map(|_| get_fingerprint(code)).collect();

    for fp in &fingerprints[1..] {
        assert_eq!(*fp, fingerprints[0], "Fingerprints should be deterministic");
    }
}

#[test]
fn test_whitespace_ignored() {
    let compact = get_fingerprint("<?php class Foo{}");
    let spaced = get_fingerprint("<?php class Foo {  }");
    let multiline = get_fingerprint("<?php\nclass Foo {\n\n}");

    assert_eq!(compact, spaced, "Extra spaces should be ignored");
    assert_eq!(compact, multiline, "Newlines should be ignored");
}

#[test]
fn test_indentation_ignored() {
    let no_indent = get_fingerprint("<?php class Foo { function bar() {} }");
    let indented = get_fingerprint("<?php\nclass Foo {\n    function bar() {}\n}");
    let deep_indent = get_fingerprint("<?php\nclass Foo {\n        function bar() {}\n}");

    assert_eq!(no_indent, indented, "Indentation should be ignored");
    assert_eq!(no_indent, deep_indent, "Deep indentation should be ignored");
}

#[test]
fn test_empty_lines_ignored() {
    let no_empty = get_fingerprint("<?php\nclass Foo {}\nfunction bar() {}");
    let with_empty = get_fingerprint("<?php\n\n\nclass Foo {}\n\n\nfunction bar() {}");

    assert_eq!(no_empty, with_empty, "Empty lines should be ignored");
}

#[test]
fn test_regular_comments_ignored() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_line = get_fingerprint("<?php // Regular comment\nclass Foo {}");
    let with_block = get_fingerprint("<?php /* Block comment */ class Foo {}");

    assert_eq!(without, with_line, "Regular line comments should be ignored");
    assert_eq!(without, with_block, "Regular block comments should be ignored");
}

#[test]
fn test_important_comments_included() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_docblock = get_fingerprint("<?php /** @return string */ class Foo {}");

    assert_ne!(without, with_docblock, "Docblocks with @ should be included");
}

#[test]
fn test_mago_directives_included() {
    let without = get_fingerprint("<?php function foo() {}");
    let with_directive = get_fingerprint("<?php // @mago-ignore\nfunction foo() {}");

    assert_ne!(without, with_directive, "Mago directives should be included");
}

#[test]
fn test_custom_comment_patterns() {
    let code_with_custom = "<?php // #custom-directive\nclass Foo {}";

    let default_fp = get_fingerprint(code_with_custom);

    let options = FingerprintOptions::default().with_comment_patterns(&["#custom-"]);
    let custom_fp = get_fingerprint_with_options(code_with_custom, options);

    let without_comment = get_fingerprint("<?php class Foo {}");
    assert_eq!(default_fp, without_comment, "Default should ignore # pattern");
    assert_ne!(custom_fp, without_comment, "Custom # pattern should include comment");
}

#[test]
fn test_inline_content_excluded() {
    let without_tag = get_fingerprint("<?php class Foo {}");
    let with_tag = get_fingerprint("<?php class Foo {} ?>");

    assert_eq!(without_tag, with_tag, "Closing tags excluded");
}

#[test]
fn test_class_name_change() {
    let foo = get_fingerprint("<?php class Foo {}");
    let bar = get_fingerprint("<?php class Bar {}");

    assert_ne!(foo, bar, "Class name change should change fingerprint");
}

#[test]
fn test_function_name_change() {
    let foo = get_fingerprint("<?php function foo() {}");
    let bar = get_fingerprint("<?php function bar() {}");

    assert_ne!(foo, bar, "Function name change should change fingerprint");
}

#[test]
fn test_adding_method_to_class() {
    let without = get_fingerprint("<?php class Foo {}");
    let with = get_fingerprint("<?php class Foo { public function bar() {} }");

    assert_ne!(without, with, "Adding method should change fingerprint");
}

#[test]
fn test_parameter_type_change() {
    let int_type = get_fingerprint("<?php function foo(int $x) {}");
    let string_type = get_fingerprint("<?php function foo(string $x) {}");

    assert_ne!(int_type, string_type, "Parameter type change should change fingerprint");
}

#[test]
fn test_return_type_change() {
    let int_return = get_fingerprint("<?php function foo(): int {}");
    let string_return = get_fingerprint("<?php function foo(): string {}");

    assert_ne!(int_return, string_return, "Return type change should change fingerprint");
}

#[test]
fn test_namespace_affects_fingerprint() {
    let global = get_fingerprint("<?php class Foo {}");
    let namespaced = get_fingerprint("<?php namespace App; class Foo {}");

    assert_ne!(global, namespaced, "Namespace should affect fingerprint");
}

#[test]
fn test_different_namespaces_different_fingerprints() {
    let ns1 = get_fingerprint("<?php namespace App; class Foo {}");
    let ns2 = get_fingerprint("<?php namespace Lib; class Foo {}");

    assert_ne!(ns1, ns2, "Different namespaces should have different fingerprints");
}

#[test]
fn test_nullable_type_syntax_variations() {
    let question_mark = get_fingerprint("<?php function foo(?string $x) {}");
    let union_null = get_fingerprint("<?php function foo(string|null $x) {}");

    assert_ne!(question_mark, union_null, "?T and T|null are syntactically different");
}

#[test]
fn test_alternative_closing_tag_equivalence() {
    let semicolon = get_fingerprint("<?php echo 'test';");
    let closing_tag = get_fingerprint("<?php echo 'test' ?>");

    assert_eq!(semicolon, closing_tag, "Trailing ; should equal ?>");
}

#[test]
fn test_comparison_operator_equivalence() {
    let not_equal = get_fingerprint("<?php if ($x != $y) {}");
    let less_greater = get_fingerprint("<?php if ($x <> $y) {}");

    assert_eq!(not_equal, less_greater, "!= should equal <>");
}

#[test]
fn test_class_names_case_insensitive() {
    let lower = get_fingerprint("<?php class foo {}");
    let upper = get_fingerprint("<?php class FOO {}");
    let mixed = get_fingerprint("<?php class Foo {}");

    assert_eq!(lower, upper, "Class names are case-insensitive");
    assert_eq!(lower, mixed, "Class names are case-insensitive");
}

#[test]
fn test_keyword_case_insensitive() {
    let lower = get_fingerprint("<?php class Foo {}");
    let upper = get_fingerprint("<?php CLASS Foo {}");
    let mixed = get_fingerprint("<?php Class Foo {}");

    assert_eq!(lower, upper, "Keyword case should not matter");
    assert_eq!(lower, mixed, "Keyword case should not matter");
}

#[test]
fn test_full_class_with_methods_and_properties() {
    let fp = get_fingerprint(
        "<?php
        class User {
            private string $name;
            private int $age;

            public function __construct(string $name, int $age) {
                $this->name = $name;
                $this->age = $age;
            }

            public function getName(): string {
                return $this->name;
            }
        }",
    );

    assert_ne!(fp, 0, "Complex class should have fingerprint");
}

#[test]
fn test_namespaced_symbols_with_use_statements() {
    let fp = get_fingerprint(
        "<?php
        namespace App\\Models;

        use App\\Traits\\Timestampable;
        use Illuminate\\Database\\Eloquent\\Model;

        class User extends Model {
            use Timestampable;
        }",
    );

    assert_ne!(fp, 0, "Namespaced class with use statements should have fingerprint");
}

#[test]
fn test_enum_with_cases() {
    let fp = get_fingerprint(
        "<?php
        enum Status: string {
            case Active = 'active';
            case Inactive = 'inactive';
            case Pending = 'pending';
        }",
    );

    assert_ne!(fp, 0, "Enum with cases should have fingerprint");
}

#[test]
fn test_interface_with_methods() {
    let fp = get_fingerprint(
        "<?php
        interface Repository {
            public function find(int $id): ?Model;
            public function save(Model $model): void;
            public function delete(int $id): bool;
        }",
    );

    assert_ne!(fp, 0, "Interface with methods should have fingerprint");
}

#[test]
fn test_trait_with_methods() {
    let fp = get_fingerprint(
        "<?php
        trait Timestampable {
            protected ?DateTime $createdAt = null;

            public function setCreatedAt(DateTime $date): void {
                $this->createdAt = $date;
            }
        }",
    );

    assert_ne!(fp, 0, "Trait with methods and properties should have fingerprint");
}

#[test]
fn test_multiple_namespaces_in_one_file() {
    let fp = get_fingerprint(
        "<?php
        namespace App\\Models {
            class User {}
        }

        namespace App\\Controllers {
            class UserController {}
        }",
    );

    assert_ne!(fp, 0, "Multiple namespaces should have fingerprint");
}

#[test]
fn test_nested_functions() {
    let fp = get_fingerprint(
        "<?php
        function outer() {
            function inner() {
                function deeplyNested() {}
            }
        }",
    );

    assert_ne!(fp, 0, "Nested functions should have fingerprint");
}

#[test]
fn test_anonymous_class() {
    let fp = get_fingerprint(
        "<?php
        $obj = new class extends BaseClass implements SomeInterface {
            public function method() {}
        };",
    );

    assert_ne!(fp, 0, "Anonymous class should have fingerprint");
}

#[test]
fn test_attributes_on_class() {
    let fp = get_fingerprint(
        "<?php
        #[Attribute]
        #[Route('/api/users')]
        class UserController {
            #[Route('GET', '/')]
            public function index() {}
        }",
    );

    assert_ne!(fp, 0, "Class with attributes should have fingerprint");
}

#[test]
fn test_readonly_class() {
    let fp = get_fingerprint(
        "<?php
        readonly class Point {
            public function __construct(
                public int $x,
                public int $y,
            ) {}
        }",
    );

    assert_ne!(fp, 0, "Readonly class with promoted properties should have fingerprint");
}
