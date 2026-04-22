//! PHP-specific view over the shared sequence primitives.
//!
//! Every syntax crate uses the same two container types, defined in
//! [`mago_syntax_core::ast`]. PHP fixes the [`TokenSeparatedSequence`]
//! separator to its own [`Token<'arena>`] via a type alias so that the
//! rest of the PHP AST can continue to write
//! `TokenSeparatedSequence<'arena, T>` (one generic) and pick up PHP
//! extension methods through [`TokenSeparatedSequenceExt`].

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::ast::TokenSeparatedSequence as CoreTokenSeparatedSequence;

use crate::token::Token;

pub use mago_syntax_core::ast::Sequence;

/// A comma-/semicolon-separated sequence of PHP AST nodes.
pub type TokenSeparatedSequence<'arena, T> = CoreTokenSeparatedSequence<'arena, T, Token<'arena>>;

/// PHP-specific helpers on a [`TokenSeparatedSequence`]. Supplies the
/// span-reconstruction methods that need to combine a file id with a
/// token's start and value length.
pub trait TokenSeparatedSequenceExt<'arena, T: HasSpan> {
    fn first_span(&self, file_id: FileId) -> Option<Span>;
    fn last_span(&self, file_id: FileId) -> Option<Span>;
    fn span(&self, file_id: FileId, from: Position) -> Span;
}

impl<'arena, T: HasSpan> TokenSeparatedSequenceExt<'arena, T> for TokenSeparatedSequence<'arena, T> {
    #[inline]
    fn first_span(&self, file_id: FileId) -> Option<Span> {
        match (self.tokens.first(), self.nodes.first()) {
            (Some(token), Some(node)) => {
                let token_end = token.start.offset + token.value.len() as u32;
                if token_end <= node.span().start.offset { Some(token.span_for(file_id)) } else { Some(node.span()) }
            }
            (Some(token), None) => Some(token.span_for(file_id)),
            (None, Some(node)) => Some(node.span()),
            (None, None) => None,
        }
    }

    #[inline]
    fn last_span(&self, file_id: FileId) -> Option<Span> {
        match (self.tokens.last(), self.nodes.last()) {
            (Some(token), Some(node)) => {
                if token.start.offset >= node.span().end.offset {
                    Some(token.span_for(file_id))
                } else {
                    Some(node.span())
                }
            }
            (Some(token), None) => Some(token.span_for(file_id)),
            (None, Some(node)) => Some(node.span()),
            (None, None) => None,
        }
    }

    #[inline]
    fn span(&self, file_id: FileId, from: Position) -> Span {
        match (self.first_span(file_id), self.last_span(file_id)) {
            (Some(first), Some(last)) => Span::new(file_id, first.start, last.end),
            _ => Span::new(file_id, from, from),
        }
    }
}
