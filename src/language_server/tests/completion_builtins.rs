use serde_json::json;

use mago_database::file::File;
use mago_php_version::PHPVersion;

use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};

use super::harness::Harness;

#[tokio::test]
async fn builtin_completions_obey_php_version_in_each_name_context() {
    let header = "<?php\nnamespace App;\nuse function json_validate as checkJson;\nuse Deprecated as Legacy;\nuse const JSON_ERROR_NON_BACKED_ENUM as UnsupportedEnum;\n";
    let cases = [
        ("json_val|;", "json_validate", PHPVersion::PHP83),
        ("\\json_val|;", "json_validate", PHPVersion::PHP83),
        ("checkJ|;", "checkJson", PHPVersion::PHP83),
        ("array_fin|;", "array_find", PHPVersion::PHP84),
        ("new Depre|;", "Deprecated", PHPVersion::PHP84),
        ("new \\Depre|;", "Deprecated", PHPVersion::PHP84),
        ("new Lega|;", "Legacy", PHPVersion::PHP84),
        ("JSON_ERROR_NON_BACKED_E|;", "JSON_ERROR_NON_BACKED_ENUM", PHPVersion::PHP83),
        ("\\JSON_ERROR_NON_BACKED_E|;", "JSON_ERROR_NON_BACKED_ENUM", PHPVersion::PHP83),
        ("Unsupp|;", "UnsupportedEnum", PHPVersion::PHP83),
        ("\\DateTime::createFromT|;", "createFromTimestamp", PHPVersion::PHP84),
        ("function test(\\ReflectionClass $r) { $r->getLazyIni|; }", "getLazyInitializer", PHPVersion::PHP84),
    ];
    let source = format!(
        "{header}{}",
        cases.iter().map(|(source, _, _)| source.replace('|', "")).collect::<Vec<_>>().join("\n")
    );
    for version in [PHPVersion::PHP82, PHPVersion::PHP83, PHPVersion::PHP84] {
        let mut config = ServerConfig::default();
        config.configuration.php_version = version;
        let mut h = Harness::start_with(&[("a.php", &source)], config, json!({})).await;
        for (line, (marked, label, since)) in cases.iter().enumerate() {
            let result = h
                .at(
                    "textDocument/completion",
                    "a.php",
                    (header.lines().count() + line) as u32,
                    marked.find('|').unwrap() as u32,
                )
                .await;
            let found = result["items"].as_array().unwrap().iter().any(|item| item["label"] == *label);
            assert_eq!(found, version >= *since, "PHP {version}, {marked}: {result}");
        }
    }
}

#[tokio::test]
async fn builtin_completion_details_keep_references_variadics_and_source_defaults() {
    let source = "<?php\narray_po;\nprintf;\njson_val;\nDateTimeImmutable::createFromF;\nfunction &reference(string &$value, int &...$rest): string { return $value; }\nreferen;\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (line, label, required) in [
        (1, "array_pop", "&$array"),
        (2, "printf", "...$values"),
        (3, "json_validate", "$depth = 512, int $flags = 0"),
        (4, "createFromFormat", "$timezone = null"),
        (6, "reference", "&reference(string &$value, int &...$rest): string"),
    ] {
        let column = source.lines().nth(line).unwrap().trim_end_matches(';').len() as u32;
        let result = h.at("textDocument/completion", "a.php", line as u32, column).await;
        let item = result["items"].as_array().unwrap().iter().find(|item| item["label"] == label).unwrap();
        assert!(item["detail"].as_str().unwrap().contains(required), "{label}: {item}");
    }
}

#[tokio::test]
async fn builtin_hover_reads_embedded_source_without_a_file_uri() {
    let source = "<?php strlen('x');";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, source.find("strlen").unwrap() as u32);
    let mut h = Harness::start(&[("a.php", source)]).await;
    let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
    assert!(hover["contents"]["value"].as_str().unwrap().contains("function strlen(string $string):"));
    assert!(hover["contents"]["value"].as_str().unwrap().contains("@pure"));
}

#[tokio::test]
async fn completion_replaces_keyword_prefixes_and_reserved_member_names() {
    for (marked, label, selected, expected) in [
        ("<?php print|(\"x\");", "printf", "print", "<?php printf(\"x\");"),
        (
            "<?php class Handler { public function match(): void {} } function test(Handler $h) { $h->mat|ch(); }",
            "match",
            "match",
            "<?php class Handler { public function match(): void {} } function test(Handler $h) { $h->match(); }",
        ),
        (
            "<?php class Handler { public static function match(): void {} } Handler::mat|ch();",
            "match",
            "match",
            "<?php class Handler { public static function match(): void {} } Handler::match();",
        ),
    ] {
        let offset = marked.find('|').unwrap();
        let source = marked.replace('|', "");
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
        let position = position_at_offset(&file, offset as u32);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
        let item = result["items"].as_array().unwrap().iter().find(|item| item["label"] == label).unwrap();
        let range: tower_lsp_server::gen_lsp_types::Range =
            serde_json::from_value(item["textEdit"]["range"].clone()).unwrap();
        let start = offset_at_position(&file, range.start) as usize;
        let end = offset_at_position(&file, range.end) as usize;
        assert_eq!(&source[start..end], selected);
        let mut applied = source.clone();
        applied.replace_range(start..end, item["textEdit"]["newText"].as_str().unwrap());
        assert_eq!(applied, expected);
    }
}

#[tokio::test]
async fn builtin_keyword_functions_complete_in_expressions_but_not_imports() {
    let cases = [
        ("use function clo|;", "clone", false),
        ("use function exi|;", "exit", false),
        ("use function di|;", "die", false),
        ("use function strle|;", "strlen", true),
        ("clo|;", "clone", true),
        ("exi|;", "exit", true),
        ("di|;", "die", true),
        ("\\clo|;", "clone", true),
        ("\\exi|;", "exit", true),
        ("\\di|;", "die", true),
    ];
    let source = format!(
        "<?php\nnamespace App;\n{}",
        cases.iter().map(|(marked, _, _)| marked.replace('|', "")).collect::<Vec<_>>().join("\n")
    );
    let mut config = ServerConfig::default();
    config.configuration.php_version = PHPVersion::PHP85;
    let mut h = Harness::start_with(&[("a.php", &source)], config, json!({})).await;
    for (line, (marked, label, expected)) in cases.iter().enumerate() {
        let result = h.at("textDocument/completion", "a.php", line as u32 + 2, marked.find('|').unwrap() as u32).await;
        let found = result["items"].as_array().unwrap().iter().any(|item| item["label"] == *label);
        assert_eq!(found, *expected, "{marked}: {result}");
    }
}

#[tokio::test]
async fn class_completion_hides_internal_prelude_contracts() {
    for marked in [
        "$x = __INTERNAL_DO|;",
        "$x = \\__INTERNAL_DO|;",
        "$x instanceof __INTERNAL_DO|;",
        "function g(__INTERNAL_DO|) {}",
        "use __INTERNAL_DO|;",
        "use \\__INTERNAL_DO|;",
        "enum E: string implements __INTERNAL_DO| {}",
        "function h(BackedE|) {}",
    ] {
        let source = format!("<?php\n{}\n", marked.replace('|', ""));
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let result = h.at("textDocument/completion", "a.php", 1, marked.find('|').unwrap() as u32).await;
        let labels: Vec<_> =
            result["items"].as_array().unwrap().iter().map(|item| item["label"].as_str().unwrap()).collect();
        assert!(labels.iter().all(|label| !label.contains("INTERNAL_DO_NOT_USE")), "{marked}: {labels:?}");
        if marked.contains("BackedE|") {
            assert!(labels.contains(&"BackedEnum"), "{marked}: {labels:?}");
        }
    }
}
