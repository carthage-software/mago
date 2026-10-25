use serde_json::json;
use tower_lsp_server::gen_lsp_types::DocumentSymbol;
use tower_lsp_server::gen_lsp_types::Range;
use tower_lsp_server::gen_lsp_types::SymbolInformation;
use tower_lsp_server::gen_lsp_types::SymbolKind;

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::ServerConfig;
use crate::language_server::position::offset_at_position;

fn children(symbol: &DocumentSymbol) -> &[DocumentSymbol] {
    symbol.children.as_deref().unwrap_or_default()
}

fn named<'a>(symbols: &'a [DocumentSymbol], name: &str) -> &'a DocumentSymbol {
    symbols.iter().find(|symbol| symbol.name == name).unwrap()
}

fn names(symbols: &[DocumentSymbol]) -> Vec<&str> {
    symbols.iter().map(|symbol| symbol.name.as_str()).collect()
}

fn assert_bounds(file: &File, symbols: &[DocumentSymbol], parent: Option<Range>) {
    for symbol in symbols {
        assert!(symbol.range.start <= symbol.selection_range.start);
        assert!(symbol.selection_range.end <= symbol.range.end);
        if let Some(parent) = parent {
            assert!(parent.start <= symbol.range.start && symbol.range.end <= parent.end);
        }
        if !symbol.name.starts_with('(') {
            let start = offset_at_position(file, symbol.selection_range.start) as usize;
            let end = offset_at_position(file, symbol.selection_range.end) as usize;
            assert_eq!(&file.contents[start..end], symbol.name.as_bytes());
        }
        assert_bounds(file, children(symbol), Some(symbol.range));
    }
}

#[tokio::test]
async fn document_symbols_keep_declarations_and_name_spans() {
    let source = r#"<?php
namespace App {
interface I {}
trait T {}
enum State { case Ready; }
class Box {
    public $a, $b;
    public string $name { get => "value"; }
    public const A = 1, B = 2;
    public function __construct(public int $id) {}
    public function work() { function inner() {} }
}
if (true) { function helper() {} }
const C = 1, D = 2;
}
namespace { function global_fn() {} }
"#;
    let config = ServerConfig { analyzer: false, linter: false, ..ServerConfig::default() };
    let caps = json!({ "textDocument": { "documentSymbol": { "hierarchicalDocumentSymbolSupport": true } } });
    let mut h = Harness::start_with(&[("a.php", source)], config, caps).await;
    let symbols: Vec<DocumentSymbol> =
        serde_json::from_value(h.for_doc("textDocument/documentSymbol", "a.php").await).unwrap();
    assert_eq!(names(&symbols), ["App", "(global)"]);
    assert_eq!(symbols[0].kind, SymbolKind::Namespace);
    let app = children(&symbols[0]);
    assert_eq!(names(app), ["I", "T", "State", "Box", "helper", "C", "D"]);
    assert_eq!(named(app, "I").kind, SymbolKind::Interface);
    assert_eq!(named(app, "T").kind, SymbolKind::Class);
    assert_eq!(named(app, "State").kind, SymbolKind::Enum);
    assert_eq!(children(named(app, "State"))[0].kind, SymbolKind::EnumMember);
    let members = children(named(app, "Box"));
    assert_eq!(names(members), ["$a", "$b", "$name", "A", "B", "__construct", "$id", "work"]);
    assert_eq!(named(members, "$name").kind, SymbolKind::Property);
    assert_eq!(named(members, "$id").kind, SymbolKind::Property);
    assert_eq!(named(members, "A").kind, SymbolKind::Constant);
    assert_eq!(named(members, "__construct").kind, SymbolKind::Constructor);
    assert_eq!(named(members, "work").kind, SymbolKind::Method);
    assert_eq!(names(children(named(members, "work"))), ["inner"]);
    assert_eq!(named(app, "helper").kind, SymbolKind::Function);
    assert_eq!(names(children(&symbols[1])), ["global_fn"]);
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    assert_bounds(&file, &symbols, None);
}

#[tokio::test]
async fn document_symbols_follow_live_text_and_utf16_positions() {
    let caps = json!({ "textDocument": { "documentSymbol": { "hierarchicalDocumentSymbolSupport": true } } });
    let mut h = Harness::start_with(&[("a.php", "<?php class Old {}")], ServerConfig::default(), caps).await;
    let source = "<?php /*😀*/ class Café { public function naïve() {} }";
    h.open("a.php", source).await;
    let symbols: Vec<DocumentSymbol> =
        serde_json::from_value(h.for_doc("textDocument/documentSymbol", "a.php").await).unwrap();
    assert_eq!(names(&symbols), ["Café"]);
    let start = source[..source.find("Café").unwrap()].encode_utf16().count() as u32;
    assert_eq!(symbols[0].selection_range.start.character, start);
    assert_eq!(symbols[0].selection_range.end.character, start + 4);
    assert_eq!(names(children(&symbols[0])), ["naïve"]);
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    assert_bounds(&file, &symbols, None);
    h.change("a.php", "<?php class Changed {} $unfinished =", 2).await;
    let symbols: Vec<DocumentSymbol> =
        serde_json::from_value(h.for_doc("textDocument/documentSymbol", "a.php").await).unwrap();
    assert_eq!(names(&symbols), ["Changed"]);
}

#[tokio::test]
async fn document_symbols_fall_back_to_flat_results() {
    let mut h = Harness::start(&[("a.php", "<?php namespace App; class Box { function work() {} }")]).await;
    let symbols: Vec<SymbolInformation> =
        serde_json::from_value(h.for_doc("textDocument/documentSymbol", "a.php").await).unwrap();
    assert_eq!(
        symbols.iter().map(|symbol| symbol.base_symbol_information.name.as_str()).collect::<Vec<_>>(),
        ["App", "Box", "work"]
    );
    assert_eq!(symbols[0].base_symbol_information.container_name, None);
    assert_eq!(symbols[1].base_symbol_information.container_name.as_deref(), Some("App"));
    assert_eq!(symbols[2].base_symbol_information.container_name.as_deref(), Some("Box"));
    assert!(symbols.iter().all(|symbol| symbol.location.uri.as_str() == h.url("a.php")));
}

#[tokio::test]
async fn document_symbols_keep_anonymous_members_in_their_class() {
    let caps = json!({ "textDocument": { "documentSymbol": { "hierarchicalDocumentSymbolSupport": true } } });
    let source = "<?php $object = new class { public function work() {} };";
    let mut h = Harness::start_with(&[("a.php", source)], ServerConfig::default(), caps).await;
    let symbols: Vec<DocumentSymbol> =
        serde_json::from_value(h.for_doc("textDocument/documentSymbol", "a.php").await).unwrap();
    assert_eq!(names(&symbols), ["(anonymous class)"]);
    assert_eq!(names(children(&symbols[0])), ["work"]);
    h.open("a.php", "<?php // no declarations").await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "a.php").await, json!([]));
}
