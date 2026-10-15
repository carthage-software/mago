use tower_lsp_server::gen_lsp_types::{DocumentLink, Location};

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::position::offset_at_position;
use crate::language_server::uri::UriFilePath;

#[tokio::test]
async fn document_links_resolve_grouped_imports_and_embedded_sources() {
    let library = "<?php\nnamespace Toolkit;\n/** Target docs. */\n/* 😀 */ final class Target {}\nfunction work(): void {}\nconst FIRST = 0, VALUE = 1;";
    let source = "<?php namespace App; use Toolkit\\{Target as Local, function work, const VALUE}; use DateTime; use function strlen; use const PHP_VERSION;";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let mut h = Harness::start(&[("a.php", source), ("lib.php", library)]).await;
    let links: Vec<DocumentLink> =
        serde_json::from_value(h.for_doc("textDocument/documentLink", "a.php").await).unwrap();
    let selected: Vec<_> = links
        .iter()
        .map(|link| {
            let start = offset_at_position(&file, link.range.start) as usize;
            let end = offset_at_position(&file, link.range.end) as usize;
            &source[start..end]
        })
        .collect();
    assert_eq!(selected, ["Target", "work", "VALUE", "DateTime", "strlen", "PHP_VERSION"]);
    for (link, fragment) in links.iter().zip(["4,22", "5,10", "6,18"]) {
        assert_eq!(link.target.as_ref().unwrap().as_str(), format!("{}#{fragment}", h.url("lib.php")));
    }
    for (link, name) in links.iter().zip(selected) {
        let target = link.target.as_ref().unwrap();
        let definition: Location = serde_json::from_value(
            h.at("textDocument/definition", "a.php", link.range.start.line, link.range.start.character).await,
        )
        .unwrap();
        assert_eq!(target.strip_fragment().as_str(), definition.uri.as_str());
        assert_eq!(
            target.fragment().unwrap().as_str(),
            format!("{},{}", definition.range.start.line + 1, definition.range.start.character + 1),
        );
        let contents = std::fs::read(target.to_file_path().unwrap()).unwrap();
        let file = File::ephemeral(b"target.php".as_slice().into(), contents.into());
        let offset = offset_at_position(&file, definition.range.start) as usize;
        assert_eq!(&file.contents[offset..offset + name.len()], name.as_bytes());
    }
}

#[tokio::test]
async fn document_links_do_not_treat_trait_uses_as_global_imports() {
    let source = "<?php namespace { trait Helper {} } namespace App { trait Helper {} class Service { use Helper; } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let links = h.for_doc("textDocument/documentLink", "a.php").await;
    assert_eq!(links, serde_json::json!([]));
}
