//! Trait aliases keep their callable name and visibility in editor output.

use mago_database::file::File;
use tower_lsp_server::gen_lsp_types::TextEdit;

use super::harness::Harness;
use crate::language_server::position::{offset_at_position, position_at_offset};

const SOURCE: &str = r"<?php
trait RawSender {
    /** Sends a message. */
    protected function send(string $message, int $attempt = 1): void {}
}
class Sender {
    use RawSender { send as public deliver; }
}
$sender = new Sender();
$sender->deliver('hello', 3);
";

#[tokio::test]
async fn trait_method_alias_completion_keeps_name_and_visibility() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), SOURCE.as_bytes().to_vec().into());
    let selector = SOURCE.rfind("deliver(").unwrap();

    // Both an empty selector and a prefix must offer the alias, not the protected original.
    for offset in [selector, selector + 3] {
        let position = position_at_offset(&file, offset as u32);
        let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
        let items = result["items"].as_array().unwrap();
        assert!(!items.iter().any(|item| item["label"] == "send"), "{items:?}");
        let alias = items
            .iter()
            .find(|item| item["label"] == "deliver")
            .unwrap_or_else(|| panic!("missing public alias: {items:?}"));
        assert_eq!(alias["detail"], "deliver(string $message, int $attempt = 1): void");
        if offset == selector {
            continue;
        }
        assert_ne!(alias["insertTextFormat"], 2);
        let edit: TextEdit = serde_json::from_value(alias["textEdit"].clone()).unwrap();
        assert_eq!(edit.new_text, "deliver");
        let mut applied = SOURCE.to_owned();
        applied.replace_range(
            offset_at_position(&file, edit.range.start) as usize..offset_at_position(&file, edit.range.end) as usize,
            &edit.new_text,
        );
        assert_eq!(applied, SOURCE);
    }
}

#[tokio::test]
async fn trait_method_alias_hover_keeps_name_visibility_and_docs() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), SOURCE.as_bytes().to_vec().into());
    let selector = SOURCE.rfind("deliver(").unwrap();
    let position = position_at_offset(&file, (selector + 1) as u32);
    let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
    assert_eq!(
        hover["contents"]["value"],
        "Sender\n\npublic function deliver(string $message, int $attempt = 1): void\n\nSends a message."
    );
}

#[tokio::test]
async fn trait_method_alias_signature_keeps_name_and_parameter_labels() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), SOURCE.as_bytes().to_vec().into());
    let position = position_at_offset(&file, (SOURCE.rfind(", 3").unwrap() + 2) as u32);
    let result = h.at("textDocument/signatureHelp", "a.php", position.line, position.character).await;
    let signature = &result["signatures"][0];
    assert_eq!(signature["label"], "function Sender::deliver(string $message, int $attempt = 1): void");
    assert_eq!(result["activeParameter"], 1);
    assert_eq!(signature["activeParameter"], 1);
    let label: Vec<_> = signature["label"].as_str().unwrap().encode_utf16().collect();
    let parameters = signature["parameters"].as_array().unwrap();
    assert_eq!(parameters.len(), 2);
    for (parameter, expected) in parameters.iter().zip(["string $message", "int $attempt = 1"]) {
        let start = parameter["label"][0].as_u64().unwrap() as usize;
        let end = parameter["label"][1].as_u64().unwrap() as usize;
        assert_eq!(String::from_utf16(&label[start..end]).unwrap(), expected);
    }
}

async fn check_mixed_case_alias(classes: &str, receiver: &str) {
    let source = format!(
        "<?php trait RawSender {{ /** Sends a message. */ \
         protected function send(string $message, int $attempt = 1): void {{}} }} \
         {classes} $sender = new {receiver}(); $sender->deliver('hello', 3);"
    );
    let mut h = Harness::start(&[("a.php", &source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let selector = source.rfind("deliver(").unwrap();
    let position = position_at_offset(&file, (selector + 3) as u32);
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    let items = result["items"].as_array().unwrap();
    assert!(!items.iter().any(|item| item["label"] == "send"), "{items:?}");
    let alias = items.iter().find(|item| item["label"] == "DeLiVeR").unwrap_or_else(|| panic!("{items:?}"));
    assert_eq!(alias["detail"], "DeLiVeR(string $message, int $attempt = 1): void");
    let edit: TextEdit = serde_json::from_value(alias["textEdit"].clone()).unwrap();
    assert_eq!(edit.new_text, "DeLiVeR");
    let mut applied = source.clone();
    applied.replace_range(
        offset_at_position(&file, edit.range.start) as usize..offset_at_position(&file, edit.range.end) as usize,
        &edit.new_text,
    );
    assert_eq!(applied, source.replace("$sender->deliver(", "$sender->DeLiVeR("));
    let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
    assert_eq!(
        hover["contents"]["value"],
        "Sender\n\npublic function DeLiVeR(string $message, int $attempt = 1): void\n\nSends a message."
    );
    let position = position_at_offset(&file, (source.rfind(", 3").unwrap() + 2) as u32);
    let signature = h.at("textDocument/signatureHelp", "a.php", position.line, position.character).await;
    assert_eq!(
        signature["signatures"][0]["label"],
        "function Sender::DeLiVeR(string $message, int $attempt = 1): void"
    );
}

#[tokio::test]
async fn trait_method_alias_uses_declared_case() {
    check_mixed_case_alias("class Sender { use RawSender { send as public DeLiVeR; } }", "Sender").await;
}

#[tokio::test]
async fn trait_method_alias_inherits_appearance_owner_and_visibility() {
    check_mixed_case_alias(
        "class Sender { use RawSender { send as public DeLiVeR; } } class ChildSender extends Sender {}",
        "ChildSender",
    )
    .await;
}

#[tokio::test]
async fn trait_method_alias_keeps_nested_trait_spelling_and_docs() {
    check_mixed_case_alias(
        "trait WrappedSender { use RawSender { send as public DeLiVeR; } } \
         class Sender { use WrappedSender; } class ChildSender extends Sender {}",
        "ChildSender",
    )
    .await;
}

#[tokio::test]
async fn trait_method_alias_completion_respects_private_and_protected_access() {
    let source = r"<?php
trait Methods {
    public function first(int $value): void {}
    public function second(int $value): void {}
}
class Sender {
    use Methods { first as private privateSend; second as protected protectedSend; }
    function inside(): void { $this->privateSend(1); }
}
class ChildSender extends Sender {
    function child(): void { $this->protectedSend(1); }
}
$sender = new Sender();
$sender->first(1);
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    for (call, private, protected) in
        [("privateSend(1)", true, true), ("protectedSend(1)", false, true), ("first(1)", false, false)]
    {
        let selector = source.rfind(call).unwrap();
        let position = position_at_offset(&file, selector as u32);
        let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
        let items = result["items"].as_array().unwrap();
        assert_eq!(items.iter().any(|item| item["label"] == "privateSend"), private, "{call}: {items:?}");
        assert_eq!(items.iter().any(|item| item["label"] == "protectedSend"), protected, "{call}: {items:?}");
        if private || protected {
            let position = position_at_offset(&file, (selector + 1) as u32);
            let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
            let (visibility, name) = if private { ("private", "privateSend") } else { ("protected", "protectedSend") };
            assert_eq!(hover["contents"]["value"], format!("Sender\n\n{visibility} function {name}(int $value): void"));
        }
    }
}

const MULTIPLE_ALIAS_ORDERS: [&str; 2] =
    ["send as FirstAlias; send as SecondAlias;", "send as SecondAlias; send as FirstAlias;"];

fn multiple_trait_alias_source(adaptations: &str) -> String {
    format!(
        r"<?php
trait RawSender {{
    /** Sends a message. */
    public function send(string $message, int $attempt = 1): void {{}}
}}
class Sender {{
    use RawSender {{ {adaptations} }}
}}
$sender = new Sender();
$sender->FirstAlias('hello', 3);
$sender->SecondAlias('hello', 3);
"
    )
}

#[tokio::test]
async fn trait_method_multiple_aliases_completion_keeps_all_names() {
    for adaptations in MULTIPLE_ALIAS_ORDERS {
        let source = multiple_trait_alias_source(adaptations);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());

        for name in ["FirstAlias", "SecondAlias"] {
            let selector = source.rfind(&format!("{name}(")).unwrap();
            let position = position_at_offset(&file, selector as u32);
            let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
            let items = result["items"].as_array().unwrap();
            let mut labels: Vec<_> = items.iter().map(|item| item["label"].as_str().unwrap()).collect();
            labels.sort_unstable();
            assert_eq!(labels, ["FirstAlias", "SecondAlias", "send"], "{adaptations}: {items:?}");
            for label in labels {
                let item = items.iter().find(|item| item["label"] == label).unwrap();
                assert_eq!(item["detail"], format!("{label}(string $message, int $attempt = 1): void"));
            }

            let position = position_at_offset(&file, (selector + 3) as u32);
            let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
            let items = result["items"].as_array().unwrap();
            let item = items
                .iter()
                .find(|item| item["label"] == name)
                .unwrap_or_else(|| panic!("missing {name}: {adaptations}: {items:?}"));
            let edit: TextEdit = serde_json::from_value(item["textEdit"].clone()).unwrap();
            assert_eq!(edit.new_text, name);
            let mut applied = source.clone();
            applied.replace_range(
                offset_at_position(&file, edit.range.start) as usize
                    ..offset_at_position(&file, edit.range.end) as usize,
                &edit.new_text,
            );
            assert_eq!(applied, source);
        }
    }
}

#[tokio::test]
async fn trait_method_multiple_aliases_hover_keeps_each_name_and_docs() {
    for adaptations in MULTIPLE_ALIAS_ORDERS {
        let source = multiple_trait_alias_source(adaptations);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
        for name in ["FirstAlias", "SecondAlias"] {
            let selector = source.rfind(&format!("{name}(")).unwrap();
            let position = position_at_offset(&file, (selector + 1) as u32);
            let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
            assert_eq!(
                hover["contents"]["value"],
                format!(
                    "Sender\n\npublic function {name}(string $message, int $attempt = 1): void\n\nSends a message."
                ),
                "{adaptations}: {hover}"
            );
        }
    }
}

#[tokio::test]
async fn trait_method_multiple_aliases_signature_keeps_each_name_and_parameters() {
    for adaptations in MULTIPLE_ALIAS_ORDERS {
        let source = multiple_trait_alias_source(adaptations);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
        for name in ["FirstAlias", "SecondAlias"] {
            let call = format!("{name}('hello', 3)");
            let offset = source.rfind(&call).unwrap() + call.find(", 3").unwrap() + 2;
            let position = position_at_offset(&file, offset as u32);
            let result = h.at("textDocument/signatureHelp", "a.php", position.line, position.character).await;
            let signature = &result["signatures"][0];
            assert_eq!(
                signature["label"],
                format!("function Sender::{name}(string $message, int $attempt = 1): void"),
                "{adaptations}: {result}"
            );
            assert_eq!(result["activeParameter"], 1);
            assert_eq!(signature["activeParameter"], 1);
            let label: Vec<_> = signature["label"].as_str().unwrap().encode_utf16().collect();
            let parameters = signature["parameters"].as_array().unwrap();
            assert_eq!(parameters.len(), 2);
            for (parameter, expected) in parameters.iter().zip(["string $message", "int $attempt = 1"]) {
                let start = parameter["label"][0].as_u64().unwrap() as usize;
                let end = parameter["label"][1].as_u64().unwrap() as usize;
                assert_eq!(String::from_utf16(&label[start..end]).unwrap(), expected);
            }
        }
    }
}

#[tokio::test]
async fn trait_method_multiple_aliases_definition_finds_the_original_declaration() {
    for adaptations in MULTIPLE_ALIAS_ORDERS {
        let source = multiple_trait_alias_source(adaptations);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
        let declaration = source.find("send(string").unwrap();
        for name in ["FirstAlias", "SecondAlias"] {
            let selector = source.rfind(&format!("{name}(")).unwrap();
            let position = position_at_offset(&file, (selector + 1) as u32);
            let result = h.at("textDocument/definition", "a.php", position.line, position.character).await;
            assert_eq!(result["uri"], h.url("a.php"), "{adaptations}: {result}");
            let location: tower_lsp_server::gen_lsp_types::Location = serde_json::from_value(result).unwrap();
            assert_eq!(offset_at_position(&file, location.range.start) as usize, declaration);
            assert_eq!(offset_at_position(&file, location.range.end) as usize, declaration + "send".len());
        }
    }
}

#[tokio::test]
async fn trait_method_realias_keeps_public_visibility_spelling_docs_and_signature() {
    for adaptation in ["wrapped as DeLiVeR", "WrappedSender::wrapped as DeLiVeR"] {
        let classes = format!(
            "trait WrappedSender {{ use RawSender {{ send as public wrapped; }} }} \
             class Sender {{ use WrappedSender {{ {adaptation}; }} }} \
             class ChildSender extends Sender {{}}"
        );
        for receiver in ["Sender", "ChildSender"] {
            check_mixed_case_alias(&classes, receiver).await;
        }
    }
}

fn trait_realias_source(raw_visibility: &str, inner_modifier: &str, outer_modifier: &str) -> String {
    format!(
        r"<?php
trait RawSender {{
    /** Sends a message. */
    {raw_visibility} function send(string $message): void {{}}
}}
trait WrappedSender {{
    use RawSender {{ send as {inner_modifier} wrapped; }}
}}
class Sender {{
    use WrappedSender {{ wrapped as {outer_modifier} DeLiVeR; }}
}}
class ChildSender extends Sender {{}}
$sender = new Sender();
$child = new ChildSender();
$sender->deliver('hello');
$child->deliver('hello');
"
    )
}

#[tokio::test]
async fn trait_method_realias_visibility_keeps_inner_private_and_outer_overrides() {
    for (inner, outer, visible) in [("private", "", false), ("private", "public", true), ("public", "private", false)] {
        let source = trait_realias_source("public", inner, outer);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
        for receiver in ["sender", "child"] {
            let call = format!("${receiver}->deliver(");
            let selector = source.find(&call).unwrap() + receiver.len() + 3;
            let position = position_at_offset(&file, selector as u32);
            let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
            let items = result["items"].as_array().unwrap();
            assert_eq!(
                items.iter().any(|item| item["label"] == "DeLiVeR"),
                visible,
                "{receiver}, inner {inner}, outer {outer}: {items:?}"
            );
        }
    }
}

#[tokio::test]
async fn trait_method_realias_keeps_inner_final_modifier() {
    let source = trait_realias_source("public", "final", "");
    let mut h = Harness::start(&[("a.php", &source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    for receiver in ["sender", "child"] {
        let call = format!("${receiver}->deliver(");
        let offset = source.find(&call).unwrap() + receiver.len() + 4;
        let position = position_at_offset(&file, offset as u32);
        let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
        assert_eq!(
            hover["contents"]["value"],
            "Sender\n\nfinal public function DeLiVeR(string $message): void\n\nSends a message.",
            "{receiver}: {hover}"
        );
    }
}

#[tokio::test]
async fn trait_method_realias_definition_keeps_original_declaration() {
    let source = trait_realias_source("protected", "public", "");
    let mut h = Harness::start(&[("a.php", &source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let declaration = source.find("send(string").unwrap();
    for receiver in ["sender", "child"] {
        let call = format!("${receiver}->deliver(");
        let offset = source.find(&call).unwrap() + receiver.len() + 4;
        let position = position_at_offset(&file, offset as u32);
        let result = h.at("textDocument/definition", "a.php", position.line, position.character).await;
        assert_eq!(result["uri"], h.url("a.php"), "{receiver}: {result}");
        let location: tower_lsp_server::gen_lsp_types::Location = serde_json::from_value(result).unwrap();
        assert_eq!(offset_at_position(&file, location.range.start) as usize, declaration);
        assert_eq!(offset_at_position(&file, location.range.end) as usize, declaration + "send".len());
    }
}

#[tokio::test]
async fn trait_method_source_follows_each_wrapper_alias() {
    for source in ["second", "SecondWrapper::second"] {
        let classes = format!(
            "trait FirstWrapper {{ use RawSender {{ send as public first; }} }} \
             trait SecondWrapper {{ use FirstWrapper {{ first as second; }} }} \
             class Sender {{ use SecondWrapper {{ {source} as DeLiVeR; }} }} \
             class ChildSender extends Sender {{}}"
        );
        for receiver in ["Sender", "ChildSender"] {
            check_mixed_case_alias(&classes, receiver).await;
        }
    }
}

#[tokio::test]
async fn trait_method_source_shared_diamond_keeps_one_visibility_source() {
    for traits in ["LeftWrapper, RightWrapper", "RightWrapper, LeftWrapper"] {
        let classes = format!(
            "trait CommonWrapper {{ use RawSender {{ send as public wrapped; }} }} \
             trait LeftWrapper {{ use CommonWrapper; }} \
             trait RightWrapper {{ use CommonWrapper; }} \
             class Sender {{ use {traits} {{ wrapped as DeLiVeR; }} }} \
             class ChildSender extends Sender {{}}"
        );
        for receiver in ["Sender", "ChildSender"] {
            check_mixed_case_alias(&classes, receiver).await;
        }
    }
}
