use serde_json::{Value, json};

use mago_database::file::File;

use crate::language_server::ServerConfig;
use crate::language_server::position::position_at_offset;

use super::harness::Harness;

#[tokio::test]
async fn class_completion_namespaces_distinguish_rows_without_changing_edits_or_docs() {
    let declarations = r"<?php
namespace First { /** First clock. */ class DateTime {} }
namespace Second { /** Second clock. */ class DateTime {} }
";
    let source = r"<?php
namespace App;
use First\DateTime as Clock;
new DateTi;
new Clo;
new \First\DateTi;
strlen('x');
";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let mut plain_rows = Vec::new();
    for supported in [false, true] {
        let capabilities = if supported {
            json!({"textDocument":{"completion":{"completionItem":{"labelDetailsSupport":true}}}})
        } else {
            json!({})
        };
        let mut h = Harness::start_with(
            &[("a.php", source), ("classes.php", declarations)],
            ServerConfig::default(),
            capabilities,
        )
        .await;
        let mut rows = Vec::new();
        for (prefix, label, expected) in [
            (
                "new DateTi",
                "DateTime",
                &[
                    (r"\DateTime", r"\", "DateTime"),
                    (r"\First\DateTime", "First", "First clock."),
                    (r"\Second\DateTime", "Second", "Second clock."),
                ][..],
            ),
            ("new Clo", "Clock", &[("Clock", "First", "First clock.")][..]),
            (r"new \First\DateTi", "DateTime", &[("DateTime", "First", "First clock.")][..]),
        ] {
            let position = position_at_offset(&file, (source.find(prefix).unwrap() + prefix.len()) as u32);
            let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
            let items: Vec<_> =
                result["items"].as_array().unwrap().iter().filter(|item| item["label"] == label).collect();
            assert_eq!(items.len(), expected.len(), "{prefix}: {result}");
            for (insert, namespace, documentation) in expected {
                let item = items.iter().find(|item| item["textEdit"]["newText"] == *insert).unwrap();
                assert_eq!(item["filterText"], label, "{item}");
                assert_eq!(item["documentation"]["value"], *documentation, "{item}");
                if supported {
                    assert_eq!(item["labelDetails"], json!({"description":namespace}), "{item}");
                    assert!(item["detail"].is_null(), "{item}");
                } else {
                    assert_eq!(item["detail"], *namespace, "{item}");
                    assert!(item["labelDetails"].is_null(), "{item}");
                }
            }
            for item in items {
                let mut item = item.clone();
                item.as_object_mut().unwrap().remove("labelDetails");
                item.as_object_mut().unwrap().remove("detail");
                rows.push(item);
            }
        }
        if supported {
            assert_eq!(rows, plain_rows, "label support must preserve edits, ranking, and docs");
        } else {
            plain_rows = rows;
        }
        let position = position_at_offset(&file, (source.find("strlen").unwrap() + 4) as u32);
        let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
        let function: &Value =
            result["items"].as_array().unwrap().iter().find(|item| item["label"] == "strlen").unwrap();
        assert!(function["labelDetails"].is_null(), "{function}");
        assert!(function["detail"].as_str().unwrap().starts_with("strlen("), "{function}");
    }
}
