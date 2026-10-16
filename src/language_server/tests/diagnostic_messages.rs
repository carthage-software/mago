//! Diagnostic messages keep the headline alone on the first line and put each detail on its own line.

use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::Event;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use serde_json::Value;
use serde_json::json;

use super::harness::{Harness, client_capabilities};
use crate::language_server::ServerConfig;

const A: &str = r"<?php

namespace App;

final class Types
{
    /** @deprecated */
    public const SIMPLE_ARRAY = 'simple_array';
}

interface CommandBusInterface
{
    public function dispatch(object $command): void;
}

/** @psalm-assert null $value */
function assert_null(mixed $value): void {}

/** @param non-empty-string $id */
function run(CommandBusInterface $bus, string $id): string
{
    $bus->dispatchh(new \stdClass());
    assert_null($id);
    return Types::SIMPLE_ARRAY;
}

$number = (integer) 1;
";

const B: &str = "<?php\n\nfinal readonly function Foo() {}\n";

#[tokio::test]
async fn messages_put_the_headline_first_and_label_each_detail() {
    for supports_links in [true, false] {
        let mut capabilities = client_capabilities();
        capabilities["textDocument"]["publishDiagnostics"]["codeDescriptionSupport"] = json!(supports_links);
        let mut config = ServerConfig::default();
        config.configuration.php_version = mago_php_version::PHPVersion::PHP85;
        let mut h = Harness::start_with(&[("a.php", A), ("b.php", B)], config, capabilities).await;
        let diagnostics: Vec<_> = h
            .diagnostics()
            .await
            .into_iter()
            .flat_map(|publication| publication["diagnostics"].as_array().unwrap().clone())
            .collect();
        let find = |code: &str| {
            diagnostics
                .iter()
                .find(|diagnostic| diagnostic["code"] == code)
                .unwrap_or_else(|| panic!("missing {code}: {diagnostics:#?}"))
        };
        let message = |code: &str| find(code)["message"].as_str().unwrap().to_owned();

        assert_eq!(
            message("deprecated-class-constant"),
            "Use of deprecated class constant `App\\Types::SIMPLE_ARRAY`.\n = This class constant is deprecated\n = Help: Check the documentation for `App\\Types::SIMPLE_ARRAY` for a replacement."
        );
        assert_eq!(
            message("non-existent-method"),
            "Method `dispatchh` does not exist on type `App\\CommandBusInterface`.\n = This method selection is invalid\n = Help: Ensure the `dispatchh` method is defined in the `App\\CommandBusInterface` class-like."
        );
        assert_eq!(
            find("non-existent-method")["relatedInformation"][0]["message"],
            "This expression has type `App\\CommandBusInterface`"
        );
        assert_eq!(
            message("impossible-type-comparison"),
            "Impossible type assertion: `$id` of type `non-empty-string` can never be `null`.\n = Argument `$id` has type `non-empty-string`\n = The assertion expects `$id` to be `null`, but no value of type `non-empty-string` can satisfy this.\n = Help: Check that the correct variable is being passed, or update the assertion type."
        );
        assert_eq!(
            message("parse"),
            "Expected one of `Class`, found `Function`\n = This error indicates that the parser encountered a parse issue.\n = Help: Check the syntax of your code."
        );
        let link = if supports_links { "" } else { "\n = See: https://wiki.php.net/rfc/deprecations_php_8_5" };
        assert_eq!(
            message("deprecated-cast"),
            format!(
                "Usage of deprecated cast `(integer)`.\n = Deprecated cast `(integer)` used here.\n = The `(integer)` cast has been deprecated since PHP 8.5.\n = Help: Replace `(integer)` with `(int)` to resolve this issue.{link}"
            )
        );

        for diagnostic in &diagnostics {
            let message = diagnostic["message"].as_str().unwrap();
            assert!(!message.contains("\n\n") && !message.contains('\r'), "{message:?}");
            let mut lines = message.lines();
            assert!(lines.next().is_some_and(|headline| !headline.starts_with(' ')), "{message:?}");
            for line in lines {
                assert!(line.starts_with(" = ") || line.starts_with("   "), "{message:?}");
            }
        }
    }
}

const LITERALS: &str = "<?php

/** @param 'a\\nb' $mode */
function mode(string $mode): void {}

mode(\"a
b\");

function sql(): int
{
    return <<<SQL
    SELECT id,
           name
    FROM users

    WHERE id = 1
    SQL;
}
";

#[tokio::test]
async fn multi_line_literal_types_keep_their_text_inside_code_spans() {
    let expected = "string('a\\nb')";
    let found = "string('a\nb')";
    let query = "string('SELECT id,\n       name\nFROM users\n\nWHERE id = 1')";
    let cases = [
        (
            "invalid-argument",
            format!(
                "Invalid argument type for argument #1 of `mode`: expected `{expected}`, but found `{found}`.\n = This has type `{found}`\n = The provided type `{found}` is not compatible with the expected type `{expected}`.\n = Help: Change the argument value to match `{expected}`, or update the parameter's type declaration."
            ),
            vec![
                format!("Invalid argument type for argument #1 of `mode`: expected `{expected}`, but found"),
                format!("```\n{found}\n"),
                ".".to_owned(),
                "= This has type".to_owned(),
                format!("```\n{found}\n"),
                "= The provided type".to_owned(),
                format!("```\n{found}\n"),
                format!(
                    "is not compatible with the expected type `{expected}`.\n= Help: Change the argument value to match `{expected}`, or update the parameter's type declaration."
                ),
            ],
        ),
        (
            "invalid-return-statement",
            format!(
                "Invalid return type for function `sql`: expected `int`, but found `{query}`.\n = This has type `{query}`\n = Help: Change the return value to match `int`, or update the function's return type declaration."
            ),
            vec![
                "Invalid return type for function `sql`: expected `int`, but found".to_owned(),
                format!("```\n{query}\n"),
                ".".to_owned(),
                "= This has type".to_owned(),
                format!("```\n{query}\n"),
                "= Help: Change the return value to match `int`, or update the function's return type declaration."
                    .to_owned(),
            ],
        ),
    ];
    for markdown in [false, true] {
        let diagnostics = published(LITERALS, markdown).await.1;
        for (code, plain, blocks) in &cases {
            let diagnostic = diagnostics
                .iter()
                .find(|diagnostic| diagnostic["code"] == *code)
                .unwrap_or_else(|| panic!("missing {code}: {diagnostics:#?}"));
            if markdown {
                assert_eq!(diagnostic["message"]["kind"], "markdown", "{diagnostic}");
                assert_eq!(markdown_blocks(diagnostic["message"]["value"].as_str().unwrap()), *blocks, "{diagnostic}");
            } else {
                assert_eq!(diagnostic["message"], *plain, "{diagnostic}");
            }
        }
    }
}

const INLINE_RETURN: &str = "<?php

function items(): array
{
    $items = [
        'first' => 1,
        'second' => 2,
    ];
    return $items;
}
";

#[tokio::test]
async fn multi_line_lint_help_keeps_the_suggested_code_pasteable() {
    for line_ending in ["\n", "\r\n"] {
        let source = INLINE_RETURN.replace('\n', line_ending);
        let snippet =
            format!("return [{line_ending}        'first' => 1,{line_ending}        'second' => 2,{line_ending}    ];");
        for markdown in [false, true] {
            let (mut h, diagnostics) = published(&source, markdown).await;
            let diagnostic = diagnostics
                .iter()
                .find(|diagnostic| diagnostic["code"] == "inline-variable-return")
                .unwrap_or_else(|| panic!("missing inline-variable-return: {diagnostics:#?}"));
            if markdown {
                assert_eq!(
                    markdown_blocks(diagnostic["message"]["value"].as_str().unwrap()),
                    [
                        "Variable assignment can be inlined into the return statement.".to_owned(),
                        "= This assignment is immediately returned\n= Assigning to a variable just to return it immediately is redundant.\n= Help: Return the expression directly:".to_owned(),
                        format!("```\n{}\n", snippet.replace("\r\n", "\n")),
                    ],
                    "{diagnostic}"
                );
            } else {
                assert_eq!(
                    diagnostic["message"],
                    format!(
                        "Variable assignment can be inlined into the return statement.\n = This assignment is immediately returned\n = Assigning to a variable just to return it immediately is redundant.\n = Help: Return the expression directly: `{snippet}`"
                    ),
                    "{diagnostic}"
                );
            }
            let actions = h
                .request(
                    "textDocument/codeAction",
                    json!({
                        "textDocument": {"uri": h.url("a.php")},
                        "range": diagnostic["range"],
                        "context": {"diagnostics": [diagnostic], "only": ["quickfix"]}
                    }),
                )
                .await;
            let action = actions
                .as_array()
                .unwrap()
                .iter()
                .find(|action| action["diagnostics"][0]["code"] == "inline-variable-return")
                .unwrap_or_else(|| panic!("missing quick fix: {actions}"));
            assert_eq!(action["diagnostics"], json!([diagnostic]), "{action}");
        }
    }
}

async fn published(source: &str, markdown: bool) -> (Harness, Vec<Value>) {
    let mut capabilities = client_capabilities();
    capabilities["textDocument"]["diagnostic"] = json!({"markupMessageSupport": markdown});
    let mut h = Harness::start_with(&[("a.php", source)], ServerConfig::default(), capabilities).await;
    let diagnostics = h
        .diagnostics()
        .await
        .into_iter()
        .flat_map(|publication| publication["diagnostics"].as_array().unwrap().clone())
        .collect();
    (h, diagnostics)
}

fn markdown_blocks(markdown: &str) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Paragraph) => blocks.push(String::new()),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) if info.is_empty() => {
                blocks.push("```\n".to_owned());
            }
            Event::Text(text) => blocks.last_mut().unwrap().push_str(&text.replace("\r\n", "\n")),
            Event::Code(code) => {
                let block = blocks.last_mut().unwrap();
                block.push('`');
                block.push_str(&code);
                block.push('`');
            }
            Event::HardBreak => blocks.last_mut().unwrap().push('\n'),
            Event::Start(Tag::Link { .. }) | Event::End(_) => {}
            event => panic!("unexpected {event:?} in {markdown:?}"),
        }
    }

    blocks
}
