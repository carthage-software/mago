use serde_json::json;

use super::harness::Harness;

#[tokio::test]
async fn member_navigation_keeps_all_union_targets_and_declaring_classes() {
    let definitions = "<?php\nclass Base { public function run(): void {} }\nclass Child extends Base {}\nclass Other { public function run(): void {} }\n";
    let source = "<?php\nfunction call_it(Child|Other $value): void {\n    $value->run();\n}\n";
    let mut h = Harness::start(&[("defs.php", definitions), ("use.php", source)]).await;
    let definitions = h.at("textDocument/definition", "use.php", 2, 13).await;
    let targets = definitions.as_array().expect("a union has two distinct definitions");
    assert_eq!(targets.len(), 2);
    let mut lines = targets.iter().map(|target| target["range"]["start"]["line"].as_u64().unwrap()).collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(lines, [1, 3]);
    assert!(targets.iter().all(|target| target["uri"] == h.url("defs.php")));
    let hover = h.at("textDocument/hover", "use.php", 2, 13).await;
    let markdown = hover["contents"]["value"].as_str().unwrap();
    assert!(
        markdown.contains("Base\n\npublic function run") && markdown.contains("Other\n\npublic function run"),
        "{markdown}"
    );
}

#[tokio::test]
async fn hover_keeps_declaring_names_variadics_and_references() {
    let source = r"<?php
namespace CarthageSoftware\CQRS;
interface Command {}
interface CommandBusInterface {
    public function Dispatch(Command ...$commands): void;
    public function &Take(string &$first, int &...$rest): string;
}
interface Child extends CommandBusInterface {}
function call_it(Child $bus, Command $command, string &$first, int &$count): void {
    $bus->dispatch($command);
    $bus->take($first, $count);
}
function &TakeGlobal(string &$first, int &...$rest): string { return $first; }
takeglobal($first, $count);
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needles, expected) in [
        (
            ["function Dispatch", "->dispatch"],
            "CarthageSoftware\\CQRS\\CommandBusInterface\n\npublic function Dispatch(Command ...$commands): void",
        ),
        (
            ["function &Take(", "->take("],
            "CarthageSoftware\\CQRS\\CommandBusInterface\n\npublic function &Take(string &$first, int &...$rest): string",
        ),
        (
            ["function &TakeGlobal(", "takeglobal("],
            "CarthageSoftware\\CQRS\n\nfunction &TakeGlobal(string &$first, int &...$rest): string",
        ),
    ] {
        for needle in needles {
            let name = needle.trim_end_matches('(');
            let offset = source.find(needle).unwrap() + name.len() - 1;
            let before = &source[..offset];
            let line = before.bytes().filter(|byte| *byte == b'\n').count() as u32;
            let column = before.rsplit('\n').next().unwrap().len() as u32;
            let hover = h.at("textDocument/hover", "a.php", line, column).await;
            assert_eq!(hover["contents"]["kind"], "plaintext");
            assert_eq!(hover["contents"]["value"], expected, "{needle}: {hover}");
        }
    }
}

#[tokio::test]
async fn chained_and_relative_method_calls_navigate_to_the_declaration() {
    let source = "<?php\nclass Base { public static function make(): static { return new static(); } public function run(): void {} }\nclass Child extends Base { public function test(): void { self::make()->run(); static::make(); parent::make(); } }\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["self::make", "static::make", "parent::make", "->run"] {
        let offset = source.find(needle).unwrap() + needle.len() - 2;
        let prefix = &source[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
        let character = prefix.rsplit('\n').next().unwrap().len() as u32;
        let target = h.at("textDocument/definition", "a.php", line, character).await;
        assert_eq!(target["range"]["start"]["line"], 1, "{needle}: {target}");
    }
}

#[tokio::test]
async fn global_navigation_distinguishes_symbol_kinds_and_constant_case() {
    let source = "<?php\nclass Same {}\nfunction Same(): int { return 1; }\nconst Same = 1;\nconst SAME = 2;\nnew Same();\nSame();\necho Same;\necho SAME;\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (line, character, expected) in [(5, 5, 1), (6, 1, 2), (7, 6, 3), (8, 6, 4)] {
        let target = h.at("textDocument/definition", "a.php", line, character).await;
        assert_eq!(target["range"]["start"]["line"], expected, "{target}");
        assert_eq!(
            target["range"]["end"]["character"].as_u64().unwrap()
                - target["range"]["start"]["character"].as_u64().unwrap(),
            4
        );
    }
}

#[tokio::test]
async fn variable_definitions_use_their_own_scope() {
    let source = "<?php\nfunction one($item) { return $item; }\nfunction two($item) { return $item; }\n// $item is only a comment\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for line in [1, 2] {
        let target = h.at("textDocument/definition", "a.php", line, 30).await;
        assert_eq!(target["range"]["start"], json!({ "line": line, "character": 13 }));
    }
    assert!(h.at("textDocument/definition", "a.php", 3, 5).await.is_null());
}

#[tokio::test]
async fn properties_constants_and_enum_cases_keep_exact_navigation_ranges() {
    let source = "<?php\nclass Base { public int $value; public static string $label; public const FLAG = 1; }\nclass Child extends Base { public function test(): void { echo $this->value, self::$label, parent::FLAG; } }\nenum Status { case Ready; }\nfunction check(Child $item): void { echo $item?->value, Child::$label, Child::FLAG, Status::Ready; }\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, name, line, expected_hover) in [
        ("$this->value", "value", 1, "Base\n\npublic int $value"),
        ("self::$label", "$label", 1, "Base\n\npublic static string $label"),
        ("parent::FLAG", "FLAG", 1, "Base\n\npublic const FLAG"),
        ("$item?->value", "value", 1, "Base\n\npublic int $value"),
        ("Child::$label", "$label", 1, "Base\n\npublic static string $label"),
        ("Child::FLAG", "FLAG", 1, "Base\n\npublic const FLAG"),
        ("Status::Ready", "Ready", 3, "Status\n\ncase Ready"),
    ] {
        let offset = source.find(needle).unwrap() + needle.len() - name.len();
        let prefix = &source[..offset];
        let row = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
        let column = prefix.rsplit('\n').next().unwrap().len() as u32;
        let target = h.at("textDocument/definition", "a.php", row, column).await;
        assert_eq!(target["range"]["start"]["line"], line, "{needle}: {target}");
        let start = target["range"]["start"]["character"].as_u64().unwrap() as usize;
        let end = target["range"]["end"]["character"].as_u64().unwrap() as usize;
        let selected = &source.lines().nth(line as usize).unwrap()[start..end];
        assert_eq!(selected.trim_start_matches('$'), name.trim_start_matches('$'), "{needle}");
        let hover = h.at("textDocument/hover", "a.php", row, column).await;
        assert!(
            hover["contents"]["value"].as_str().is_some_and(|text| text.contains(expected_hover)),
            "{needle}: {hover}"
        );
    }
}

#[tokio::test]
async fn property_unions_keep_both_targets_and_hover_ignores_comment_variables() {
    let source = "<?php\nclass A { public int $value; }\nclass B { public string $value; }\nfunction read(A|B $item) { return $item->value; }\n// $item\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let target = h.at("textDocument/definition", "a.php", 3, 42).await;
    assert_eq!(target.as_array().unwrap().len(), 2);
    let hover = h.at("textDocument/hover", "a.php", 3, 42).await;
    let text = hover["contents"]["value"].as_str().unwrap();
    assert!(text.contains("A\n\npublic int $value") && text.contains("B\n\npublic string $value"), "{text}");
    assert!(h.at("textDocument/hover", "a.php", 4, 5).await.is_null());
}

#[tokio::test]
async fn member_references_work_from_calls_and_declarations_across_files() {
    let declarations = "<?php\nclass Base { public int $value; public function run(): void {} public const FLAG = 1; }\nclass Child extends Base {}\nclass Other { public int $value; public function run(): void {} public const FLAG = 2; }\nenum Status { case Ready; }\n";
    let calls = "<?php\nfunction check(Child $a, Other $b) { $a->run(); echo $a->value, Child::FLAG, Status::Ready; $b->run(); echo $b->value, Other::FLAG; }\n";
    let mut h = Harness::start(&[("defs.php", declarations), ("calls.php", calls)]).await;
    for (name, call, declaration_line) in
        [("run", "$a->run", 1), ("$value", "$a->value", 1), ("FLAG", "Child::FLAG", 1), ("Ready", "Status::Ready", 4)]
    {
        let declaration = declarations.lines().nth(declaration_line).unwrap().find(name).unwrap() as u32;
        let call_column =
            calls.lines().nth(1).unwrap().find(call).unwrap() + call.len() - name.trim_start_matches('$').len();
        for (file, line, character) in
            [("defs.php", declaration_line as u32, declaration), ("calls.php", 1, call_column as u32)]
        {
            let result = h
                .request(
                    "textDocument/references",
                    json!({
                        "textDocument": { "uri": h.url(file) }, "position": { "line": line, "character": character },
                        "context": { "includeDeclaration": true }
                    }),
                )
                .await;
            let references = result.as_array().unwrap();
            assert_eq!(references.len(), 2, "{name}: {result}");
            assert!(
                references
                    .iter()
                    .any(|r| r["uri"] == h.url("defs.php") && r["range"]["start"]["line"] == declaration_line),
                "{result}"
            );
            assert!(
                references
                    .iter()
                    .any(|r| r["uri"] == h.url("calls.php") && r["range"]["start"]["character"] == call_column),
                "{result}"
            );
        }
    }
}

#[tokio::test]
async fn constructor_and_invoke_references_include_implicit_calls() {
    let declarations = "<?php\nclass Base { public function __construct(int $value) {} }\nclass Child extends Base {}\nclass Own extends Base { public function __construct() { parent::__construct(1); } }\nfinal class Handler { public function __invoke(int $value): int { return $value; } }\nfinal class Other { public function __invoke(): void {} }\n";
    let calls = "<?php\nfunction make(Handler $handler, ?Handler $maybe, Other $other): void {\n    new Base(1);\n    new Child(2);\n    new Own();\n    $handler(3);\n    $handler->__invoke(4);\n    $maybe(5);\n    $other();\n}\n";
    let mut h = Harness::start(&[("defs.php", declarations), ("calls.php", calls)]).await;
    let at = |source: &str, needle: &str| {
        let offset = source.find(needle).unwrap();
        let line = source[..offset].matches('\n').count();
        (line as u32, (offset - source[..offset].rfind('\n').map_or(0, |newline| newline + 1)) as u32)
    };
    for (file, source, needle, expected) in [
        (
            "defs.php",
            declarations,
            "__construct(int",
            vec![
                ("defs.php", at(declarations, "__construct(int")),
                ("defs.php", at(declarations, "__construct(1)")),
                ("calls.php", at(calls, "Base(1)")),
                ("calls.php", at(calls, "Child(2)")),
            ],
        ),
        (
            "defs.php",
            declarations,
            "__construct() {",
            vec![("defs.php", at(declarations, "__construct() {")), ("calls.php", at(calls, "Own()"))],
        ),
        (
            "calls.php",
            calls,
            "__invoke(4)",
            vec![
                ("defs.php", at(declarations, "__invoke(int")),
                ("calls.php", at(calls, "$handler(3)")),
                ("calls.php", at(calls, "__invoke(4)")),
                ("calls.php", at(calls, "$maybe(5)")),
            ],
        ),
    ] {
        let (line, character) = at(source, needle);
        let result = h
            .request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": h.url(file) }, "position": { "line": line, "character": character },
                    "context": { "includeDeclaration": true }
                }),
            )
            .await;
        let mut found: Vec<_> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|location| {
                let start = &location["range"]["start"];
                (
                    location["uri"].as_str().unwrap().to_owned(),
                    start["line"].as_u64().unwrap(),
                    start["character"].as_u64().unwrap(),
                )
            })
            .collect();
        found.sort_unstable();
        let mut expected: Vec<_> = expected
            .into_iter()
            .map(|(file, (line, character))| (h.url(file), u64::from(line), u64::from(character)))
            .collect();
        expected.sort_unstable();
        assert_eq!(found, expected, "{needle}: {result}");
    }
}
