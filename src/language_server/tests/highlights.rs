use serde_json::Value;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::DocumentHighlight;

use mago_database::file::File;

use super::harness::Harness;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};

async fn at(h: &mut Harness, source: &str, needle: &str) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, source.find(needle).unwrap() as u32);
    h.at("textDocument/documentHighlight", "a.php", position.line, position.character).await
}

#[tokio::test]
async fn highlights_keep_lexical_scope_and_work_without_analyzer() {
    let source = "<?php\nfunction one($value) { $copy = fn() => $value; $own = fn($value) => $value; return $value; }\nfunction two($value) { return $value; }\n// $value\n";
    let config = ServerConfig { analyzer: false, ..ServerConfig::default() };
    let mut h =
        Harness::start_with(&[("a.php", source), ("b.php", "<?php $value = 1; echo $value;")], config, json!({})).await;
    for needle in ["$value) { $copy", "$value; $own", "$value; }\nfunction"] {
        let result = at(&mut h, source, needle).await;
        assert_eq!(result.as_array().unwrap().len(), 3, "{result}");
        assert!(result.as_array().unwrap().iter().all(|item| item["range"]["start"]["line"] == 1));
    }
    let inner = at(&mut h, source, "$value) =>").await;
    assert_eq!(inner.as_array().unwrap().len(), 2);
    assert!(at(&mut h, source, "$value\n").await.is_null());
}

#[tokio::test]
async fn member_highlights_include_declaration_and_known_read_write_kinds() {
    let source = "<?php\nclass Box { public int $value; function run() { $this->value = 1; return $this->value; } }\nclass Other { public int $value; function run() { return $this->value; } }\n";
    let mut h =
        Harness::start(&[("a.php", source), ("b.php", "<?php function useBox(Box $b) { return $b->value; }")]).await;
    for needle in ["$value; function run() { $this->value =", "value =", "value; } }\nclass"] {
        let result = at(&mut h, source, needle).await;
        assert_eq!(result.as_array().unwrap().len(), 3, "{result}");
        assert_eq!(
            result.as_array().unwrap().iter().map(|item| item["kind"].as_u64().unwrap()).collect::<Vec<_>>(),
            [1, 3, 2]
        );
        assert!(result.as_array().unwrap().iter().all(|item| item["range"]["start"]["line"] == 1));
    }
}

#[tokio::test]
async fn aliases_keep_their_namespace_binding_and_exact_utf16_prefix() {
    let source = "<?php\nnamespace Lib { class Thing {} }\nnamespace One { /*😀*/ use Lib\\Thing as Item; new Item(); new Item(); use Lib as L; new L\\Thing(); }\nnamespace Two { use Lib\\Thing as Item; new Item(); }\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    for needle in ["Item; new", "Item(); new"] {
        let result = at(&mut h, source, needle).await;
        let highlights: Vec<DocumentHighlight> = serde_json::from_value(result).unwrap();
        assert_eq!(highlights.len(), 3);
        for item in highlights {
            assert_eq!(item.range.start.line, 2);
            assert_eq!(
                &source[offset_at_position(&file, item.range.start) as usize
                    ..offset_at_position(&file, item.range.end) as usize],
                "Item"
            );
        }
    }
    let prefix = at(&mut h, source, "L\\Thing()").await;
    assert_eq!(prefix.as_array().unwrap().len(), 2);
    assert!(prefix.as_array().unwrap().iter().all(|item| item["range"]["end"]["character"].as_u64().unwrap()
        - item["range"]["start"]["character"].as_u64().unwrap()
        == 1));
    let mut config = ServerConfig::default();
    config.configuration.language_server.disable = vec![Provider::DocumentHighlight];
    let mut h = Harness::start_with(&[("a.php", source)], config, json!({})).await;
    assert!(at(&mut h, source, "Item; new").await.is_null());
}
