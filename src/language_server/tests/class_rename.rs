use super::harness::Harness;
use super::references::{apply, rename_at};

#[tokio::test]
async fn class_rename_declines_literal_callable_receivers_and_new_name_capture() {
    for expression in [
        "['C', 'run']",
        "'C::run'",
        "['\\C', 'run']",
        "'\\c::run'",
        "[\"\\x43\", 'run']",
        "[$key => 'C', 1 => 'run']",
        "['C', 'run', ...$items]",
        "[...['C'], 'run']",
        "['C', $method]",
        "['Changed', 'run']",
        "'Changed::run'",
    ] {
        let source = format!(
            "<?php class C {{ static function run(): void {{}} }} $key = 0; $items = []; $method = 'run'; $cb = {expression};"
        );
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(rename_at(&mut h, "a.php", &source, "C {", "Changed").await.is_err(), "{expression}");
    }
}

#[tokio::test]
async fn class_rename_preserves_unrelated_runtime_names_and_class_expressions() {
    let declaration = "<?php namespace Lib; class C { static function run(): void {} }";
    let caller = "<?php class C { static function run(): void {} } $a = 'C::run'; $b = ['C', 'run']; $c = [\\Lib\\C::class, 'run']; \\Lib\\C::run();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", caller)]).await;
    let edit = rename_at(&mut h, "a.php", declaration, "C {", "Changed").await.unwrap();
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("class C", "class Changed"));
    assert_eq!(apply(caller, &edit["changes"][h.url("b.php")]), caller.replace("\\Lib\\C", "\\Lib\\Changed"));
}

#[tokio::test]
async fn class_callable_guards_keep_only_possible_last_receiver_values() {
    for expression in [
        "[0 => 'C', 1 => 'run', 0 => 'Other']",
        "[0 => 'C', 1 => 'run', ...$items, 0 => 'Other']",
        "[$key => 'Other', 1 => 'run']",
    ] {
        let source = format!(
            "<?php class C {{}} class Other {{ static function run(): void {{}} }} $key = 0; $items = []; $cb = {expression}; new C();"
        );
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let edit = rename_at(&mut h, "a.php", &source, "C {", "Changed").await.unwrap();
        let expected = source.replace("class C", "class Changed").replace("new C()", "new Changed()");
        assert_eq!(apply(&source, &edit["changes"][h.url("a.php")]), expected, "{expression}");
    }
    let source = "<?php class C { static function run(): void {} } $cb = ['C', 'run']; new C();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "C {", "c").await.unwrap();
    let expected = source.replace("class C", "class c").replace("new C()", "new c()");
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected);
}

#[tokio::test]
async fn class_rename_clears_constructor_errors_when_the_definition_arrives_last() {
    use super::harness::client_capabilities;
    use crate::language_server::ServerConfig;

    let declaration = "<?php namespace Entity; class Contact { public function __construct(public string $name) {} }";
    let fixture = "<?php namespace App; use Entity\\Contact;
        interface Manager { public function persist(object $value): void; }
        final class Fixtures {
            private const array CONTACTS = ['A', 'B'];
            public function load(Manager $manager): void { $this->populate($manager); }
            private function populate(Manager $manager): void {
                foreach (self::CONTACTS as $name) { $manager->persist(new Contact(name: $name)); }
            }
        }";
    for buffers in [false, true] {
        let config = ServerConfig { linter: false, ..ServerConfig::default() };
        let mut h = Harness::start_with(
            &[("Contact.php", declaration), ("Fixtures.php", fixture)],
            config,
            client_capabilities(),
        )
        .await;
        if buffers {
            h.open("Contact.php", declaration).await;
            h.open("Fixtures.php", fixture).await;
        }
        h.diagnostics().await;
        for (version, rename) in [(2, true), (3, false)] {
            for (name, original) in [("Fixtures.php", fixture), ("Contact.php", declaration)] {
                let source = if rename { original.replace("Contact", "SomePerson") } else { original.to_owned() };
                if buffers {
                    h.change(name, &source, version).await;
                } else {
                    std::fs::write(h.workspace.join(name), source).unwrap();
                    h.watched(name, 2).await;
                }
                let publications = h.diagnostics().await;
                let diagnostics = publications
                    .iter()
                    .find(|entry| entry["uri"] == h.url("Fixtures.php"))
                    .expect("the fixture's diagnostics must refresh after each edit")["diagnostics"]
                    .as_array()
                    .unwrap();
                if name == "Fixtures.php" {
                    assert!(diagnostics.iter().any(|issue| issue["code"] == "non-existent-class"), "{diagnostics:#?}");
                } else {
                    assert!(diagnostics.iter().all(|issue| issue["severity"] != 1), "{diagnostics:#?}");
                }
            }
        }
    }
}
