//! `textDocument/inlayHint`.
//!
//! Annotates positional argument lists with parameter names. For
//! `take_user($name, 42)` we emit `name:` before `$name` and `id:` before
//! `42` (when the function metadata names the param `id`).
//!
//! This is a token-level scan; for each call expression we resolve the
//! callee via the codebase metadata and pair token-level argument
//! positions with `FunctionLikeParameterMetadata.name`. Named arguments
//! (`name: $value`) are skipped; already explicit. Method calls and
//! static calls are deferred until we wire analyzer artifacts in.

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_database::file::File as MagoFile;
use mago_names::ResolvedNames;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;
use tower_lsp::lsp_types::InlayHint;
use tower_lsp::lsp_types::InlayHintKind;
use tower_lsp::lsp_types::InlayHintLabel;

use crate::language_server::capabilities::lookup;
use crate::language_server::position::position_at_offset;

pub fn compute(
    codebase: &CodebaseMetadata,
    resolved: &ResolvedNames<'_>,
    file: &MagoFile,
    range_start: u32,
    range_end: u32,
) -> Vec<InlayHint> {
    let tokens = lookup::lex(file);
    let mut hints = Vec::new();
    let mut prev_significant: Option<TokenKind> = None;

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        if token.start.offset > range_end {
            break;
        }

        if !lookup::is_trivia(token.kind) {
            let is_callable = matches!(
                token.kind,
                TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier
            );
            if is_callable
                && !is_declaration_context(prev_significant)
                && let Some(open_paren_idx) = next_paren(&tokens, i + 1)
                && let Some(fqcn) = resolved.resolve(&token.start)
                && let Some(meta) = codebase.get_function(fqcn)
            {
                emit_argument_hints(&tokens, open_paren_idx, file, range_start, range_end, meta, &mut hints);
            }
            prev_significant = Some(token.kind);
        }

        i += 1;
    }

    hints
}

fn is_declaration_context(prev: Option<TokenKind>) -> bool {
    matches!(
        prev,
        Some(
            TokenKind::Function
                | TokenKind::Class
                | TokenKind::Interface
                | TokenKind::Trait
                | TokenKind::Enum
                | TokenKind::Const
                | TokenKind::New
                | TokenKind::Use
                | TokenKind::Namespace
                | TokenKind::MinusGreaterThan
                | TokenKind::QuestionMinusGreaterThan
                | TokenKind::ColonColon
        )
    )
}

fn next_paren(tokens: &[Token<'_>], start: usize) -> Option<usize> {
    for (k, t) in tokens.iter().enumerate().skip(start) {
        if lookup::is_trivia(t.kind) {
            continue;
        }
        if matches!(t.kind, TokenKind::LeftParenthesis) {
            return Some(k);
        }
        return None;
    }
    None
}

fn emit_argument_hints(
    tokens: &[Token<'_>],
    open_paren_idx: usize,
    file: &MagoFile,
    range_start: u32,
    range_end: u32,
    meta: &FunctionLikeMetadata,
    out: &mut Vec<InlayHint>,
) {
    let mut depth = 1i32;
    let mut arg_index: usize = 0;
    let mut first_token_of_argument: bool = true;

    for k in (open_paren_idx + 1)..tokens.len() {
        let t = &tokens[k];
        if lookup::is_trivia(t.kind) {
            continue;
        }

        match t.kind {
            TokenKind::LeftParenthesis => depth += 1,
            TokenKind::RightParenthesis => {
                depth -= 1;
                if depth == 0 {
                    return;
                }
            }
            TokenKind::Comma if depth == 1 => {
                arg_index += 1;
                first_token_of_argument = true;
                continue;
            }
            _ => {}
        }

        if depth != 1 || !first_token_of_argument {
            continue;
        }

        if matches!(t.kind, TokenKind::Identifier)
            && tokens.get(k + 1).is_some_and(|next| matches!(next.kind, TokenKind::Colon))
        {
            first_token_of_argument = false;
            continue;
        }

        first_token_of_argument = false;

        if t.start.offset < range_start || t.start.offset > range_end {
            continue;
        }

        let Some(param) = meta.parameters.get(arg_index) else { continue };
        let label = format!("{}:", param.name.0.as_str().trim_start_matches('$'));

        out.push(InlayHint {
            position: position_at_offset(file, t.start.offset),
            label: InlayHintLabel::String(label),
            kind: Some(InlayHintKind::PARAMETER),
            text_edits: None,
            tooltip: None,
            padding_left: None,
            padding_right: Some(true),
            data: None,
        });
    }
}
