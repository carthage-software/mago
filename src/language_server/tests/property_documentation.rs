use mago_database::file::File;
use serde_json::json;

use super::harness::Harness;
use crate::language_server::ServerConfig;
use crate::language_server::position::position_at_offset;

#[tokio::test]
async fn promoted_property_docs_follow_the_declaring_constructor_and_prefer_inline_docs() {
    let declarations = r"<?php
namespace Lib;
class Base {
    /**
     * @param string $name Original **name**.
     * @param string $inline Constructor docs lose to inline docs.
     */
    public function __construct(public string $name, /** Inline **docs**. */ public string $inline) {}
}
class Child extends Base {
    /** @param string $name Unrelated child constructor docs. */
    public function __construct(string $name) { parent::__construct($name, ''); }
}
class Other {
    public string $name;
    /** @param string $name Ordinary parameters do not document properties. */
    public function __construct(string $name) { $this->name = $name; }
}
trait Named {
    /** @param string $name Trait **name**. */
    public function __construct(public string $name) {}
}
class UsesTrait { use Named; }
";
    let source = r"<?php
function show(\Lib\Child $child, \Lib\Other $other, \Lib\UsesTrait $trait): void {
    echo $child->name, $child->inline, $other->name, $trait->name;
}
";
    let mut h = Harness::start_with(
        &[("a.php", source), ("declarations.php", declarations)],
        ServerConfig::default(),
        json!({"textDocument":{
            "hover":{"contentFormat":["markdown"]},
            "completion":{"completionItem":{"documentationFormat":["markdown"]}}
        }}),
    )
    .await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    for (receiver, name, expected) in [
        ("$child->", "name", Some("Original **name**.")),
        ("$child->", "inline", Some("Inline **docs**.")),
        ("$other->", "name", None),
        ("$trait->", "name", Some("Trait **name**.")),
    ] {
        let offset = source.find(&format!("{receiver}{name}")).unwrap() + receiver.len() + 2;
        let position = position_at_offset(&file, offset as u32);
        let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
        let item = result["items"].as_array().unwrap().iter().find(|item| item["label"] == name).unwrap();
        assert_eq!(item["documentation"]["value"].as_str(), expected, "{item}");
        let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
        let text = hover["contents"]["value"].as_str().unwrap();
        if let Some(expected) = expected {
            assert!(text.ends_with(expected), "{hover}");
        } else {
            assert!(!text.contains("---"), "{hover}");
        }
    }
}
