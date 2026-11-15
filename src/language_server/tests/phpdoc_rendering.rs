use serde_json::json;

use super::harness::Harness;
use crate::language_server::ServerConfig;

async fn hover(h: &mut Harness, source: &str) -> String {
    let offset = source.find("sample(").unwrap();
    let prefix = &source[..offset];
    let result = h
        .at(
            "textDocument/hover",
            "a.php",
            prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
            prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
        )
        .await;
    result["contents"]["value"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn phpdoc_hover_labels_known_tags_without_losing_descriptions() {
    let source = r"<?php
/** Greets **people**.
 *
 * Keep the [guide](https://example.test/guide).
 * @param string $name First line.
 *   Second line with `inline code`.
 * @return string The greeting.
 * @throws \RuntimeException When unavailable.
 * @deprecated Use `next()`.
 */
function sample(string $name): string { return $name; }
";
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{"hover":{"contentFormat":["markdown"]}}}),
    )
    .await;
    let value = hover(&mut h, source).await;
    for expected in [
        "Greets **people**.\n\nKeep the [guide](https://example.test/guide).",
        "**Parameter** `$name` `string` First line.\n  Second line with `inline code`.",
        "**Returns** `string` The greeting.",
        "**Throws** `\\RuntimeException` When unavailable.",
        "**Deprecated** Use `next()`.",
    ] {
        assert!(value.contains(expected), "missing {expected:?}: {value}");
    }
    assert!(!value.contains("@param"), "{value}");
    assert!(!value.contains("@return"), "{value}");
}

#[tokio::test]
async fn phpdoc_hover_preserves_code_unknown_and_invalid_tags() {
    let source = r"<?php
/** Example `@return bool`.
 *
 * ````php
 * ```
 * @param string $example
 * ````
 *
 * ~~~php
 * @throws Example
 * ~~~
 *
 * @custom Keep **this**.
 * @param | broken
 * @phpstan-return non-empty-string
 * @return string The result.
 */
function sample(): string { return 'value'; }
";
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{"hover":{"contentFormat":["markdown"]}}}),
    )
    .await;
    let value = hover(&mut h, source).await;
    for expected in [
        "Example `@return bool`.",
        "````php\n```\n@param string $example\n````",
        "~~~php\n@throws Example\n~~~",
        "@custom Keep **this**.",
        "@param | broken",
        "@phpstan-return non-empty-string",
        "**Returns** `string` The result.",
    ] {
        assert!(value.contains(expected), "missing {expected:?}: {value}");
    }
    assert!(!value.contains("**Parameter**"), "{value}");
    assert!(!value.contains("**Throws**"), "{value}");
}

#[tokio::test]
async fn phpdoc_signature_help_uses_its_own_documentation_preference() {
    let source = "<?php\n/** Greets a person.\n * @param string $name Their name.\n * @return string The greeting.\n */\nfunction sample(string $name): string { return $name; }\nsample('x');\n";
    for (formats, kind, label) in [
        (json!(["markdown", "plaintext"]), "markdown", "**Parameter** `$name` `string`"),
        (json!(["plaintext", "markdown"]), "plaintext", "Parameter $name string"),
        (json!([]), "plaintext", "Parameter $name string"),
    ] {
        let mut h = Harness::start_with(
            &[("a.php", source)],
            ServerConfig::default(),
            json!({"textDocument":{
                "hover":{"contentFormat":["markdown"]},
                "signatureHelp":{"signatureInformation":{"documentationFormat":formats}}
            }}),
        )
        .await;
        let result = h.at("textDocument/signatureHelp", "a.php", 6, 7).await;
        let documentation = &result["signatures"][0]["documentation"];
        assert_eq!(documentation["kind"], kind, "{result}");
        assert!(documentation["value"].as_str().unwrap().contains(label), "{result}");
        assert!(documentation["value"].as_str().unwrap().contains("Their name."), "{result}");
        if kind == "markdown" {
            let hover = hover(&mut h, source).await;
            assert_eq!(hover.split_once("---\n\n").unwrap().1, documentation["value"].as_str().unwrap());
        }
    }
}
