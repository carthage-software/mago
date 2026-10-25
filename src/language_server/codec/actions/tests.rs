use mago_database::DatabaseConfiguration;
use mago_database::file::{FileId, FileType};
use mago_server::{CodeActionKind as ActionKind, Range, TextReplacement};

use crate::language_server::uri::UriFilePath;

use super::*;

#[test]
fn unresolved_edit_targets_omit_the_whole_action() {
    let root = tempfile::tempdir().unwrap();
    let mut database = Database::new(DatabaseConfiguration::new(root.path(), vec![], vec![], vec![], vec![]));
    let source = MagoFile::new(
        b"source.php".as_slice().into(),
        FileType::Host,
        Some(root.path().join("source.php")),
        b"<?php".as_slice().into(),
    );
    let source_id = source.id;
    database.add(source);
    let pathless = MagoFile::ephemeral(b"pathless.php".as_slice().into(), b"<?php".as_slice().into());
    let pathless_id = pathless.id;
    database.add(pathless);
    let source = database.get(&source_id).unwrap();
    let document_uri = |file: &MagoFile| file.path.as_ref().and_then(Uri::from_file_path);
    let uri = document_uri(&source).unwrap();
    for target in [FileId::new(b"missing.php"), pathless_id] {
        for literals in [false, true] {
            let replacement = TextReplacement { range: Range::new(0, 0), new_text: " ".into() };
            let action = CodeActionItem {
                title: "Change both files".into(),
                kind: ActionKind::QuickFix,
                edits: HashMap::from_iter([(source_id, vec![replacement.clone()]), (target, vec![replacement])]),
                diagnostic: None,
            };
            assert!(
                code_actions(
                    &database,
                    document_uri,
                    vec![action],
                    CodeActionSource { uri: &uri, file: &source, diagnostics: &[], markdown_diagnostics: false },
                    literals,
                    false,
                )
                .is_empty()
            );
        }
    }
}

#[test]
fn action_diagnostics_reuse_matching_previews_and_reject_unrelated_context() {
    let root = tempfile::tempdir().unwrap();
    let mut database = Database::new(DatabaseConfiguration::new(root.path(), vec![], vec![], vec![], vec![]));
    let file = database.add(MagoFile::new(
        b"source.php".as_slice().into(),
        FileType::Host,
        Some(root.path().join("source.php")),
        b"<?php (integer) 1;".as_slice().into(),
    ));
    let data = DiagnosticData {
        file,
        range: Range::new(6, 15),
        severity: Severity::Warning,
        code: Some("deprecated-cast".into()),
        message: "Deprecated cast.\n = Help: Use int.".into(),
    };
    let snapshot = database.get(&file).unwrap();
    let uri = Uri::from_file_path(root.path().join("source.php")).unwrap();
    let source = CodeActionSource { uri: &uri, file: &snapshot, diagnostics: &[], markdown_diagnostics: false };
    let canonical = diagnostic(&database, data.clone(), &source).unwrap();
    let mut published = canonical.clone();
    published.code_description =
        Some(tower_lsp_server::gen_lsp_types::CodeDescription { href: "https://example.com/cast".parse().unwrap() });
    published.tags = Some(vec![tower_lsp_server::gen_lsp_types::DiagnosticTag::Deprecated]);
    published.data = Some(serde_json::json!({"issue": "cast"}));
    let mut fallback = published.clone();
    fallback.code_description = None;
    fallback.message = Message::String(format!("{}\n = See: https://example.com/cast", data.message));
    let mut markup = published.clone();
    markup.message = Message::MarkupContent(tower_lsp_server::gen_lsp_types::MarkupContent {
        kind: tower_lsp_server::gen_lsp_types::MarkupKind::Markdown,
        value: "Deprecated cast\\.\n\n\\= Help\\: Use int\\.".into(),
    });
    let mut markup_fallback = markup.clone();
    markup_fallback.code_description = None;
    if let Message::MarkupContent(content) = &mut markup_fallback.message {
        content.value.push_str("\\\n\\= See\\: <https://example.com/cast>");
    }
    for preview in [published.clone(), fallback, markup, markup_fallback] {
        let context =
            CodeActionSource { diagnostics: std::slice::from_ref(&preview), markdown_diagnostics: true, ..source };
        assert_eq!(diagnostic(&database, data.clone(), &context), Some(preview.clone()));
        if matches!(&preview.message, Message::MarkupContent(_)) {
            assert_eq!(
                diagnostic(&database, data.clone(), &CodeActionSource { markdown_diagnostics: false, ..context }),
                Some(canonical.clone()),
            );
        }
    }
    for field in ["source", "code", "range", "severity", "message", "message-kind"] {
        let mut unrelated = published.clone();
        match field {
            "source" => unrelated.source = Some("other".into()),
            "code" => unrelated.code = Some(Code::String("other-code".into())),
            "range" => unrelated.range.start.character += 1,
            "severity" => unrelated.severity = Some(DiagnosticSeverity::Error),
            "message" => unrelated.message = Message::String(format!("{} Stale detail.", data.message)),
            "message-kind" => {
                unrelated.message = Message::MarkupContent(tower_lsp_server::gen_lsp_types::MarkupContent {
                    kind: tower_lsp_server::gen_lsp_types::MarkupKind::PlainText,
                    value: data.message.clone(),
                });
            }
            _ => unreachable!(),
        }
        let context =
            CodeActionSource { diagnostics: std::slice::from_ref(&unrelated), markdown_diagnostics: true, ..source };
        assert_eq!(diagnostic(&database, data.clone(), &context), Some(canonical.clone()), "{field}");
    }
    let other = MagoFile::ephemeral(b"other.php".as_slice().into(), b"<?php".as_slice().into());
    let context = CodeActionSource { file: &other, diagnostics: std::slice::from_ref(&published), ..source };
    assert_eq!(diagnostic(&database, data, &context), Some(canonical));
}
