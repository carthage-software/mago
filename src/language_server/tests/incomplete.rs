use serde_json::Value;

use super::harness::Harness;
use crate::language_server::position::position_at_offset;
use mago_database::file::File;

async fn at_end(h: &mut Harness, method: &str, source: &str) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, file.size);
    h.at(method, "a.php", position.line, position.character).await
}

#[tokio::test]
async fn unfinished_bodies_keep_local_scope_and_member_types() {
    let declarations = "<?php class Leaf { public int $value; public function work(int $number): void {} } ";
    for body in [
        "function test(Leaf $leaf) { $leaf->",
        "function test(Leaf $leaf) { if (true) { $leaf->",
        "class C { function test(Leaf $leaf) { $leaf->",
        "namespace { class C { function test(Leaf $leaf) { $leaf->",
    ] {
        let source = format!("{declarations}{body}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let completion = at_end(&mut h, "textDocument/completion", &source).await;
        assert!(
            completion["items"].as_array().unwrap().iter().any(|item| item["label"] == "work"),
            "{body}: {completion}"
        );
        let changed = source.strip_suffix("$leaf->").unwrap().to_owned() + "$";
        h.open("a.php", &source).await;
        h.change("a.php", &changed, 2).await;
        let completion = at_end(&mut h, "textDocument/completion", &changed).await;
        assert!(
            completion["items"].as_array().unwrap().iter().any(|item| item["label"] == "$leaf"),
            "{body}: {completion}"
        );
    }
}

#[tokio::test]
async fn unfinished_argument_lists_keep_the_callee_and_active_parameter() {
    let declarations = "<?php class Leaf { public function work(array $items, int $count): void {} } ";
    for expression in ["$leaf->work(", "$leaf->work([1, 2], "] {
        let source = format!("{declarations} function test(Leaf $leaf) {{ {expression}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let help = at_end(&mut h, "textDocument/signatureHelp", &source).await;
        assert!(help["signatures"][0]["label"].as_str().is_some_and(|label| label.contains("work")), "{help}");
        assert_eq!(help["activeParameter"], u32::from(expression.contains(',')), "{help}");
    }
}

#[tokio::test]
async fn unfinished_assignment_keeps_navigation_and_reports_a_parse_error() {
    let source = "<?php class Leaf {} function test() { $leaf = new Leaf()";
    let mut h = Harness::start(&[("a.php", source)]).await;
    h.open("a.php", source).await;
    let symbols = h.for_doc("textDocument/documentSymbol", "a.php").await;
    assert!(symbols.as_array().unwrap().iter().any(|symbol| symbol["name"] == "test"), "{symbols}");
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, source.rfind("Leaf").unwrap() as u32);
    let definition = h.at("textDocument/definition", "a.php", position.line, position.character).await;
    assert_eq!(definition["uri"], h.url("a.php"), "{definition}");
    let diagnostics = h.diagnostics().await;
    assert!(
        diagnostics.iter().any(|message| message["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "parse")),
        "{diagnostics:?}"
    );
}

#[tokio::test]
async fn completion_keeps_member_visibility_at_eof() {
    for expression in ["$this->", "self::"] {
        let declaration =
            if expression == "self::" { "private static function hidden() {}" } else { "private function hidden() {}" };
        let source = format!("<?php class C {{ {declaration} function work() {{ {expression}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let completion = at_end(&mut h, "textDocument/completion", &source).await;
        assert!(completion["items"].as_array().unwrap().iter().any(|item| item["label"] == "hidden"), "{completion}");
    }
}

#[tokio::test]
async fn unclosed_control_heads_keep_narrowing_in_their_existing_bodies() {
    let declarations = "<?php\n\nfinal class Widget\n{\n    public function render(): void {}\n}\n\nfinal class Gadget\n{\n    public function spin(): void {}\n}\n\nfunction run(mixed $value): void\n{\n    ";
    for (body, expected) in [
        (
            "if ($value instanceof Widget {\n        $value->render();\n    } elseif ($value instanceof Gadget) {\n        $value->spin();\n    } else {\n        echo 1;\n    }\n}\n",
            [("Widget", "render"), ("Gadget", "spin")].as_slice(),
        ),
        (
            "if ($value instanceof Widget) {\n        $value->render();\n    } elseif ($value instanceof Gadget {\n        $value->spin();\n    }\n}\n",
            [("Widget", "render"), ("Gadget", "spin")].as_slice(),
        ),
        ("while ($value instanceof Widget {\n        $value->render();\n    }\n}\n", [("Widget", "render")].as_slice()),
    ] {
        let source = format!("{declarations}{body}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        h.open("a.php", &source).await;
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
        for (class, method) in expected {
            let offset = source.find(&format!("$value->{method}")).unwrap() as u32;
            let position = position_at_offset(&file, offset + 1);
            let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
            assert_eq!(hover["contents"]["value"], format!("{class} $value"), "{body}: {hover}");
            let position = position_at_offset(&file, offset + "$value->".len() as u32);
            let completion = h.at("textDocument/completion", "a.php", position.line, position.character).await;
            let items = completion["items"].as_array().unwrap();
            assert!(items.iter().any(|item| item["label"] == *method), "{body}: {completion}");
        }
        let publications = h.diagnostics().await;
        let url = h.url("a.php");
        let publication = publications.iter().rev().find(|publication| publication["uri"] == url).unwrap();
        let diagnostics = publication["diagnostics"].as_array().unwrap();
        assert_eq!(
            diagnostics.iter().filter(|diagnostic| diagnostic["code"] == "parse").count(),
            1,
            "{body}: {diagnostics:?}"
        );
    }
}
