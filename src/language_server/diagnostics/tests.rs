use crate::language_server::uri::UriFilePath;
use mago_database::DatabaseConfiguration;
use mago_database::file::{File, FileType};
use mago_span::{Position, Span};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde_json::json;

use super::*;

#[test]
fn diagnostics_keep_links_tags_and_each_distinct_related_location() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::new(DatabaseConfiguration::new(directory.path(), vec![], vec![], vec![], vec![]));
    let source = "<?php $value = '💡';\nfoo();\nbar();\n";
    let path = directory.path().join("main.php");
    let file = database.add(File::new(
        b"main.php".as_slice().into(),
        FileType::Host,
        Some(path.clone()),
        source.as_bytes().into(),
    ));

    let annotation = |kind, name: &str, message| {
        let start = source.find(name).unwrap() as u32;
        Annotation::new(kind, Span::new(file, Position::new(start), Position::new(start + name.len() as u32)))
            .with_message(message)
    };

    let primary = annotation(mago_reporting::AnnotationKind::Primary, "foo", "The first call.");
    let late = annotation(mago_reporting::AnnotationKind::Primary, "bar", "The other call.");
    let duplicate = annotation(mago_reporting::AnnotationKind::Secondary, "bar", " The other call. ");
    let blank = annotation(mago_reporting::AnnotationKind::Secondary, "$value", " ");
    let issue = Issue::warning("Deprecated call.")
        .with_code("deprecated-function")
        .with_link("https://wiki.php.net/rfc/deprecations_php_8_5")
        .with_annotations([late, blank, primary, duplicate]);
    let uri = Uri::from_file_path(path).unwrap();
    let diagnostics = build_diagnostics(&database, [&issue]);
    let diagnostic = &diagnostics[&uri][0];
    assert_eq!(diagnostic.range.start, tower_lsp_server::gen_lsp_types::Position { line: 1, character: 0 });
    assert_eq!(diagnostic.range.end, tower_lsp_server::gen_lsp_types::Position { line: 1, character: 3 });
    assert_eq!(diagnostic.tags, Some(vec![DiagnosticTag::Deprecated]));
    assert_eq!(diagnostic.code_description.as_ref().unwrap().href.as_str(), issue.link.as_deref().unwrap());
    let related = diagnostic.related_information.as_ref().unwrap();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].location.uri, uri);
    assert_eq!(related[0].location.range.start, tower_lsp_server::gen_lsp_types::Position { line: 2, character: 0 });
    assert_eq!(related[0].message, "The other call.");
    assert_eq!(diagnostic.message, Message::String("Deprecated call.\n = The first call.".into()));
    let mut issue = issue;
    issue.code = None;
    let diagnostics = build_diagnostics(&database, [&issue]);
    let diagnostic = &diagnostics[&uri][0];
    assert!(diagnostic.code_description.is_none());
    assert_eq!(
        diagnostic.message,
        Message::String(
            "Deprecated call.\n = The first call.\n = See: https://wiki.php.net/rfc/deprecations_php_8_5".into()
        )
    );
}

#[test]
fn documentation_links_require_absolute_http_or_https_urls() {
    for link in ["https://wiki.php.net/rfc/deprecations_php_8_5", "HTTP://example.com/reference#code"] {
        assert_eq!(code_description(link).unwrap().href.as_str(), link);
    }

    for link in [
        "",
        "#code",
        "/reference",
        "https:/reference",
        "https:///reference",
        "javascript:alert(1)",
        "file:///reference",
        "https://example.com/has spaces",
    ] {
        assert!(code_description(link).is_none(), "{link}");
    }
}

#[test]
fn published_message_matching_accepts_only_the_link_fallback() {
    for canonical in ["The title.\n = Help: The next step.", "The title."] {
        assert!(published_message_matches(canonical, &Message::String(canonical.into())));
        for suffix in ["\n = See: https://example.com/reference", "\n = See: HTTP://example.com/reference#code"] {
            assert!(published_message_matches(canonical, &Message::String(format!("{canonical}{suffix}"))));
        }

        for suffix in [
            " Different detail.",
            "\n = See: javascript:alert(1)",
            "\n = See: /local",
            "\n = See: https://example.com/reference\n = Extra prose",
            "\n\n = See: https://example.com/reference",
            "\n\nMore information: https://example.com/reference",
            "\n  see: https://example.com/reference",
        ] {
            assert!(
                !published_message_matches(canonical, &Message::String(format!("{canonical}{suffix}"))),
                "{suffix}"
            );
        }
    }
}

#[test]
fn diagnostic_fields_follow_the_client_capabilities() {
    let original = Diagnostic {
        message: "The title.\n = The detail.\n = The context.\n = Help: The next step.".into(),
        code_description: code_description("https://wiki.php.net/rfc/deprecations_php_8_5"),
        tags: Some(vec![DiagnosticTag::Unnecessary, DiagnosticTag::Deprecated]),
        related_information: Some(vec![DiagnosticRelatedInformation {
            location: Location {
                uri: "file:///test.php".parse().unwrap(),
                range: tower_lsp_server::gen_lsp_types::Range::default(),
            },
            message: "Related cause.".into(),
        }]),
        ..Diagnostic::default()
    };
    let Message::String(original_message) = &original.message else { unreachable!() };
    for (capabilities, related, description, tags) in [
        (None, false, false, None),
        (Some(json!({})), false, false, None),
        (
            Some(
                json!({"relatedInformation": true, "codeDescriptionSupport": true, "tagSupport": {"valueSet": [1, 2]}}),
            ),
            true,
            true,
            original.tags.clone(),
        ),
        (
            Some(
                json!({"relatedInformation": false, "codeDescriptionSupport": false, "tagSupport": {"valueSet": [2]}}),
            ),
            false,
            false,
            Some(vec![DiagnosticTag::Deprecated]),
        ),
    ] {
        let capabilities =
            capabilities.map(|value| serde_json::from_value::<PublishDiagnosticsClientCapabilities>(value).unwrap());
        let mut diagnostic = original.clone();
        DiagnosticPreferences::from_capabilities(capabilities.as_ref(), None).apply(&mut diagnostic);
        assert_eq!(diagnostic.related_information.is_some(), related);
        assert_eq!(diagnostic.code_description.is_some(), description);
        assert_eq!(diagnostic.tags, tags);
        let expected = if description {
            original.message.clone()
        } else {
            Message::String(format!("{original_message}\n = See: https://wiki.php.net/rfc/deprecations_php_8_5"))
        };
        assert_eq!(diagnostic.message, expected);
        DiagnosticPreferences::from_capabilities(capabilities.as_ref(), None).apply(&mut diagnostic);
        assert_eq!(diagnostic.message, expected);
    }
}

#[test]
fn markdown_messages_require_support_and_keep_matching_the_published_preview() {
    let canonical = "Expected `int`, found `string('value')`.\n = Help: Return an `int`.";
    for supported in [None, Some(false), Some(true)] {
        for description_supported in [false, true] {
            let capabilities = PublishDiagnosticsClientCapabilities {
                diagnostics_capabilities: tower_lsp_server::gen_lsp_types::DiagnosticsCapabilities {
                    code_description_support: Some(description_supported),
                    ..Default::default()
                },
                ..Default::default()
            };
            let diagnostic_capabilities =
                DiagnosticClientCapabilities { markup_message_support: supported, ..Default::default() };
            let preferences =
                DiagnosticPreferences::from_capabilities(Some(&capabilities), Some(&diagnostic_capabilities));
            let mut diagnostic = Diagnostic {
                message: canonical.into(),
                code_description: code_description("https://example.com/reference#code"),
                ..Default::default()
            };
            preferences.apply(&mut diagnostic);
            assert_eq!(matches!(diagnostic.message, Message::MarkupContent(_)), supported == Some(true));
            assert!(published_message_matches(canonical, &diagnostic.message));
            if supported != Some(true) {
                let expected = if description_supported {
                    canonical.to_owned()
                } else {
                    format!("{canonical}\n = See: https://example.com/reference#code")
                };
                assert_eq!(diagnostic.message, Message::String(expected));
            } else {
                let mut expected = markdown::render(canonical);
                if !description_supported {
                    expected.push_str("\\\n\\= See\\: <https://example.com/reference#code>");
                }
                assert_eq!(diagnostic.message, MarkupContent { kind: MarkupKind::Markdown, value: expected }.into());
            }
            let once = diagnostic.clone();
            preferences.apply(&mut diagnostic);
            assert_eq!(diagnostic, once);
        }
    }
    for value in [
        format!("{} Different detail.", markdown::render(canonical)),
        format!("{}\\\n\\= See\\: <javascript:alert(1)>", markdown::render(canonical)),
        format!("{}\\\n\\= See\\: <https://example.com/>\nExtra prose", markdown::render(canonical)),
        format!("{}\n\n\\= See\\: <https://example.com/>", markdown::render(canonical)),
        format!("{}\n\nMore information: <https://example.com/>", markdown::render(canonical)),
    ] {
        assert!(!published_message_matches(canonical, &MarkupContent { kind: MarkupKind::Markdown, value }.into()));
    }
    let headline = "Expected `int`.";
    let mut diagnostic = Diagnostic {
        message: headline.into(),
        code_description: code_description("https://example.com/reference"),
        ..Default::default()
    };
    DiagnosticPreferences { markdown: true, ..DiagnosticPreferences::default() }.apply(&mut diagnostic);
    assert_eq!(
        diagnostic.message,
        MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Expected `int`\\.\n\n\\= See\\: <https://example.com/reference>".into()
        }
        .into()
    );
    assert!(published_message_matches(headline, &diagnostic.message));
    let value = "Expected `int`\\.\\\n\\= See\\: <https://example.com/reference>".into();
    assert!(!published_message_matches(headline, &MarkupContent { kind: MarkupKind::Markdown, value }.into()));
}

#[test]
fn markdown_shows_multi_line_code_spans_as_blocks_and_keeps_matching() {
    let link = "https://example.com/reference";
    let primary = Annotation::primary(Span::dummy(0, 1));
    for (issue, inline_code, code_blocks) in [
        (
            Issue::error("expected `string('a\\nb')`, but found `string('a\nb')`.")
                .with_annotation(primary.clone().with_message("This has type `string('x\n\n  y')`")),
            vec!["string('a\\nb')"],
            vec!["string('a\nb')\n", "string('x\n\n  y')\n"],
        ),
        (Issue::error("found `string('a\n\nb')`").with_annotation(primary.clone()), vec![], vec!["string('a\n\nb')\n"]),
    ] {
        let canonical = diagnostic_message(&issue, primary_annotation(&issue).unwrap());
        for supports_links in [false, true] {
            let mut diagnostic = Diagnostic {
                message: canonical.as_str().into(),
                code_description: code_description(link),
                ..Default::default()
            };
            DiagnosticPreferences {
                markdown: true,
                code_description: supports_links,
                ..DiagnosticPreferences::default()
            }
            .apply(&mut diagnostic);
            assert!(published_message_matches(&canonical, &diagnostic.message));
            let Message::MarkupContent(content) = &diagnostic.message else { unreachable!() };
            let (mut inline, mut blocks, mut links) = (Vec::new(), Vec::new(), Vec::new());
            let mut block: Option<String> = None;
            for event in Parser::new(&content.value) {
                match event {
                    Event::Start(Tag::CodeBlock(_)) => block = Some(String::new()),
                    Event::End(TagEnd::CodeBlock) => blocks.push(block.take().unwrap()),
                    Event::Text(text) => {
                        if let Some(block) = block.as_mut() {
                            block.push_str(&text);
                        }
                    }
                    Event::Code(code) => inline.push(code.into_string()),
                    Event::Start(Tag::Link { dest_url, .. }) => links.push(dest_url.into_string()),
                    _ => {}
                }
            }
            assert_eq!(inline, inline_code, "{content:?}");
            assert_eq!(blocks, code_blocks, "{content:?}");
            assert_eq!(links, if supports_links { vec![] } else { vec![link] }, "{content:?}");
        }
        let rendered = markdown::render(&canonical);
        let separator = markdown::detail_separator(&rendered);
        for wrong in ["\\\n", "\n", "\n\n"].into_iter().filter(|wrong| *wrong != separator) {
            let value = format!("{rendered}{wrong}\\= See\\: <{link}>");
            assert!(!published_message_matches(
                &canonical,
                &MarkupContent { kind: MarkupKind::Markdown, value }.into()
            ));
        }
    }
}
