//! `get_completion`: context-sensitive completion candidates.
//!
//! Five contexts:
//! - **Variable** (`$<prefix>`): variables in the enclosing function scope.
//! - **Instance member** (`$obj-><prefix>`): receiver type comes from analyzer
//!   artifacts; methods, properties, and constants of the hierarchy are offered.
//! - **Static member** (`Class::<prefix>`): constants, enum cases, methods.
//! - **Bare identifier**: global functions, constants, class-likes (scored by
//!   the [`matcher`]).
//! - **Qualified** (`Foo\<prefix>`, `\<prefix>`): namespace-scoped class search.

mod items;
mod matcher;

use mago_codex::metadata::CodebaseMetadata;
use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_database::file::FileId;
use mago_names::scope::NamespaceScope;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::Server;
use crate::domain::CompletionEntry;
use crate::domain::CompletionList;
use crate::domain::Range;
use crate::lookup;
use crate::server::ExpressionTypeIndex;

const MAX_RESULTS: usize = 50;

#[derive(Debug)]
enum Context<'a> {
    Variable { prefix: &'a [u8], scope_start: u32 },
    InstanceMember { receiver_span: (u32, u32), prefix: &'a [u8] },
    StaticMember { class: &'a [u8], prefix: &'a [u8] },
    Qualified { qualifier: &'a [u8], prefix: &'a [u8] },
    Bare { prefix: &'a [u8], classes_only: bool },
}

impl Server {
    /// Completion candidates for the cursor at `offset` in `file_id`.
    pub fn get_completion(&mut self, file_id: FileId, offset: u32) -> CompletionList {
        let Ok(file) = self.database().get(&file_id) else {
            return CompletionList::default();
        };
        let type_index = self.type_index_for(file_id).cloned();
        let scope = match self.file_analysis_for(file_id) {
            Some(analysis) => analysis.scope_at(offset),
            None => return CompletionList::default(),
        };

        let items = compute(self.database(), self.codebase(), type_index.as_ref(), &scope, &file, offset);
        CompletionList { is_incomplete: true, items }
    }
}

fn byte_before(file: &MagoFile, offset: u32) -> Option<u8> {
    if offset == 0 {
        return None;
    }

    file.contents.get((offset - 1) as usize).copied()
}

fn compute(
    database: &mago_database::Database<'_>,
    codebase: &CodebaseMetadata,
    type_index: Option<&ExpressionTypeIndex>,
    scope: &NamespaceScope,
    file: &MagoFile,
    offset: u32,
) -> Vec<CompletionEntry> {
    let tokens = lookup::lex(file);
    let context = classify(file, &tokens, offset);
    match context {
        Context::Variable { prefix, scope_start } => {
            let replace = Range::new(offset.saturating_sub(prefix.len() as u32 + 1), offset);
            items::variable_items(&tokens, scope_start, offset, prefix, replace)
        }
        Context::InstanceMember { receiver_span, prefix } => {
            items::instance_member_items(codebase, type_index, receiver_span, prefix)
        }
        Context::StaticMember { class, prefix } => items::static_member_items(codebase, scope, class, prefix),
        Context::Qualified { qualifier, prefix } => items::qualified_items(database, codebase, qualifier, prefix),
        Context::Bare { prefix, classes_only } => items::bare_items(database, codebase, scope, prefix, classes_only),
    }
}

fn classify<'a>(file: &MagoFile, tokens: &'a [Token<'a>], offset: u32) -> Context<'a> {
    let mut current_idx: Option<usize> = None;
    for (i, t) in tokens.iter().enumerate() {
        if lookup::is_trivia(t.kind) {
            continue;
        }

        let start = t.start.offset;
        let end = start + t.value.len() as u32;
        if start <= offset && offset <= end {
            let starts_here_after_prev = offset == start
                && current_idx.is_some_and(|c| tokens[c].start.offset + tokens[c].value.len() as u32 == offset);
            if !starts_here_after_prev {
                current_idx = Some(i);
            }
        }
        if start > offset {
            break;
        }
    }

    let scope_start = enclosing_function_start(tokens, offset);

    let Some(current_idx) = current_idx else {
        return classify_after_punctuation(file, tokens, offset, scope_start);
    };

    let cur = &tokens[current_idx];

    if matches!(cur.kind, TokenKind::Variable) {
        let raw = cur.value;
        let consumed = (offset - cur.start.offset) as usize;
        let prefix: &[u8] = if consumed >= 1 && consumed <= raw.len() { &raw[1..consumed] } else { b"" };
        return Context::Variable { prefix, scope_start };
    }

    let cursor_at_end_of_cur = offset == cur.start.offset + cur.value.len() as u32;
    if cursor_at_end_of_cur {
        match cur.kind {
            TokenKind::Dollar => return Context::Variable { prefix: b"", scope_start },
            TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan => {
                if let Some(receiver_span) = receiver_before(tokens, current_idx) {
                    return Context::InstanceMember { receiver_span, prefix: b"" };
                }

                return Context::Bare { prefix: b"", classes_only: false };
            }
            TokenKind::ColonColon => {
                if let Some(class) = static_receiver_before(tokens, current_idx) {
                    return Context::StaticMember { class, prefix: b"" };
                }

                return Context::Bare { prefix: b"", classes_only: false };
            }
            _ => {}
        }
    }

    let prev_idx = (0..current_idx).rev().find(|j| !lookup::is_trivia(tokens[*j].kind));
    let prev = prev_idx.map(|j| tokens[j].kind);

    if matches!(prev, Some(TokenKind::MinusGreaterThan) | Some(TokenKind::QuestionMinusGreaterThan))
        && let Some(prev_idx) = prev_idx
    {
        let prefix: &[u8] = if matches!(cur.kind, TokenKind::Identifier) { cur.value } else { b"" };
        if let Some(receiver_span) = receiver_before(tokens, prev_idx) {
            return Context::InstanceMember { receiver_span, prefix };
        }

        return Context::Bare { prefix, classes_only: false };
    }

    if matches!(prev, Some(TokenKind::ColonColon))
        && let Some(prev_idx) = prev_idx
        && let Some(class) = static_receiver_before(tokens, prev_idx)
    {
        let prefix: &[u8] = if matches!(cur.kind, TokenKind::Identifier) { cur.value } else { b"" };
        return Context::StaticMember { class, prefix };
    }

    match cur.kind {
        TokenKind::Identifier => Context::Bare { prefix: cur.value, classes_only: expects_class_name(prev) },
        TokenKind::QualifiedIdentifier => match rsplit_once_byte(cur.value, b'\\') {
            Some((qual, last)) => Context::Qualified { qualifier: qual, prefix: last },
            None => Context::Bare { prefix: cur.value, classes_only: expects_class_name(prev) },
        },
        TokenKind::FullyQualifiedIdentifier => {
            let stripped = mago_bytes::trim_start_byte(cur.value, b'\\');
            match rsplit_once_byte(stripped, b'\\') {
                Some((qual, last)) => Context::Qualified { qualifier: qual, prefix: last },
                None => Context::Qualified { qualifier: b"", prefix: stripped },
            }
        }
        TokenKind::NamespaceSeparator => match prev_idx.map(|j| &tokens[j]) {
            Some(name)
                if matches!(
                    name.kind,
                    TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier
                ) =>
            {
                Context::Qualified { qualifier: mago_bytes::trim_start_byte(name.value, b'\\'), prefix: b"" }
            }
            _ => Context::Qualified { qualifier: b"", prefix: b"" },
        },
        _ => Context::Bare { prefix: b"", classes_only: false },
    }
}

fn expects_class_name(prev: Option<TokenKind>) -> bool {
    matches!(
        prev,
        Some(TokenKind::New | TokenKind::Instanceof | TokenKind::Extends | TokenKind::Implements | TokenKind::Catch)
    )
}

fn rsplit_once_byte(s: &[u8], byte: u8) -> Option<(&[u8], &[u8])> {
    memchr::memrchr(byte, s).map(|i| (&s[..i], &s[i + 1..]))
}

fn classify_after_punctuation<'a>(
    file: &MagoFile,
    tokens: &'a [Token<'a>],
    offset: u32,
    scope_start: u32,
) -> Context<'a> {
    if byte_before(file, offset) == Some(b'$') {
        return Context::Variable { prefix: b"", scope_start };
    }

    let prev_idx = (0..tokens.len()).rev().find(|i| {
        let token = &tokens[*i];
        let token_end = token.start.offset + token.value.len() as u32;
        token_end <= offset && !lookup::is_trivia(token.kind)
    });

    let Some(prev_idx) = prev_idx else {
        return Context::Bare { prefix: b"", classes_only: false };
    };

    let prev_kind = tokens[prev_idx].kind;

    if matches!(prev_kind, TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan) {
        if let Some(receiver_span) = receiver_before(tokens, prev_idx) {
            return Context::InstanceMember { receiver_span, prefix: b"" };
        }

        return Context::Bare { prefix: b"", classes_only: false };
    }

    if matches!(prev_kind, TokenKind::ColonColon)
        && let Some(class) = static_receiver_before(tokens, prev_idx)
    {
        return Context::StaticMember { class, prefix: b"" };
    }

    Context::Bare { prefix: b"", classes_only: expects_class_name(Some(prev_kind)) }
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

/// The static receiver name written before `::`. The scope resolves it to a
/// fully-qualified name (handling `use` aliases and namespace relativity).
fn static_receiver_before<'a>(tokens: &'a [Token<'a>], colon_idx: usize) -> Option<&'a [u8]> {
    let mut k = colon_idx;
    while k > 0 {
        k -= 1;
        if lookup::is_trivia(tokens[k].kind) {
            continue;
        }

        return match tokens[k].kind {
            TokenKind::Self_ | TokenKind::Static | TokenKind::Parent => Some(tokens[k].value),
            TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier => {
                Some(mago_bytes::trim_start_byte(tokens[k].value, b'\\'))
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
