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
fn test_whitespace_changes_ignored() {
    let compact = get_fingerprint("<?php class Foo{}");
    let spaced = get_fingerprint("<?php class Foo {  }");
    let multiline = get_fingerprint("<?php\nclass Foo {\n\n}");

    assert_eq!(compact, spaced, "Extra spaces should be ignored");
    assert_eq!(compact, multiline, "Newlines should be ignored");
}

#[test]
fn test_indentation_changes_ignored() {
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
fn test_trailing_commas_ignored() {
    let without = get_fingerprint("<?php $x = [1, 2, 3];");
    let with = get_fingerprint("<?php $x = [1, 2, 3,];");

    assert_eq!(without, with, "Trailing commas should be ignored");
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
fn test_method_name_change() {
    let foo = get_fingerprint("<?php class A { function foo() {} }");
    let bar = get_fingerprint("<?php class A { function bar() {} }");

    assert_ne!(foo, bar, "Method name change should change fingerprint");
}

#[test]
fn test_property_name_change() {
    let foo = get_fingerprint("<?php class A { public $foo; }");
    let bar = get_fingerprint("<?php class A { public $bar; }");

    assert_ne!(foo, bar, "Property name change should change fingerprint");
}

#[test]
fn test_parameter_addition() {
    let no_param = get_fingerprint("<?php function foo() {}");
    let with_param = get_fingerprint("<?php function foo($x) {}");

    assert_ne!(no_param, with_param, "Adding parameter should change fingerprint");
}

#[test]
fn test_parameter_name_change() {
    let x = get_fingerprint("<?php function foo($x) {}");
    let y = get_fingerprint("<?php function foo($y) {}");

    assert_ne!(x, y, "Parameter name change should change fingerprint");
}

#[test]
fn test_return_type_addition() {
    let no_type = get_fingerprint("<?php function foo() {}");
    let with_type = get_fingerprint("<?php function foo(): int {}");

    assert_ne!(no_type, with_type, "Adding return type should change fingerprint");
}

#[test]
fn test_return_type_change() {
    let int_type = get_fingerprint("<?php function foo(): int {}");
    let string_type = get_fingerprint("<?php function foo(): string {}");

    assert_ne!(int_type, string_type, "Return type change should change fingerprint");
}

#[test]
fn test_visibility_change() {
    let public = get_fingerprint("<?php class A { public function foo() {} }");
    let private = get_fingerprint("<?php class A { private function foo() {} }");

    assert_ne!(public, private, "Visibility change should change fingerprint");
}

#[test]
fn test_static_modifier_addition() {
    let instance = get_fingerprint("<?php class A { public function foo() {} }");
    let static_method = get_fingerprint("<?php class A { public static function foo() {} }");

    assert_ne!(instance, static_method, "Adding static modifier should change fingerprint");
}

#[test]
fn test_abstract_modifier_addition() {
    let concrete = get_fingerprint("<?php class A { public function foo() {} }");
    let abstract_method = get_fingerprint("<?php abstract class A { abstract public function foo(); }");

    assert_ne!(concrete, abstract_method, "Adding abstract modifier should change fingerprint");
}

#[test]
fn test_class_extends_addition() {
    let no_extends = get_fingerprint("<?php class Foo {}");
    let with_extends = get_fingerprint("<?php class Foo extends Bar {}");

    assert_ne!(no_extends, with_extends, "Adding extends should change fingerprint");
}

#[test]
fn test_class_implements_addition() {
    let no_implements = get_fingerprint("<?php class Foo {}");
    let with_implements = get_fingerprint("<?php class Foo implements Bar {}");

    assert_ne!(no_implements, with_implements, "Adding implements should change fingerprint");
}

#[test]
fn test_interface_extends_addition() {
    let no_extends = get_fingerprint("<?php interface Foo {}");
    let with_extends = get_fingerprint("<?php interface Foo extends Bar {}");

    assert_ne!(no_extends, with_extends, "Adding interface extends should change fingerprint");
}

#[test]
fn test_use_trait_addition() {
    let no_trait = get_fingerprint("<?php class Foo {}");
    let with_trait = get_fingerprint("<?php class Foo { use Bar; }");

    assert_ne!(no_trait, with_trait, "Adding trait use should change fingerprint");
}

#[test]
fn test_constant_value_change() {
    let value_1 = get_fingerprint("<?php const X = 1;");
    let value_2 = get_fingerprint("<?php const X = 2;");

    assert_ne!(value_1, value_2, "Constant value change should change fingerprint");
}

#[test]
fn test_enum_case_addition() {
    let one_case = get_fingerprint("<?php enum Status { case Active; }");
    let two_cases = get_fingerprint("<?php enum Status { case Active; case Inactive; }");

    assert_ne!(one_case, two_cases, "Adding enum case should change fingerprint");
}

#[test]
fn test_enum_backed_type_change() {
    let string_backed = get_fingerprint("<?php enum Status: string { case Active = 'active'; }");
    let int_backed = get_fingerprint("<?php enum Status: int { case Active = 1; }");

    assert_ne!(string_backed, int_backed, "Enum backed type change should change fingerprint");
}

#[test]
fn test_nullable_type_addition() {
    let not_null = get_fingerprint("<?php function foo(string $x) {}");
    let nullable = get_fingerprint("<?php function foo(?string $x) {}");

    assert_ne!(not_null, nullable, "Adding nullable type should change fingerprint");
}

#[test]
fn test_union_type_addition() {
    let single = get_fingerprint("<?php function foo(string $x) {}");
    let union = get_fingerprint("<?php function foo(string|int $x) {}");

    assert_ne!(single, union, "Adding union type should change fingerprint");
}

#[test]
fn test_intersection_type() {
    let single = get_fingerprint("<?php function foo(A $x) {}");
    let intersection = get_fingerprint("<?php function foo(A&B $x) {}");

    assert_ne!(single, intersection, "Adding intersection type should change fingerprint");
}

#[test]
fn test_function_body_changes() {
    let body1 = get_fingerprint("<?php function foo() { return 1; }");
    let body2 = get_fingerprint("<?php function foo() { return 2; }");

    assert_ne!(body1, body2, "Function body change should change fingerprint");
}

#[test]
fn test_adding_statement_to_body() {
    let one_stmt = get_fingerprint("<?php function foo() { $x = 1; }");
    let two_stmts = get_fingerprint("<?php function foo() { $x = 1; $y = 2; }");

    assert_ne!(one_stmt, two_stmts, "Adding statement should change fingerprint");
}

#[test]
fn test_removing_statement_from_body() {
    let two_stmts = get_fingerprint("<?php function foo() { $x = 1; $y = 2; }");
    let one_stmt = get_fingerprint("<?php function foo() { $x = 1; }");

    assert_ne!(two_stmts, one_stmt, "Removing statement should change fingerprint");
}

#[test]
fn test_reordering_statements() {
    let order1 = get_fingerprint("<?php function foo() { $x = 1; $y = 2; }");
    let order2 = get_fingerprint("<?php function foo() { $y = 2; $x = 1; }");

    assert_ne!(order1, order2, "Statement order matters");
}

#[test]
fn test_changing_variable_names_in_body() {
    let var_x = get_fingerprint("<?php function foo() { $x = 1; }");
    let var_y = get_fingerprint("<?php function foo() { $y = 1; }");

    assert_ne!(var_x, var_y, "Variable name changes should change fingerprint");
}

#[test]
fn test_operator_change() {
    let addition = get_fingerprint("<?php $x = $a + $b;");
    let subtraction = get_fingerprint("<?php $x = $a - $b;");

    assert_ne!(addition, subtraction, "Operator change should change fingerprint");
}

#[test]
fn test_literal_value_change() {
    let value_1 = get_fingerprint("<?php $x = 1;");
    let value_2 = get_fingerprint("<?php $x = 2;");

    assert_ne!(value_1, value_2, "Literal value change should change fingerprint");
}

#[test]
fn test_string_content_change() {
    let str1 = get_fingerprint("<?php $x = 'hello';");
    let str2 = get_fingerprint("<?php $x = 'world';");

    assert_ne!(str1, str2, "String content change should change fingerprint");
}

#[test]
fn test_array_element_addition() {
    let one = get_fingerprint("<?php $x = [1];");
    let two = get_fingerprint("<?php $x = [1, 2];");

    assert_ne!(one, two, "Adding array element should change fingerprint");
}

#[test]
fn test_nullable_vs_union_null() {
    let question_mark = get_fingerprint("<?php function foo(?string $x) {}");
    let union_null = get_fingerprint("<?php function foo(string|null $x) {}");

    assert_ne!(question_mark, union_null, "?T and T|null are syntactically different");
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
fn test_final_modifier_addition() {
    let regular = get_fingerprint("<?php class A { public function foo() {} }");
    let final_method = get_fingerprint("<?php class A { final public function foo() {} }");

    assert_ne!(regular, final_method, "Adding final should change fingerprint");
}

#[test]
fn test_readonly_property_addition() {
    let regular = get_fingerprint("<?php class A { public string $x; }");
    let readonly = get_fingerprint("<?php class A { public readonly string $x; }");

    assert_ne!(regular, readonly, "Adding readonly should change fingerprint");
}

#[test]
fn test_readonly_class() {
    let regular = get_fingerprint("<?php class A {}");
    let readonly = get_fingerprint("<?php readonly class A {}");

    assert_ne!(regular, readonly, "Making class readonly should change fingerprint");
}

#[test]
fn test_parameter_default_value_addition() {
    let without = get_fingerprint("<?php function foo($x) {}");
    let with = get_fingerprint("<?php function foo($x = 1) {}");

    assert_ne!(without, with, "Adding default value should change fingerprint");
}

#[test]
fn test_parameter_default_value_change() {
    let default_1 = get_fingerprint("<?php function foo($x = 1) {}");
    let default_2 = get_fingerprint("<?php function foo($x = 2) {}");

    assert_ne!(default_1, default_2, "Changing default value should change fingerprint");
}

#[test]
fn test_property_default_value() {
    let without = get_fingerprint("<?php class A { public $x; }");
    let with = get_fingerprint("<?php class A { public $x = 1; }");

    assert_ne!(without, with, "Adding property default should change fingerprint");
}

#[test]
fn test_combined_changes() {
    let original = get_fingerprint("<?php class Foo { public function bar() {} }");
    let modified = get_fingerprint("<?php abstract class Foo { abstract public static function bar(): int; }");

    assert_ne!(original, modified, "Combined changes should change fingerprint");
}

#[test]
fn test_cosmetic_vs_semantic() {
    let code = "<?php class Foo { public function bar(): int { return 1; } }";
    let whitespace_changed =
        "<?php\n\nclass   Foo\n{\n    public    function   bar()  :  int\n    {\n        return   1;\n    }\n}";
    let semantic_changed = "<?php class Foo { public function bar(): string { return '1'; } }";

    let original = get_fingerprint(code);
    let cosmetic = get_fingerprint(whitespace_changed);
    let semantic = get_fingerprint(semantic_changed);

    assert_eq!(original, cosmetic, "Cosmetic changes should not affect fingerprint");
    assert_ne!(original, semantic, "Semantic changes should affect fingerprint");
}
