use serde_json::{Value, json};

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};

async fn completions(marked: &str, supported: bool) -> (String, Vec<Value>) {
    let capabilities = json!({"textDocument": {"completion": {"completionItem": {"snippetSupport": supported}}}});
    completions_with(marked, capabilities).await
}

async fn completions_with(marked: &str, capabilities: Value) -> (String, Vec<Value>) {
    let offset = marked.find('|').unwrap();
    let source = marked.replacen('|', "", 1);
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let mut h = Harness::start_with(&[("a.php", &source)], ServerConfig::default(), capabilities).await;
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    (source, result["items"].as_array().unwrap().clone())
}

async fn snippets(marked: &str, supported: bool) -> (String, Vec<Value>) {
    let (source, mut items) = completions(marked, supported).await;
    items.retain(|item| item["kind"] == 15);
    (source, items)
}

fn apply(source: &str, item: &Value) -> String {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let range =
        serde_json::from_value::<tower_lsp_server::gen_lsp_types::Range>(item["textEdit"]["range"].clone()).unwrap();
    let mut edited = source.to_owned();
    let start = offset_at_position(&file, range.start) as usize;
    let text = item["textEdit"]["newText"].as_str().unwrap();
    // Snippet clients indent continuation lines from the insertion line.
    let text = if item["insertTextMode"] == 1 {
        text.to_owned()
    } else {
        let line = source[..start].rsplit('\n').next().unwrap();
        let indent = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
        text.replace('\n', &format!("\n{indent}"))
    };
    edited.replace_range(start..offset_at_position(&file, range.end) as usize, &text);
    edited
}

fn expand_declaration_snippet(text: &str) -> (String, Vec<(u32, std::ops::Range<usize>)>) {
    let mut text = text.chars().peekable();
    let mut expanded = String::new();
    let mut stops = Vec::new();
    while let Some(character) = text.next() {
        match character {
            '\\' => expanded.push(text.next().expect("escaped character")),
            '$' => {
                let braced = text.next_if_eq(&'{').is_some();
                let mut digits = String::new();
                while text.peek().is_some_and(char::is_ascii_digit) {
                    digits.push(text.next().unwrap());
                }
                let index = digits.parse().expect("numeric tab stop");
                let start = expanded.len();
                if braced {
                    if text.next_if_eq(&':').is_some() {
                        while text.peek().is_some_and(|character| *character != '}') {
                            expanded.push(text.next().unwrap());
                        }
                    }
                    assert_eq!(text.next(), Some('}'));
                }
                stops.push((index, start..expanded.len()));
            }
            character => expanded.push(character),
        }
    }
    (expanded, stops)
}

#[tokio::test]
async fn property_snippets_expand_the_php_dollar_before_the_name_tab_stop() {
    let (source, items) = snippets("<?php class C { p| }", true).await;
    let properties: Vec<_> =
        items.iter().filter(|item| item["label"].as_str().unwrap().ends_with(" property")).collect();
    assert_eq!(properties.len(), 9);
    for item in properties {
        let (expanded, stops) = expand_declaration_snippet(item["textEdit"]["newText"].as_str().unwrap());
        let values: Vec<_> = stops.iter().map(|(index, range)| (*index, &expanded[range.clone()])).collect();
        assert_eq!(values, [(1, "mixed"), (2, "name"), (0, "")], "{item}");
        assert!(expanded.contains("mixed $name;"), "{expanded}");
        let mut item = item.clone();
        item["textEdit"]["newText"] = json!(expanded);
        let php = apply(&source, &item);
        let file = File::ephemeral(b"a.php".as_slice().into(), php.as_bytes().to_vec().into());
        let arena = mago_allocator::LocalArena::new();
        let program = mago_syntax::parser::parse_file_borrowed_with_settings(&arena, &file, Default::default());
        assert!(!program.has_errors(), "{php}: {:?}", program.errors);
    }
}

#[tokio::test]
async fn declaration_snippets_replace_partial_keywords_and_keep_modifiers() {
    for (marked, label, expected) in [
        ("<?php fin|", "final class", "<?php final class ${1:Name}\n{\n    $0\n}"),
        ("<?php final class |", "final class", "<?php final class ${1:Name}\n{\n    $0\n}"),
        ("<?php final cl|ass", "final class", "<?php final class ${1:Name}\n{\n    $0\n}"),
        ("<?php abstract cl|", "abstract class", "<?php abstract class ${1:Name}\n{\n    $0\n}"),
        ("<?php final readonly class |", "final readonly class", "<?php final readonly class ${1:Name}\n{\n    $0\n}"),
        ("<?php interface |", "interface", "<?php interface ${1:Name}\n{\n    $0\n}"),
        ("<?php trait |", "trait", "<?php trait ${1:Name}\n{\n    $0\n}"),
        ("<?php enum |", "enum", "<?php enum ${1:Name}\n{\n    $0\n}"),
        (
            "<?php #[Example(Foo::class)] final class |",
            "final class",
            "<?php #[Example(Foo::class)] final class ${1:Name}\n{\n    $0\n}",
        ),
        ("<?php function |", "function", "<?php function ${1:name}(${2}): ${3:void}\n{\n    $0\n}"),
    ] {
        let (source, items) = snippets(marked, true).await;
        let item = items.iter().find(|item| item["label"] == label).unwrap_or_else(|| panic!("{marked}: {items:?}"));
        assert_eq!(item["insertTextFormat"], 2);
        assert_eq!(apply(&source, item), expected, "{marked}");
    }
}

#[tokio::test]
async fn middle_declaration_keywords_offer_only_complete_snippet_edits() {
    let declarations = concat!(
        "<?php class Clock {} class FunctionValue {} function clock(): void {}\n",
        "class InterfaceValue {} class TraitValue {} class EnumValue {} class ConstValue {} class CaseValue {}\n",
    );
    for (marked, label, expected) in [
        ("final cl|ass", "final class", "final class ${1:Name}\n{\n    $0\n}"),
        (
            "#[AllowDynamicProperties] final cl|ass",
            "final class",
            "#[AllowDynamicProperties] final class ${1:Name}\n{\n    $0\n}",
        ),
        (
            "class A {\n    public fun|ction\n}",
            "public function",
            "class A {\n    public function ${1:name}(${2}): ${3:void}\n    {\n        $0\n    }\n}",
        ),
        ("in|terface", "interface", "interface ${1:Name}\n{\n    $0\n}"),
        ("tr|ait", "trait", "trait ${1:Name}\n{\n    $0\n}"),
        ("en|um", "enum", "enum ${1:Name}\n{\n    $0\n}"),
        ("co|nst", "const", "const ${1:NAME} = ${2:null};$0"),
        ("class A {\n    public co|nst\n}", "public const", "class A {\n    public const ${1:NAME} = ${2:null};$0\n}"),
        ("enum A {\n    ca|se\n}", "case", "enum A {\n    case ${1:Name};$0\n}"),
        ("enum A: int {\n    ca|se\n}", "case", "enum A: int {\n    case ${1:Name} = ${2:value};$0\n}"),
    ] {
        let marked = format!("{declarations}{marked}");
        let (source, items) = completions(&marked, true).await;
        assert!(!items.is_empty(), "{marked}");
        assert!(items.iter().all(|item| item["kind"] == 15), "{marked}: {items:?}");
        let item = items.iter().find(|item| item["label"] == label).unwrap();
        assert_eq!(apply(&source, item), format!("{declarations}{expected}"));
        assert!(completions(&marked, false).await.1.is_empty(), "{marked}");
    }
}

#[tokio::test]
async fn middle_keyword_filter_keeps_expression_and_property_names() {
    for (marked, label) in [
        ("<?php function functionValue(): void {} fun|ctionValue();", "functionValue"),
        ("<?php class FunctionValue {} class A { public Fun|ctionValue $value; }", "FunctionValue"),
    ] {
        let (source, items) = completions(marked, true).await;
        let item = items.iter().find(|item| item["label"] == label).unwrap_or_else(|| panic!("{marked}: {items:?}"));
        assert_ne!(item["kind"], 15, "{item}");
        assert_eq!(apply(&source, item), source);
    }
}

#[tokio::test]
async fn keyword_tokens_keep_type_and_member_completion_contexts() {
    for (marked, label, before, after) in [
        ("<?php class Clock {} new cl|ass;", "Clock", "new class;", "new Clock;"),
        ("<?php class InterfaceValue {} new in|terface;", "InterfaceValue", "new interface;", "new InterfaceValue;"),
        (
            "<?php class Foo { public static function caseValue(): void {} } Foo::ca|se();",
            "caseValue",
            "Foo::case();",
            "Foo::caseValue();",
        ),
        (
            "<?php class Clock {} class A { public cl|ass $value; }",
            "Clock",
            "public class $value",
            "public Clock $value",
        ),
        (
            "<?php class Foo { public static function className(): void {} } Foo::cl|ass();",
            "className",
            "Foo::class();",
            "Foo::className();",
        ),
        (
            "<?php class Foo { public function functionValue(): void {} } function run(Foo $obj): void { $obj->fun|ction(); }",
            "functionValue",
            "$obj->function();",
            "$obj->functionValue();",
        ),
    ] {
        let (source, items) = completions(marked, true).await;
        let item = items.iter().find(|item| item["label"] == label).unwrap_or_else(|| panic!("{marked}: {items:?}"));
        assert!(items.iter().all(|item| item["kind"] != 15), "{marked}: {items:?}");
        assert_eq!(apply(&source, item), source.replace(before, after));
    }
    for (source, label, kind) in
        [("<?php function f(): void {} use fun|ction;", "f", 3), ("<?php const V = 1; use co|nst;", "V", 21)]
    {
        let (_, items) = completions(source, true).await;
        assert!(items.iter().any(|item| item["label"] == label), "{source}: {items:?}");
        assert!(items.iter().all(|item| item["kind"] == kind), "{source}: {items:?}");
    }
}

#[tokio::test]
async fn declaration_snippets_match_members_and_keep_indentation() {
    for (marked, label, expected) in [
        (
            "<?php class A {\n    public function |\n}",
            "public function",
            "<?php class A {\n    public function ${1:name}(${2}): ${3:void}\n    {\n        $0\n    }\n}",
        ),
        (
            "<?php class A { protected static fun| }",
            "protected static function",
            "<?php class A { protected static function ${1:name}(${2}): ${3:void}\n{\n    $0\n} }",
        ),
        (
            "<?php class A { public readonly | }",
            "public readonly property",
            "<?php class A { public readonly ${1:mixed} \\$${2:name};$0 }",
        ),
        (
            "<?php interface A { public function | }",
            "public function",
            "<?php interface A { public function ${1:name}(${2}): ${3:void};$0 }",
        ),
        ("<?php enum A { case | }", "case", "<?php enum A { case ${1:Name};$0 }"),
        ("<?php enum A: string { case | }", "case", "<?php enum A: string { case ${1:Name} = ${2:value};$0 }"),
        (
            "<?php abstract class A { abstract public fun| }",
            "abstract public function",
            "<?php abstract class A { abstract public function ${1:name}(${2}): ${3:void};$0 }",
        ),
    ] {
        let (source, items) = snippets(marked, true).await;
        let item = items.iter().find(|item| item["label"] == label).unwrap_or_else(|| panic!("{marked}: {items:?}"));
        assert_eq!(apply(&source, item), expected, "{marked}");
    }
}

#[tokio::test]
async fn method_inside_class_snippet_adds_the_base_indent_once() {
    let capabilities = json!({"textDocument": {"completion": {
        "insertTextMode": 2,
        "completionItem": {"snippetSupport": true, "insertTextModeSupport": {"valueSet": [1, 2]}}
    }}});
    let (source, items) = completions_with("<?php\n\nfinal class |", capabilities.clone()).await;
    let class = items.iter().find(|item| item["label"] == "final class").unwrap();
    let source = apply(&source, class).replace("${1:Name}", "SnippetCheck").replace("$0", "public function |");
    let (source, items) = completions_with(&source, capabilities).await;
    let method = items.iter().find(|item| item["label"] == "public function").unwrap();
    assert_eq!(method["insertTextMode"], 2);
    assert_eq!(method["textEdit"]["newText"], "public function ${1:name}(${2}): ${3:void}\n{\n    $0\n}");
    let edited = apply(&source, method)
        .replace("${1:name}", "answer")
        .replace("${2}", "int $value")
        .replace("${3:void}", "int")
        .replace("$0", "return $value;");
    assert_eq!(
        edited,
        "<?php\n\nfinal class SnippetCheck\n{\n    public function answer(int $value): int\n    {\n        return $value;\n    }\n}",
    );
}

#[tokio::test]
async fn declaration_snippets_stay_out_of_expressions_comments_and_existing_declarations() {
    for marked in [
        "<?php // final class |",
        "<?php 'final class |';",
        "<?php new final|;",
        "<?php function take(final|) {}",
        "<?php foo(final|);",
        "<?php for (;; final|) {}",
        "<?php final class |Existing {}",
        "<?php final cl|ass Existing {}",
        "<?php class A { final class | }",
        "<?php function a() { public fun| }",
        "<?php interface A { private fun| }",
        "<?php enum A { public readonly | }",
    ] {
        assert!(snippets(marked, true).await.1.is_empty(), "{marked}");
    }
    assert!(snippets("<?php final class |", false).await.1.is_empty());
}

#[tokio::test]
async fn space_trigger_offers_declarations_without_global_symbol_noise() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    assert!(h.server_capabilities["completionProvider"]["triggerCharacters"].as_array().unwrap().contains(&json!(" ")));
    h.open("a.php", "<?php").await;
    for (index, (source, expected)) in
        [("<?php final class ", true), ("<?php $value = ", false), ("<?php // final class ", false)]
            .into_iter()
            .enumerate()
    {
        h.change("a.php", source, index as i32 + 2).await;
        let result = h
            .request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": h.url("a.php") },
                    "position": { "line": 0, "character": source.len() },
                    "context": { "triggerKind": 2, "triggerCharacter": " " }
                }),
            )
            .await;
        let items = result["items"].as_array().unwrap();
        assert_eq!(!items.is_empty(), expected, "{source}: {result}");
        assert!(items.iter().all(|item| item["kind"] == 15), "{result}");
    }
}

#[tokio::test]
async fn space_trigger_after_name_keywords_offers_names() {
    let library = "<?php namespace App; class Target {} interface Contract {} trait Mixin {} function helper(): void {} const LIMIT = 1;";
    let mut h = Harness::start(&[("lib.php", library), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    type Case = (&'static str, &'static [&'static str], &'static [&'static str], &'static [u64]);
    let cases: &[Case] = &[
        ("<?php namespace App; new ", &["Target"], &["Contract", "Mixin", "helper", "LIMIT"], &[7]),
        ("<?php namespace App; throw new ", &["Target"], &["Contract", "Mixin", "helper"], &[7]),
        ("<?php namespace App; function f($x) { return $x instanceof ", &["Target", "Contract"], &["helper"], &[]),
        ("<?php namespace App; class A extends ", &["Target"], &["Contract", "Mixin", "helper"], &[7]),
        ("<?php namespace App; class A implements ", &["Contract"], &["Target", "Mixin", "helper"], &[8]),
        ("<?php namespace App; class A implements Contract, ", &["Contract"], &["Target", "Mixin"], &[8]),
        ("<?php namespace App; class A { use ", &["Mixin"], &["Target", "Contract", "helper"], &[7]),
        ("<?php use ", &[], &["helper", "LIMIT"], &[7, 8, 13]),
        ("<?php use function ", &[], &[], &[3]),
        ("<?php use const ", &[], &[], &[21]),
        ("<?php $value = ", &[], &["Target"], &[]),
        ("<?php $x->new ", &[], &["Target"], &[]),
        ("<?php // new ", &[], &["Target"], &[]),
    ];
    for (index, (source, present, absent, kinds)) in cases.iter().enumerate() {
        h.change("a.php", source, index as i32 + 2).await;
        let result = h
            .request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": h.url("a.php") },
                    "position": { "line": 0, "character": source.len() },
                    "context": { "triggerKind": 2, "triggerCharacter": " " }
                }),
            )
            .await;
        let items = result["items"].as_array().unwrap();
        for label in *present {
            assert!(items.iter().any(|item| item["label"] == *label), "{source} missed {label}: {result}");
        }
        for label in *absent {
            assert!(items.iter().all(|item| item["label"] != *label), "{source} offered {label}: {result}");
        }
        if !kinds.is_empty() {
            assert!(!items.is_empty(), "{source}: {result}");
            assert!(items.iter().all(|item| kinds.contains(&item["kind"].as_u64().unwrap())), "{source}: {result}");
        } else if present.is_empty() {
            assert!(items.iter().all(|item| item["kind"] == 15), "{source}: {result}");
        }
    }
}

#[tokio::test]
async fn declaration_names_offer_only_snippets() {
    for marked in [
        "<?php final class |",
        "<?php interface |",
        "<?php trait |",
        "<?php enum |",
        "<?php function |",
        "<?php const |",
        "<?php class A { public function | }",
        "<?php\nnamespace MagoEditorQa;\nenum State\n{\n    case |\n}",
        "<?php enum State: string { case | }",
        "<?php enum State { #[Deprecated] case | }",
    ] {
        let (_, items) = completions(marked, true).await;
        assert!(!items.is_empty(), "{marked}");
        assert!(items.iter().all(|item| item["kind"] == 15), "{marked}: {items:?}");
    }
    for marked in [
        "<?php class Ex|ception {}",
        "<?php function str|len() {}",
        "<?php class A { public function &str|len() {} }",
        "<?php enum State { case Str|; }",
        "<?php const PHP_VERS|;",
        "<?php const FIRST = 1, PHP_VERS|;",
        "<?php class A { private const int PHP_VERS|; }",
        "<?php class A { private const FIRST = 1, PHP_VERS|; }",
    ] {
        assert!(completions(marked, true).await.1.is_empty(), "{marked}");
    }
    assert!(completions("<?php final class |", false).await.1.is_empty());
    assert!(completions("<?php enum State { case | }", false).await.1.is_empty());
}

#[tokio::test]
async fn enum_values_and_switch_cases_keep_expression_completions() {
    for marked in [
        "<?php enum State: string { case Ready = PHP_VERS|; }",
        "<?php switch ($value) { case PHP_VERS|: break; }",
        "<?php enum State { public function test($value): void { switch ($value) { case PHP_VERS|: break; } } }",
        "<?php enum State { public function test($value): void { switch ($value): case PHP_VERS|: break; endswitch; } }",
    ] {
        let (_, items) = completions(marked, true).await;
        assert!(items.iter().any(|item| item["label"] == "PHP_VERSION"), "{marked}: {items:?}");
        assert!(items.iter().all(|item| item["kind"] != 15), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn completion_inside_import_keywords_keeps_keywords_comments_and_aliases() {
    for (marked, label, expected) in [
        ("<?php function f(): void {} use fun|ction;", "f", "<?php function f(): void {} use function f;"),
        ("<?php function f(): void {} use function|;", "f", "<?php function f(): void {} use function f;"),
        (
            "<?php function f(): void {} use fun|ction /* keep */ old as F;",
            "f",
            "<?php function f(): void {} use function /* keep */ f as F;",
        ),
        ("<?php const V = 1; use con|st;", "V", "<?php const V = 1; use const V;"),
        (
            "<?php namespace Lib { function f(): void {} } namespace App { use Lib\\{fun|ction}; }",
            "f",
            "<?php namespace Lib { function f(): void {} } namespace App { use Lib\\{function f}; }",
        ),
    ] {
        let (source, items) = completions(marked, false).await;
        let item = items.iter().find(|item| item["label"] == label).unwrap_or_else(|| panic!("{marked}: {items:?}"));
        assert_eq!(apply(&source, item), expected, "{marked}");
        let keyword = if marked.contains("con|st") { "const" } else { "function" };
        assert!(item["filterText"].as_str().is_some_and(|filter| filter.starts_with(keyword)), "{item}");
    }
    for marked in [
        "<?php namespace Lib { function f(): void {} } namespace App { use fun|ction Lib\\{f}; }",
        "<?php namespace Lib { const V = 1; } namespace App { use con|st Lib\\{V}; }",
    ] {
        let (_, items) = completions(marked, false).await;
        assert!(items.is_empty(), "A keyword completion must not replace an import group prefix: {marked}: {items:?}");
    }
    let (_, items) = completions("<?php function f(): void {} use fun|ction\nold;", false).await;
    assert!(items.is_empty(), "A completion edit cannot span the keyword and a target on another line: {items:?}");
}

#[tokio::test]
async fn declaration_headers_keep_type_and_import_completions() {
    for (marked, label) in [
        ("<?php class A extends Ex|ception {}", "Exception"),
        ("<?php class A implements Count|able {}", "Countable"),
        ("<?php function f(Ex|ception $error) {}", "Exception"),
        ("<?php class A { public Ex|ception $error; }", "Exception"),
        ("<?php use function str|len;", "strlen"),
    ] {
        let (_, items) = completions(marked, true).await;
        assert!(items.iter().any(|item| item["label"] == label), "{marked}: {items:?}");
    }
}
