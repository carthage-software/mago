use serde_json::json;

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};

#[tokio::test]
async fn diagnostic_metadata_respects_client_support() {
    let source = "<?php declare(strict_types=1); function expectsInt(int $value): void {} expectsInt('wrong'); 1; $number = (integer) 1;";
    for supported in [false, true] {
        let capabilities = if supported {
            json!({"textDocument":{"publishDiagnostics":{
                "relatedInformation":true, "codeDescriptionSupport":true, "tagSupport":{"valueSet":[1,2]}
            }}})
        } else {
            json!({})
        };
        let mut config = ServerConfig::default();
        config.configuration.php_version = mago_php_version::PHPVersion::PHP85;
        let mut harness = Harness::start_with(&[("a.php", source)], config, capabilities).await;
        let publications = harness.diagnostics().await;
        let diagnostics: Vec<_> =
            publications.iter().flat_map(|publication| publication["diagnostics"].as_array().unwrap()).collect();
        for code in ["invalid-argument", "unused-statement", "deprecated-cast"] {
            assert!(diagnostics.iter().any(|diagnostic| diagnostic["code"] == code), "{diagnostics:?}");
        }
        let cast = diagnostics.iter().find(|diagnostic| diagnostic["code"] == "deprecated-cast").unwrap();
        assert_eq!(
            cast["message"].as_str().unwrap().contains("\n = See: https://wiki.php.net/rfc/deprecations_php_8_5"),
            !supported,
            "{cast}"
        );
        assert_eq!(
            diagnostics.iter().any(|diagnostic| diagnostic["relatedInformation"].is_array()),
            supported,
            "{diagnostics:?}"
        );
        assert_eq!(
            diagnostics.iter().any(|diagnostic| diagnostic["codeDescription"].is_object()),
            supported,
            "{diagnostics:?}"
        );
        assert_eq!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic["tags"].as_array().is_some_and(|tags| tags.contains(&json!(1)))),
            supported,
            "{diagnostics:?}"
        );
        assert_eq!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic["tags"].as_array().is_some_and(|tags| tags.contains(&json!(2)))),
            supported,
            "{diagnostics:?}"
        );
    }
}

#[tokio::test]
async fn diagnostic_markup_messages_require_explicit_diagnostic_support() {
    let source = "<?php declare(strict_types=1); function expectsInt(int $value): void {} expectsInt('wrong');";
    for support in [None, Some(false), Some(true)] {
        let mut capabilities = super::harness::client_capabilities();
        capabilities["textDocument"]["hover"] = json!({"contentFormat": ["markdown"]});
        if let Some(support) = support {
            capabilities["textDocument"]["diagnostic"] = json!({"markupMessageSupport": support});
        }
        let mut harness = Harness::start_with(&[("a.php", source)], ServerConfig::default(), capabilities).await;
        let publications = harness.diagnostics().await;
        let diagnostic = publications
            .iter()
            .flat_map(|publication| publication["diagnostics"].as_array().unwrap())
            .find(|diagnostic| diagnostic["code"] == "invalid-argument")
            .unwrap();
        let message = &diagnostic["message"];
        if support == Some(true) {
            assert_eq!(message["kind"], "markdown", "{diagnostic}");
            let value = message["value"].as_str().unwrap();
            assert_eq!(value.matches("\n\n").count(), 1, "{diagnostic}");
            assert!(value.contains("\\\n\\= Help\\: "), "{diagnostic}");
        } else {
            let message = message.as_str().unwrap();
            assert!(!message.contains("\n\n"), "{diagnostic}");
            assert!(message.contains("\n = Help: "), "{diagnostic}");
        }
    }
}

#[tokio::test]
async fn completion_insertion_respects_snippet_support() {
    for supported in [false, true] {
        for (marked, label, plain, snippet) in [
            ("<?php strle|;", "strlen", "strlen", "strlen($1)"),
            ("<?php DateTimeImmutable::createFromF|;", "createFromFormat", "createFromFormat", "createFromFormat($1)"),
            (
                "<?php namespace Lib; function work() {} namespace App; wor|;",
                "work",
                "\\Lib\\work",
                "\\\\Lib\\\\work($1)",
            ),
            ("<?php strle|('x');", "strlen", "strlen", "strlen"),
        ] {
            let offset = marked.find('|').unwrap();
            let source = marked.replace('|', "");
            let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
            let position = position_at_offset(&file, offset as u32);
            let capabilities = if supported {
                json!({"textDocument": {"completion": {"completionItem": {"snippetSupport": true}}}})
            } else {
                json!({})
            };
            let mut h = Harness::start_with(&[("a.php", &source)], ServerConfig::default(), capabilities).await;
            let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
            let item = result["items"].as_array().unwrap().iter().find(|item| item["label"] == label).unwrap();
            let expected = if supported { snippet } else { plain };
            assert_eq!(item["textEdit"]["newText"], expected, "{marked}: {item}");
            assert_eq!(item["insertTextFormat"] == 2, supported && snippet != plain, "{item}");
            if !supported {
                let range: tower_lsp_server::gen_lsp_types::Range =
                    serde_json::from_value(item["textEdit"]["range"].clone()).unwrap();
                let start = offset_at_position(&file, range.start) as usize;
                let end = offset_at_position(&file, range.end) as usize;
                let mut applied = source.clone();
                applied.replace_range(start..end, expected);
                assert!(!applied.contains("$1"), "{applied}");
                assert!(applied.contains(plain));
            }
        }
    }
}

#[tokio::test]
async fn completion_snippet_indentation_matches_the_negotiated_mode() {
    let source = "<?php class A {\n\tpublic function \n}";
    for (modes, default, expected, as_is) in [
        (None, None, None, false),
        (None, Some(1), None, true),
        (None, Some(2), None, false),
        (Some(vec![2]), Some(2), Some(2), false),
        (Some(vec![1, 2]), Some(2), Some(2), false),
        (Some(vec![1, 2]), Some(1), Some(2), false),
        (Some(vec![1]), Some(1), Some(1), true),
    ] {
        let mut capabilities = json!({"textDocument": {"completion": {"completionItem": {"snippetSupport": true}}}});
        if let Some(modes) = modes {
            capabilities["textDocument"]["completion"]["completionItem"]["insertTextModeSupport"] =
                json!({"valueSet": modes});
        }
        if let Some(default) = default {
            capabilities["textDocument"]["completion"]["insertTextMode"] = json!(default);
        }
        let mut h = Harness::start_with(&[("a.php", source)], ServerConfig::default(), capabilities).await;
        let result = h.at("textDocument/completion", "a.php", 1, 17).await;
        let item = result["items"].as_array().unwrap().iter().find(|item| item["label"] == "public function").unwrap();
        assert_eq!(item["insertTextMode"], json!(expected));
        let indent = if as_is { "\t" } else { "" };
        assert_eq!(
            item["textEdit"]["newText"],
            format!("public function ${{1:name}}(${{2}}): ${{3:void}}\n{indent}{{\n{indent}    $0\n{indent}}}"),
        );
    }
}

#[tokio::test]
async fn signature_labels_respect_client_support() {
    let source = "<?php function café(string $名, int $count): void {} café('', 1);";
    let offset = source.rfind('1').unwrap() as u32;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset);
    for supported in [false, true] {
        let capabilities = json!({"textDocument": {"signatureHelp": {"signatureInformation": {
            "parameterInformation": {"labelOffsetSupport": supported}, "activeParameterSupport": supported
        }}}});
        let mut h = Harness::start_with(&[("a.php", source)], ServerConfig::default(), capabilities).await;
        let result = h.at("textDocument/signatureHelp", "a.php", position.line, position.character).await;
        assert_eq!(result["activeParameter"], 1, "{result}");
        let signature = &result["signatures"][0];
        assert_eq!(signature["activeParameter"], if supported { json!(1) } else { json!(null) }, "{result}");
        for (parameter, expected) in
            signature["parameters"].as_array().unwrap().iter().zip(["string $名", "int $count"])
        {
            if supported {
                let label: Vec<_> = signature["label"].as_str().unwrap().encode_utf16().collect();
                let start = parameter["label"][0].as_u64().unwrap() as usize;
                let end = parameter["label"][1].as_u64().unwrap() as usize;
                assert_eq!(String::from_utf16(&label[start..end]).unwrap(), expected);
            } else {
                assert_eq!(parameter["label"], expected, "{result}");
            }
        }
    }
}

#[tokio::test]
async fn rename_options_require_prepare_support() {
    for prepare in [None, Some(false), Some(true)] {
        let mut capabilities = json!({});
        if let Some(prepare) = prepare {
            capabilities = json!({ "textDocument": { "rename": { "prepareSupport": prepare } } });
        }
        let mut h = Harness::start_with(&[("a.php", "<?php $value = 1;")], ServerConfig::default(), capabilities).await;
        let expected = if prepare == Some(true) { json!({ "prepareProvider": true }) } else { json!(true) };
        assert_eq!(h.server_capabilities["renameProvider"], expected);
        let result = h.request("textDocument/rename", json!({
            "textDocument": { "uri": h.url("a.php") }, "position": { "line": 0, "character": 7 }, "newName": "item"
        })).await;
        assert_eq!(
            super::references::apply("<?php $value = 1;", &result["changes"][h.url("a.php")]),
            "<?php $item = 1;"
        );
    }
}
