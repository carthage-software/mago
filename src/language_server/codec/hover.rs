//! Render hover declarations and documentation in the client format.

use tower_lsp_server::gen_lsp_types::{Contents, Hover, MarkupContent, MarkupKind};

use mago_database::file::File as MagoFile;
use mago_server::{HoverEntry, HoverInfo};

use crate::language_server::position::range_at_offsets;

/// The extension may register a PHP grammar that accepts tag-free snippets.
#[derive(Clone, Copy, Default)]
pub enum HoverCodeLanguage {
    #[default]
    Php,
    PhpOnly,
}

impl HoverCodeLanguage {
    pub fn from_options(options: Option<&serde_json::Value>) -> Self {
        match options.and_then(|options| options.get("hoverCodeLanguage")).and_then(serde_json::Value::as_str) {
            Some("php_only") => Self::PhpOnly,
            _ => Self::Php,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Php => "php",
            Self::PhpOnly => "php_only",
        }
    }
}

/// Convert the selected token's byte range and descriptions to an LSP hover.
pub fn hover(file: &MagoFile, info: HoverInfo, markdown: bool, language: HoverCodeLanguage) -> Hover {
    let entries: Vec<_> = info.entries.into_iter().map(|entry| render_entry(entry, markdown, language)).collect();
    Hover {
        contents: Contents::MarkupContent(MarkupContent {
            kind: if markdown { MarkupKind::Markdown } else { MarkupKind::PlainText },
            value: entries.join(if markdown { "\n\n---\n\n" } else { "\n\n" }),
        }),
        range: Some(range_at_offsets(file, info.range.start, info.range.end)),
    }
}

pub(super) fn render_entry(entry: HoverEntry, markdown: bool, language: HoverCodeLanguage) -> String {
    let mut parts = Vec::new();
    if let Some(scope) = entry.scope {
        parts.push(if markdown { super::documentation::code_span(&scope) } else { scope });
    }
    parts.push(if markdown { declaration(&entry.declaration, language) } else { entry.declaration });
    if let Some(documentation) = entry.documentation {
        let documentation = super::documentation::render(&documentation, markdown);
        parts.push(if markdown { format!("---\n\n{documentation}") } else { documentation });
    }
    parts.join("\n\n")
}

fn declaration(source: &str, language: HoverCodeLanguage) -> String {
    let longest = source.split(|character| character != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(3.max(longest + 1));
    format!("{fence}{}\n{source}\n{fence}", language.name())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn snippet_language_is_explicit_and_cannot_inject_markdown() {
        for (options, expected) in [
            (None, "php"),
            (Some(json!({"hoverCodeLanguage": "php_only"})), "php_only"),
            (Some(json!({"hoverCodeLanguage": "php\n```\ntext"})), "php"),
        ] {
            let language = HoverCodeLanguage::from_options(options.as_ref());
            assert_eq!(
                declaration("interface DefaultInterface", language),
                format!("```{expected}\ninterface DefaultInterface\n```")
            );
        }
    }

    #[test]
    fn symbols_show_their_namespace_and_members_show_their_class() {
        for (entry, expected_plain, expected_markdown) in [
            (HoverEntry::for_symbol(b"Item", "class Item".into()), "class Item", "```php\nclass Item\n```"),
            (
                HoverEntry::for_symbol(b"App\\Domain\\Item", "class Item".into()),
                "App\\Domain\n\nclass Item",
                "`App\\Domain`\n\n```php\nclass Item\n```",
            ),
            (
                HoverEntry::in_class(b"App\\Domain\\Item", "public int $id".into()),
                "App\\Domain\\Item\n\npublic int $id",
                "`App\\Domain\\Item`\n\n```php\npublic int $id\n```",
            ),
        ] {
            assert_eq!(render_entry(entry.clone(), false, HoverCodeLanguage::Php), expected_plain);
            assert_eq!(render_entry(entry, true, HoverCodeLanguage::Php), expected_markdown);
        }
    }

    #[test]
    fn anonymous_context_uses_a_safe_code_span() {
        let context = "{anonymous-class:demo`case.php:2:1}";
        let entry = HoverEntry::in_class(context.as_bytes(), "public function run(): void".into());
        assert_eq!(
            render_entry(entry.clone(), true, HoverCodeLanguage::Php),
            format!("``{context}``\n\n```php\npublic function run(): void\n```"),
        );
        assert_eq!(
            render_entry(entry, false, HoverCodeLanguage::Php),
            format!("{context}\n\npublic function run(): void"),
        );
    }
}
