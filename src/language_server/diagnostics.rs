//! Issue → LSP `Diagnostic` conversion and publish helpers.

use crate::language_server::uri::UriFilePath;
use foldhash::HashMap;
use tower_lsp_server::gen_lsp_types::Code;
use tower_lsp_server::gen_lsp_types::CodeDescription;
use tower_lsp_server::gen_lsp_types::Diagnostic;
use tower_lsp_server::gen_lsp_types::DiagnosticClientCapabilities;
use tower_lsp_server::gen_lsp_types::DiagnosticRelatedInformation;
use tower_lsp_server::gen_lsp_types::DiagnosticSeverity;
use tower_lsp_server::gen_lsp_types::DiagnosticTag;
use tower_lsp_server::gen_lsp_types::Location;
use tower_lsp_server::gen_lsp_types::MarkupContent;
use tower_lsp_server::gen_lsp_types::MarkupKind;
use tower_lsp_server::gen_lsp_types::Message;
use tower_lsp_server::gen_lsp_types::PublishDiagnosticsClientCapabilities;
use tower_lsp_server::gen_lsp_types::Uri;

use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_server::diagnostics::diagnostic_message;
use mago_server::diagnostics::diagnostic_tags;
use mago_server::diagnostics::primary_annotation;
use mago_server::diagnostics::related_annotations;

use crate::language_server::position::range_at_offsets;

mod markdown;
#[cfg(test)]
mod tests;

const LINK: &str = "\n = See: ";
const MARKDOWN_LINK: &str = "\\= See\\: ";

/// Optional diagnostic fields accepted by the client.
#[derive(Clone, Copy, Default)]
pub struct DiagnosticPreferences {
    related_information: bool,
    code_description: bool,
    unnecessary: bool,
    deprecated: bool,
    markdown: bool,
}

impl DiagnosticPreferences {
    pub(super) fn supports_markdown(self) -> bool {
        self.markdown
    }

    pub fn from_capabilities(
        capabilities: Option<&PublishDiagnosticsClientCapabilities>,
        diagnostic: Option<&DiagnosticClientCapabilities>,
    ) -> Self {
        let capabilities = capabilities.map(|capabilities| &capabilities.diagnostics_capabilities);
        let accepts = |tag| {
            capabilities
                .and_then(|capabilities| capabilities.tag_support.as_ref())
                .is_some_and(|support| support.value_set.contains(&tag))
        };
        Self {
            related_information: capabilities
                .and_then(|capabilities| capabilities.related_information)
                .unwrap_or(false),
            code_description: capabilities
                .and_then(|capabilities| capabilities.code_description_support)
                .unwrap_or(false),
            unnecessary: accepts(DiagnosticTag::Unnecessary),
            deprecated: accepts(DiagnosticTag::Deprecated),
            markdown: diagnostic.and_then(|capabilities| capabilities.markup_message_support).unwrap_or(false),
        }
    }

    pub fn apply(self, diagnostic: &mut Diagnostic) {
        if !self.related_information {
            diagnostic.related_information = None;
        }
        if self.markdown
            && let Message::String(message) = &diagnostic.message
        {
            diagnostic.message = MarkupContent { kind: MarkupKind::Markdown, value: markdown::render(message) }.into();
        }
        if !self.code_description
            && let Some(description) = diagnostic.code_description.take()
        {
            append_link(&mut diagnostic.message, &description);
        }
        if let Some(tags) = diagnostic.tags.as_mut() {
            tags.retain(|tag| {
                (*tag == DiagnosticTag::Unnecessary && self.unnecessary)
                    || (*tag == DiagnosticTag::Deprecated && self.deprecated)
            });
            if tags.is_empty() {
                diagnostic.tags = None;
            }
        }
    }
}

/// Convert borrowed issues into diagnostics, grouped by document URI.
/// Issues without a source path cannot be shown by the client.
pub fn build_diagnostics<'a>(
    database: &Database<'_>,
    issues: impl IntoIterator<Item = &'a Issue>,
) -> HashMap<Uri, Vec<Diagnostic>> {
    let mut by_file = HashMap::default();
    let mut urls = HashMap::default();
    for issue in issues {
        push_issue(database, &mut by_file, &mut urls, issue);
    }
    by_file.into_iter().filter_map(|(file, diagnostics)| Some((urls.remove(&file)??, diagnostics))).collect()
}

fn push_issue(
    database: &Database<'_>,
    by_file: &mut HashMap<FileId, Vec<Diagnostic>>,
    urls: &mut HashMap<FileId, Option<Uri>>,
    issue: &Issue,
) {
    let Some(annotation) = primary_annotation(issue) else {
        return;
    };

    let Ok(file) = database.get_ref(&annotation.span.file_id) else {
        return;
    };

    if urls.entry(file.id).or_insert_with(|| file.path.as_ref().and_then(Uri::from_file_path)).is_none() {
        return;
    }

    let range = range_at_offsets(file, annotation.span.start.offset, annotation.span.end.offset);

    let related = related_information(database, urls, issue, annotation);
    let mut message = Message::String(diagnostic_message(issue, annotation));
    let mut code_description = issue.link.as_deref().and_then(code_description);
    if issue.code.is_none()
        && let Some(description) = code_description.take()
    {
        append_link(&mut message, &description);
    }
    let flags = diagnostic_tags(issue);
    let tags: Vec<_> = [(flags.unnecessary, DiagnosticTag::Unnecessary), (flags.deprecated, DiagnosticTag::Deprecated)]
        .into_iter()
        .filter_map(|(enabled, tag)| enabled.then_some(tag))
        .collect();

    by_file.entry(file.id).or_default().push(Diagnostic {
        range,
        severity: Some(level_to_severity(issue.level)),
        code: issue.code.clone().map(Code::String),
        code_description,
        source: Some("mago".into()),
        message,
        related_information: if related.is_empty() { None } else { Some(related) },
        tags: if tags.is_empty() { None } else { Some(tags) },
        ..Diagnostic::default()
    });
}

fn related_information(
    database: &Database<'_>,
    urls: &mut HashMap<FileId, Option<Uri>>,
    issue: &Issue,
    primary: &Annotation,
) -> Vec<DiagnosticRelatedInformation> {
    let mut related = Vec::new();

    for annotation in related_annotations(issue, primary) {
        let Ok(file) = database.get_ref(&annotation.span.file_id) else {
            continue;
        };
        let Some(url) = urls.entry(file.id).or_insert_with(|| file.path.as_ref().and_then(Uri::from_file_path)) else {
            continue;
        };

        let range = range_at_offsets(file, annotation.span.start.offset, annotation.span.end.offset);
        let message = annotation.message.as_deref().unwrap_or_default().trim().to_owned();

        related.push(DiagnosticRelatedInformation { location: Location { uri: url.clone(), range }, message });
    }

    related
}

fn code_description(link: &str) -> Option<CodeDescription> {
    let href: Uri = link.parse().ok()?;
    let scheme = href.scheme().as_str();
    if (scheme.eq_ignore_ascii_case("https") || scheme.eq_ignore_ascii_case("http"))
        && href.authority().is_some_and(|authority| !authority.host().is_empty())
    {
        Some(CodeDescription { href })
    } else {
        None
    }
}

fn append_link(message: &mut Message, description: &CodeDescription) {
    let link = description.href.as_str();
    match message {
        Message::String(message) => {
            if !message.contains(link) {
                message.push_str(LINK);
                message.push_str(link);
            }
        }
        Message::MarkupContent(content) => {
            if !content.value.contains(link) {
                if content.kind == MarkupKind::Markdown {
                    content.value.push_str(markdown::detail_separator(&content.value));
                    content.value.push_str(MARKDOWN_LINK);
                    content.value.push('<');
                    content.value.push_str(link);
                    content.value.push('>');
                } else {
                    content.value.push_str(LINK);
                    content.value.push_str(link);
                }
            }
        }
    }
}

/// Match issue text after the client's optional documentation-link fallback.
pub(super) fn published_message_matches(canonical: &str, published: &Message) -> bool {
    let (expected, published, markdown) = match published {
        Message::String(published) => (std::borrow::Cow::Borrowed(canonical), published.as_str(), false),
        Message::MarkupContent(content) if content.kind == MarkupKind::Markdown => {
            (std::borrow::Cow::Owned(markdown::render(canonical)), content.value.as_str(), true)
        }
        Message::MarkupContent(_) => return false,
    };
    expected == published
        || published
            .strip_prefix(expected.as_ref())
            .and_then(|suffix| {
                if markdown {
                    suffix
                        .strip_prefix(markdown::detail_separator(&expected))?
                        .strip_prefix(MARKDOWN_LINK)?
                        .strip_prefix('<')?
                        .strip_suffix('>')
                } else {
                    suffix.strip_prefix(LINK)
                }
            })
            .is_some_and(|link| code_description(link).is_some())
}

#[must_use]
pub fn level_to_severity(level: Level) -> DiagnosticSeverity {
    match level {
        Level::Error => DiagnosticSeverity::Error,
        Level::Warning => DiagnosticSeverity::Warning,
        Level::Help => DiagnosticSeverity::Hint,
        Level::Note => DiagnosticSeverity::Information,
    }
}
