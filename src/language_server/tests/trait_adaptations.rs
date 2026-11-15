//! Qualified trait adaptations keep each method's source and signature.

use mago_database::file::File;
use tower_lsp_server::gen_lsp_types::TextEdit;

use super::harness::Harness;
use crate::language_server::position::{offset_at_position, position_at_offset};

const QUALIFIED_ALIAS_SOURCE: &str = r"<?php
trait FirstSender {
    /** Sends a number. */
    public function send(int $number): int { return $number; }
}
trait SecondSender {
    /** Sends a message. */
    public function send(string $message): string { return $message; }
}
class Sender {
    use FirstSender, SecondSender {
        SecondSender::send insteadof FirstSender;
        FirstSender::send as SendNumber;
        SecondSender::send as SendText;
    }
}
$sender = new Sender();
$sender->send('hello');
$sender->SendNumber(1);
$sender->SendText('hello');
";

const QUALIFIED_ALIAS_METHODS: [(&str, &str, &str, &str); 3] = [
    ("send", "string $message", "string", "Sends a message."),
    ("SendNumber", "int $number", "int", "Sends a number."),
    ("SendText", "string $message", "string", "Sends a message."),
];

#[tokio::test]
async fn trait_method_qualified_aliases_signature_uses_each_source() {
    let mut h = Harness::start(&[("a.php", QUALIFIED_ALIAS_SOURCE)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), QUALIFIED_ALIAS_SOURCE.as_bytes().to_vec().into());
    for (name, parameter, return_type, _) in QUALIFIED_ALIAS_METHODS {
        let call = format!("$sender->{name}(");
        let offset = QUALIFIED_ALIAS_SOURCE.find(&call).unwrap() + call.len();
        let position = position_at_offset(&file, offset as u32);
        let result = h.at("textDocument/signatureHelp", "a.php", position.line, position.character).await;
        let signature = &result["signatures"][0];
        assert_eq!(signature["label"], format!("function Sender::{name}({parameter}): {return_type}"), "{result}");
        assert_eq!(result["activeParameter"], 0);
        let label: Vec<_> = signature["label"].as_str().unwrap().encode_utf16().collect();
        let parameters = signature["parameters"].as_array().unwrap();
        assert_eq!(parameters.len(), 1);
        let start = parameters[0]["label"][0].as_u64().unwrap() as usize;
        let end = parameters[0]["label"][1].as_u64().unwrap() as usize;
        assert_eq!(String::from_utf16(&label[start..end]).unwrap(), parameter);
    }
}

#[tokio::test]
async fn trait_method_qualified_aliases_hover_keeps_source_docs_and_alias_names() {
    let mut h = Harness::start(&[("a.php", QUALIFIED_ALIAS_SOURCE)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), QUALIFIED_ALIAS_SOURCE.as_bytes().to_vec().into());
    for (name, parameter, return_type, docs) in QUALIFIED_ALIAS_METHODS {
        let offset = QUALIFIED_ALIAS_SOURCE.find(&format!("->{name}(")).unwrap() + 3;
        let position = position_at_offset(&file, offset as u32);
        let result = h.at("textDocument/hover", "a.php", position.line, position.character).await;
        assert_eq!(
            result["contents"]["value"],
            format!("Sender\n\npublic function {name}({parameter}): {return_type}\n\n{docs}"),
            "{result}"
        );
    }
}

#[tokio::test]
async fn trait_method_qualified_aliases_definition_uses_each_original_declaration() {
    let mut h = Harness::start(&[("a.php", QUALIFIED_ALIAS_SOURCE)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), QUALIFIED_ALIAS_SOURCE.as_bytes().to_vec().into());
    for (name, parameter, _, _) in QUALIFIED_ALIAS_METHODS {
        let declaration = QUALIFIED_ALIAS_SOURCE.find(&format!("send({parameter})")).unwrap();
        let offset = QUALIFIED_ALIAS_SOURCE.find(&format!("->{name}(")).unwrap() + 3;
        let position = position_at_offset(&file, offset as u32);
        let result = h.at("textDocument/definition", "a.php", position.line, position.character).await;
        assert_eq!(result["uri"], h.url("a.php"), "{name}: {result}");
        let location: tower_lsp_server::gen_lsp_types::Location = serde_json::from_value(result).unwrap();
        assert_eq!(offset_at_position(&file, location.range.start) as usize, declaration, "{name}");
        assert_eq!(offset_at_position(&file, location.range.end) as usize, declaration + "send".len(), "{name}");
    }
}

#[tokio::test]
async fn trait_method_qualified_aliases_parameter_completion_uses_each_source() {
    for (name, parameter, _, _) in QUALIFIED_ALIAS_METHODS {
        let call = format!("$sender->{name}(");
        let argument_start = QUALIFIED_ALIAS_SOURCE.find(&call).unwrap() + call.len();
        let argument_end = argument_start + QUALIFIED_ALIAS_SOURCE[argument_start..].find(')').unwrap();
        let mut source = QUALIFIED_ALIAS_SOURCE.to_owned();
        source.replace_range(argument_start..argument_end, "");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
        let position = position_at_offset(&file, argument_start as u32);
        let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
        let items = result["items"].as_array().unwrap();
        let parameter_name = parameter.split_once('$').unwrap().1;
        let expected = format!("{parameter_name}:");
        let labels: Vec<_> =
            items.iter().filter_map(|item| item["label"].as_str()).filter(|label| label.ends_with(':')).collect();
        assert_eq!(labels, [expected.as_str()], "{name}: {items:?}");
        let item = items.iter().find(|item| item["label"] == expected).unwrap();
        assert_eq!(item["detail"], parameter);
        let edit: TextEdit = serde_json::from_value(item["textEdit"].clone()).unwrap();
        assert_eq!(edit.new_text, format!("{parameter_name}: "));
        assert_eq!(offset_at_position(&file, edit.range.start) as usize, argument_start);
        assert_eq!(edit.range.start, edit.range.end);
    }
}

use serde_json::Value;

async fn adaptation_query(h: &mut Harness, method: &str, source: &str, needle: &str, relative: usize) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, (source.rfind(needle).unwrap() + relative) as u32);
    h.at(method, "a.php", position.line, position.character).await
}

async fn check_qualified_presentations(h: &mut Harness, source: &str, namespaced: bool, first: bool) -> Vec<Value> {
    let mut snapshot = Vec::new();
    for (name, mut parameter, mut return_type, mut docs) in QUALIFIED_ALIAS_METHODS {
        if first && name == "send" {
            (parameter, return_type, docs) = ("int $number", "int", "Sends a number.");
        }
        let call = format!("$sender->{name}(");
        let namespace = if namespaced { "App\\" } else { "" };
        let owner = if namespaced { "App\\Sender" } else { "Sender" };
        let hover = adaptation_query(h, "textDocument/hover", source, &call, "$sender->".len() + 1).await;
        assert_eq!(
            hover["contents"]["value"],
            format!("{namespace}Sender\n\npublic function {name}({parameter}): {return_type}\n\n{docs}")
        );
        let signature = adaptation_query(h, "textDocument/signatureHelp", source, &call, call.len()).await;
        assert_eq!(
            signature["signatures"][0]["label"],
            format!("function {owner}::{name}({parameter}): {return_type}")
        );
        let result = adaptation_query(h, "textDocument/completion", source, &call, "$sender->".len()).await;
        let items = result["items"].as_array().unwrap();
        let item = items.iter().find(|item| item["label"] == name).unwrap_or_else(|| panic!("{result}"));
        assert_eq!(item["detail"], format!("{name}({parameter}): {return_type}"));
        snapshot.extend([hover["contents"].clone(), signature["signatures"].clone(), item.clone()]);
    }
    snapshot
}

#[tokio::test]
async fn qualified_trait_modifiers_only_change_the_selected_original() {
    for origin in ["FirstSender", "SecondSender"] {
        for modifier in ["private", "protected", "final"] {
            let source = QUALIFIED_ALIAS_SOURCE
                .replace("class Sender {", "class Sender {\npublic function inspect(): void { $this->send('hello'); }")
                .replace(
                    "SecondSender::send insteadof FirstSender;",
                    &format!("SecondSender::send insteadof FirstSender; {origin}::send as {modifier};"),
                );
            let mut h = Harness::start(&[("a.php", &source)]).await;
            let selected = origin == "SecondSender";
            let visibility = if selected && modifier != "final" { modifier } else { "public" };
            let final_prefix = if selected && modifier == "final" { "final " } else { "" };
            let hover = adaptation_query(&mut h, "textDocument/hover", &source, "$this->send(", 8).await;
            assert_eq!(
                hover["contents"]["value"],
                format!(
                    "Sender\n\n{final_prefix}{visibility} function send(string $message): string\n\nSends a message."
                ),
                "{origin} as {modifier}: {hover}"
            );
            for (call, visible) in [("$this->send(", true), ("$sender->send(", visibility == "public")] {
                let offset = call.find("->").unwrap() + 2;
                let result = adaptation_query(&mut h, "textDocument/completion", &source, call, offset).await;
                let items = result["items"].as_array().unwrap();
                assert_eq!(
                    items.iter().any(|item| item["label"] == "send"),
                    visible,
                    "{origin} as {modifier}: {result}"
                );
                for alias in ["SendNumber", "SendText"] {
                    assert!(items.iter().any(|item| item["label"] == alias), "{result}");
                }
            }
            // Changing one imported original must not change either renamed copy.
            for (name, parameter, return_type, docs) in QUALIFIED_ALIAS_METHODS.into_iter().skip(1) {
                let call = format!("$sender->{name}(");
                let hover = adaptation_query(&mut h, "textDocument/hover", &source, &call, "$sender->".len() + 1).await;
                assert_eq!(
                    hover["contents"]["value"],
                    format!("Sender\n\npublic function {name}({parameter}): {return_type}\n\n{docs}"),
                    "{origin} as {modifier}: {hover}"
                );
            }
        }
    }
}

#[tokio::test]
async fn qualified_trait_sources_resolve_imports_wrappers_and_inherited_aliases() {
    for wrapped in [false, true] {
        let (wrapper, number_source) =
            if wrapped { ("trait NumberWrapper { use FirstSender; }\n", "NumberWrapper") } else { ("", "FirstSender") };
        let source = QUALIFIED_ALIAS_SOURCE
            .replacen("<?php", "<?php\nnamespace Traits;", 1)
            .replace("class Sender {", &format!(
                "{wrapper}namespace App;\nuse Traits\\{number_source} as Numbers;\nuse Traits\\SecondSender as Text;\nclass Sender {{"
            ))
            .replace("use FirstSender, SecondSender", "use Numbers, Text")
            .replace("FirstSender::send", "Numbers::send")
            .replace("SecondSender::send", "Text::send")
            .replace("insteadof FirstSender", "insteadof Numbers")
            .replace("$sender = new Sender();", "class ChildSender extends Sender {}\n$sender = new ChildSender();");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        check_qualified_presentations(&mut h, &source, true, false).await;
    }
}

#[tokio::test]
async fn qualified_trait_precedence_edits_match_fresh_analysis() {
    let mut h = Harness::start(&[("a.php", QUALIFIED_ALIAS_SOURCE)]).await;
    let original = check_qualified_presentations(&mut h, QUALIFIED_ALIAS_SOURCE, false, false).await;
    h.open("a.php", QUALIFIED_ALIAS_SOURCE).await;
    let switched = QUALIFIED_ALIAS_SOURCE
        .replace("SecondSender::send insteadof FirstSender", "FirstSender::send insteadof SecondSender")
        .replace("$sender->send('hello')", "$sender->send(1)");
    h.change("a.php", &switched, 2).await;
    let edited = check_qualified_presentations(&mut h, &switched, false, true).await;
    let mut fresh = Harness::start(&[("a.php", &switched)]).await;
    assert_eq!(edited, check_qualified_presentations(&mut fresh, &switched, false, true).await);
    h.change("a.php", QUALIFIED_ALIAS_SOURCE, 3).await;
    assert_eq!(original, check_qualified_presentations(&mut h, QUALIFIED_ALIAS_SOURCE, false, false).await);
}

fn shared_trait_source(modifier: &str, traits: &str) -> String {
    format!(
        r"<?php
trait Inner {{
    /** Runs a number. */
    public function run(int $number): int {{ return $number; }}
}}
trait Left {{ use Inner {{ run as {modifier}; }} }}
trait Right {{ use Inner; }}
class Host {{
    use {traits} {{
        Right::run insteadof Left;
        Left::run as leftRun;
        Left::run as public publicLeft;
    }}
    public function inspect(): void {{ $this->leftRun(1); $this->publicLeft(1); }}
}}
$host = new Host();
$host->run(1);
"
    )
}

#[tokio::test]
async fn trait_method_source_exclusion_keeps_selected_modifiers() {
    for modifier in ["private", "final"] {
        for traits in ["Left, Right", "Right, Left"] {
            let source = shared_trait_source(modifier, traits);
            let mut h = Harness::start(&[("a.php", &source)]).await;
            let hover = adaptation_query(&mut h, "textDocument/hover", &source, "$host->run(", 8).await;
            assert_eq!(
                hover["contents"]["value"], "Host\n\npublic function run(int $number): int\n\nRuns a number.",
                "{modifier}, {traits}: {hover}"
            );
            let completion = adaptation_query(&mut h, "textDocument/completion", &source, "$host->run(", 7).await;
            assert!(
                completion["items"].as_array().unwrap().iter().any(|item| item["label"] == "run"),
                "{modifier}, {traits}: {completion}"
            );
        }
    }
}

#[tokio::test]
async fn trait_method_source_qualified_alias_keeps_excluded_modifiers() {
    for modifier in ["private", "final"] {
        let source = shared_trait_source(modifier, "Left, Right");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let final_prefix = if modifier == "final" { "final " } else { "" };
        for name in ["leftRun", "publicLeft"] {
            let visibility = if name == "leftRun" && modifier == "private" { "private" } else { "public" };
            let call = format!("$this->{name}(");
            let hover = adaptation_query(&mut h, "textDocument/hover", &source, &call, 8).await;
            assert_eq!(
                hover["contents"]["value"],
                format!("Host\n\n{final_prefix}{visibility} function {name}(int $number): int\n\nRuns a number."),
                "{modifier}: {hover}"
            );
        }
        let completion = adaptation_query(&mut h, "textDocument/completion", &source, "$host->run(", 7).await;
        let items = completion["items"].as_array().unwrap();
        assert_eq!(items.iter().any(|item| item["label"] == "leftRun"), modifier != "private", "{completion}");
        assert!(items.iter().any(|item| item["label"] == "publicLeft"), "{completion}");
    }
}
