//! `textDocument/completion`.
//!
//! Five contexts:
//! - **Variable** (`$<prefix>`); variables in the enclosing function scope.
//! - **Instance member** (`$obj-><prefix>`); receiver type comes from
//!   analyzer artifacts; methods, properties and constants of the class
//!   hierarchy are offered.
//! - **Static member** (`Class::<prefix>`); constants, enum cases, methods.
//! - **Bare identifier**; global functions, constants, class-likes
//!   (case-insensitive prefix match).
//! - **Qualified** (`Foo\<prefix>`, `\\<prefix>`); namespace-scoped
//!   class-like search.

use mago_codex::metadata::CodebaseMetadata;
use mago_database::Database;
use mago_database::file::File as MagoFile;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;
use tower_lsp::lsp_types::CompletionResponse;

use crate::language_server::capabilities::lookup;
use crate::language_server::state::ExpressionTypeIndex;

mod items;

const MAX_RESULTS: usize = 50;

#[derive(Debug)]
pub(super) enum Context<'a> {
    Variable { prefix: &'a str, scope_start: u32 },
    InstanceMember { receiver_span: (u32, u32), prefix: &'a str },
    StaticMember { class: &'a str, prefix: &'a str },
    Qualified { qualifier: &'a str, prefix: &'a str },
    Bare { prefix: &'a str },
}

pub fn compute(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    type_index: Option<&ExpressionTypeIndex>,
    file: &MagoFile,
    offset: u32,
) -> CompletionResponse {
    let tokens = lookup::lex(file);
    let context = classify(&tokens, offset);
    let items = match context {
        Context::Variable { prefix, scope_start } => items::variable_items(&tokens, scope_start, offset, prefix),
        Context::InstanceMember { receiver_span, prefix } => {
            items::instance_member_items(codebase, type_index, receiver_span, prefix)
        }
        Context::StaticMember { class, prefix } => items::static_member_items(codebase, class, prefix),
        Context::Qualified { qualifier, prefix } => items::qualified_items(database, codebase, qualifier, prefix),
        Context::Bare { prefix } => items::bare_items(database, codebase, file, offset, prefix),
    };
    CompletionResponse::Array(items)
}

fn classify<'a>(tokens: &'a [Token<'a>], offset: u32) -> Context<'a> {
    let mut current_idx: Option<usize> = None;
    for (i, t) in tokens.iter().enumerate() {
        if lookup::is_trivia(t.kind) {
            continue;
        }
        let start = t.start.offset;
        let end = start + t.value.len() as u32;
        if start <= offset && offset <= end {
            current_idx = Some(i);
        }
        if start > offset {
            break;
        }
    }

    let scope_start = enclosing_function_start(tokens, offset);

    let cur = match current_idx {
        Some(i) => &tokens[i],
        None => return Context::Bare { prefix: "" },
    };

    if matches!(cur.kind, TokenKind::Variable) {
        let raw = cur.value;
        let consumed = (offset - cur.start.offset) as usize;
        let prefix = if consumed >= 1 && consumed <= raw.len() { &raw[1..consumed] } else { "" };
        return Context::Variable { prefix, scope_start };
    }

    let prev_idx = current_idx.and_then(|i| (0..i).rev().find(|j| !lookup::is_trivia(tokens[*j].kind)));
    let prev = prev_idx.map(|j| tokens[j].kind);

    if matches!(prev, Some(TokenKind::MinusGreaterThan) | Some(TokenKind::QuestionMinusGreaterThan)) {
        let prefix = if matches!(cur.kind, TokenKind::Identifier) { cur.value } else { "" };
        if let Some(receiver_span) = receiver_before(tokens, prev_idx.unwrap()) {
            return Context::InstanceMember { receiver_span, prefix };
        }
        return Context::Bare { prefix };
    }

    if matches!(prev, Some(TokenKind::ColonColon))
        && let Some(class) = static_receiver_before(tokens, prev_idx.unwrap())
    {
        let prefix = if matches!(cur.kind, TokenKind::Identifier) { cur.value } else { "" };
        return Context::StaticMember { class, prefix };
    }

    match cur.kind {
        TokenKind::Identifier => Context::Bare { prefix: cur.value },
        TokenKind::QualifiedIdentifier => match cur.value.rsplit_once('\\') {
            Some((qual, last)) => Context::Qualified { qualifier: qual, prefix: last },
            None => Context::Bare { prefix: cur.value },
        },
        TokenKind::FullyQualifiedIdentifier => {
            let stripped = cur.value.trim_start_matches('\\');
            match stripped.rsplit_once('\\') {
                Some((qual, last)) => Context::Qualified { qualifier: qual, prefix: last },
                None => Context::Qualified { qualifier: "", prefix: stripped },
            }
        }
        _ => Context::Bare { prefix: "" },
    }
}

fn receiver_before(tokens: &[Token<'_>], arrow_idx: usize) -> Option<(u32, u32)> {
    let mut j = arrow_idx;
    while j > 0 {
        j -= 1;
        if !lookup::is_trivia(tokens[j].kind) {
            let end_token = &tokens[j];
            let end = end_token.start.offset + end_token.value.len() as u32;
            let start = walk_chain_start(tokens, j);
            return Some((start, end));
        }
    }
    None
}

fn walk_chain_start(tokens: &[Token<'_>], end_idx: usize) -> u32 {
    let mut idx = end_idx;
    let mut start_offset = tokens[idx].start.offset;
    loop {
        if idx == 0 {
            return start_offset;
        }
        let mut k = idx - 1;
        while k > 0 && lookup::is_trivia(tokens[k].kind) {
            k -= 1;
        }
        if lookup::is_trivia(tokens[k].kind) {
            return start_offset;
        }
        match tokens[k].kind {
            TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan | TokenKind::ColonColon => {
                if k == 0 {
                    return start_offset;
                }
                let mut m = k - 1;
                while m > 0 && lookup::is_trivia(tokens[m].kind) {
                    m -= 1;
                }
                start_offset = tokens[m].start.offset;
                idx = m;
            }
            _ => return start_offset,
        }
    }
}

fn static_receiver_before<'a>(tokens: &'a [Token<'a>], colon_idx: usize) -> Option<&'a str> {
    let mut k = colon_idx;
    while k > 0 {
        k -= 1;
        if lookup::is_trivia(tokens[k].kind) {
            continue;
        }
        return match tokens[k].kind {
            TokenKind::Self_ | TokenKind::Static | TokenKind::Parent => Some(tokens[k].value),
            TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier => {
                Some(tokens[k].value.trim_start_matches('\\'))
            }
            _ => None,
        };
    }
    None
}

fn enclosing_function_start(tokens: &[Token<'_>], offset: u32) -> u32 {
    for t in tokens.iter().rev() {
        if t.start.offset >= offset {
            continue;
        }
        if matches!(t.kind, TokenKind::Function | TokenKind::Fn) {
            return t.start.offset;
        }
    }
    0
}
