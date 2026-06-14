//! `get_inlay_hints`: parameter-name hints for positional arguments.
//!
//! For `take_user($name, 42)` we emit `name:`/`id:` before each positional
//! argument, pairing token-level argument positions with the callee's parameter
//! names from the codebase. Named arguments are skipped; method/static calls are
//! deferred until analyzer artifacts are wired in.

use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::Server;
use crate::domain::HintKind;
use crate::domain::InlayHintItem;
use crate::lookup;

impl Server {
    /// Parameter-name inlay hints for the calls within `[range_start, range_end]`
    /// of `file_id`.
    pub fn get_inlay_hints(&mut self, file_id: FileId, range_start: u32, range_end: u32) -> Vec<InlayHintItem> {
        let Ok(file) = self.database().get(&file_id) else {
            return Vec::new();
        };

        let Some(analysis) = self.file_analysis_for(file_id) else {
            return Vec::new();
        };

        let resolved = analysis.resolved();
        let codebase = self.codebase();
        let tokens = lookup::lex(&file);
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
                    emit_argument_hints(&tokens, open_paren_idx, range_start, range_end, meta, &mut hints);
                }

                prev_significant = Some(token.kind);
            }

            i += 1;
        }

        hints
    }
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
    range_start: u32,
    range_end: u32,
    meta: &FunctionLikeMetadata,
    out: &mut Vec<InlayHintItem>,
) {
    let mut depth = 1i32;
    let mut arg_index: usize = 0;
    let mut first_token_of_argument = true;

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
        let label =
            format!("{}:", mago_bytes::BytesDisplay(mago_bytes::trim_start_byte(param.name.0.as_bytes(), b'$')));

        out.push(InlayHintItem { offset: t.start.offset, label, kind: HintKind::Parameter });
    }
}
