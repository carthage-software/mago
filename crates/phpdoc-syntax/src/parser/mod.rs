use bumpalo::Bump;
use bumpalo::collections::Vec as BVec;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::input::Input;

use crate::cst::Document;
use crate::cst::inherit_doc::InheritDoc;
use crate::error::ParseError;
use crate::lexer::DocblockLexer;
use crate::parser::internal::stream::DocblockTokenStream;

pub(crate) mod internal;

#[derive(Debug)]
#[allow(clippy::field_scoped_visibility_modifiers)]
pub struct PHPDocParser<'arena> {
    pub(crate) arena: &'arena Bump,
    pub(crate) stream: DocblockTokenStream<'arena>,
    pub(crate) errors: BVec<'arena, ParseError>,
    pub(crate) inherit_docs: BVec<'arena, InheritDoc>,
}

impl<'arena> PHPDocParser<'arena> {
    #[must_use]
    pub fn new(arena: &'arena Bump, content: &'arena [u8], span: Span) -> Self {
        let input = Input::anchored_at(span.file_id, content, span.start);
        let lexer = DocblockLexer::new(input);
        let stream = DocblockTokenStream::new(arena, lexer, content, span.start);

        Self { arena, stream, errors: BVec::new_in(arena), inherit_docs: BVec::new_in(arena) }
    }

    #[must_use]
    pub fn parse_with_span(arena: &'arena Bump, content: &'arena [u8], span: Span) -> Document<'arena> {
        let mut parser = Self::new(arena, content, span);

        parser.parse_document(span)
    }

    #[must_use]
    pub fn parse(arena: &'arena Bump, file_id: FileId, content: &'arena [u8]) -> Document<'arena> {
        let span = Span::new(file_id, Position::new(0), Position::new(content.len() as u32));

        Self::parse_with_span(arena, content, span)
    }

    #[inline]
    pub(crate) fn file_id(&self) -> FileId {
        self.stream.file_id()
    }
}

/// Parses a standalone type expression (the PHPStan/Psalm type mini-language) from `content`.
///
/// # Errors
///
/// Returns a [`ParseError`] if the input is not a well-formed type.
pub fn parse_type<'arena>(
    arena: &'arena Bump,
    content: &'arena [u8],
    span: Span,
) -> Result<crate::cst::r#type::Type<'arena>, ParseError> {
    let mut parser = PHPDocParser::new(arena, content, span);

    parser.parse_type()
}
