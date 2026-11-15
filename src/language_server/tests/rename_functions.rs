use super::harness::Harness;
use super::references::{apply, prepare_at, rename_at};

#[tokio::test]
async fn function_rename_declines_quoted_callbacks_in_other_files() {
    let declaration = "<?php function run(): void {}";
    for tail in [
        "$fn = 'run'; $fn();",
        "('run')();",
        "array_map('run', []);",
        "$fn = 'RuN'; $fn();",
        "$fn = \"ru\\x6e\"; $fn();",
        "$fn = b'run'; $fn();",
        "$fn = '\\run'; $fn();",
    ] {
        let caller = format!("<?php {tail}");
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", &caller)]).await;
        let position = declaration.find("run").unwrap() as u32;
        assert!(prepare_at(&mut h, "a.php", 0, position).await.is_err(), "{tail}");
        assert!(rename_at(&mut h, "a.php", declaration, "run", "execute").await.is_err(), "{tail}");
    }
}

#[tokio::test]
async fn function_rename_checks_decoded_fully_qualified_callback_names() {
    let declaration = "<?php namespace Lib; function run(): void {}";
    for literal in ["'Lib\\run'", "'\\LIB\\RUN'", "\"Lib\\\\ru\\x6e\"", "B'Lib\\run'"] {
        let caller = format!("<?php namespace App; $fn = {literal}; $fn();");
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", &caller)]).await;
        let position = declaration.find("run").unwrap() as u32;
        assert!(prepare_at(&mut h, "a.php", 0, position).await.is_err(), "{literal}");
        assert!(rename_at(&mut h, "a.php", declaration, "run", "execute").await.is_err(), "{literal}");
    }
}

#[tokio::test]
async fn function_rename_does_not_capture_a_proposed_callback_name() {
    for (namespace, literal) in [("", "'execute'"), ("namespace Lib;", "'\\LIB\\EXECUTE'")] {
        let source =
            format!("<?php {namespace} function run(): void {{}} $fn = {literal}; if (is_callable($fn)) {{ $fn(); }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let position = source.find("run").unwrap() as u32;
        let prepared = h.at("textDocument/prepareRename", "a.php", 0, position).await;
        assert_eq!(prepared["placeholder"], "run");
        assert!(rename_at(&mut h, "a.php", &source, "run", "execute").await.is_err(), "{source}");
    }
}

#[tokio::test]
async fn function_rename_keeps_unrelated_literals_and_updates_first_class_callables() {
    let source = "<?php function run(): void {} run(); $fn = run(...); $fn(); $a = 'other'; $b = 'Other\\run'; $c = 'C::run'; // 'run'\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let position = source.find("run").unwrap() as u32;
    assert_eq!(h.at("textDocument/prepareRename", "a.php", 0, position).await["placeholder"], "run");
    let edit = rename_at(&mut h, "a.php", source, "run", "execute").await.unwrap();
    assert_eq!(
        apply(source, &edit["changes"][h.url("a.php")]),
        source.replace("run()", "execute()").replace("run(...)", "execute(...)")
    );
}

#[tokio::test]
async fn function_rename_does_not_resolve_callback_strings_through_imports_or_namespaces() {
    let declaration = "<?php namespace Lib; function run(): void {}";
    let caller = "<?php namespace App; use function Lib\\run as invoke; invoke(); $fn = 'run'; $fn();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", caller)]).await;
    let position = declaration.find("run").unwrap() as u32;
    assert_eq!(h.at("textDocument/prepareRename", "a.php", 0, position).await["placeholder"], "run");
    let edit = rename_at(&mut h, "a.php", declaration, "run", "execute").await.unwrap();
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("run", "execute"));
    assert_eq!(apply(caller, &edit["changes"][h.url("b.php")]), caller.replace("Lib\\run", "Lib\\execute"));
}

#[tokio::test]
async fn function_alias_rename_leaves_matching_runtime_strings_alone() {
    let declaration = "<?php namespace Lib; function run(): void {}";
    let caller = "<?php namespace App; use function Lib\\run as invoke; invoke(); $fn = 'invoke'; $fn();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", caller)]).await;
    let position = caller.find("invoke").unwrap() as u32;
    assert_eq!(h.at("textDocument/prepareRename", "b.php", 0, position).await["placeholder"], "invoke");
    let edit = rename_at(&mut h, "b.php", caller, "invoke", "perform").await.unwrap();
    assert_eq!(edit["changes"].as_object().unwrap().len(), 1);
    assert_eq!(
        apply(caller, &edit["changes"][h.url("b.php")]),
        caller.replace("as invoke;", "as perform;").replace("invoke()", "perform()")
    );
}
