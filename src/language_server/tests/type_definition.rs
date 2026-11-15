use serde_json::Value;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::Location;

use mago_database::file::File;

use super::harness::Harness;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};

async fn at(h: &mut Harness, source: &str, needle: &str) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, source.find(needle).unwrap() as u32);
    h.at("textDocument/typeDefinition", "a.php", position.line, position.character).await
}

fn names(source: &str, result: Value) -> Vec<String> {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let items: Vec<Location> = serde_json::from_value(result).unwrap();
    items
        .iter()
        .map(|item| {
            source[offset_at_position(&file, item.range.start) as usize
                ..offset_at_position(&file, item.range.end) as usize]
                .to_owned()
        })
        .collect()
}

#[tokio::test]
async fn type_definitions_use_inference_property_metadata_unions_and_intersections() {
    let source = "<?php\nclass A {}\nclass B {}\nclass Box { public A|B $value; public int $count; public function make(): B { return new B(); } }\ninterface I {} interface J {}\nfunction both(I&J $pair) { return $pair; }\n$box = new Box(); echo $box; echo $box->value; $box->make();\n$child = $box->make(); echo $child;\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, expected) in [
        ("$value;", vec!["A", "B"]),
        ("value; $box", vec!["A", "B"]),
        ("$box;", vec!["Box"]),
        ("make();", vec!["B"]),
        ("$child;", vec!["B"]),
        ("$pair;", vec!["I", "J"]),
        ("Box {", vec!["Box"]),
        ("A|B", vec!["A"]),
    ] {
        assert_eq!(names(source, at(&mut h, source, needle).await), expected, "{needle}");
    }
}

#[tokio::test]
async fn scalar_arguments_and_members_never_use_the_enclosing_call_type() {
    let source = "<?php class Box { public int $count = 1; } function factory(int $n): Box { return new Box(); } $box = new Box(); $number = 1; factory($number); factory($box->count); factory(7); factory($unknown); factory(/* comment */ 1);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(names(source, at(&mut h, source, "factory($number)").await), ["Box"]);
    for needle in ["$number);", "count);", "7);", "$unknown);", "comment", "<?php"] {
        assert!(at(&mut h, source, needle).await.is_null(), "{needle}");
    }
}

#[tokio::test]
async fn type_definition_uses_utf16_and_obeys_switches() {
    let source = "<?php /*😀*/ class Café {} $value = new Café(); echo $value;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = at(&mut h, source, "$value;").await;
    assert_eq!(result[0]["range"]["start"]["character"], source[..source.find("Café").unwrap()].encode_utf16().count());
    assert_eq!(names(source, result), ["Café"]);
    for analyzer in [true, false] {
        let mut config = ServerConfig { analyzer, ..ServerConfig::default() };
        if analyzer {
            config.configuration.language_server.disable = vec![Provider::TypeDefinition];
        }
        let mut h = Harness::start_with(&[("a.php", source)], config, json!({})).await;
        assert!(at(&mut h, source, "$value;").await.is_null());
    }
}
