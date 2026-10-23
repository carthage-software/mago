use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use super::references::apply;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;
use crate::language_server::capabilities::server_capabilities;

async fn actions(h: &mut Harness, source: &str, needle: &str, only: Value) -> Value {
    let offset = source.find(needle).unwrap();
    let before = &source[..offset];
    let position = json!({
        "line": before.bytes().filter(|byte| *byte == b'\n').count(),
        "character": before.rsplit('\n').next().unwrap().encode_utf16().count()
    });
    let mut context = json!({ "diagnostics": [] });
    if !only.is_null() {
        context["only"] = only;
    }
    h.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": h.url("a.php") },
            "range": { "start": position, "end": position }, "context": context
        }),
    )
    .await
}

#[tokio::test]
async fn import_refactor_preserves_comments_line_endings_and_utf16_edits() {
    let source = "<?php\r\nnamespace App; // keep\r\n\r\n$😀 = new \\Lib\\Thing();\r\n";
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", source).await;
    let result = actions(&mut h, source, "\\Lib", json!(["refactor"])).await;
    let rows = result.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["kind"], "refactor.rewrite");
    assert!(rows[0]["isPreferred"].is_null());
    assert_eq!(
        apply(source, &rows[0]["edit"]["changes"][h.url("a.php")]),
        "<?php\r\nnamespace App; // keep\r\n\r\nuse Lib\\Thing;\r\n\r\n$😀 = new Thing();\r\n"
    );
    for only in [json!(["quickfix"]), json!(["refactor.extract"]), json!(["refac"]), json!([])] {
        assert!(actions(&mut h, source, "\\Lib", only).await.as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn import_refactor_reuses_aliases_and_keeps_namespace_blocks_separate() {
    for (source, expected) in [
        (
            "<?php namespace App; use Lib\\{Thing as Alias}; new \\Lib\\Thing();",
            "<?php namespace App; use Lib\\{Thing as Alias}; new Alias();",
        ),
        ("<?php namespace Lib; new \\Lib\\Thing();", "<?php namespace Lib; new Thing();"),
        ("<?php new \\DateTime();", "<?php new DateTime();"),
        ("<?=new \\Lib\\Thing()?>", "<?php use Lib\\Thing; ?><?=new Thing()?>"),
        (
            "<?php namespace A { use Lib\\Thing as Alias; } namespace B { new \\Lib\\Thing(); }",
            "<?php namespace A { use Lib\\Thing as Alias; } namespace B {\n\nuse Lib\\Thing; new Thing(); }",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        let result = actions(&mut h, source, "new ", json!(["refactor.rewrite"])).await;
        assert!(result.as_array().unwrap().is_empty(), "a cursor outside the name must not select it");
        let needle = if source.contains("\\DateTime") { "\\DateTime" } else { "\\Lib\\Thing" };
        let result = actions(&mut h, source, needle, json!(["refactor.rewrite"])).await;
        assert_eq!(result.as_array().unwrap().len(), 1, "{source}: {result}");
        assert_eq!(apply(source, &result[0]["edit"]["changes"][h.url("a.php")]), expected);
    }
}

#[tokio::test]
async fn import_refactor_declines_conflicts_other_name_kinds_and_broken_php() {
    for source in [
        "<?php namespace App; use Other\\Thing; new \\Lib\\Thing();",
        "<?php namespace App; class Thing {} new \\Lib\\Thing();",
        "<?php namespace App; new \\Lib\\Thing(); class Thing {}",
        "<?php namespace App; new \\Lib\\Thing(); new Thing();",
        "<?php namespace App; new \\Lib\\Thing(); Thing\\work();",
        "<?php namespace App; new \\Lib\\Thing(); echo Thing\\VALUE;",
        "<?php namespace App; new \\Lib\\Thing(); use Other\\Thing;",
        "<?php \\Lib\\Thing();",
        "<?php echo \\Lib\\Thing;",
        "<?php use \\Lib\\Thing;",
        "<?php new \\Lib\\Thing(;",
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(
            actions(&mut h, source, "\\Lib\\Thing", json!(["refactor"])).await.as_array().unwrap().is_empty(),
            "{source}"
        );
    }
}

#[tokio::test]
async fn import_refactor_obeys_provider_settings() {
    let source = "<?php new \\Lib\\Thing();";
    let mut config = ServerConfig::default();
    let capabilities = serde_json::to_value(server_capabilities(&config)).unwrap();
    assert_eq!(
        capabilities["codeActionProvider"]["codeActionKinds"],
        json!(["quickfix", "refactor.rewrite", "refactor.extract"])
    );
    config.configuration.language_server.disable.push(Provider::CodeAction);
    assert!(server_capabilities(&config).code_action_provider.is_none());
    let mut h = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
    assert!(actions(&mut h, source, "\\Lib", Value::Null).await.is_null());
}

#[tokio::test]
async fn import_refactor_obeys_short_tag_settings() {
    let source = "<? new \\Lib\\Thing();";
    for enabled in [true, false] {
        let mut config = ServerConfig::default();
        config.configuration.parser.enable_short_tags = enabled;
        let mut h = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
        assert_eq!(
            actions(&mut h, source, "\\Lib", json!(["refactor"])).await.as_array().unwrap().len(),
            usize::from(enabled)
        );
    }
}

#[tokio::test]
async fn unresolved_class_imports_offer_sorted_choices_and_fix_repeated_bindings() {
    let library = "<?php namespace Zebra; class Thing {} namespace Alpha; class Thing {}";
    let source = "<?php\nnamespace App; // keep\nfunction Thing() {} const Thing = 1;\nfunction build(Thing $v): Thing { return new Thing(); }\nThing(); echo Thing;\n";
    let mut h = Harness::start(&[("a.php", source), ("lib.php", library)]).await;
    let result = actions(&mut h, source, "Thing $v", json!(["quickfix"])).await;
    let rows = result.as_array().unwrap();
    assert_eq!(
        rows.iter().map(|row| row["title"].as_str().unwrap()).collect::<Vec<_>>(),
        ["Import Alpha\\Thing", "Import Zebra\\Thing"]
    );
    for (row, namespace) in rows.iter().zip(["Alpha", "Zebra"]) {
        assert_eq!(row["kind"], "quickfix");
        assert!(row["isPreferred"].is_null());
        let edits = &row["edit"]["changes"][h.url("a.php")];
        assert_eq!(edits.as_array().unwrap().len(), 1, "only the import needs an edit");
        assert_eq!(apply(source, edits), source.replace("// keep", &format!("// keep\n\nuse {namespace}\\Thing;")));
    }
    assert!(actions(&mut h, source, "Thing $v", json!(["refactor"])).await.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn unresolved_class_imports_keep_aliases_casing_and_namespace_blocks() {
    let library = "<?php namespace Lib; class Thing {}";
    for (source, needle, expected) in [
        (
            "<?php namespace App; use Lib\\Thing as Alias; new Thing();",
            "Thing();",
            "<?php namespace App; use Lib\\Thing as Alias; new Alias();",
        ),
        ("<?php namespace App; new thing();", "thing();", "<?php namespace App;\n\nuse Lib\\Thing; new Thing();"),
        (
            "<?php namespace App { new Thing(); } namespace App { new Thing(); }",
            "Thing();",
            "<?php namespace App {\n\nuse Lib\\Thing; new Thing(); } namespace App { new Thing(); }",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", source), ("lib.php", library)]).await;
        let result = actions(&mut h, source, needle, json!(["quickfix"])).await;
        assert_eq!(result.as_array().unwrap().len(), 1, "{source}: {result}");
        assert_eq!(apply(source, &result[0]["edit"]["changes"][h.url("a.php")]), expected);
    }
}

#[tokio::test]
async fn unresolved_class_imports_decline_captures_aliases_and_nonclass_names() {
    let library = "<?php namespace Lib; class Thing {}";
    for (source, needle) in [
        ("<?php namespace App; use Missing\\Thing; new Thing();", "Thing();"),
        ("<?php namespace App; use Missing as Thing; new Thing();", "Thing();"),
        ("<?php namespace App; new Thing(); use Other\\Thing;", "Thing();"),
        ("<?php namespace App; new Thing(); class Thing {}", "Thing();"),
        ("<?php namespace App; new Thing(); new Thing\\Child();", "Thing();"),
        ("<?php namespace App; new Thing(); Thing\\run();", "Thing();"),
        ("<?php namespace App; new Thing(); echo Thing\\VALUE;", "Thing();"),
        ("<?php new Thing\\Child();", "Thing\\Child"),
        ("<?php use Thing;", "Thing;"),
        ("<?php Thing();", "Thing();"),
        ("<?php echo Thing;", "Thing;"),
        ("<?php new NothingLikeThing();", "NothingLikeThing"),
        ("<?php new Thing(", "Thing("),
        ("<?php new Thing", "Thing"),
    ] {
        let mut h = Harness::start(&[("a.php", source), ("lib.php", library)]).await;
        let result = actions(&mut h, source, needle, json!(["quickfix"])).await;
        assert!(
            result.as_array().unwrap().iter().all(|row| !row["title"].as_str().unwrap().starts_with("Import ")),
            "{source}: {result}"
        );
    }
}

#[tokio::test]
async fn unresolved_class_import_choices_are_bounded() {
    let library =
        format!("<?php {}", (0..23).rev().map(|i| format!("namespace N{i:02}; class Thing {{}} ")).collect::<String>());
    let source = "<?php new Thing();";
    let mut h = Harness::start(&[("a.php", source), ("lib.php", &library)]).await;
    let result = actions(&mut h, source, "Thing", json!(["quickfix"])).await;
    let rows = result.as_array().unwrap();
    assert_eq!(rows.len(), 20);
    assert_eq!(rows[0]["title"], "Import N00\\Thing");
    assert_eq!(rows[19]["title"], "Import N19\\Thing");
}

#[tokio::test]
async fn import_actions_do_not_capture_phpdoc_names() {
    let library = "<?php namespace Lib; class Thing {}";
    for (source, needle, count) in [
        ("<?php namespace App; /** @var Thing\\Child $v */ new Thing();", "Thing();", 0),
        ("<?php namespace App; /** @var Thing $v */ new \\Lib\\Thing();", "\\Lib\\Thing", 0),
        ("<?php namespace App; /** @var Other\\Thing $v */ new Thing();", "Thing();", 1),
        ("<?php namespace App; /** @var \\Thing\\Child $v */ new Thing();", "Thing();", 1),
    ] {
        let mut h = Harness::start(&[("a.php", source), ("lib.php", library)]).await;
        let result = actions(&mut h, source, needle, Value::Null).await;
        let imports: Vec<_> = result
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["title"].as_str().unwrap().starts_with("Import ") || row["kind"] == "refactor.rewrite")
            .collect();
        assert_eq!(imports.len(), count, "{source}: {result}");
        for action in imports {
            apply(source, &action["edit"]["changes"][h.url("a.php")]);
        }
    }
}
