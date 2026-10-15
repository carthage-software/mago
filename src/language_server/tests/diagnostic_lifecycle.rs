//! Diagnostics follow edits across files, even when definitions arrive last.

use std::collections::BTreeMap;

use serde_json::Value;

use super::harness::{Harness, client_capabilities};
use crate::language_server::ServerConfig;

async fn collect(h: &mut Harness, current: &mut BTreeMap<String, Value>) {
    for publication in h.diagnostics().await {
        current.insert(publication["uri"].as_str().unwrap().to_owned(), publication["diagnostics"].clone());
    }
}

fn assert_no_errors(current: &BTreeMap<String, Value>) {
    let errors: Vec<_> = current
        .iter()
        .flat_map(|(uri, diagnostics)| diagnostics.as_array().unwrap().iter().map(move |diagnostic| (uri, diagnostic)))
        .filter(|(_, diagnostic)| diagnostic["severity"] == 1)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[tokio::test]
async fn parse_errors_hide_cascades_without_losing_editor_recovery() {
    let prefix = "<?php declare(strict_types=1); function validSibling(int $value): int { return $value; } function caller(): int { return validSibling(1); } ";
    let source = format!("{prefix}function unsavedOnlyTarget(): int {{ return 1; }}");
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(
        &[("a.php", &source), ("other.php", "<?php missing_function();")],
        config,
        client_capabilities(),
    )
    .await;
    h.open("a.php", &source).await;
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    for (version, expression, expected) in
        [(2, "1 +", "parse"), (3, "'wrong'", "invalid-return-statement"), (4, "1 +", "parse"), (5, "1", "")]
    {
        let source = format!("{prefix}function unsavedOnlyTarget(): int {{ return {expression}; }}");
        h.change("a.php", &source, version).await;
        collect(&mut h, &mut current).await;
        let diagnostics = current[&h.url("a.php")].as_array().unwrap();
        if expected.is_empty() {
            assert!(diagnostics.is_empty(), "{diagnostics:?}");
        } else {
            assert!(diagnostics.iter().any(|issue| issue["code"] == expected), "{diagnostics:?}");
        }
        if expected == "parse" {
            assert!(
                diagnostics.iter().all(|issue| matches!(issue["code"].as_str(), Some("parse" | "syntax"))),
                "{diagnostics:?}"
            );
            let error = diagnostics.iter().find(|issue| issue["code"] == "parse").unwrap();
            let message = error["message"].as_str().unwrap();
            assert_eq!(message.lines().next(), Some("Unexpected token `Semicolon`"), "{message}");
            assert!(!message.contains("Parse error encountered during parsing"), "{message}");
        }
        if expected == "invalid-return-statement" {
            let issue = diagnostics.iter().find(|issue| issue["code"] == expected).unwrap();
            let message = issue["message"].as_str().unwrap();
            assert!(message.starts_with("Invalid return type for function"), "{message}");
            assert!(message.contains("expected `int`, but found `string('wrong')`"), "{message}");
            assert!(message.contains("\n = Help: Change the return value to match `int`"), "{message}");
            assert!(!message.contains("returned here is not compatible with the declared return type"), "{message}");
        }
        assert!(
            current[&h.url("other.php")]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue["code"] == "non-existent-function")
        );
        let outline = h.for_doc("textDocument/documentSymbol", "a.php").await;
        assert!(outline.as_array().unwrap().iter().any(|symbol| symbol["name"] == "validSibling"));
        let hover = h.at("textDocument/hover", "a.php", 0, source.find("$value;").unwrap() as u32 + 1).await;
        assert_eq!(hover["contents"]["value"], "int $value");
        let signature = h
            .at(
                "textDocument/signatureHelp",
                "a.php",
                0,
                source.find("validSibling(1)").unwrap() as u32 + "validSibling(".len() as u32,
            )
            .await;
        assert_eq!(signature["signatures"][0]["label"], "function validSibling(int $value): int");
    }
}

#[tokio::test]
async fn php_version_errors_do_not_hide_analyzer_diagnostics() {
    let mut config = ServerConfig { linter: false, ..ServerConfig::default() };
    config.configuration.php_version = mago_php_version::PHPVersion::PHP74;
    let mut h = Harness::start_with(
        &[("a.php", "<?php function take(mixed $value): void { missing_function(); }")],
        config,
        client_capabilities(),
    )
    .await;
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    let diagnostics = current[&h.url("a.php")].as_array().unwrap();
    for code in ["semantics", "non-existent-function"] {
        assert!(diagnostics.iter().any(|issue| issue["code"] == code), "{diagnostics:?}");
    }
    assert!(diagnostics.iter().all(|issue| !matches!(issue["code"].as_str(), Some("parse" | "syntax"))));
}

#[tokio::test]
async fn class_rename_and_undo_clear_errors_when_declaration_changes_last() {
    let declaration = "<?php class Contact { public string $name = 'contact'; }";
    let caller = "<?php function contact(): Contact { return new Contact(); }";
    let property = "<?php class Holder { public Contact $contact; }";
    let repository = "<?php /** @template T of object */ abstract class Repository { /** @return T|null */ abstract public function find(): ?object; } /** @extends Repository<Contact> */ abstract class ContactRepository extends Repository {}";
    let handler = "<?php function run(ContactRepository $repository): void { $contact = $repository->find(); if ($contact !== null) { echo $contact->name; } }";
    let originals = [
        ("Contact.php", declaration),
        ("caller.php", caller),
        ("Holder.php", property),
        ("repository.php", repository),
        ("handler.php", handler),
    ];
    for buffers in [true, false] {
        let config = ServerConfig { linter: false, ..ServerConfig::default() };
        let mut h = Harness::start_with(&originals, config, client_capabilities()).await;
        let mut current = BTreeMap::new();
        if buffers {
            for (name, contents) in originals {
                h.open(name, contents).await;
            }
        }
        collect(&mut h, &mut current).await;
        assert_no_errors(&current);
        for (version, rename) in [(2, true), (3, false)] {
            for (name, contents) in [
                ("caller.php", caller),
                ("Holder.php", property),
                ("repository.php", repository),
                ("Contact.php", declaration),
            ] {
                let contents = if rename {
                    contents.replace("Contact", "SomePerson").replace("SomePersonRepository", "ContactRepository")
                } else {
                    contents.to_owned()
                };
                if buffers {
                    h.change(name, &contents, version).await;
                } else {
                    std::fs::write(h.workspace.join(name), contents).unwrap();
                    h.watched(name, 2).await;
                }
                collect(&mut h, &mut current).await;
            }
            assert_no_errors(&current);
        }
    }
}

#[tokio::test]
async fn saved_class_rename_and_disk_discard_restore_open_and_closed_file_diagnostics() {
    let files = [
        ("Contact.php", "<?php final class Contact { public function name(): string { return 'contact'; } }"),
        (
            "Listener.php",
            "<?php final class Listener { /** @param list<Contact> $contacts @return list<string> */ public function names(array $contacts): array { return array_map(static fn (Contact $contact): string => $contact->name(), $contacts); } }",
        ),
        ("Consumer.php", "<?php final class Consumer { public function contact(): Contact { return new Contact(); } }"),
    ];
    for watcher_first in [false, true] {
        let mut h = Harness::start(&files).await;
        for (name, source) in &files[..2] {
            h.open(name, source).await;
        }
        let mut current = BTreeMap::new();
        collect(&mut h, &mut current).await;
        assert_no_errors(&current);
        let baseline = current.clone();
        for (index, (name, source)) in files.iter().enumerate() {
            let renamed = source.replace("Contact", "SomePerson");
            std::fs::write(h.workspace.join(name), &renamed).unwrap();
            if index < 2 {
                h.change(name, &renamed, 2).await;
                h.client
                    .send_notification(
                        "textDocument/didSave",
                        serde_json::json!({
                            "textDocument":{"uri":h.url(name)},
                        }),
                    )
                    .await;
            }
            h.watched(name, 2).await;
        }
        collect(&mut h, &mut current).await;
        assert!(current[&h.url("Contact.php")].as_array().unwrap().iter().any(|issue| {
            issue["code"] == "file-name" && issue["message"].as_str().unwrap().contains("SomePerson")
        }));
        for (name, source) in files {
            std::fs::write(h.workspace.join(name), source).unwrap();
        }
        if watcher_first {
            h.watched("", 2).await;
        }
        // Reloaded open buffers keep their versions; closed consumers arrive through the watcher.
        for (name, source) in files[..2].iter().rev() {
            h.change(name, source, 3).await;
        }
        h.watched("", 2).await;
        collect(&mut h, &mut current).await;
        for (name, _) in files {
            let uri = h.url(name);
            assert_eq!(current.get(&uri), baseline.get(&uri), "{name}; watcher first: {watcher_first}");
        }
    }
}

#[tokio::test]
async fn declaration_rename_and_restore_refresh_unchanged_closure_parameter_types() {
    let declaration = "<?php class Contact { public function name(): string { return 'contact'; } }";
    let callers = [
        (
            "used.php",
            "<?php function used(): Closure { return static fn (Contact $contact): string => $contact->name(); }",
        ),
        (
            "closure.php",
            "<?php function unusedClosure(): Closure { return static function (Contact $unused): void {}; }",
        ),
        ("arrow.php", "<?php function unusedArrow(): Closure { return static fn (Contact $unused): int => 1; }"),
        ("global.php", "<?php $unused = static fn (Contact $unused): int => 1;"),
        (
            "nested.php",
            "<?php class Factory { public function make(): Closure { return static fn (): Closure => static function (Contact $unused): void {}; } }",
        ),
        (
            "return.php",
            "<?php function returnContact(): Closure { return static function (): Contact { throw new RuntimeException(); }; }",
        ),
    ];
    let files = std::iter::once(("Contact.php", declaration)).chain(callers).collect::<Vec<_>>();
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&files, config, client_capabilities()).await;
    h.open("Contact.php", declaration).await;
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    assert_no_errors(&current);
    for (version, source) in [(2, declaration.replace("Contact", "SomePerson")), (3, declaration.to_owned())] {
        std::fs::write(h.workspace.join("Contact.php"), &source).unwrap();
        h.change("Contact.php", &source, version).await;
        h.client
            .send_notification(
                "textDocument/didSave",
                serde_json::json!({
                    "textDocument":{"uri":h.url("Contact.php")},
                }),
            )
            .await;
        h.watched("Contact.php", 2).await;
        collect(&mut h, &mut current).await;
        if version == 2 {
            for (name, _) in callers {
                assert!(
                    current.get(&h.url(name)).is_some_and(|issues| {
                        issues.as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-class-like")
                    }),
                    "missing Contact must invalidate {name}: {current:#?}"
                );
            }
        } else {
            assert_no_errors(&current);
        }
    }
}

#[tokio::test]
async fn directory_events_remove_and_restore_dependency_symbols_and_errors() {
    let source = "<?php function contact(): Contact { return new Contact(); }";
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(
        &[("a.php", source), ("mago.toml", "[source]\npaths = ['a.php']\nincludes = ['vendor']\n")],
        config,
        client_capabilities(),
    )
    .await;
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    for directory in ["vendor", "vendor/package", "vendor"] {
        std::fs::create_dir_all(h.workspace.join("vendor/package")).unwrap();
        std::fs::write(h.workspace.join("vendor/package/Contact.php"), "<?php class Contact {}").unwrap();
        h.watched(directory, 1).await;
        collect(&mut h, &mut current).await;
        assert_no_errors(&current);
        assert_eq!(h.for_doc("textDocument/documentSymbol", "vendor/package/Contact.php").await[0]["name"], "Contact");
        std::fs::remove_dir_all(h.workspace.join(directory)).unwrap();
        h.watched(directory, 3).await;
        collect(&mut h, &mut current).await;
        assert!(current[&h.url("a.php")].as_array().unwrap().iter().any(|diagnostic| diagnostic["severity"] == 1));
        assert!(h.for_doc("textDocument/documentSymbol", "vendor/package/Contact.php").await.is_null());
    }
}

#[tokio::test]
async fn directory_events_keep_open_buffers_until_close() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php")], config, client_capabilities()).await;
    std::fs::create_dir(h.workspace.join("src")).unwrap();
    std::fs::write(h.workspace.join("src/Contact.php"), "<?php class Contact {}").unwrap();
    h.watched("src", 1).await;
    h.open("src/Contact.php", "<?php class UnsavedContact {}").await;
    std::fs::remove_dir_all(h.workspace.join("src")).unwrap();
    h.watched("src", 3).await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "src/Contact.php").await[0]["name"], "UnsavedContact");
    std::fs::create_dir(h.workspace.join("src")).unwrap();
    std::fs::write(h.workspace.join("src/Contact.php"), "<?php class ReinstalledContact {}").unwrap();
    h.watched("src", 1).await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "src/Contact.php").await[0]["name"], "UnsavedContact");
    h.close("src/Contact.php").await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "src/Contact.php").await[0]["name"], "ReinstalledContact");
}

#[tokio::test]
async fn directory_replacement_uses_disk_state_after_coalesced_events() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php")], config, client_capabilities()).await;
    std::fs::create_dir(h.workspace.join("src")).unwrap();
    std::fs::write(h.workspace.join("src/Old.php"), "<?php class Old {}").unwrap();
    h.watched("src", 1).await;
    h.for_doc("textDocument/documentSymbol", "src/Old.php").await;
    std::fs::remove_dir_all(h.workspace.join("src")).unwrap();
    std::fs::create_dir(h.workspace.join("src")).unwrap();
    std::fs::write(h.workspace.join("src/New.php"), "<?php class NewClass {}").unwrap();
    h.client
        .send_notification(
            "workspace/didChangeWatchedFiles",
            serde_json::json!({"changes": [
                {"uri": h.url("src"), "type": 3},
                {"uri": h.url("src"), "type": 1},
                {"uri": h.url("src/Old.php"), "type": 3},
                {"uri": h.url("src/New.php"), "type": 1}
            ]}),
        )
        .await;
    assert!(h.for_doc("textDocument/documentSymbol", "src/Old.php").await.is_null());
    assert_eq!(h.for_doc("textDocument/documentSymbol", "src/New.php").await[0]["name"], "NewClass");
}

#[tokio::test]
async fn directory_replaced_by_file_clears_closed_dependencies_and_keeps_buffers() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(
        &[
            ("a.php", "<?php function contact(): Contact { return new Contact(); }"),
            ("mago.toml", "[source]\npaths = ['a.php']\nincludes = ['vendor']\n"),
        ],
        config,
        client_capabilities(),
    )
    .await;
    let package = h.workspace.join("vendor/package");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join("Contact.php"), "<?php class Contact {}").unwrap();
    std::fs::write(package.join("Buffered.php"), "<?php class Buffered {}").unwrap();
    h.watched("vendor", 1).await;
    h.open("vendor/package/Buffered.php", "<?php class UnsavedBuffered {}").await;
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    assert_no_errors(&current);

    std::fs::remove_dir_all(&package).unwrap();
    std::fs::write(&package, "replacement file").unwrap();
    h.client
        .send_notification(
            "workspace/didChangeWatchedFiles",
            serde_json::json!({"changes": [
                {"uri": h.url("vendor/package"), "type": 3},
                {"uri": h.url("vendor/package"), "type": 1}
            ]}),
        )
        .await;
    assert_eq!(
        h.for_doc("textDocument/documentSymbol", "vendor/package/Buffered.php").await[0]["name"],
        "UnsavedBuffered"
    );
    assert!(h.for_doc("textDocument/documentSymbol", "vendor/package/Contact.php").await.is_null());
    collect(&mut h, &mut current).await;
    assert!(current[&h.url("a.php")].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-class"));

    std::fs::remove_file(&package).unwrap();
    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("Contact.php"), "<?php class Contact {}").unwrap();
    std::fs::write(package.join("Buffered.php"), "<?php class RestoredBuffered {}").unwrap();
    h.watched("vendor/package", 1).await;
    collect(&mut h, &mut current).await;
    assert_no_errors(&current);
    assert_eq!(
        h.for_doc("textDocument/documentSymbol", "vendor/package/Buffered.php").await[0]["name"],
        "UnsavedBuffered"
    );
    h.close("vendor/package/Buffered.php").await;
    assert_eq!(
        h.for_doc("textDocument/documentSymbol", "vendor/package/Buffered.php").await[0]["name"],
        "RestoredBuffered"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn directory_events_restore_symlinked_vendor_packages() {
    let source = "<?php function contact(): Contact { return new Contact(); }";
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(
        &[("a.php", source), ("mago.toml", "[source]\npaths = ['a.php']\nincludes = ['vendor']\n")],
        config,
        client_capabilities(),
    )
    .await;
    let package = tempfile::tempdir().unwrap();
    std::fs::write(package.path().join("Contact.php"), "<?php class Contact {}").unwrap();
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    for _ in 0..2 {
        std::fs::create_dir(h.workspace.join("vendor")).unwrap();
        std::os::unix::fs::symlink(package.path(), h.workspace.join("vendor/package")).unwrap();
        h.watched("vendor", 1).await;
        collect(&mut h, &mut current).await;
        assert_no_errors(&current);
        let definition = h.at("textDocument/definition", "a.php", 0, source.find("Contact").unwrap() as u32).await;
        assert_eq!(definition["uri"], h.url("vendor/package/Contact.php"));
        std::fs::remove_dir_all(h.workspace.join("vendor")).unwrap();
        h.watched("vendor", 3).await;
        assert!(h.for_doc("textDocument/documentSymbol", "vendor/package/Contact.php").await.is_null());
    }
}

#[cfg(unix)]
#[tokio::test]
async fn watcher_keeps_an_open_file_after_its_loaded_alias_changes_target() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php")], config, client_capabilities()).await;
    let external = tempfile::tempdir().unwrap();
    let physical = external.path().join("Original.php");
    std::fs::write(&physical, "<?php class Original {}").unwrap();
    let alias = h.workspace.join("Alias.php");
    std::os::unix::fs::symlink(&physical, &alias).unwrap();
    h.watched("Alias.php", 1).await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "Alias.php").await[0]["name"], "Original");
    let uri = super::harness::url(&physical.canonicalize().unwrap());
    h.client
        .send_notification(
            "textDocument/didOpen",
            serde_json::json!({"textDocument": {
                "uri": uri, "languageId": "php", "version": 1, "text": "<?php class Unsaved {}"
            }}),
        )
        .await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "Alias.php").await[0]["name"], "Unsaved");
    let replacement = h.workspace.join("Replacement.php");
    std::fs::write(&replacement, "<?php class Replacement {}").unwrap();
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&replacement, &alias).unwrap();
    h.watched("Replacement.php", 1).await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "Alias.php").await[0]["name"], "Unsaved");
}

#[cfg(unix)]
#[tokio::test]
async fn physical_package_deletion_removes_files_behind_a_dangling_symlink() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php")], config, client_capabilities()).await;
    let external = tempfile::tempdir().unwrap();
    let package = external.path().join("package");
    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("Contact.php"), "<?php class Contact {}").unwrap();
    std::fs::create_dir(h.workspace.join("vendor")).unwrap();
    std::os::unix::fs::symlink(&package, h.workspace.join("vendor/package")).unwrap();
    h.watched("vendor", 1).await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "vendor/package/Contact.php").await[0]["name"], "Contact");
    let uri = super::harness::url(&package.canonicalize().unwrap());
    std::fs::remove_dir_all(&package).unwrap();
    h.client
        .send_notification("workspace/didChangeWatchedFiles", serde_json::json!({"changes": [{"uri": uri, "type": 3}]}))
        .await;
    assert!(h.for_doc("textDocument/documentSymbol", "vendor/package/Contact.php").await.is_null());
}

#[cfg(unix)]
#[tokio::test]
async fn physical_package_restore_reloads_closed_dependencies_and_keeps_buffers() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(
        &[
            ("a.php", "<?php function contact(): Contact { return new Contact(); }"),
            ("mago.toml", "[source]\npaths = ['a.php']\nincludes = ['vendor']\n"),
        ],
        config,
        client_capabilities(),
    )
    .await;
    let external = tempfile::tempdir().unwrap();
    let package = external.path().join("package");
    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("Contact.php"), "<?php class Contact {}").unwrap();
    std::fs::write(package.join("Buffered.php"), "<?php class Buffered {}").unwrap();
    std::fs::create_dir(h.workspace.join("vendor")).unwrap();
    std::os::unix::fs::symlink(&package, h.workspace.join("vendor/package")).unwrap();
    h.watched("vendor", 1).await;
    h.open("vendor/package/Buffered.php", "<?php class UnsavedBuffered {}").await;
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    assert_no_errors(&current);
    let physical_uri = super::harness::url(&package.canonicalize().unwrap());

    std::fs::remove_dir_all(&package).unwrap();
    h.client
        .send_notification(
            "workspace/didChangeWatchedFiles",
            serde_json::json!({"changes": [{"uri": physical_uri, "type": 3}]}),
        )
        .await;
    assert_eq!(
        h.for_doc("textDocument/documentSymbol", "vendor/package/Buffered.php").await[0]["name"],
        "UnsavedBuffered"
    );
    assert!(h.for_doc("textDocument/documentSymbol", "vendor/package/Contact.php").await.is_null());
    collect(&mut h, &mut current).await;
    assert!(current[&h.url("a.php")].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-class"));

    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("Contact.php"), "<?php class Contact {}").unwrap();
    std::fs::write(package.join("Buffered.php"), "<?php class RestoredBuffered {}").unwrap();
    h.client
        .send_notification(
            "workspace/didChangeWatchedFiles",
            serde_json::json!({"changes": [{"uri": physical_uri, "type": 1}]}),
        )
        .await;
    collect(&mut h, &mut current).await;
    assert_no_errors(&current);
    assert_eq!(h.for_doc("textDocument/documentSymbol", "vendor/package/Contact.php").await[0]["name"], "Contact");
    assert_eq!(
        h.for_doc("textDocument/documentSymbol", "vendor/package/Buffered.php").await[0]["name"],
        "UnsavedBuffered"
    );
    h.close("vendor/package/Buffered.php").await;
    assert_eq!(
        h.for_doc("textDocument/documentSymbol", "vendor/package/Buffered.php").await[0]["name"],
        "RestoredBuffered"
    );
}

#[tokio::test]
async fn dependency_symbol_queries_do_not_publish_lint_on_later_disk_changes() {
    let mut h = Harness::start(&[
        ("a.php", "<?php class HostClass {}"),
        ("dependency.php", "<?php class DependencyClass {}"),
        ("mago.toml", "[source]\npaths = ['a.php']\nincludes = ['dependency.php']\n"),
    ])
    .await;
    let initial = h.diagnostics().await;
    assert!(
        initial
            .iter()
            .any(|entry| entry["uri"] == h.url("a.php") && !entry["diagnostics"].as_array().unwrap().is_empty())
    );
    assert_eq!(h.for_doc("textDocument/documentSymbol", "dependency.php").await[0]["name"], "DependencyClass");
    std::fs::write(h.workspace.join("a.php"), "<?php class ChangedHostClass {}").unwrap();
    h.watched("a.php", 2).await;
    let diagnostics = h.diagnostics().await;
    assert!(
        diagnostics.iter().all(|entry| entry["uri"] != h.url("dependency.php") || entry["diagnostics"].as_array().unwrap().is_empty()),
        "{diagnostics:#?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn physical_package_events_use_startup_aliases_without_open_documents() {
    let workspace = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let package = external.path().join("package");
    std::fs::create_dir(&package).unwrap();
    std::fs::create_dir(workspace.path().join("vendor")).unwrap();
    std::os::unix::fs::symlink(&package, workspace.path().join("vendor/package")).unwrap();
    let source = "<?php function contact(): Contact { return new Contact(); }";
    std::fs::write(workspace.path().join("a.php"), source).unwrap();
    std::fs::write(workspace.path().join("mago.toml"), "[source]\npaths = ['a.php']\nincludes = ['vendor']\n").unwrap();
    std::fs::write(package.join("Contact.php"), "<?php class Contact {}").unwrap();
    let config = ServerConfig {
        linter: false,
        workspace_override: Some(workspace.path().to_path_buf()),
        ..ServerConfig::default()
    };
    let mut h = Harness::start_with(&[], config, client_capabilities()).await;
    let root = workspace.path().canonicalize().unwrap();
    let caller_uri = super::harness::url(&root.join("a.php"));
    let alias_uri = super::harness::url(&root.join("vendor/package/Contact.php"));
    let package_uri = super::harness::url(&package.canonicalize().unwrap());
    let file_uri = super::harness::url(&package.join("Contact.php").canonicalize().unwrap());
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    assert_no_errors(&current);

    std::fs::write(package.join("Contact.php"), "<?php class Contact { public const CHANGED = 1; }").unwrap();
    h.client
        .send_notification(
            "workspace/didChangeWatchedFiles",
            serde_json::json!({"changes": [{"uri": file_uri, "type": 2}]}),
        )
        .await;
    let outline =
        h.request("textDocument/documentSymbol", serde_json::json!({"textDocument": {"uri": alias_uri}})).await;
    assert!(outline.as_array().unwrap().iter().any(|symbol| symbol["name"] == "CHANGED"), "{outline}");

    for _ in 0..2 {
        std::fs::remove_dir_all(&package).unwrap();
        h.client
            .send_notification(
                "workspace/didChangeWatchedFiles",
                serde_json::json!({"changes": [{"uri": package_uri, "type": 3}]}),
            )
            .await;
        collect(&mut h, &mut current).await;
        assert!(current[&caller_uri].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-class"));
        assert!(
            h.request("textDocument/documentSymbol", serde_json::json!({"textDocument": {"uri": alias_uri}}))
                .await
                .is_null()
        );

        std::fs::create_dir(&package).unwrap();
        std::fs::write(package.join("Contact.php"), "<?php class Contact {}").unwrap();
        h.client
            .send_notification(
                "workspace/didChangeWatchedFiles",
                serde_json::json!({"changes": [{"uri": package_uri, "type": 1}]}),
            )
            .await;
        collect(&mut h, &mut current).await;
        assert_no_errors(&current);
        let definition =
            h.at_uri("textDocument/definition", &caller_uri, 0, source.find("Contact").unwrap() as u32).await;
        assert_eq!(definition["uri"], alias_uri);
    }
}

#[tokio::test]
async fn unfinished_control_heads_publish_only_their_parse_errors() {
    const LIBRARY: &str = "<?php\n\ndeclare(strict_types=1);\n\nnamespace Demo;\n\nenum Kind\n{\n    case Alpha;\n}\n";
    const RETURN: &str = "\n        return 1;\n    }\n}\n";
    const BRACE: &str = "\n    }\n}\n";
    const UNCLOSED_BODIES: [&str; 3] = [
        "Expected one of `RightBrace` before end of file",
        "Unexpected end of file",
        "Expected one of `RightBrace` before end of file",
    ];
    let method = |body: &str| {
        format!(
            "<?php\n\ndeclare(strict_types=1);\n\nnamespace Demo;\n\nfinal class Controller\n{{\n    public function run(int $a, Kind $kind, Controller $other): int\n    {{\n        {body}"
        )
    };
    let script = |body: &str| {
        format!("<?php\n\ndeclare(strict_types=1);\n\nfunction run(int $a): int\n{{\n    return $a;\n}}\n\n{body}")
    };
    let position = |source: &str, offset: usize| {
        let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
        serde_json::json!({ "line": source[..offset].matches('\n').count(), "character": offset - line_start })
    };
    let unclosed = (
        Some(("Expected one of `RightParenthesis`, found `Return`", "return")),
        "Expected one of `RightParenthesis`, found `RightBrace`",
        "Expected one of `RightParenthesis` before end of file",
    );
    let member = (
        Some(("Expected one of `RightParenthesis`, found `LiteralInteger`", "1;")),
        "Expected one of `Variable`, `DollarLeftBrace`, `Dollar`, `LeftBrace`, `Identifier`, found `RightBrace`",
        unclosed.2,
    );
    let bodiless = (None, "Unexpected token `RightBrace`", "Unexpected end of file");
    let valid = method("return $a;\n    }\n}\n");
    for config in [ServerConfig::default(), ServerConfig { analyzer: false, ..ServerConfig::default() }] {
        let mut h =
            Harness::start_with(&[("Kind.php", LIBRARY), ("Controller.php", &valid)], config, client_capabilities())
                .await;
        h.open("Controller.php", &valid).await;
        let mut current = BTreeMap::new();
        collect(&mut h, &mut current).await;
        assert_eq!(current[&h.url("Controller.php")], serde_json::json!([]));
        let mut version = 1;
        for (head, (at_return, at_brace, at_eof)) in [
            ("if ($a", unclosed),
            ("if (true", unclosed),
            ("if ($a) {} elseif ($a", unclosed),
            ("while ($a", unclosed),
            ("while ($kind === Kind::Alpha", (unclosed.0, unclosed.1, "Unexpected end of file")),
            ("do {} while ($a", unclosed),
            ("switch ($a", unclosed),
            ("if ($this->", member),
            ("while ($other->", member),
            ("if ($a)", bodiless),
            ("while ($a)", bodiless),
            ("foreach ([1] as $item)", bodiless),
        ] {
            let mut cases = vec![
                (method(&format!("{head}{BRACE}")), vec![at_brace], "}"),
                (method(head), [at_eof].into_iter().chain(UNCLOSED_BODIES).collect(), ""),
                (script(head), vec![at_eof], ""),
            ];
            if let Some((message, anchor)) = at_return {
                cases.push((method(&format!("{head}{RETURN}")), vec![message], anchor));
            }
            for (source, expected, anchor) in cases {
                version += 1;
                h.change("Controller.php", &source, version).await;
                collect(&mut h, &mut current).await;
                let diagnostics = current[&h.url("Controller.php")].as_array().unwrap();
                let published: Vec<_> = diagnostics
                    .iter()
                    .map(|diagnostic| {
                        (diagnostic["code"].as_str().unwrap(), diagnostic["message"].as_str().unwrap().lines().next())
                    })
                    .collect();
                let expected: Vec<_> = expected.into_iter().map(|message| ("parse", Some(message))).collect();
                assert_eq!(published, expected, "{source}");
                let after = source.rfind(head).unwrap() + head.len();
                let start = position(&source, after + source[after..].find(anchor).unwrap());
                assert!(diagnostics.iter().all(|diagnostic| diagnostic["range"]["start"] == start), "{source}");
            }
        }
        version += 1;
        h.change("Controller.php", &valid, version).await;
        collect(&mut h, &mut current).await;
        assert_eq!(current[&h.url("Controller.php")], serde_json::json!([]));
    }
}

#[tokio::test]
async fn unfinished_control_heads_keep_lints_of_their_complete_bodies() {
    const FUNCTION: &str =
        "<?php\n\ndeclare(strict_types=1);\n\nnamespace Demo;\n\nfunction run(int $a, array $items): int\n{\n    ";
    const TEMPLATE: &str = "<?php declare(strict_types=1); ?>\n<ul>\n<?php foreach ($items as $item): ?>\n  <?php ";
    const IF_CHAIN: &str = " {\n        if ($a == 2) {\n            return 2;\n        }\n    } elseif ($a < 0) {\n        foreach ($items as $item) echo $item;\n    } else {\n        return 3;\n    }\n\n    return 1;\n}\n";
    const SWITCH: &str = " {\n        case 1:\n            foreach ($items as $item) echo $item;\n            break;\n        default:\n            if ($a == 2) {\n                return 2;\n            }\n    }\n\n    return 1;\n}\n";
    const ALTERNATIVE_IF: &str = ": ?>\n    <li><?php echo $item == 1 ? 'a' : 'b'; ?></li>\n    <?php foreach ($item->children as $child) echo $child; ?>\n  <?php endif; ?>\n<?php endforeach; ?>\n</ul>\n";
    const ALTERNATIVE_WHILE: &str = ":\n        if ($a == 2) {\n            return 2;\n        }\n        $a--;\n    endwhile;\n\n    return 1;\n}\n";
    let branches =
        format!("{FUNCTION}if ($a > 5) {{\n        if ($a == 6) {{\n            return 6;\n        }}\n    }} ");
    let do_body = format!(
        "{FUNCTION}do {{\n        if ($a == 4) {{\n            return 4;\n        }}\n        foreach ($items as $item) echo $item;\n    }} "
    );
    let offset = |source: &str, position: &Value| {
        let line = position["line"].as_u64().unwrap() as usize;
        let start: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
        (start + position["character"].as_u64().unwrap() as usize) as u32
    };
    let lints = |source: &str, diagnostics: &Value| -> Vec<(String, u32, u32)> {
        diagnostics
            .as_array()
            .unwrap()
            .iter()
            .filter(|diagnostic| diagnostic["code"] != "parse")
            .map(|diagnostic| {
                let range = &diagnostic["range"];
                (
                    diagnostic["code"].as_str().unwrap().to_owned(),
                    offset(source, &range["start"]),
                    offset(source, &range["end"]),
                )
            })
            .collect()
    };
    let config = ServerConfig { analyzer: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php\n")], config, client_capabilities()).await;
    h.open("a.php", "<?php\n").await;
    let mut current = BTreeMap::new();
    collect(&mut h, &mut current).await;
    let mut version = 1;
    for (before, head, complete, edited, after, kept) in [
        (
            FUNCTION,
            "if ($a > 1",
            ")",
            "",
            IF_CHAIN,
            ["identity-comparison", "block-statement", "no-else-clause"].as_slice(),
        ),
        (FUNCTION, "switch ($a", ")", "", SWITCH, ["block-statement", "identity-comparison"].as_slice()),
        (
            TEMPLATE,
            "if ($item->visible",
            ")",
            "",
            ALTERNATIVE_IF,
            ["identity-comparison", "block-statement"].as_slice(),
        ),
        (FUNCTION, "while ($a > 1", ")", "", ALTERNATIVE_WHILE, ["identity-comparison"].as_slice()),
        (
            do_body.as_str(),
            "while ($a > 1",
            ")",
            "",
            ";\n\n    return 1;\n}\n",
            ["identity-comparison", "block-statement"].as_slice(),
        ),
        (branches.as_str(), "", "", "elseif ($a > 6", "\n\n    return 1;\n}\n", ["identity-comparison"].as_slice()),
    ] {
        let complete_source = format!("{before}{head}{complete}{after}");
        let edited_source = format!("{before}{head}{edited}{after}");
        let mut published = Vec::new();
        for source in [&complete_source, &edited_source] {
            version += 1;
            h.change("a.php", source, version).await;
            collect(&mut h, &mut current).await;
            published.push(current[&h.url("a.php")].clone());
        }
        let parse = published[1].as_array().unwrap().iter().filter(|diagnostic| diagnostic["code"] == "parse").count();
        assert_eq!(parse, 1, "{edited_source}: {:?}", published[1]);
        let start = before.len() as u32;
        let at = start + head.len() as u32;
        let (complete_length, edited_length) = (complete.len() as u32, edited.len() as u32);
        let shift = |offset: u32| match offset {
            offset if offset <= at => Some(offset),
            offset if offset >= at + edited_length => Some(offset - edited_length + complete_length),
            _ => None,
        };
        let mut suppressed = lints(&complete_source, &published[0]);
        let reported = lints(&edited_source, &published[1]);
        for (code, issue_start, issue_end) in &reported {
            let issue = shift(*issue_start).zip(shift(*issue_end)).map(|(start, end)| (code.clone(), start, end));
            let index = issue.and_then(|issue| suppressed.iter().position(|expected| *expected == issue));
            let Some(index) = index else { panic!("{edited_source}: {code} at {issue_start}..{issue_end}") };
            suppressed.swap_remove(index);
        }
        for (code, issue_start, issue_end) in &suppressed {
            assert!(*issue_end > start && *issue_start < at + complete_length, "{edited_source}: lost {code}");
        }
        for code in kept {
            assert!(reported.iter().any(|(reported, ..)| reported == code), "{edited_source}: missing {code}");
        }
    }
}
