use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use super::references::apply;
use crate::language_server::ServerConfig;

async fn actions(h: &mut Harness, start: (u32, u32), end: (u32, u32), kind: &str) -> Value {
    h.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": h.url("a.php") },
            "range": {
                "start": { "line": start.0, "character": start.1 },
                "end": { "line": end.0, "character": end.1 }
            },
            "context": { "diagnostics": [], "only": [kind] }
        }),
    )
    .await
}

fn assert_no_actions(result: &Value) {
    assert!(result.is_null() || result.as_array().is_some_and(Vec::is_empty), "{result}");
}

const LINT_SOURCE: &str = "<?php function check(mixed $x): bool { return is_null($x); }";

async fn lint_fixture() -> (Harness, u32) {
    let mut config = ServerConfig::default();
    config.configuration.linter.rules.no_is_null.enabled = true;
    let mut h = Harness::start_with(&[("a.php", LINT_SOURCE)], config, client_capabilities()).await;
    let column = LINT_SOURCE.find("is_null").unwrap() as u32;
    let result = actions(&mut h, (0, column), (0, column), "quickfix").await;
    let action =
        result.as_array().unwrap().iter().find(|action| action["diagnostics"][0]["code"] == "no-is-null").unwrap();
    assert_eq!(
        apply(LINT_SOURCE, &action["edit"]["changes"][h.url("a.php")]),
        "<?php function check(mixed $x): bool { return null === $x; }"
    );
    (h, column)
}

#[tokio::test]
async fn code_actions_reject_reversed_ranges_inside_a_diagnostic() {
    let (mut h, column) = lint_fixture().await;
    let result = actions(&mut h, (0, column + 3), (0, column + 1), "quickfix").await;
    assert_no_actions(&result);
}

#[tokio::test]
async fn code_actions_reject_lines_outside_the_document() {
    let (mut h, column) = lint_fixture().await;
    let result = actions(&mut h, (0, column), (99, 0), "quickfix").await;
    assert_no_actions(&result);
}

#[tokio::test]
async fn code_actions_reject_positions_inside_a_utf16_surrogate_pair() {
    let source = "<?php new \\😀Thing();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, (0, 11), (0, 11), "refactor.rewrite").await;
    assert_eq!(result.as_array().unwrap().len(), 1, "{result}");
    assert_eq!(apply(source, &result[0]["edit"]["changes"][h.url("a.php")]), "<?php new 😀Thing();");
    for (start, end) in [((0, 12), (0, 12)), ((0, 10), (0, 12))] {
        assert_no_actions(&actions(&mut h, start, end, "refactor.rewrite").await);
    }
}

#[tokio::test]
async fn code_actions_keep_the_lsp_line_end_column_clamp() {
    let source = "<?php\nnew \\DateTime\n();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let exact = actions(&mut h, (1, 4), (1, 13), "refactor.rewrite").await;
    assert_eq!(exact.as_array().unwrap().len(), 1, "{exact}");
    assert_eq!(apply(source, &exact[0]["edit"]["changes"][h.url("a.php")]), "<?php\nnew DateTime\n();");
    // LSP Position defines columns past the text as the line's end.
    let clamped = actions(&mut h, (1, 4), (1, 999), "refactor.rewrite").await;
    assert_eq!(clamped, exact);
}
