//! `textDocument/documentLink`.
//!
//! Turns each `use Foo\Bar;` (and `use function`, `use const`) into a
//! clickable link to the file declaring the symbol. The `use` keyword's
//! position is the trigger; we walk forward through the name token(s) and
//! resolve the FQCN via [`mago_codex::metadata::CodebaseMetadata`].

use mago_codex::metadata::CodebaseMetadata;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_span::Span;
use mago_syntax::token::TokenKind;
use tower_lsp::lsp_types::DocumentLink;
use tower_lsp::lsp_types::Url;

use crate::language_server::capabilities::lookup;
use crate::language_server::position::range_at_offsets;

#[derive(Debug, Clone, Copy)]
enum UseKind {
    Class,
    Function,
    Constant,
}

pub fn compute(database: &Database<'_>, codebase: &CodebaseMetadata, file: &MagoFile) -> Vec<DocumentLink> {
    let tokens = lookup::lex(file);
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
                let name = t.value.trim_start_matches('\\');
                if let Some(span) = resolve(codebase, kind, name)
                    && let Some(uri) = file_url(database, span)
                {
                    let start = t.start.offset;
                    let end = start + t.value.len() as u32;
                    out.push(DocumentLink {
                        range: range_at_offsets(file, start, end),
                        target: Some(uri),
                        tooltip: Some(name.to_string()),
                        data: None,
                    });
                }
            }

            j += 1;
        }

        i = j + 1;
    }

    out
}

fn resolve(codebase: &CodebaseMetadata, kind: UseKind, name: &str) -> Option<Span> {
    match kind {
        UseKind::Class => codebase.get_class_like(name).map(|m| m.name_span.unwrap_or(m.span)),
        UseKind::Function => codebase.get_function(name).map(|m| m.name_span.unwrap_or(m.span)),
        UseKind::Constant => codebase.get_constant(name).map(|m| m.span),
    }
}

fn file_url(database: &Database<'_>, span: Span) -> Option<Url> {
    let file = database.get(&span.file_id).ok()?;
    let path = file.path.as_ref()?;
    Url::from_file_path(path).ok()
}
