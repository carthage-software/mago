//! The source range and call syntax shared by a site's completion candidates.

use mago_syntax::token::{Token, TokenKind};

use crate::domain::{CompletionEntry, CompletionInsert, Range};
use crate::lookup;

use super::context::Context;
use super::imports::ImportTarget;

pub(super) struct CompletionEdit {
    range: Range,
    has_arguments: bool,
}

impl CompletionEdit {
    pub(super) fn at(
        tokens: &[Token<'_>],
        offset: u32,
        context: Context<'_>,
        import: Option<&ImportTarget>,
    ) -> Option<Self> {
        if matches!(context, Context::Variable { .. }) {
            return None;
        }
        // Replace the whole selector, including its suffix and a static property's `$`.
        let mut range = tokens
            .iter()
            .find(|token| {
                token.start.offset <= offset
                    && offset <= token.start.offset + token.value.len() as u32
                    && (token.kind.is_identifier_maybe_reserved()
                        || matches!(
                            token.kind,
                            TokenKind::QualifiedIdentifier
                                | TokenKind::FullyQualifiedIdentifier
                                | TokenKind::Variable
                                | TokenKind::Dollar
                        ))
            })
            .map_or(Range::new(offset, offset), |token| {
                Range::new(token.start.offset, token.start.offset + token.value.len() as u32)
            });
        if let Some(import) = import {
            range = import.replace;
        } else {
            if let Context::Qualified { prefix, .. } = context {
                range.start = offset.saturating_sub(prefix.len() as u32);
                range.end = range.end.max(range.start);
            }
            let next = tokens.partition_point(|token| token.start.offset < range.end);
            if tokens
                .get(next)
                .is_some_and(|token| token.start.offset == range.end && token.kind == TokenKind::NamespaceSeparator)
            {
                range.end += 1;
            }
        }
        let has_arguments = tokens
            .iter()
            .find(|token| token.start.offset >= range.end && !lookup::is_trivia(token.kind))
            .is_some_and(|token| token.value == b"(");
        Some(Self { range, has_arguments })
    }

    pub(super) fn apply(&self, items: &mut [CompletionEntry]) {
        for item in items {
            item.replace = Some(self.range);
            if self.has_arguments {
                item.insert = item.insert.take().map(|insert| match insert {
                    CompletionInsert::Call(name) => CompletionInsert::Text(name),
                    text => text,
                });
            }
        }
    }
}
