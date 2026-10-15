//! A caret at the end of a name still selects that name for symbol queries.

use mago_database::file::File;
use serde_json::{Value, json};

use super::harness::Harness;
use crate::language_server::position::position_at_offset;

const SOURCE: &str = r"<?php
namespace App;
use App\Factory as Alias;
interface Contract { public function create(): Item; }
class Item {}
class Factory implements Contract { public function create(): Item { return new Item(); } }
function make(): Item { return new Item(); }
$local = new Factory();
$local->create();
make();
$alias = new Alias();
";

async fn query(h: &mut Harness, method: &str, source: &str, offset: usize) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let mut params = json!({ "textDocument": { "uri": h.url("a.php") }, "position": position });
    if method == "references" {
        params["context"] = json!({ "includeDeclaration": true });
    }
    if method == "rename" {
        params["newName"] = json!("Renamed");
    }
    h.request(&format!("textDocument/{method}"), params).await
}

fn has_result(value: &Value) -> bool {
    !value.is_null() && !value.as_array().is_some_and(Vec::is_empty)
}

#[tokio::test]
async fn symbol_queries_accept_the_right_edge_of_names() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    for (text, skip, name) in [
        ("class Factory", 6, "Factory"),
        ("new Factory", 4, "Factory"),
        ("function make", 9, "make"),
        ("make();", 0, "make"),
        ("$local->create", 8, "create"),
        ("$local->create", 0, "$local"),
        ("new Alias", 4, "Alias"),
    ] {
        let start = SOURCE.find(text).unwrap() + skip;
        for method in ["hover", "definition", "references", "documentHighlight", "prepareRename", "rename"] {
            let inside = query(&mut h, method, SOURCE, start + 1).await;
            assert!(has_result(&inside), "{method} at {text}: {inside}");
            let edge = query(&mut h, method, SOURCE, start + name.len()).await;
            assert_eq!(edge, inside, "{method} at the end of {text}");
        }
    }
}

#[tokio::test]
async fn hierarchy_and_type_queries_accept_the_right_edge_of_names() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    for (method, text, skip, name) in [
        ("prepareTypeHierarchy", "class Factory", 6, "Factory"),
        ("prepareCallHierarchy", "make();", 0, "make"),
        ("implementation", "interface Contract", 10, "Contract"),
        ("typeDefinition", "$local->create", 0, "$local"),
    ] {
        let start = SOURCE.find(text).unwrap() + skip;
        let inside = query(&mut h, method, SOURCE, start + 1).await;
        assert!(has_result(&inside), "{method}: {inside}");
        assert_eq!(query(&mut h, method, SOURCE, start + name.len()).await, inside, "{method}");
    }
}

#[tokio::test]
async fn symbol_queries_do_not_cross_whitespace_or_comments() {
    let source = "<?php\nfunction make(): int { return 1; }\nmake  (); // make\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for offset in [source.find("make  ").unwrap() + 5, source.find("// make").unwrap() + 7] {
        for method in ["hover", "definition", "prepareRename"] {
            assert!(query(&mut h, method, source, offset).await.is_null(), "{method} at {offset}");
        }
    }
}

#[tokio::test]
async fn symbol_queries_accept_a_variable_at_end_of_file() {
    let source = "<?php $value = 1; $value";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for method in ["hover", "definition", "references", "prepareRename"] {
        let inside = query(&mut h, method, source, source.len() - 2).await;
        assert!(has_result(&inside), "{method}: {inside}");
        assert_eq!(query(&mut h, method, source, source.len()).await, inside, "{method}");
    }
}

#[tokio::test]
async fn alias_queries_accept_the_right_edge_before_a_namespace_separator() {
    let source = "<?php namespace Lib { class Item {} } namespace App { use Lib as Alias; new Alias\\Item(); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let start = source.find("new Alias").unwrap() + 4;
    for method in ["references", "documentHighlight", "prepareRename", "rename"] {
        let inside = query(&mut h, method, source, start + 1).await;
        assert!(has_result(&inside), "{method}: {inside}");
        assert_eq!(query(&mut h, method, source, start + 5).await, inside, "{method}");
        if method == "rename" {
            assert_eq!(
                super::references::apply(source, &inside["changes"][h.url("a.php")]),
                source.replace("Alias", "Renamed")
            );
        }
    }
    let suffix = query(&mut h, "prepareRename", source, start + 6).await;
    assert_eq!(suffix["placeholder"], "Item");
}
