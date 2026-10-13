use std::borrow::Cow;

use foldhash::HashSet;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;
use mago_word::WordSet;
use mago_word::empty_word;
use mago_word::word;

use super::populate_codebase;
use super::populate_codebase_targeted;
use crate::metadata::CodebaseMetadata;
use crate::reference::SymbolReferences;
use crate::scanner::scan_program;

fn scan(source: &str) -> CodebaseMetadata {
    let file = File::ephemeral(Cow::Borrowed(b"ancestry.php"), Cow::Owned(source.as_bytes().to_vec()));
    let arena = LocalArena::new();
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "fixture did not parse: {:?}", program.errors);
    let names = NameResolver::new(&arena).resolve(program);
    scan_program(&arena, &file, program, &names, PHPVersion::LATEST)
}

fn populate_full(codebase: &mut CodebaseMetadata) {
    populate_codebase(codebase, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());
}

fn check_update(codebase: &mut CodebaseMetadata, source: &str, rescanned: &[&str], invalidated: &[&str]) {
    let mut scanned = scan(source);
    let mut expected = scanned.clone();
    populate_full(&mut expected);
    let rescanned: WordSet = rescanned.iter().map(|name| word(*name)).collect();
    for name in &rescanned {
        if let Some(metadata) = scanned.class_likes.remove(name) {
            codebase.class_likes.insert(*name, metadata);
        }
    }

    codebase.function_likes.retain(|(name, _), _| !rescanned.contains(name));
    codebase
        .function_likes
        .extend(scanned.function_likes.into_iter().filter(|((name, _), _)| rescanned.contains(name)));
    codebase.symbols = scanned.symbols;
    codebase.file_signatures = scanned.file_signatures;

    let unsafe_classes: WordSet = invalidated.iter().map(|name| word(*name)).chain(rescanned.iter().copied()).collect();
    let safe = codebase.class_likes.keys().filter(|name| !unsafe_classes.contains(*name)).copied().collect();
    let dirty = rescanned
        .iter()
        .map(|name| (*name, empty_word()))
        .chain(codebase.function_likes.keys().filter(|(name, _)| rescanned.contains(name)).copied())
        .collect();

    populate_codebase_targeted(codebase, &mut SymbolReferences::new(), safe, HashSet::default(), dirty);

    assert_eq!(codebase.direct_classlike_descendants, expected.direct_classlike_descendants, "{source}");
    assert_eq!(codebase.all_class_like_descendants, expected.all_class_like_descendants, "{source}");
    for (name, expected) in &expected.class_likes {
        let actual = &codebase.class_likes[name];
        assert_eq!(
            actual.flags.allows_dynamic_properties(),
            expected.flags.allows_dynamic_properties(),
            "{name}: {source}",
        );
        assert_eq!(actual.direct_parent_class, expected.direct_parent_class, "{name}: {source}");
        assert_eq!(actual.direct_parent_interfaces, expected.direct_parent_interfaces, "{name}: {source}");
        assert_eq!(actual.all_parent_classes, expected.all_parent_classes, "{name}: {source}");
        assert_eq!(actual.all_parent_interfaces, expected.all_parent_interfaces, "{name}: {source}");
        assert_eq!(actual.used_traits, expected.used_traits, "{name}: {source}");
        assert_eq!(actual.require_extends, expected.require_extends, "{name}: {source}");
        assert_eq!(actual.require_implements, expected.require_implements, "{name}: {source}");
        assert_eq!(actual.child_class_likes, expected.child_class_likes, "{name}: {source}");
        assert_eq!(actual.constants.len(), expected.constants.len(), "{name}: {source}");
        for (constant, expected) in &expected.constants {
            let actual = &actual.constants[constant];
            assert_eq!(actual.inferred_type, expected.inferred_type, "{name}::{constant}: {source}");
            assert_eq!(actual.visibility, expected.visibility, "{name}::{constant}: {source}");
        }
        assert_eq!(actual.trait_constant_ids, expected.trait_constant_ids, "{name}: {source}");
    }
}

#[test]
fn dynamic_properties_flags_follow_class_inheritance_and_updates() {
    let before = include_str!("../../tests/fixtures/allow_dynamic_properties/before.php");
    let after = include_str!("../../tests/fixtures/allow_dynamic_properties/after.php");
    let mut codebase = scan(before);

    for name in ["base", "own"] {
        assert!(codebase.class_likes[&word(name)].flags.allows_dynamic_properties());
    }

    for name in ["child", "leaf", "unrelated"] {
        assert!(!codebase.class_likes[&word(name)].flags.allows_dynamic_properties());
    }

    populate_full(&mut codebase);

    for name in ["base", "child", "leaf", "own"] {
        assert!(codebase.class_likes[&word(name)].flags.allows_dynamic_properties());
    }

    assert!(!codebase.class_likes[&word("unrelated")].flags.allows_dynamic_properties());

    check_update(&mut codebase, after, &["base"], &["child", "leaf", "own"]);
    check_update(&mut codebase, before, &["base"], &["child", "leaf", "own"]);
}

#[test]
fn targeted_population_rebuilds_ancestry_without_rescanning_descendants() {
    let roots = concat!(
        "interface oldcontract {} interface newcontract {} interface stringable {} ",
        "class oldroot implements oldcontract {} class newroot implements newcontract {} ",
        "trait oldtrait {} trait keep {} class safe {} ",
    );
    let descendants = "class child extends base { use keep; } class leaf extends child {}";
    let cases: &[(&str, &str, &[&str], &[&str])] = &[
        ("class base extends oldroot {}", "class base extends newroot {}", &["base"], &["child", "leaf"]),
        ("class base { use oldtrait, keep; }", "class base {}", &["base"], &["child", "leaf"]),
        (
            "trait shared { use oldtrait; } class base { use shared; }",
            "trait shared {} class base { use shared; }",
            &["shared"],
            &["base", "child", "leaf"],
        ),
        (
            "interface contract extends oldcontract {} class base implements contract {}",
            "interface contract extends newcontract {} class base implements contract {}",
            &["contract"],
            &["base", "child", "leaf"],
        ),
        (
            "trait shared { public function __toString(): string { return ''; } } class base { use shared; }",
            "trait shared {} class base { use shared; }",
            &["shared"],
            &["base", "child", "leaf"],
        ),
        (
            "class base extends oldroot {} /** @require-extends base */ trait shared {} class user { use shared; }",
            "class base extends newroot {} /** @require-extends base */ trait shared {} class user { use shared; }",
            &["base"],
            &["shared", "user", "child", "leaf"],
        ),
    ];

    for (before, after, rescanned, invalidated) in cases {
        let before = format!("<?php {roots} {before} {descendants}");
        let after = format!("<?php {roots} {after} {descendants}");
        let mut codebase = scan(&before);
        populate_full(&mut codebase);
        check_update(&mut codebase, &after, rescanned, invalidated);
        check_update(&mut codebase, &before, rescanned, invalidated);
    }
}

#[test]
fn targeted_population_rebuilds_inherited_constants_without_rescanning_descendants() {
    let descendants =
        "class Child extends Base { public const KEEP = 99, SECOND = 100; } class Leaf extends Child {} class Safe {}";
    let cases: &[(&str, &str, &[&str], &[&str])] = &[
        (
            "class Base { public const VALUE = 1; public const KEEP = 1; }",
            "class Base { protected const VALUE = 2; public const KEEP = 2; }",
            &["base"],
            &["child", "leaf"],
        ),
        (
            "class Base { public const FIRST = 1; }",
            "class Base { public const FIRST = 1, ADDED = 2; }",
            &["base"],
            &["child", "leaf"],
        ),
        ("class base { public const VALUE = 1; }", "class base {}", &["base"], &["child", "leaf"]),
        (
            "trait shared { public const VALUE = 1; } class base { use shared; }",
            "trait shared { public const VALUE = 2; } class base { use shared; }",
            &["shared"],
            &["base", "child", "leaf"],
        ),
        (
            "trait shared { public const VALUE = 1; } class base { use shared; }",
            "trait shared {} class base { use shared; }",
            &["shared"],
            &["base", "child", "leaf"],
        ),
        (
            "interface contract { public const VALUE = 1; } class base implements contract {}",
            "interface contract {} class base implements contract {}",
            &["contract"],
            &["base", "child", "leaf"],
        ),
        (
            "class oldroot { public const OLD = 1; } class newroot { public const NEW = 2; } class base extends oldroot {}",
            "class oldroot { public const OLD = 1; } class newroot { public const NEW = 2; } class base extends newroot {}",
            &["base"],
            &["child", "leaf"],
        ),
    ];
    let declared = scan(&format!("<?php class Base {{}} {descendants}"));
    let mut populated = declared.clone();
    populate_full(&mut populated);
    assert_eq!(populated.class_likes[&word("child")].constants.len(), 2);
    assert_eq!(
        populated.class_likes[&word("child")].constants[&word("SECOND")].inferred_type,
        declared.class_likes[&word("child")].constants[&word("SECOND")].inferred_type,
    );
    for (before, after, rescanned, invalidated) in cases {
        let before = format!("<?php {before} {descendants}");
        let after = format!("<?php {after} {descendants}");
        let mut codebase = scan(&before);
        populate_full(&mut codebase);
        check_update(&mut codebase, &after, rescanned, invalidated);
        check_update(&mut codebase, &before, rescanned, invalidated);
    }
}
