use std::borrow::Cow;
use std::sync::LazyLock;

use bumpalo::Bump;
use foldhash::HashSet;
use indoc::indoc;

use mago_atom::AtomSet;
use mago_codex::populator::populate_codebase;
use mago_codex::scanner::scan_program;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_guard::ArchitecturalGuard;
use mago_guard::settings::Settings;
use mago_guard::settings::StructuralInheritanceConstraint;
use mago_guard::settings::StructuralRule;
use mago_guard::settings::StructuralSettings;
use mago_names::resolver::NameResolver;
use mago_prelude::Prelude;
use mago_syntax::parser::parse_file;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);

fn apply_fix(code: &'static str, settings: Settings) -> String {
    let Prelude { mut database, mut metadata, mut symbol_references } = PRELUDE.clone();

    let file = File::ephemeral(Cow::Borrowed("test_fix"), Cow::Borrowed(code));
    let file_id = database.add(file);
    let source_file = database.get_ref(&file_id).expect("File just added should exist");

    let arena = Bump::new();
    let program = parse_file(&arena, source_file);
    if program.has_errors() {
        panic!("Failed to parse code for guard test, errors: {:?}", program.errors);
    }

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    metadata.extend(scan_program(&arena, source_file, program, &resolved_names));
    populate_codebase(&mut metadata, &mut symbol_references, AtomSet::default(), HashSet::default());

    let guard = ArchitecturalGuard::new(settings);
    let report = guard.check(&metadata, program, &resolved_names);

    use mago_text_edit::TextEditor;
    let mut editor = TextEditor::new(&source_file.contents);
    for flaw in &report.structural_flaws {
        for edit in &flaw.edits {
            match editor.apply(edit.clone(), None::<fn(&str) -> bool>) {
                mago_text_edit::ApplyResult::Applied => {}
                result => panic!("Failed to apply edit: {:?}", result),
            }
        }
    }

    editor.finish()
}

#[test]
fn test_add_final_modifier() {
    let code = indoc! {r"<?php
        class MyCommand {}
    "};

    let settings = Settings {
        structural: StructuralSettings {
            rules: vec![StructuralRule {
                on: "MyCommand".to_string(),
                must_be_final: Some(true),
                ..Default::default()
            }],
        },
        ..Default::default()
    };

    assert!(apply_fix(code, settings).contains("final class MyCommand"));
}

#[test]
fn test_remove_final_modifier() {
    let code = indoc! {r"<?php
        final class MyCommand {}
    "};

    let settings = Settings {
        structural: StructuralSettings {
            rules: vec![StructuralRule {
                on: "MyCommand".to_string(),
                must_be_final: Some(false),
                ..Default::default()
            }],
        },
        ..Default::default()
    };

    let result = apply_fix(code, settings);
    assert!(!result.contains("final class") && result.contains("class MyCommand"));
}

#[test]
fn test_add_readonly_modifier() {
    let code = indoc! {r"<?php
        class MyCommand {}
    "};

    let settings = Settings {
        structural: StructuralSettings {
            rules: vec![StructuralRule {
                on: "MyCommand".to_string(),
                must_be_readonly: Some(true),
                ..Default::default()
            }],
        },
        ..Default::default()
    };

    assert!(apply_fix(code, settings).contains("readonly class MyCommand"));
}

#[test]
fn test_add_interface_no_existing() {
    let code = indoc! {r"<?php
        class MyCommand {}
    "};

    let settings = Settings {
        structural: StructuralSettings {
            rules: vec![StructuralRule {
                on: "MyCommand".to_string(),
                must_implement: Some(StructuralInheritanceConstraint::Single(
                    "App\\Domain\\Command".to_string(),
                )),
                ..Default::default()
            }],
        },
        ..Default::default()
    };

    let result = apply_fix(code, settings);
    assert!(result.contains("MyCommand") && result.contains("implements") && result.contains("Domain"));
}

#[test]
fn test_add_interface_existing() {
    let code = indoc! {r"<?php
        class MyCommand implements ExistingInterface {}
    "};

    let settings = Settings {
        structural: StructuralSettings {
            rules: vec![StructuralRule {
                on: "MyCommand".to_string(),
                must_implement: Some(StructuralInheritanceConstraint::AllOf(vec![
                    "ExistingInterface".to_string(),
                    "App\\Domain\\Command".to_string(),
                ])),
                ..Default::default()
            }],
        },
        ..Default::default()
    };

    let result = apply_fix(code, settings);
    assert!(result.contains("implements ExistingInterface, \\App\\Domain\\Command"));
}