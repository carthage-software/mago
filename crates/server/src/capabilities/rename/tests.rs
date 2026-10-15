#![allow(clippy::unwrap_used)]

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use mago_allocator::LocalArena;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::signature_builder::build_file_signature;
use mago_database::file::{File, FileType};
use mago_database::{Database, DatabaseConfiguration, DatabaseReader};
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::settings::ParserSettings;
use mago_word::word;

use super::*;

fn server(files: &[(&str, &str)]) -> Server {
    typed_server(&files.iter().map(|(name, source)| (*name, *source, FileType::Host)).collect::<Vec<_>>())
}

fn typed_server(files: &[(&str, &str, FileType)]) -> Server {
    let mut database = Database::new(DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]));
    let mut codebase = CodebaseMetadata::new();
    for (name, source, file_type) in files {
        let file = File::new(name.as_bytes().to_vec().into(), *file_type, None, source.as_bytes().to_vec().into());
        assert!(file.path.is_none());
        let arena = LocalArena::new();
        let program = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
        assert!(!program.has_errors(), "{source}");
        let names = NameResolver::new(&arena).resolve(program);
        let mut scanned = scan_program(&arena, &file, program, &names, PHPVersion::PHP85);
        scanned.set_file_signature(file.id, build_file_signature(&file, program, &names));
        codebase.extend(scanned);
        database.add(file);
    }
    let mut references = SymbolReferences::new();
    populate_codebase(&mut codebase, &mut references, HashSet::default(), HashSet::default());
    Server::new(
        database,
        codebase,
        references,
        crate::Settings {
            php_version: PHPVersion::PHP85,
            features: crate::Features { analyzer: false, linter: false, formatter: false },
            parser: ParserSettings::default(),
            analyzer: mago_analyzer::settings::Settings::default(),
            linter: mago_linter::settings::Settings::default(),
            formatter: mago_formatter::settings::FormatSettings::default(),
            plugin_registry: Arc::new(mago_analyzer::plugin::PluginRegistry::new()),
        },
    )
}

fn apply(source: &str, edits: &[TextReplacement]) -> String {
    let mut edits: Vec<_> = edits.iter().collect();
    edits.sort_unstable_by_key(|edit| std::cmp::Reverse(edit.range.start));
    let mut result = source.to_owned();
    for edit in edits {
        result.replace_range(edit.range.start as usize..edit.range.end as usize, &edit.new_text);
    }
    result
}

#[test]
fn rename_returns_byte_edits_for_files_without_paths() {
    let declaration = "<?php namespace Lib; class Target {}";
    let docs = "<?php namespace App; use Lib\\Target; /** @var Target */ $value = new Target();";
    let local = "<?php /* 😀 */ use Lib\\Target; $item = new Target(); echo $item;";
    let files = [("lib.php", declaration), ("docs.php", docs), ("local.php", local)];
    let mut server = server(&files);
    let declaration_id = server.database().get_id(b"lib.php").unwrap();
    let offset = declaration.find("Target").unwrap() as u32;
    assert_eq!(server.get_prepare_rename(declaration_id, offset), Ok(Range::new(offset, offset + 6)));
    let edits = server.get_rename(declaration_id, offset, "Renamed").unwrap();
    assert_eq!(edits.len(), files.len());
    for (name, source) in files {
        let file = server.database().get_id(name.as_bytes()).unwrap();
        assert_eq!(apply(source, &edits[&file]), source.replace("Target", "Renamed"));
    }

    let file = server.database().get_id(b"local.php").unwrap();
    let offset = local.find("Target();").unwrap() as u32;
    let edits = server.get_rename(file, offset, "Alias").unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(
        apply(local, &edits[&file]),
        "<?php /* 😀 */ use Lib\\Target as Alias; $item = new Alias(); echo $item;"
    );
    let offset = local.find("$item").unwrap() as u32;
    let edits = server.get_rename(file, offset, "value").unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(apply(local, &edits[&file]), local.replace("$item", "$value"));
}

#[test]
fn refusals_carry_reason_and_location() {
    let callback = "<?php function run(): void {} $fn = 'run';";
    let local = "<?php function f(array $commands) { foreach ($commands as $command) {} }";
    let first = "<?php class DeleteController {}";
    let second = "<?php class IndexController {}";
    let files = [("callback.php", callback), ("local.php", local), ("a.php", first), ("b.php", second)];
    let mut server = server(&files);
    let id = |server: &Server, name: &str| server.database().get_id(name.as_bytes()).unwrap();
    let at = |reason, file, source: &str, needle: &str| {
        let start = source.find(needle).unwrap() as u32;
        RenameRefusal::at(reason, file, Range::new(start, start + needle.len() as u32))
    };

    let file = id(&server, "callback.php");
    let offset = callback.find("run").unwrap() as u32;
    let expected = at(RefusalReason::StringReference { proposed: false }, file, callback, "'run'");
    assert_eq!(server.get_prepare_rename(file, offset), Err(expected));
    assert_eq!(server.get_rename(file, offset, "execute").unwrap_err(), expected);

    let file = id(&server, "local.php");
    let offset = local.find("$command)").unwrap() as u32;
    assert_eq!(server.get_prepare_rename(file, offset), Ok(Range::new(offset + 1, offset + 8)));
    assert_eq!(
        server.get_rename(file, offset, "commands").unwrap_err(),
        at(RefusalReason::BindingTaken, file, local, "$commands")
    );

    let file = id(&server, "a.php");
    let offset = first.find("DeleteController").unwrap() as u32;
    assert_eq!(
        server.get_rename(file, offset, "IndexController").unwrap_err(),
        at(RefusalReason::NameTaken, id(&server, "b.php"), second, "IndexController")
    );
    assert_eq!(
        server.get_rename(file, offset, "123bad").unwrap_err(),
        RenameRefusal::new(RefusalReason::InvalidName(crate::NameProblem::NotIdentifier))
    );

    let unknown = FileId::new(b"unknown.php");
    assert_eq!(server.get_prepare_rename(unknown, 0), Err(RenameRefusal::new(RefusalReason::NoSymbol)));
    assert_eq!(server.get_prepare_rename(file, 0).unwrap_err().reason, RefusalReason::NoSymbol);
}

fn at(reason: RefusalReason, file: FileId, source: &str, needle: &str, name: &str) -> RenameRefusal {
    let start = (source.find(needle).unwrap() + needle.find(name).unwrap()) as u32;
    RenameRefusal::at(reason, file, Range::new(start, start + name.len() as u32))
}

#[test]
fn php_provided_enum_methods_refuse_the_families_they_join() {
    let source =
        "<?php interface HasCases { public static function cases(): array; } enum Kind implements HasCases { case A; }";
    let mut server = server(&[("a.php", source)]);
    let file = server.database().get_id(b"a.php").unwrap();
    let offset = source.find("cases").unwrap() as u32;
    let expected = at(RefusalReason::EnumMethod, file, source, "enum Kind", "Kind");
    assert_eq!(server.get_prepare_rename(file, offset), Err(expected));
    assert_eq!(server.get_rename(file, offset, "all").unwrap_err(), expected);
}

#[test]
fn outside_family_refusals_name_the_overridden_declaration() {
    let base = "<?php abstract class Base { public function run(int $value): void {} }";
    let local = "<?php class Local extends Base { public function run(int $value): void {} }";
    let siblings: Vec<_> = (0..6)
        .map(|index| {
            (
                format!("vendor/a_sibling_{index}.php"),
                format!("<?php class Sibling{index} extends Base {{ public function run(int $value): void {{}} }}"),
            )
        })
        .collect();
    for _ in 0..3 {
        let mut files = vec![("vendor/z_base.php", base, FileType::Vendored), ("local.php", local, FileType::Host)];
        files.extend(siblings.iter().map(|(name, source)| (name.as_str(), source.as_str(), FileType::Vendored)));
        let mut server = typed_server(&files);
        let base_id = server.database().get_id(b"vendor/z_base.php").unwrap();
        let file = server.database().get_id(b"local.php").unwrap();
        for (needle, name) in [("run(", "run"), ("$value", "$value")] {
            let offset = local.find(needle).unwrap() as u32;
            let expected = at(RefusalReason::External, base_id, base, needle, name);
            for _ in 0..3 {
                assert_eq!(server.get_prepare_rename(file, offset), Err(expected), "{needle}");
                assert_eq!(server.get_rename(file, offset, "renamed").unwrap_err(), expected, "{needle}");
            }
        }
    }
}

#[test]
fn runtime_declarations_are_refused_before_renaming() {
    let source =
        "<?php define('LIMIT', 1); echo LIMIT; class Target {} class_alias(Target::class, 'Alias'); new Alias();";
    let mut server = server(&[("a.php", source)]);
    let file = server.database().get_id(b"a.php").unwrap();
    for (needle, call) in [("LIMIT;", "define('LIMIT', 1)"), ("Alias()", "class_alias(Target::class, 'Alias')")] {
        let offset = source.find(needle).unwrap() as u32;
        let expected = at(RefusalReason::RuntimeDeclaration, file, source, call, call);
        assert_eq!(server.get_prepare_rename(file, offset), Err(expected), "{needle}");
        assert_eq!(server.get_rename(file, offset, "Renamed").unwrap_err(), expected, "{needle}");
    }
}

#[test]
fn incomplete_hierarchies_keep_the_written_ancestor_name() {
    let source = "<?php class Child extends Missing\\Parent_ implements MissingB, MissingA { function run(): void {} }";
    let mut server = server(&[("a.php", source)]);
    let file = server.database().get_id(b"a.php").unwrap();
    let offset = source.find("run").unwrap() as u32;
    assert_eq!(
        server.get_prepare_rename(file, offset),
        Err(at(RefusalReason::IncompleteHierarchy(word("Missing\\Parent_")), file, source, "Child", "Child"))
    );
}
