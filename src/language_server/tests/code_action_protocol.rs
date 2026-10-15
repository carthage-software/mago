use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use super::references::apply;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;

const SOURCE: &str = "<?php new \\DateTime();";
const COMMAND: &str = "mago.applyCodeAction";

#[tokio::test]
async fn quick_fixes_attach_the_exact_published_preview() {
    let source = "<?php final class Holder { protected private(set) int $value = 1; } $number = (integer) 1;";
    for markup in [None, Some(false), Some(true)] {
        for supports_links in [false, true] {
            let mut capabilities = client_capabilities();
            capabilities["textDocument"]["publishDiagnostics"] = json!({
                "relatedInformation": true, "codeDescriptionSupport": supports_links, "tagSupport": {"valueSet": [1, 2]}
            });
            if let Some(markup) = markup {
                capabilities["textDocument"]["diagnostic"] = json!({"markupMessageSupport": markup});
            }
            let mut config = ServerConfig::default();
            config.configuration.php_version = mago_php_version::PHPVersion::PHP85;
            config.configuration.linter.rules.no_protected_in_final.enabled = true;
            let mut h = Harness::start_with(&[("a.php", source)], config, capabilities).await;
            let published: Vec<_> = h
                .diagnostics()
                .await
                .into_iter()
                .flat_map(|publication| publication["diagnostics"].as_array().unwrap().clone())
                .collect();
            for code in ["deprecated-cast", "no-protected-in-final"] {
                let diagnostic = published.iter().find(|diagnostic| diagnostic["code"] == code).unwrap();
                let actions = h
                    .request(
                        "textDocument/codeAction",
                        json!({
                            "textDocument": {"uri": h.url("a.php")},
                            "range": diagnostic["range"],
                            "context": {"diagnostics": [diagnostic], "only": ["quickfix"]}
                        }),
                    )
                    .await;
                let action =
                    actions.as_array().unwrap().iter().find(|action| action["diagnostics"][0]["code"] == code).unwrap();
                assert_eq!(action["diagnostics"], json!([diagnostic]), "{action}");
                if markup == Some(true) {
                    assert_eq!(diagnostic["message"]["kind"], "markdown", "{diagnostic}");
                    assert!(diagnostic["message"]["value"].is_string(), "{diagnostic}");
                } else {
                    assert!(diagnostic["message"].is_string(), "{diagnostic}");
                }
                if code == "deprecated-cast" {
                    assert_eq!(diagnostic["codeDescription"].is_object(), supports_links);
                    assert_eq!(diagnostic["tags"], json!([2]));
                    if markup != Some(true) && supports_links {
                        let mut unsolicited = diagnostic.clone();
                        unsolicited["message"] = json!({"kind": "markdown", "value": diagnostic["message"]});
                        let actions = h
                            .request(
                                "textDocument/codeAction",
                                json!({
                                    "textDocument": {"uri": h.url("a.php")},
                                    "range": diagnostic["range"],
                                    "context": {"diagnostics": [unsolicited], "only": ["quickfix"]}
                                }),
                            )
                            .await;
                        let action = actions
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|action| action["diagnostics"][0]["code"] == code)
                            .unwrap();
                        assert_eq!(action["diagnostics"][0]["message"], diagnostic["message"], "{action}");
                    }
                } else {
                    assert!(!diagnostic["relatedInformation"].as_array().unwrap().is_empty(), "{diagnostic}");
                }
            }
        }
    }
}

#[tokio::test]
async fn code_action_context_rejects_malformed_markup_messages_without_stopping_the_session() {
    let mut capabilities = client_capabilities();
    capabilities["textDocument"]["diagnostic"] = json!({"markupMessageSupport": true});
    let mut h = Harness::start_with(&[("a.php", SOURCE)], ServerConfig::default(), capabilities).await;
    let range = json!({"start": {"line": 0, "character": 10}, "end": {"line": 0, "character": 10}});
    for message in [json!(false), json!({"value": "missing kind"}), json!({"kind": "markdown", "value": 123})] {
        let id = h
            .client
            .send_request(
                "textDocument/codeAction",
                json!({
                    "textDocument": {"uri": h.url("a.php")}, "range": range,
                    "context": {"diagnostics": [{"range": range, "message": message}]}
                }),
            )
            .await;
        let response = h.client.await_response(id, 10).await;
        assert_eq!(response["error"]["code"], -32602, "{response}");
        assert!(h.client.server_requests().is_empty());
    }
    assert_eq!(actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await.as_array().unwrap().len(), 1);
}

async fn actions(h: &mut Harness, source: &str, needle: &str, kind: &str) -> Value {
    let before = &source[..source.find(needle).unwrap()];
    let position = json!({
        "line": before.bytes().filter(|byte| *byte == b'\n').count(),
        "character": before.rsplit('\n').next().unwrap().encode_utf16().count()
    });
    h.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": h.url("a.php") },
            "range": { "start": position, "end": position },
            "context": { "diagnostics": [], "only": [kind] }
        }),
    )
    .await
}

async fn apply_request(h: &mut Harness) -> Value {
    loop {
        let message = h.client.read_message(10).await;
        if message["method"] == "workspace/applyEdit" {
            assert!(message.get("id").is_some(), "{message}");
            return message;
        }
        assert!(message.get("id").is_none(), "expected workspace/applyEdit, got {message}");
    }
}

fn edit(uri: &str) -> Value {
    json!({ "changes": { (uri): [{
        "range": { "start": { "line": 0, "character": 10 }, "end": { "line": 0, "character": 11 } },
        "newText": ""
    }] } })
}

fn arguments(command: &Value, source: &str, target: &str) -> Value {
    let original = &command["arguments"];
    let edits = &original[1]["changes"][original[0].as_str().unwrap()];
    let mut source_snapshot = original[2][0].clone();
    source_snapshot["uri"] = json!(source);
    let mut snapshots = vec![source_snapshot.clone()];
    if source != target {
        source_snapshot["uri"] = json!(target);
        snapshots.push(source_snapshot);
    }
    json!([source, { "changes": { (target): edits } }, snapshots])
}

#[tokio::test]
async fn code_action_advertisement_and_results_follow_client_support() {
    for capabilities in
        [json!({}), json!({ "workspace": { "applyEdit": false }, "textDocument": { "codeAction": {} } })]
    {
        let mut h = Harness::start_with(&[("a.php", SOURCE)], ServerConfig::default(), capabilities).await;
        assert!(h.server_capabilities["codeActionProvider"].is_null());
        assert!(h.server_capabilities["executeCommandProvider"].is_null());
        assert!(actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await.is_null());
    }

    let mut modern = Harness::start(&[("a.php", SOURCE)]).await;
    assert_eq!(
        modern.server_capabilities["codeActionProvider"]["codeActionKinds"],
        json!(["quickfix", "refactor.rewrite", "refactor.extract"])
    );
    assert!(modern.server_capabilities["executeCommandProvider"].is_null());
    let result = actions(&mut modern, SOURCE, "\\DateTime", "refactor.rewrite").await;
    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(result[0]["kind"], "refactor.rewrite");
    assert!(result[0]["command"].is_null());
    assert_eq!(apply(SOURCE, &result[0]["edit"]["changes"][modern.url("a.php")]), "<?php new DateTime();");

    let mut legacy = Harness::start_with(
        &[("a.php", SOURCE)],
        ServerConfig::default(),
        json!({ "workspace": { "applyEdit": true } }),
    )
    .await;
    assert_eq!(legacy.server_capabilities["codeActionProvider"], true);
    assert_eq!(legacy.server_capabilities["executeCommandProvider"]["commands"], json!([COMMAND]));
    let commands = actions(&mut legacy, SOURCE, "\\DateTime", "refactor.rewrite").await;
    assert_eq!(commands.as_array().unwrap().len(), 1);
    assert_eq!(commands[0]["command"], COMMAND);
    assert_eq!(commands[0]["title"], result[0]["title"]);
    assert!(commands[0]["kind"].is_null());
    assert!(commands[0]["edit"].is_null());
    assert_eq!(commands[0]["arguments"].as_array().unwrap().len(), 3);
    assert_eq!(commands[0]["arguments"][2].as_array().unwrap().len(), 1);
    assert_eq!(commands[0]["arguments"][2][0]["uri"], legacy.url("a.php"));
    let hash = commands[0]["arguments"][2][0]["content_hash"].as_str().unwrap();
    assert_eq!(hash.len(), 32);
    assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(commands[0]["arguments"][0], legacy.url("a.php"));
    assert_eq!(apply(SOURCE, &commands[0]["arguments"][1]["changes"][legacy.url("a.php")]), "<?php new DateTime();");
}

#[tokio::test]
async fn preferred_code_actions_require_client_support() {
    let source = "<?php function check(mixed $x): bool { return is_null($x); }";
    for preferred in [None, Some(false), Some(true)] {
        let mut capabilities = json!({ "textDocument": { "codeAction": {
            "codeActionLiteralSupport": { "codeActionKind": { "valueSet": [] } }
        } } });
        if let Some(preferred) = preferred {
            capabilities["textDocument"]["codeAction"]["isPreferredSupport"] = json!(preferred);
        }
        let mut config = ServerConfig::default();
        config.configuration.linter.rules.no_is_null.enabled = true;
        let mut h = Harness::start_with(&[("a.php", source)], config, capabilities).await;
        let result = actions(&mut h, source, "is_null", "quickfix").await;
        let action =
            result.as_array().unwrap().iter().find(|action| action["diagnostics"][0]["code"] == "no-is-null").unwrap();
        assert_eq!(action["kind"], "quickfix");
        assert_eq!(action["isPreferred"], if preferred == Some(true) { json!(true) } else { Value::Null });
        assert_eq!(
            apply(source, &action["edit"]["changes"][h.url("a.php")]),
            "<?php function check(mixed $x): bool { return null === $x; }"
        );
    }
}

#[tokio::test]
async fn legacy_code_actions_apply_edits_while_document_changes_wait() {
    let mut h = Harness::start_with(
        &[("a.php", SOURCE)],
        ServerConfig::default(),
        json!({ "workspace": { "applyEdit": true } }),
    )
    .await;
    h.open("a.php", SOURCE).await;
    let commands = actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await;
    let command = &commands[0];
    assert_eq!(command["command"], COMMAND);
    let id = h
        .client
        .send_request(
            "workspace/executeCommand",
            json!({
                "command": command["command"], "arguments": command["arguments"]
            }),
        )
        .await;
    let request = apply_request(&mut h).await;
    assert_eq!(request["params"]["edit"], command["arguments"][1]);
    let changed = apply(SOURCE, &request["params"]["edit"]["changes"][h.url("a.php")]);
    assert_eq!(changed, "<?php new DateTime();");
    h.change("a.php", &changed, 2).await;
    h.client.respond(request["id"].clone(), json!({ "applied": true })).await;
    let response = h.client.await_response(id, 10).await;
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"], Value::Null);
    assert!(actions(&mut h, &changed, "DateTime", "refactor.rewrite").await.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn legacy_code_actions_report_client_refusal() {
    let mut h = Harness::start_with(
        &[("a.php", SOURCE)],
        ServerConfig::default(),
        json!({ "workspace": { "applyEdit": true } }),
    )
    .await;
    let commands = actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await;
    let id = h
        .client
        .send_request(
            "workspace/executeCommand",
            json!({
                "command": COMMAND, "arguments": commands[0]["arguments"]
            }),
        )
        .await;
    let request = apply_request(&mut h).await;
    h.client.respond(request["id"].clone(), json!({ "applied": false, "failureReason": "Document changed" })).await;
    let response = h.client.await_response(id, 10).await;
    assert_eq!(response["error"]["code"], -32803);
    assert_eq!(response["error"]["message"], "Document changed");
}

#[tokio::test]
async fn legacy_code_actions_forward_client_errors() {
    let mut h = Harness::start_with(
        &[("a.php", SOURCE)],
        ServerConfig::default(),
        json!({ "workspace": { "applyEdit": true } }),
    )
    .await;
    let commands = actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await;
    let id = h
        .client
        .send_request("workspace/executeCommand", json!({ "command": COMMAND, "arguments": commands[0]["arguments"] }))
        .await;
    let request = apply_request(&mut h).await;
    let error = json!({ "code": -32603, "message": "Edit failed" });
    h.client.respond_error(request["id"].clone(), error.clone()).await;
    assert_eq!(h.client.await_response(id, 10).await["error"], error);
}

#[tokio::test]
async fn code_action_commands_reject_bad_arguments_and_unknown_commands() {
    let mut h = Harness::start_with(
        &[("a.php", SOURCE)],
        ServerConfig::default(),
        json!({ "workspace": { "applyEdit": true } }),
    )
    .await;
    let commands = actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await;
    let snapshots = &commands[0]["arguments"][2];
    let uri = h.url("a.php");
    for params in [
        json!({ "command": "mago.unknown", "arguments": [uri, edit(&uri)] }),
        json!({ "command": COMMAND }),
        json!({ "command": COMMAND, "arguments": [uri] }),
        json!({ "command": COMMAND, "arguments": [uri, edit(&uri)] }),
        json!({ "command": COMMAND, "arguments": [false, edit(&uri)] }),
        json!({ "command": COMMAND, "arguments": [uri, { "changes": {} }, snapshots] }),
        json!({ "command": COMMAND, "arguments": [uri, { "documentChanges": [] }, snapshots] }),
        json!({ "command": COMMAND, "arguments": [uri, { "changes": edit(&uri)["changes"], "changeAnnotations": {} }, snapshots] }),
    ] {
        let id = h.client.send_request("workspace/executeCommand", params).await;
        let response = h.client.await_response(id, 10).await;
        assert_eq!(response["error"]["code"], -32602, "{response}");
        assert!(h.client.server_requests().is_empty());
    }
}

#[tokio::test]
async fn code_action_commands_obey_client_and_provider_settings() {
    let mut generator = Harness::start_with(
        &[("a.php", SOURCE)],
        ServerConfig::default(),
        json!({ "workspace": { "applyEdit": true } }),
    )
    .await;
    let commands = actions(&mut generator, SOURCE, "\\DateTime", "refactor.rewrite").await;
    let mut modern_with_apply = client_capabilities();
    modern_with_apply["workspace"] = json!({ "applyEdit": true });
    for (capabilities, disabled) in [
        (json!({}), false),
        (client_capabilities(), false),
        (modern_with_apply, false),
        (json!({}), true),
        (client_capabilities(), true),
        (json!({ "workspace": { "applyEdit": true } }), true),
    ] {
        let mut config = ServerConfig::default();
        if disabled {
            config.configuration.language_server.disable.push(Provider::CodeAction);
        }
        let mut h = Harness::start_with(&[("a.php", SOURCE)], config, capabilities).await;
        if disabled {
            assert!(h.server_capabilities["codeActionProvider"].is_null());
        }
        assert!(h.server_capabilities["executeCommandProvider"].is_null());
        let uri = h.url("a.php");
        let id = h
            .client
            .send_request(
                "workspace/executeCommand",
                json!({ "command": COMMAND, "arguments": arguments(&commands[0], &uri, &uri) }),
            )
            .await;
        assert!(h.client.await_response(id, 10).await.get("error").is_some());
        assert!(h.client.server_requests().is_empty());
    }
}

#[tokio::test]
async fn code_action_commands_check_source_and_target_workspaces() {
    let mut h = Harness::start_with(
        &[("a.php", SOURCE)],
        ServerConfig::default(),
        json!({ "workspace": { "applyEdit": true } }),
    )
    .await;
    h.add_folder("disabled", &[("a.php", SOURCE), ("mago.toml", "[language-server]\ndisable = ['code-action']\n")])
        .await;
    let commands = actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await;
    let enabled = h.url("a.php");
    let disabled = h.url("disabled/a.php");
    let missing = h.url("missing.php");
    let definition = h.at("textDocument/definition", "a.php", 0, 12).await;
    let builtin = definition["uri"].as_str().expect("builtin definition URI");
    for (source, target) in [
        (disabled.as_str(), enabled.as_str()),
        (&enabled, &disabled),
        (&enabled, &missing),
        (&enabled, builtin),
        (builtin, &enabled),
    ] {
        let id = h
            .client
            .send_request(
                "workspace/executeCommand",
                json!({ "command": COMMAND, "arguments": arguments(&commands[0], source, target) }),
            )
            .await;
        assert!(h.client.await_response(id, 10).await.get("error").is_some());
        assert!(h.client.server_requests().is_empty());
    }
}

#[tokio::test]
async fn legacy_code_actions_reject_changed_source_before_requesting_an_edit() {
    for open in [false, true] {
        let mut h = Harness::start_with(
            &[("a.php", SOURCE)],
            ServerConfig::default(),
            json!({ "workspace": { "applyEdit": true } }),
        )
        .await;
        if open {
            h.open("a.php", SOURCE).await;
        }
        let commands = actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await;
        assert_eq!(commands.as_array().unwrap().len(), 1);
        let changed = "<?php /* edited */ new \\DateTime();";
        if open {
            h.change("a.php", changed, 2).await;
        } else {
            std::fs::write(h.workspace.join("a.php"), changed).unwrap();
            h.watched("a.php", 2).await;
        }
        let id = h
            .client
            .send_request(
                "workspace/executeCommand",
                json!({
                    "command": COMMAND, "arguments": commands[0]["arguments"]
                }),
            )
            .await;
        let response = h.client.await_response(id, 10).await;
        assert!(h.client.server_requests().is_empty(), "stale edit reached the client");
        assert_eq!(response["error"]["code"], -32801, "{response}");
    }
}

#[tokio::test]
async fn legacy_code_actions_validate_source_and_other_edit_targets() {
    for changed_file in [None, Some("a.php"), Some("b.php")] {
        let mut h = Harness::start_with(
            &[("a.php", SOURCE), ("b.php", SOURCE)],
            ServerConfig::default(),
            json!({ "workspace": { "applyEdit": true } }),
        )
        .await;
        let commands = actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await;
        let arguments = arguments(&commands[0], &h.url("a.php"), &h.url("b.php"));
        if let Some(file) = changed_file {
            std::fs::write(h.workspace.join(file), "<?php /* edited */ new \\DateTime();").unwrap();
            h.watched(file, 2).await;
        }
        let id = h
            .client
            .send_request("workspace/executeCommand", json!({ "command": COMMAND, "arguments": arguments }))
            .await;
        if changed_file.is_some() {
            let response = h.client.await_response(id, 10).await;
            assert_eq!(response["error"]["code"], -32801, "{response}");
            assert!(h.client.server_requests().is_empty(), "stale edit reached the client");
        } else {
            let request = apply_request(&mut h).await;
            assert_eq!(request["params"]["edit"], arguments[1]);
            h.client.respond(request["id"].clone(), json!({ "applied": true })).await;
            let response = h.client.await_response(id, 10).await;
            assert!(response.get("error").is_none(), "{response}");
        }
    }
}

#[tokio::test]
async fn legacy_code_actions_require_every_document_snapshot() {
    let mut h = Harness::start_with(
        &[("a.php", SOURCE), ("b.php", SOURCE)],
        ServerConfig::default(),
        json!({ "workspace": { "applyEdit": true } }),
    )
    .await;
    let commands = actions(&mut h, SOURCE, "\\DateTime", "refactor.rewrite").await;
    for missing in ["a.php", "b.php"] {
        let mut arguments = arguments(&commands[0], &h.url("a.php"), &h.url("b.php"));
        arguments[2].as_array_mut().unwrap().retain(|snapshot| snapshot["uri"] != h.url(missing));
        let id = h
            .client
            .send_request("workspace/executeCommand", json!({ "command": COMMAND, "arguments": arguments }))
            .await;
        let response = h.client.await_response(id, 10).await;
        assert_eq!(response["error"]["code"], -32602, "{response}");
        assert!(h.client.server_requests().is_empty(), "incomplete snapshots reached the client");
    }
}
