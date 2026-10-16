//! Import targets use global names, even inside a namespace or a group.

use mago_names::kind::NameKind;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::domain::Range;
use crate::lookup;

pub(super) struct ImportTarget {
    pub kind: NameKind,
    pub qualifier: Vec<u8>,
    pub prefix: Vec<u8>,
    pub replace: Range,
    pub keyword_prefix: Option<Range>,
    pub alias: bool,
    pub group_prefix: bool,
}

/// Read the unfinished import without relying on parser recovery. The caller
/// excludes class bodies; `use (...)` captures fail the token check below.
pub(super) fn target(tokens: &[Token<'_>], offset: u32) -> Option<ImportTarget> {
    let start = tokens.iter().rposition(|token| token.kind == TokenKind::Use && token.start.offset < offset)?;
    let mut kind = NameKind::Default;
    let mut group_kind = kind;
    let mut group = Vec::new();
    let mut name = Vec::new();
    let mut alias = false;
    let mut end = offset;
    let mut keyword_prefix = None;
    let mut group_prefix = false;
    let mut last_end = offset;
    for token in tokens[start + 1..].iter().take_while(|token| token.start.offset <= offset) {
        if lookup::is_trivia(token.kind) {
            continue;
        }
        let consumed = (offset - token.start.offset) as usize;
        let typed = &token.value[..consumed.min(token.value.len())];
        let is_name = matches!(
            token.kind,
            TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier
        );
        if token.start.offset == offset && !is_name {
            break;
        }
        match token.kind {
            TokenKind::Function | TokenKind::Const if name.is_empty() && !alias => {
                kind = if token.kind == TokenKind::Function { NameKind::Function } else { NameKind::Constant };
                if group.is_empty() {
                    group_kind = kind;
                }
                let keyword_end = token.start.offset + token.value.len() as u32;
                if offset <= keyword_end {
                    // Replace a target after the keyword, keeping intervening comments and space.
                    let mut following =
                        tokens.iter().filter(|next| next.start.offset >= keyword_end && !lookup::is_trivia(next.kind));
                    let target = following.next().filter(|next| {
                        matches!(
                            next.kind,
                            TokenKind::Identifier
                                | TokenKind::QualifiedIdentifier
                                | TokenKind::FullyQualifiedIdentifier
                                | TokenKind::NamespaceSeparator
                        )
                    });
                    // A group prefix is not a single symbol that this edit can replace.
                    group_prefix = following.next().is_some_and(|next| {
                        next.kind == TokenKind::LeftBrace
                            || next.kind == TokenKind::NamespaceSeparator
                                && following.next().is_some_and(|next| next.kind == TokenKind::LeftBrace)
                    });
                    end = target.map_or(keyword_end, |target| target.start.offset + target.value.len() as u32);
                    keyword_prefix =
                        Some(Range::new(token.start.offset, target.map_or(keyword_end, |target| target.start.offset)));
                }
            }
            TokenKind::LeftBrace if group.is_empty() && name.last() == Some(&b'\\') => {
                group = std::mem::take(&mut name);
            }
            TokenKind::Comma => {
                name.clear();
                alias = false;
                kind = group_kind;
            }
            TokenKind::As if !name.is_empty() => alias = true,
            TokenKind::NamespaceSeparator if !alias => name.extend_from_slice(typed),
            _ if is_name => {
                if !alias {
                    if !name.is_empty() && name.last() != Some(&b'\\') {
                        return None;
                    }
                    name.extend_from_slice(typed);
                    end = token.start.offset + token.value.len() as u32;
                }
            }
            _ => return None,
        }
        last_end = token.start.offset + token.value.len() as u32;
    }
    // Whitespace after a complete target is an alias position, not a new name.
    alias |= !name.is_empty() && last_end < offset;
    let leaf_start = memchr::memrchr(b'\\', &name).map_or(0, |index| index + 1);
    let prefix = name[leaf_start..].to_vec();
    let mut qualifier = group;
    qualifier.extend_from_slice(&name[..leaf_start]);
    if qualifier.last() == Some(&b'\\') {
        qualifier.pop();
    }
    Some(ImportTarget {
        kind,
        qualifier,
        replace: Range::new(
            keyword_prefix.map_or(offset.saturating_sub(prefix.len() as u32), |keyword| keyword.start),
            end.max(offset),
        ),
        keyword_prefix,
        prefix,
        alias,
        group_prefix,
    })
}
