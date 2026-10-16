#![allow(clippy::unwrap_used)]
use super::*;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_database::file::File;
use mago_database::{Database, DatabaseConfiguration};
use mago_php_version::PHPVersion;
use mago_syntax::settings::ParserSettings;
use std::path::Path;
use std::sync::Arc;

fn server(files: &[(&str, &str)]) -> Server {
    let mut database = Database::new(DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]));
    let mut codebase = CodebaseMetadata::new();
    for (name, source) in files {
        let mut file = File::ephemeral(name.as_bytes().to_vec().into(), source.as_bytes().to_vec().into());
        if *name == "prelude.php" {
            file.file_type = mago_database::file::FileType::Builtin;
        }
        let arena = LocalArena::new();
        let program = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
        assert!(!program.has_errors(), "{source}");
        let names = NameResolver::new(&arena).resolve(program);
        codebase.extend(scan_program(&arena, &file, program, &names, PHPVersion::PHP85));
        database.add(file);
    }
    let mut references = SymbolReferences::new();
    populate_codebase(&mut codebase, &mut references, std::collections::HashSet::default(), HashSet::default());
    Server::new(
        database,
        codebase,
        references,
        crate::Settings {
            php_version: PHPVersion::PHP85,
            features: crate::Features { analyzer: false, linter: false, formatter: true },
            parser: ParserSettings::default(),
            analyzer: mago_analyzer::settings::Settings::default(),
            linter: mago_linter::settings::Settings::default(),
            formatter: mago_formatter::settings::FormatSettings::default(),
            plugin_registry: Arc::new(mago_analyzer::plugin::PluginRegistry::new()),
            excludes: crate::Excludes::default(),
        },
    )
}

#[test]
fn stubs_preserve_native_signatures_and_qualify_source_names() {
    let library = "<?php namespace Types; class Value {} interface Left {} interface Right {} const LIMIT = 7; namespace Contracts; use Types\\Value as Alias; use const Types\\LIMIT; interface Contract { function &Build(?Alias &$value, array $options = [LIMIT, Alias::class], int ...$extra): (\\Types\\Left&\\Types\\Right)|null; static function make(self $value): static; }";
    let server =
        server(&[("lib.php", library), ("c.php", "<?php namespace App; class C implements \\Contracts\\Contract {}")]);
    let stubs = server.get_method_stubs(b"App\\C").unwrap();
    assert_eq!(stubs.len(), 2);
    let text = stubs.join("\n");
    for expected in [
        "public function &Build",
        "?\\Types\\Value &$value",
        "\\Types\\LIMIT",
        "\\Types\\Value::class",
        "int ...$extra",
        "(\\Types\\Left&\\Types\\Right)|null",
        "public static function make(\\Contracts\\Contract $value): static",
    ] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
    assert_eq!(text.matches("throw new \\LogicException('Not implemented.');").count(), 2);
}

#[test]
fn stubs_handle_abstract_parents_enums_and_satisfied_methods() {
    let server = server(&[
        (
            "prelude.php",
            "<?php /** @enum-interface */ interface UnitEnum { public static function cases(): array; } /** @enum-interface */ interface BackedEnum extends UnitEnum { public static function from(int|string $value): static; public static function tryFrom(int|string $value): ?static; }",
        ),
        (
            "a.php",
            "<?php class Grand {} abstract class Base extends Grand { protected const DEFAULTS = 3; abstract protected function run(parent $value, int $default = self::DEFAULTS): self; } class C extends Base {} interface I { function text(): string; } enum E: int implements I { case A = 1; } class Done implements I { public function text(): string { return ''; } }",
        ),
    ]);
    let stubs = server.get_method_stubs(b"C").unwrap();
    assert_eq!(stubs.len(), 1);
    for expected in ["public function run(\\Grand $value", "\\Base::DEFAULTS", ": \\Base"] {
        assert!(stubs[0].contains(expected), "{}", stubs[0]);
    }
    assert_eq!(server.get_method_stubs(b"E").unwrap().len(), 1);
    assert!(server.get_method_stubs(b"Done").unwrap().is_empty());
}

#[test]
fn duplicate_obligations_require_matching_native_signatures() {
    let server = server(&[(
        "a.php",
        "<?php interface A { function run(int $value = 1): void; } interface B { public function run( int $value=1 ):void; } class C implements A, B {}",
    )]);
    assert_eq!(server.get_method_stubs(b"C").unwrap().len(), 1);
    for second in ["string $value = ''", "int $other = 1", "int $value = 2"] {
        let source = format!(
            "<?php interface A {{ function run(int $value = 1): void; }} interface B {{ function run({second}): void; }} class C implements A, B {{}}"
        );
        assert!(self::server(&[("a.php", &source)]).get_method_stubs(b"C").is_none(), "{source}");
    }
}

#[test]
fn unsupported_obligations_decline_the_whole_set() {
    for source in [
        "<?php interface I { function good(): void; #[Marker] function bad(): void; } class C implements I {}",
        "<?php interface I { function good(): void; function bad(#[Marker] int $value): void; } class C implements I {}",
        "<?php interface I { function good(): void; function bad(int $value = __LINE__): void; } class C implements I {}",
        "<?php interface I { function good(): void; function bad(int $value = 'invalid'): void; } class C implements I {}",
        "<?php class Value {} interface I { function good(): void; function bad($value = new Value()): void; } class C implements I {}",
        "<?php /** @template T */ interface I { function good(): void; } class C implements I {}",
        "<?php trait T { abstract function good(): void; } class C { use T; }",
        "<?php abstract class Base { private const VALUE = 1; abstract function good(): void; abstract function bad($value = self::VALUE): void; } class C extends Base {}",
        "<?php interface I { static function bad(): void; function good(): void; } class Base { function bad(): void {} } class C extends Base implements I {}",
    ] {
        assert!(
            server(&[("a.php", source)]).get_method_stubs(b"C").is_none(),
            "must not return a partial set: {source}"
        );
    }
}
