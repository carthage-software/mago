//! `get_document_links`: links from `use` statements to declarations.
//!
//! Turns each `use Foo\Bar;` (and `use function`/`use const`) into a link to
//! the file declaring that symbol. The `use` keyword triggers a forward token
//! walk; the name is resolved against the codebase.

use mago_codex::metadata::CodebaseMetadata;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_span::Span as FileSpan;
use mago_syntax::token::TokenKind;

use crate::Server;
use crate::domain::DocumentLinkItem;
use crate::domain::Range;
use crate::lookup;

#[derive(Debug, Clone, Copy)]
enum UseKind {
    Class,
    Function,
    Constant,
}

impl Server {
    /// Document links for the `use` statements in `file_id`.
    #[must_use]
    pub fn get_document_links(&self, file_id: FileId) -> Vec<DocumentLinkItem> {
        let Ok(file) = self.database().get(&file_id) else {
            return Vec::new();
        };

        let codebase = self.codebase();
        let tokens = lookup::lex(&file);
        let mut out = Vec::new();

        let mut i = 0;
        while i < tokens.len() {
            if !matches!(tokens[i].kind, TokenKind::Use) {
                i += 1;
                continue;
            }

            let mut j = i + 1;
            while j < tokens.len() && lookup::is_trivia(tokens[j].kind) {
                j += 1;
            }

            let kind = match tokens.get(j).map(|t| t.kind) {
                Some(TokenKind::Function) => {
                    j += 1;
                    UseKind::Function
                }
                Some(TokenKind::Const) => {
                    j += 1;
                    UseKind::Constant
                }
                _ => UseKind::Class,
            };

            while j < tokens.len() {
                let t = &tokens[j];
                if matches!(t.kind, TokenKind::Semicolon | TokenKind::LeftBrace) {
                    break;
                }

                if matches!(t.kind, TokenKind::Comma) {
                    j += 1;
                    continue;
                }

                if matches!(t.kind, TokenKind::As) {
                    while j < tokens.len()
                        && !matches!(tokens[j].kind, TokenKind::Comma | TokenKind::Semicolon | TokenKind::LeftBrace)
                    {
                        j += 1;
                    }

                    continue;
                }

                if matches!(
                    t.kind,
                    TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier
                ) {
                    let name = mago_bytes::trim_start_byte(t.value, b'\\');
                    if let Some(span) = resolve(codebase, kind, name) {
                        let start = t.start.offset;
                        let end = start + t.value.len() as u32;
                        out.push(DocumentLinkItem {
                            range: Range::new(start, end),
                            target: span.file_id,
                            tooltip: String::from_utf8_lossy(name).into_owned(),
                        });
                    }
                }

                j += 1;
            }

            i = j + 1;
        }

        out
    }
}

fn resolve(codebase: &CodebaseMetadata, kind: UseKind, name: &[u8]) -> Option<FileSpan> {
    match kind {
        UseKind::Class => codebase.get_class_like(name).map(|m| m.name_span.unwrap_or(m.span)),
        UseKind::Function => codebase.get_function(name).map(|m| m.name_span.unwrap_or(m.span)),
        UseKind::Constant => codebase.get_constant(name).map(|m| m.span),
    }
}
