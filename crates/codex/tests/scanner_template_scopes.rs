#![allow(clippy::unwrap_used)]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::scanner::scan_program;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::reference::TReference;
use mago_codex::ttype::union::TUnion;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;
use mago_word::word;

fn scan(source: &str) -> CodebaseMetadata {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"scopes.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "{:?}", program.errors);
    let names = NameResolver::new(&arena).resolve(program);
    scan_program(&arena, &file, program, &names, PHPVersion::LATEST)
}

fn return_type(function: &FunctionLikeMetadata) -> &TUnion {
    &function.return_type_metadata.as_ref().unwrap().type_union
}

fn assert_class(ty: &TUnion, expected: &str) {
    let TAtomic::Reference(TReference::Symbol { name, .. }) = ty.get_single() else {
        panic!("Expected class {expected}, got {ty:?}");
    };

    assert_eq!(*name, word(expected), "resolved class name");
}

fn assert_template(ty: &TUnion, expected: &str, parent: GenericParent) {
    let TAtomic::GenericParameter(template) = ty.get_single() else {
        panic!("Expected template {expected}, got {ty:?}");
    };

    assert_eq!(template.parameter_name, word(expected), "template name");
    assert_eq!(template.defining_entity, parent, "template scope");
}

#[test]
fn template_names_do_not_escape_into_sibling_declarations() {
    for source in [
        include_str!("cases/scanner_template_scopes/sibling_after_class.php"),
        include_str!("cases/scanner_template_scopes/sibling_after_function.php"),
    ] {
        let codebase = scan(source);
        assert_class(return_type(codebase.get_function(b"Other\\control").unwrap()), "Other\\Item");
        assert_class(return_type(codebase.get_function(b"App\\sibling").unwrap()), "App\\Item");
    }
}

#[test]
fn template_names_do_not_replace_imports_in_sibling_methods() {
    let codebase = scan(include_str!("cases/scanner_template_scopes/sibling_methods.php"));
    assert_template(
        return_type(codebase.get_method(b"App\\Container", b"generic").unwrap()),
        "Item",
        GenericParent::FunctionLike((word("app\\container"), word("generic"))),
    );

    assert_class(return_type(codebase.get_method(b"App\\Container", b"sibling").unwrap()), "Lib\\Item");
    assert_class(return_type(codebase.get_function(b"App\\after_class").unwrap()), "Lib\\Item");
}

#[test]
fn nested_templates_keep_their_type_context() {
    let codebase = scan(include_str!("cases/scanner_template_scopes/nested_templates.php"));
    let class_parent = GenericParent::ClassLike(word("app\\container"));
    let method = codebase.get_method(b"App\\Container", b"generic").unwrap();
    assert_template(&method.template_types[&word("U")].constraint, "T", class_parent);
    assert_template(return_type(method), "U", GenericParent::FunctionLike((word("app\\container"), word("generic"))));
    assert_template(return_type(codebase.get_function(b"App\\nested").unwrap()), "T", class_parent);
    assert_template(return_type(codebase.get_method(b"App\\Container", b"sibling").unwrap()), "T", class_parent);
}

#[test]
fn forced_class_names_keep_imports_when_a_template_has_the_same_name() {
    let codebase = scan(include_str!("cases/scanner_template_scopes/forced_class_names.php"));
    for name in [b"magic".as_slice(), b"value"] {
        let ty = return_type(codebase.get_method(b"App\\Container", name).unwrap());
        let TAtomic::Reference(TReference::Member { class_like_name, .. }) = ty.get_single() else {
            panic!("Expected a class constant, got {ty:?}");
        };

        assert_eq!(*class_like_name, word("Lib\\Source"), "resolved class name");
    }
}

#[test]
fn template_bounds_and_defaults_keep_imports_before_the_template_is_bound() {
    let codebase = scan(include_str!("cases/scanner_template_scopes/template_bounds_and_defaults.php"));
    let class = codebase.get_class_like(b"App\\Container").unwrap();
    let function = codebase.get_function(b"App\\generic").unwrap();
    for template in [&class.template_types[&word("Source")], &function.template_types[&word("Source")]] {
        assert_class(&template.constraint, "Lib\\Source");
        assert_class(template.default.as_ref().unwrap(), "Lib\\Source");
    }
}

#[test]
fn imported_type_sources_agree_in_class_and_method_contexts() {
    let codebase = scan(include_str!("cases/scanner_template_scopes/imported_type_sources.php"));
    let class = codebase.get_class_like(b"App\\Container").unwrap();
    assert_eq!(class.imported_type_aliases[&word("LocalPayload")].0, word("lib\\source"), "import source");
    for name in [b"magic".as_slice(), b"value"] {
        let ty = return_type(codebase.get_method(b"App\\Container", name).unwrap());
        let TAtomic::Alias(alias) = ty.get_single() else {
            panic!("Expected an imported type alias, got {ty:?}");
        };

        assert_eq!(alias.get_class_name(), word("lib\\source"), "alias source");
        assert_eq!(alias.get_alias_name(), word("Payload"), "alias name");
    }
}
